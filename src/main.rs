// src/main.rs — animi 交织器 CLI 入口
//
// 用法：animi <file.anim>
//       animi --help
//       animi --version
// 输出：<file>.json

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
        eprintln!("用法: animi <file.anim>");
        eprintln!("      animi --help    显示帮助");
        eprintln!("      animi --version 显示版本");
        process::exit(1);
    }

    let path = &args[1];

    if path == "--help" || path == "-h" {
        println!("animi — Anim 交织语言工具链");
        println!();
        println!("用法: animi <file.anim>");
        println!("      animi --help    显示此帮助");
        println!("      animi --version 显示版本信息");
        println!();
        println!("输入 .anim 源码，输出 FSIR JSON。");
        println!("交织器，不是编译器——animi = Anim Interlinker。");
        return;
    }

    if path == "--version" || path == "-V" {
        println!("animi v0.1.0");
        return;
    }

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

    let checker = animi::typeck::TypeChecker::new();
    die(checker.check(&ast));

    die(animi::rule::check(&ast));
    die(animi::safety::check(&ast));
    die(animi::guard::inject(&ast));

    let doc = animi::fsir::FsirDoc::from_ast(&ast, None);
    let json = die(doc.to_json());

    let out_path = format!("{}.json", path.trim_end_matches(".anim"));
    if let Err(e) = fs::write(&out_path, &json) {
        eprintln!("无法写入 {}: {}", out_path, e);
        process::exit(1);
    }

    println!("✅ {} → {}", path, out_path);
}
