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
use animi::log::A;

/// 错误处理辅助——Ok 则返回值，Err 则打印并退出。
fn die<T>(r: Result<T, AnimiError>) -> T {
    match r {
        Ok(v) => v,
        Err(e) => {
            A.error(format_args!("{}", e));
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

    // 解析可选参数：--cap <N>、--config <path> 和 <registry.json>
    let mut user_cap: u32 = 100;
    let mut config_path: Option<String> = None;
    let mut reg_path: Option<&String> = None;
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
        } else if reg_path.is_none() {
            reg_path = Some(&args[i]);
            i += 1;
        } else {
            A.warn(format_args!("未知参数 '{}'，已忽略", args[i]));
            i += 1;
        }
    }

    // Registry——外部 JSON 或内建 fallback
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

    // Config——外部 YAML 或默认值
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

    // 设置当前文件名——oi! 宏自动从中读取
    animi::error::CURRENT_FILE.with(|f| *f.borrow_mut() = path.clone());

    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            A.error(format_args!("无法读取文件 {}: {}", path, e));
            process::exit(1);
        }
    };

    let mut lexer = animi::lexer::Lexer::new(&src);
    let tokens = die(lexer.tokenize());

    let mut parser = animi::parser::Parser::new(tokens);
    let ast = die(parser.parse());

    let checker = animi::typeck::TypeChecker::new(&registry);
    die(checker.check(&ast));

    // ── Pipeline + Hook 架构 —— ADR 013 ──
    let pipeline = animi::pipeline::Pipeline::build(&config);
    let mut ctx = animi::pipeline::HookContext::new(&config, user_cap);
    ctx.ast = Some(&ast);
    ctx.registry = Some(&registry);

    // AfterTypeCheck —— static_safety hook
    die(pipeline.run_stage(animi::pipeline::PipelineStage::AfterTypeCheck, &ctx));

    // Pass 3——用户安全检查 + 强度缩放
    let scaled = die(animi::safety::check_with_scale(&ast, user_cap, &config));
    *ctx.scaled_intensity.borrow_mut() = Some(scaled.clone());

    // AfterIntensityScale —— oi_smoothing / leaky_bucket_intake hooks
    die(pipeline.run_stage(animi::pipeline::PipelineStage::AfterIntensityScale, &ctx));

    // 从 ctx 读取 oi_smoothing 产出
    let smoothing = ctx.smoothing_output.borrow().clone()
        .expect("oi_smoothing hook must produce output");
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

    // 源码哈希——SPL 锚定用
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

    let out_path = format!("{}.json", path.trim_end_matches(".anim"));
    if let Err(e) = fs::write(&out_path, &json) {
        A.error(format_args!("无法写入 {}: {}", out_path, e));
        process::exit(1);
    }

    A.info(format_args!("✅ {} → {}", path, out_path));
}
