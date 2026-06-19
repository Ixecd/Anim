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
use animi::error::Severity;
use animi::log::A;

/// 硬错误——Deny 级别，直接退出。
fn die<T>(r: Result<T, AnimiError>) -> T {
    match r {
        Ok(v) => v,
        Err(e) => {
            A.error(format_args!("{}", e));
            process::exit(1);
        }
    }
}

/// 软错误——按 severity + flags 分发。Warn 打印继续，Note 静默吞掉。
/// 返回 None 表示被吞掉了，调用方应继续。
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
        println!("animi v0.1.0");
        return;
    }

    // 解析可选参数：--cap <N>、--config <path>、--strict、--verbose 和 <registry.json>
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

    // ── Pass 0a-expand：宏展开 ──
    let (macros, expanded_src) = animi::macros::extract_macros(&src);
    let processed_src = animi::macros::expand_macros(&expanded_src, &macros);
    if !macros.is_empty() {
        A.info(format_args!("展开 {} 个宏定义", macros.len()));
    }

    let mut lexer = animi::lexer::Lexer::new(&processed_src);
    let tokens = die(lexer.tokenize());

    let mut parser = animi::parser::Parser::new(tokens);
    let ast = die(parser.parse());

    let checker = animi::typeck::TypeChecker::new(&registry);
    die(checker.check(&ast));

    // ── Pipeline + Hook 架构 —— ADR 013 ──
    let pipeline = animi::pipeline::Pipeline::build(&config);
    let mut ctx = animi::pipeline::Ctx::new(&config, user_cap);
    ctx.ast = Some(&ast);
    ctx.registry = Some(&registry);

    // AfterTypeCheck —— static_safety hook（可 Warn）
    die_soft(
        pipeline.run_stage(animi::pipeline::PipelineStage::AfterTypeCheck, &ctx),
        strict,
        verbose,
    );

    // Pass 3——用户安全检查 + 强度缩放（硬线——Deny）
    let scaled = die(animi::safety::check_with_scale(&ast, user_cap, &config));
    *ctx.scaled_intensity.borrow_mut() = Some(scaled.clone());

    // AfterIntensityScale —— oi_smoothing / leaky_bucket hook（可 Warn/Note）
    die_soft(
        pipeline.run_stage(animi::pipeline::PipelineStage::AfterIntensityScale, &ctx),
        strict,
        verbose,
    );

    // ── Sandbox routing + Governance —— ADR 009 §十 ──
    if animi::sandbox::is_sandbox(scaled.max) {
        A.info(format_args!(
            "沙箱激活: 强度 max={} >= {}",
            scaled.max,
            animi::sandbox::SANDBOX_THRESHOLD
        ));
        let mut responses: Vec<animi::registry::SandboxResponse> = Vec::new();
        if let Some(r) = registry.sandbox_response(&ast.mix.main.name) {
            responses.push(r);
        }
        for acc in &ast.mix.accents {
            if let Some(r) = registry.sandbox_response(&acc.atom.name) {
                responses.push(r);
            }
        }
        let worst = animi::sandbox::worst_response(&responses);
        let accent_ratios: Vec<(String, f64)> = ast
            .mix
            .accents
            .iter()
            .map(|a| (a.atom.name.clone(), a.ratio))
            .collect();

        let combined_action =
            animi::sandbox::check_combined(scaled.max, &accent_ratios, user_cap, worst, None);
        let mut actions = vec![combined_action];
        for acc in &ast.mix.accents {
            let r = registry
                .sandbox_response(&acc.atom.name)
                .unwrap_or_default();
            actions.push(animi::sandbox::check_accent_absolute(
                acc.ratio, scaled.max, r, None,
            ));
        }
        let final_action = animi::sandbox::strictest(&actions);
        if !final_action.is_passthrough() {
            A.info(format_args!("沙箱治理: {}", final_action.description()));
            match &final_action {
                animi::sandbox::GovernanceAction::Steer {
                    blend_atom,
                    blend_intensity,
                    ..
                } => {
                    A.info(format_args!(
                        "  → G1 温和拉回: 叠加 {} @ {} 强度",
                        blend_atom, blend_intensity
                    ));
                }
                animi::sandbox::GovernanceAction::Redirect {
                    atom, intensity, ..
                } => {
                    A.info(format_args!(
                        "  → G2 重新导向: 替换为 {} @ {} 强度",
                        atom, intensity
                    ));
                }
                animi::sandbox::GovernanceAction::Anchor {
                    atom,
                    intensity,
                    escalate_defence,
                    ..
                } => {
                    A.info(format_args!(
                        "  → G3 安全锚点: {} @ {} 强度{}",
                        atom,
                        intensity,
                        if *escalate_defence {
                            " (升级 DefenceLevel)"
                        } else {
                            ""
                        }
                    ));
                }
                _ => {}
            }
        }
    }

    // 从 ctx 读取 oi_smoothing 产出
    let smoothing = ctx
        .smoothing_output
        .borrow()
        .clone()
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

    // ── Pass 7: DeviceMap — PSIR × DeviceSet → DSIR ──
    let psir = doc.to_psir_stub();
    let device_set = animi::dsir::DeviceSet::default();
    let dsir = die(animi::device_map::device_map(&psir, &device_set));
    let dsir_json = die(dsir.to_json());
    let dsir_out = format!("{}.dsir.json", path.trim_end_matches(".anim"));
    if let Err(e) = fs::write(&dsir_out, &dsir_json) {
        A.error(format_args!("无法写入 {}: {}", dsir_out, e));
        process::exit(1);
    }
    A.info(format_args!("✅ {} → {}", path, dsir_out));

    // ── Pass 8: CodeGen — DSIR → ESIR ──
    let esir = die(animi::codegen::codegen(&dsir));
    let esir_bytes = die(esir.to_binary());
    let esir_out = format!("{}.esir", path.trim_end_matches(".anim"));
    if let Err(e) = fs::write(&esir_out, &esir_bytes) {
        A.error(format_args!("无法写入 {}: {}", esir_out, e));
        process::exit(1);
    }
    A.info(format_args!(
        "✅ {} → {} ({} 帧, {}ms)",
        path, esir_out, esir.frame_count, esir.duration_ms
    ));
}
