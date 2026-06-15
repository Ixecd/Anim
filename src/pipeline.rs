// src/pipeline.rs — Pipeline + Hook 架构 —— ADR 013
//
// 核心管线只做数据流转，不可变。
// 每个安全/检测/校准机制 = 独立的 PipelineHook，在 PipelineStage 注入点注册。
// Hook 列表在 Session 启动时从 config 一次性构建，运行期不变。

use crate::ast::FeelingSource;
use crate::config::{AnimConfig, HooksConfig};
use crate::error::AnimiError;
use crate::fsir::FsirDoc;
use crate::pbm::PbmDimension;
use crate::psir::PsirDoc;
use crate::registry::Registry;
use crate::safety::{monotonic_ns, NeuroEnergyTracker, UserSafetyProfile};
use std::cell::RefCell;

// ═══════════════════════════════════════════════════════════════
// PipelineStage
// ═══════════════════════════════════════════════════════════════

/// 管线阶段——Hook 注入点。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineStage {
    AfterParse,
    AfterTypeCheck,
    AfterIntensityScale,
    AfterPersonalize,
    OnSessionStart,
    OnSessionEnd,
}

// ═══════════════════════════════════════════════════════════════
// Ctx
// ═══════════════════════════════════════════════════════════════

/// Hook 上下文——只读管线数据 + 有状态 hook 的内部可变性。
///
/// 每个阶段只暴露该阶段已就绪的数据。
pub struct Ctx<'a> {
    pub stage: PipelineStage,

    // —— 只读管线数据（随阶段逐步就绪）——
    pub ast: Option<&'a FeelingSource>,
    pub fsir: Option<&'a FsirDoc>,
    pub psir: Option<&'a PsirDoc>,

    // —— 只读配置 ——
    pub config: &'a AnimConfig,

    // —— 管线初始化参数 ——
    pub user_cap: u32,
    pub registry: Option<&'a Registry>,

    // —— 有状态 hook 通过 RefCell 实现内部可变性 ——
    pub tracker: RefCell<Option<NeuroEnergyTracker>>,
    pub previous_intensities: RefCell<Option<[u32; 4]>>,
    pub monotony_counters: RefCell<Option<[u32; 4]>>,

    // —— Pass 6 产出（用于 main.rs 消费）——
    pub scaled_intensity: RefCell<Option<crate::ast::Intensity>>,
    pub smoothing_output: RefCell<Option<crate::guard::OiSmoothing>>,
    pub psir_output: RefCell<Option<PsirDoc>>,
}

