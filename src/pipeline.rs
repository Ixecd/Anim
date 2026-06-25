// src/pipeline.rs — Pipeline + Hook 架构 —— ADR 013
//
// Anim 职责收敛。漏桶/脱敏/阻尼 Hook 已迁移至 Feelings-Core。
// 仅保留 static_safety + oi_smoothing 两个 Hook。

use crate::ast::FeelingSource;
use crate::config::AnimConfig;
use crate::error::AnimiError;
use crate::fsir::FsirDoc;
use crate::registry::Registry;
use std::cell::RefCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineStage {
    AfterParse,
    AfterTypeCheck,
    AfterIntensityScale,
    AfterPersonalize,
    OnSessionStart,
    OnSessionEnd,
}

pub struct Ctx<'a> {
    pub stage: PipelineStage,
    pub ast: Option<&'a FeelingSource>,
    pub fsir: Option<&'a FsirDoc>,
    pub config: &'a AnimConfig,
    pub user_cap: u32,
    pub registry: Option<&'a Registry>,
    pub scaled_intensity: RefCell<Option<crate::ast::Intensity>>,
    pub smoothing_output: RefCell<Option<crate::guard::OiSmoothing>>,
}

impl<'a> Ctx<'a> {
    pub fn new(config: &'a AnimConfig, user_cap: u32) -> Self {
        Ctx {
            stage: PipelineStage::AfterParse,
            ast: None,
            fsir: None,
            config,
            user_cap,
            registry: None,
            scaled_intensity: RefCell::new(None),
            smoothing_output: RefCell::new(None),
        }
    }
}

/// Hook 基 trait——每个安全/检测/校准机制实现此 trait。
pub trait PipelineHook {
    fn name(&self) -> &'static str;
    fn run(&self, ctx: &Ctx) -> Result<(), AnimiError>;
}

/// 管线——持有 hook 列表，按阶段执行。
pub struct Pipeline {
    hooks: Vec<(PipelineStage, Box<dyn PipelineHook>)>,
}

impl Pipeline {
    pub fn build(_config: &AnimConfig) -> Self {
        let hooks: Vec<(PipelineStage, Box<dyn PipelineHook>)> = vec![
            (PipelineStage::AfterTypeCheck, Box::new(StaticSafetyHook)),
            (
                PipelineStage::AfterIntensityScale,
                Box::new(OiSmoothingHook),
            ),
        ];
        Pipeline { hooks }
    }

    pub fn run_stage(&self, stage: PipelineStage, ctx: &Ctx) -> Result<(), AnimiError> {
        for (hook_stage, hook) in &self.hooks {
            if *hook_stage == stage {
                hook.run(ctx)?;
            }
        }
        Ok(())
    }
}

// ── static_safety hook —— Pass 2b ──

pub struct StaticSafetyHook;

impl PipelineHook for StaticSafetyHook {
    fn name(&self) -> &'static str {
        "static_safety"
    }
    fn run(&self, ctx: &Ctx) -> Result<(), AnimiError> {
        if let Some(ast) = ctx.ast {
            let reg = ctx.registry.unwrap_or_else(|| {
                // static safety rules work without a registry (global cap, abrupt stop)
                // We need a reference, so create a default registry if none
                panic!("static_safety hook requires registry")
            });
            crate::rule::check(ast, reg, ctx.config)
        } else {
            Ok(())
        }
    }
}

// ── oi_smoothing hook —— Pass 3b ──

pub struct OiSmoothingHook;

impl PipelineHook for OiSmoothingHook {
    fn name(&self) -> &'static str {
        "oi_smoothing"
    }
    fn run(&self, ctx: &Ctx) -> Result<(), AnimiError> {
        if let Some(ast) = ctx.ast {
            let smoothing = crate::guard::inject(ast, ctx.config)?;
            *ctx.smoothing_output.borrow_mut() = Some(smoothing);
        }
        Ok(())
    }
}
