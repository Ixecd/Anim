// src/main.rs — animi 交织器 CLI 入口
//
// 用法：animi <file.anim> [registry.json]
//       animi --help
//       animi --version
// 输出：<file>.json (FSIR)
//
// Anim 职责收敛：.anim → FSIR only (Pass 0-5)
// Pass 6+ 归属 Feelings-Core

use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::process;

use animi::error::AnimiError;
use animi::error::Severity;
use animi::log::A;

fn die<T>(r: Result<T, AnimiError>) -> T {
    match r {
        Ok(v) => v,
        Err(e) => {
            A.error(format_args!("{}", e));
            process::exit(1);
        }
    }
}

fn die_soft(r: Result<(), AnimiError>, strict: bool, verbose: bool) {
    match r {
        Ok(()) => {}
        Err(e) => match e.severity() {
            Severity::Deny => {
                A.error(format_args!("{}", e));
                process::exit(1);
            }
            Severity::Warn => {
                if strict {
                    A.error(format_args!("{} (--strict)", e));
                    process::exit(1);
                } else {
                    A.warn(format_args!("{}", e));
                }
            }
            Severity::Note => {
                if verbose {
                    A.debug(format_args!("{}", e));
                }
            }
        },
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
        println!("animi v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let mut user_cap: u32 = 100;
    let mut config_path: Option<String> = None;
    let mut reg_path: Option<&String> = None;
    let mut strict = false;
    let mut verbose = false;
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--cap" && i + 1 < args.len() {
            user_cap = match args[i + 1].parse() {
                Ok(v) => v,
                Err(_) => {
                    A.warn(format_args!(
                        "无效的 cap 值 '{}'，使用默认 100",
                        args[i + 1]
                    ));
                    100
                }
            };
            i += 2;
        } else if args[i] == "--config" && i + 1 < args.len() {
            config_path = Some(args[i + 1].clone());
            i += 2;
        } else if args[i] == "--log-level" && i + 1 < args.len() {
            let lvl = match args[i + 1].to_lowercase().as_str() {
                "debug" => animi::log::Level::Debug,
                "info" => animi::log::Level::Info,
                "warn" => animi::log::Level::Warn,
                "error" => animi::log::Level::Error,
                _ => {
                    A.warn(format_args!(
                        "无效的日志级别 '{}'，使用默认 info",
                        args[i + 1]
                    ));
                    animi::log::Level::Info
                }
            };
            animi::log::set_log_level(lvl);
            i += 2;
        } else if args[i] == "--strict" {
            strict = true;
            i += 1;
        } else if args[i] == "--verbose" {
            verbose = true;
            i += 1;
        } else if reg_path.is_none() {
            reg_path = Some(&args[i]);
            i += 1;
        } else {
            A.warn(format_args!("未知参数 '{}'，已忽略", args[i]));
            i += 1;
        }
    }

    let registry = if let Some(path) = reg_path {
        match animi::registry::Registry::from_file(path) {
            Ok(r) => {
                A.info(format_args!("✅ 加载外部 Registry: {}", path));
                r
            }
            Err(e) => {
                A.warn(format_args!(
                    "⚠️ 加载外部 Registry 失败: {}，使用内建列表",
                    e
                ));
                animi::registry::Registry::default()
            }
        }
    } else {
        animi::registry::Registry::default()
    };

    let config = if let Some(ref cfg_path) = config_path {
        match animi::config::AnimConfig::load(cfg_path) {
            Ok(c) => {
                A.info(format_args!("✅ 加载外部 Config: {}", cfg_path));
                c
            }
            Err(e) => {
                A.warn(format_args!("⚠️ 加载 Config 失败: {}，使用默认值", e));
                animi::config::AnimConfig::default()
            }
        }
    } else {
        animi::config::AnimConfig::default()
    };

    animi::error::CURRENT_FILE.with(|f| *f.borrow_mut() = path.clone());

    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            A.error(format_args!("无法读取文件 {}: {}", path, e));
            process::exit(1);
        }
    };

    // ── Pass 0a-expand：宏展开 ──
    let (macros, expanded_src) = animi::macros::extract_macros(&src);
    let processed_src = animi::macros::expand_macros(&expanded_src, &macros);
    if !macros.is_empty() {
        A.info(format_args!("展开 {} 个宏定义", macros.len()));
    }
    die(animi::macros::validate_expanded(&processed_src));

    let mut lexer = animi::lexer::Lexer::new(&processed_src);
    let tokens = die(lexer.tokenize());
    let mut parser = animi::parser::Parser::new(tokens);
    let ast = die(parser.parse());
    let checker = animi::typeck::TypeChecker::new(&registry);
    die(checker.check(&ast));

    // ── Pass 2b：静态安全规则 ──
    die_soft(
        animi::rule::check(&ast, &registry, &config),
        strict,
        verbose,
    );

    // Pass 3——用户安全检查 + 强度缩放
    let scaled = die(animi::safety::check_with_scale(&ast, user_cap, &config));

    // ── Pass 3b：oi 帧平滑 ──
    let smoothing = die(animi::guard::inject(&ast, &config));
    A.info(format_args!(
        "oi 帧平滑: {} 帧 {}ms 衰减{}",
        smoothing.frame_count(),
        smoothing.duration_ms(),
        if smoothing.is_hard_cut() {
            " (硬截断——强度≤20)"
        } else {
            ""
        }
    ));

    let src_hash = hex::encode(Sha256::digest(src.as_bytes()));
    let multipliers: Vec<f64> = smoothing.steps.iter().map(|s| s.multiplier).collect();
    let doc = animi::fsir::FsirDoc::from_ast(
        &ast,
        Some(src_hash),
        Some(registry.hash()),
        &animi::fsir::FsirIntensity {
            min: scaled.min,
            max: scaled.max,
        },
        Some(&multipliers),
    );

    let json = die(doc.to_json());
    let basename = path.strip_suffix(".anim").unwrap_or(path);
    let out_path = format!("{}.json", basename);
    if let Err(e) = fs::write(&out_path, &json) {
        A.error(format_args!("无法写入 {}: {}", out_path, e));
        process::exit(1);
    }
    A.info(format_args!("✅ {} → {}", path, out_path));
}
