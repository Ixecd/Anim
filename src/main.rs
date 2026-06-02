// src/main.rs — animi 交织器 CLI 入口
//
// 用法：animi <file.anim> [registry.json]
//       animi --help
//       animi --version
// 输出：<file>.json

use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::process;

use animi::error::AnimiError;
use animi::{A_info, A_warn};

/// 错误处理辅助——Ok 则返回值，Err 则打印并退出。
fn die<T>(r: Result<T, AnimiError>) -> T {
    match r {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: animi <file.anim> [registry.json]");
        eprintln!("      animi --help    显示帮助");
        eprintln!("      animi --version 显示版本");
        process::exit(1);
    }

    let path = &args[1];

    if path == "--help" || path == "-h" {
        println!("animi — Anim 交织语言工具链");
        println!();
        println!("用法: animi <file.anim> [registry.json]");
        println!("      animi --help    显示此帮助");
        println!("      animi --version 显示版本信息");
        println!();
        println!("输入 .anim 源码，输出 FSIR JSON。");
        println!("可选第二个参数——外部 Registry JSON 文件。");
        println!("交织器，不是编译器——animi = Anim Interlinker。");
        return;
    }

    if path == "--version" || path == "-V" {
        println!("animi v0.1.0");
        return;
    }

    // 解析可选参数：--cap <N> 和 <registry.json>
    let mut user_cap: u32 = 100;
    let mut reg_path: Option<&String> = None;
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--cap" && i + 1 < args.len() {
            user_cap = args[i + 1].parse().unwrap_or(100);
            i += 2;
        } else if reg_path.is_none() {
            reg_path = Some(&args[i]);
            i += 1;
        } else {
            i += 1;
        }
    }

    // Registry——外部 JSON 或内建 fallback
    let registry = if let Some(path) = reg_path {
        match animi::registry::Registry::from_file(path) {
            Ok(r) => {
                A_info!("加载外部 Registry: {}", path);
                r
            }
            Err(e) => {
                A_warn!("加载外部 Registry 失败: {}，使用内建列表", e);
                animi::registry::Registry::default()
            }
        }
    } else {
        animi::registry::Registry::default()
    };

    // 设置当前文件名——oi! 宏自动从中读取
    animi::error::CURRENT_FILE.with(|f| *f.borrow_mut() = path.clone());

    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("无法读取文件 {}: {}", path, e);
            process::exit(1);
        }
    };

    let mut lexer = animi::lexer::Lexer::new(&src);
    let tokens = die(lexer.tokenize());

    let mut parser = animi::parser::Parser::new(tokens);
    let ast = die(parser.parse());

    let checker = animi::typeck::TypeChecker::new(&registry);
    die(checker.check(&ast));

    die(animi::rule::check(&ast, &registry));

    // Pass 3——用户安全检查 + 强度缩放（user_cap 从 --cap 参数或取默认值 100）
    let scaled = animi::safety::check_with_scale(&ast, user_cap).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    die(animi::guard::inject(&ast));

    // 源码哈希——SPL 锚定用
    let src_hash = hex::encode(Sha256::digest(src.as_bytes()));
    let doc = animi::fsir::FsirDoc::from_ast(
        &ast,
        Some(src_hash),
        Some(registry.hash()),
        &animi::fsir::FsirIntensity {
            min: scaled.min,
            max: scaled.max,
        },
    );

    let json = die(doc.to_json());

    let out_path = format!("{}.json", path.trim_end_matches(".anim"));
    if let Err(e) = fs::write(&out_path, &json) {
        eprintln!("无法写入 {}: {}", out_path, e);
        process::exit(1);
    }

    println!("✅ {} → {}", path, out_path);
}