impl<'a> Ctx<'a> {
    pub fn new(config: &'a AnimConfig, user_cap: u32) -> Self {
        Ctx {
            stage: PipelineStage::AfterParse,
            ast: None,
            fsir: None,
            psir: None,
            config,
            user_cap,
            registry: None,
            tracker: RefCell::new(None),
            previous_intensities: RefCell::new(None),
            monotony_counters: RefCell::new(None),
            scaled_intensity: RefCell::new(None),
            smoothing_output: RefCell::new(None),
            psir_output: RefCell::new(None),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// PipelineHook trait
// ═══════════════════════════════════════════════════════════════

/// 管线钩子——在 PipelineStage 注入点被调用。
pub trait PipelineHook {
    fn name(&self) -> &str;
    fn stage(&self) -> PipelineStage;
    fn run(&self, ctx: &Ctx) -> Result<(), AnimiError>;
}

// ═══════════════════════════════════════════════════════════════
// Pipeline
// ═══════════════════════════════════════════════════════════════

/// 管线——持有 Session 生命周期内不变的 hook 列表。
pub struct Pipeline {
    hooks: Vec<Box<dyn PipelineHook>>,
    #[allow(dead_code)]
    stage_order: [PipelineStage; 6],
}

impl Pipeline {
    /// 从 config 构建 Hook 列表。
    pub fn build(config: &AnimConfig) -> Self {
        let hooks: Vec<Box<dyn PipelineHook>> = vec![
            Box::new(StaticSafetyHook),
            Box::new(LowAnchorCapHook),
            Box::new(OiSmoothingHook),
            Box::new(LeakyBucketIntakeHook),
            Box::new(MonotonyHook),
            Box::new(CrossDimCouplingHook),
            Box::new(ColdStartHook),
        ];
        // 按 config.hooks 过滤——只保留 enabled 的 hook
        let hooks: Vec<_> = hooks
            .into_iter()
            .filter(|h| hook_enabled(&config.hooks, h.name()))
            .collect();

        Pipeline {
            hooks,
            stage_order: [
                PipelineStage::AfterParse,
                PipelineStage::AfterTypeCheck,
                PipelineStage::AfterIntensityScale,
                PipelineStage::AfterPersonalize,
                PipelineStage::OnSessionStart,
                PipelineStage::OnSessionEnd,
            ],
        }
    }

    /// 执行指定阶段的所有已启用 hook。
    pub fn run_stage(
        &self,
        stage: PipelineStage,
        ctx: &Ctx,
    ) -> Result<(), AnimiError> {
        for hook in &self.hooks {
            if hook.stage() == stage {
                hook.run(ctx)?;
            }
        }
        Ok(())
    }

    /// 按顺序遍历所有阶段——等同显式调用每个 run_stage。
    /// 仅用于 invoke_all(AfterTypeCheck, AfterIntensityScale, AfterPersonalize) 等紧凑调用。
    pub fn invoke_all(
        &self,
        stages: &[PipelineStage],
        ctx: &Ctx,
    ) -> Result<(), AnimiError> {
        for &stage in stages {
            self.run_stage(stage, ctx)?;
        }
        Ok(())
    }
}

fn hook_enabled(cfg: &HooksConfig, name: &str) -> bool {
    match name {
        "static_safety" => cfg.static_safety.enabled,
        "low_anchor_cap" => cfg.low_anchor_cap.enabled,
        "leaky_bucket_intake" => cfg.leaky_bucket_intake.enabled,
        "monotony" => cfg.monotony.enabled,
        "cross_dim_coupling" => cfg.cross_dim_coupling.enabled,
        "cold_start" => cfg.cold_start.enabled,
        "oi_smoothing" => cfg.oi_smoothing.enabled,
        _ => true, // 未知 hook——默认启用
    }
}

// ═══════════════════════════════════════════════════════════════
// Hook 实现
// ═══════════════════════════════════════════════════════════════

// —— static_safety ——

struct StaticSafetyHook;
impl PipelineHook for StaticSafetyHook {
    fn name(&self) -> &str { "static_safety" }
    fn stage(&self) -> PipelineStage { PipelineStage::AfterTypeCheck }
    fn run(&self, ctx: &Ctx) -> Result<(), AnimiError> {
        let ast = ctx.ast.expect("static_safety: AST must be set");
        let reg = ctx.registry.expect("static_safety: registry must be set");
        crate::rule::check(ast, reg, ctx.config)
    }
}

// —— low_anchor_cap ——

struct LowAnchorCapHook;
impl PipelineHook for LowAnchorCapHook {
    fn name(&self) -> &str { "low_anchor_cap" }
    fn stage(&self) -> PipelineStage { PipelineStage::AfterIntensityScale }
    fn run(&self, _ctx: &Ctx) -> Result<(), AnimiError> {
        // low_anchor_cap 在 safety::check_with_scale / personalize 中已经生效。
        // 这里是钩子占位——实际逻辑在强度缩放路径中。
        Ok(())
    }
}

// —— oi_smoothing ——

struct OiSmoothingHook;
impl PipelineHook for OiSmoothingHook {
    fn name(&self) -> &str { "oi_smoothing" }
    fn stage(&self) -> PipelineStage { PipelineStage::AfterIntensityScale }
    fn run(&self, ctx: &Ctx) -> Result<(), AnimiError> {
        let ast = ctx.ast.expect("oi_smoothing: AST must be set");
        let smoothing = crate::guard::inject(ast, ctx.config)?;
        *ctx.smoothing_output.borrow_mut() = Some(smoothing);
        Ok(())
    }
}

// —— leaky_bucket_intake ——

struct LeakyBucketIntakeHook;
impl PipelineHook for LeakyBucketIntakeHook {
    fn name(&self) -> &str { "leaky_bucket_intake" }
    fn stage(&self) -> PipelineStage { PipelineStage::AfterIntensityScale }
    fn run(&self, ctx: &Ctx) -> Result<(), AnimiError> {
        // 离线 CLI 模式——无 tracker 状态——跳过。
        // 在线 Session 模式——tracker 由 main.rs 在 Session 启动时注入 ctx.tracker。
        if let Some(ref mut t) = *ctx.tracker.borrow_mut() {
            let scaled = ctx.scaled_intensity.borrow();
            if let Some(ref scaled) = *scaled {
                let now = monotonic_ns();
                // 使用主旋律维度（默认 Emotional——离线模式不确定维度）
                t.intake_and_verify(
                    scaled.max,
                    PbmDimension::Emotional,
                    &UserSafetyProfile::standard(),
                    now,
                )?;
            }
        }
        Ok(())
    }
}

// —— monotony ——

struct MonotonyHook;
impl PipelineHook for MonotonyHook {
    fn name(&self) -> &str { "monotony" }
    fn stage(&self) -> PipelineStage { PipelineStage::AfterPersonalize }
    fn run(&self, _ctx: &Ctx) -> Result<(), AnimiError> {
        // 离线 CLI 模式——无跨帧状态——跳过。
        // 在线 Session 模式——由 personalize() 内部的 monotony 检测处理。
        Ok(())
    }
}

// —— cross_dim_coupling ——

struct CrossDimCouplingHook;
impl PipelineHook for CrossDimCouplingHook {
    fn name(&self) -> &str { "cross_dim_coupling" }
    fn stage(&self) -> PipelineStage { PipelineStage::AfterPersonalize }
    fn run(&self, ctx: &Ctx) -> Result<(), AnimiError> {
        if let Some(ref t) = *ctx.tracker.borrow() {
            t.verify_cross_dimension(&UserSafetyProfile::standard())?;
        }
        Ok(())
    }
}

// —— cold_start ——

struct ColdStartHook;
impl PipelineHook for ColdStartHook {
    fn name(&self) -> &str { "cold_start" }
    fn stage(&self) -> PipelineStage { PipelineStage::OnSessionStart }
    fn run(&self, _ctx: &Ctx) -> Result<(), AnimiError> {
        // 离线 CLI 模式——无 Session 概念——跳过。
        // 在线 Session 模式——由 personalize() 内部的 ColdStartGuard 处理。
        Ok(())
    }
}
