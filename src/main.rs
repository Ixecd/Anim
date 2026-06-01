// src/main.rs — animi 交织器 CLI 入口
//
// 用法：animi <file.anim>
// 输出：fsir.json

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: animi <file.anim>");
        process::exit(1);
    }

    let path = &args[1];
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("无法读取文件 {}: {}", path, e);
            process::exit(1);
        }
    };

    // Pass 0a — 词法
    let mut lexer = animi::lexer::Lexer::new(&src);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    // Pass 0b — 语法
    let mut parser = animi::parser::Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    // Pass 1 — 类型检查
    let checker = animi::typeck::TypeChecker::new();
    if let Err(e) = checker.check(&ast) {
        eprintln!("{}", e);
        process::exit(1);
    }

    // Pass 2 — 静态安全规则
    animi::rule::check(&ast).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    // Pass 3 — 用户安全（v1.1 桩——直接通过）
    animi::safety::check(&ast).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    // Pass 4 — 运行期插桩预埋（v1.1 桩——直接通过）
    animi::guard::inject(&ast).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    // Pass 5 — FSIR 生成
    let doc = animi::fsir::FsirDoc::from_ast(&ast, "");
    let json = match doc.to_json() {
        Ok(j) => j,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    // 输出
    let out_path = format!("{}.json", path.trim_end_matches(".anim"));
    if let Err(e) = fs::write(&out_path, &json) {
        eprintln!("无法写入 {}: {}", out_path, e);
        process::exit(1);
    }

    println!("✅ {} → {}", path, out_path);
}
