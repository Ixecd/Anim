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

    // Registry——外部 JSON 或内建 fallback
    let registry = if let Some(reg_path) = args.get(2) {
        match animi::registry::Registry::from_file(reg_path) {
            Ok(r) => {
                eprintln!("✅ 加载外部 Registry: {}", reg_path);
                r
            }
            Err(e) => {
                eprintln!("⚠️ 加载外部 Registry 失败: {}，使用内建列表", e);
                animi::registry::Registry::default()
            }
        }
    } else {
        animi::registry::Registry::default()
    };

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

    // Pass 3——用户安全检查 + 强度缩放（v1.1 桩，user_cap 从环境变量取）
    let user_cap: u32 = std::env::var("ANIMI_USER_CAP")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    let _scaled = animi::safety::check_with_scale(&ast, user_cap).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    die(animi::guard::inject(&ast));

    // 源码哈希——SPL 锚定用
    let src_hash = hex::encode(Sha256::digest(src.as_bytes()));
    let doc = animi::fsir::FsirDoc::from_ast(&ast, Some(src_hash));

    let json = die(doc.to_json());

    let out_path = format!("{}.json", path.trim_end_matches(".anim"));
    if let Err(e) = fs::write(&out_path, &json) {
        eprintln!("无法写入 {}: {}", out_path, e);
        process::exit(1);
    }

    println!("✅ {} → {}", path, out_path);
}
