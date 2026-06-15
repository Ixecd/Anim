// src/personalize.rs — Pass 6：FSIR × PBM → PSIR

use crate::error::{AnimiError, Severity};
use crate::fsir::FsirDoc;
use crate::pbm::{
    ColdStartGuard, DampingMatrix, DampingState, PbmDimension, SessionLabel, StepState,
};
use crate::psir::{
    PersonalizedAccent, PersonalizedFeeling, PsirDecayStep, PsirDoc, PsirFeelingInput,
    PsirIntensityInput, PsirMetaInput, PsirSmoothing,
};
use crate::registry::Registry;
use std::collections::HashMap;

// ── 冷启动四维系数 ──────────────────────────────────────────

/// PBM 四维差异化冷启动基线偏移系数。
///
/// 来自 Feelings-ROADMAP §1.2：
///   内脏 0.75 / 情绪 0.40 / 触觉 0.80 / 听觉 0.85
///
/// 系数 < 1.0 表示该维度的感受触发基线低于通用模板——
/// 新用户需要更强的信号才能达到同样的感受强度。
#[derive(Debug, Clone, Copy)]
pub struct PbmColdStartCoefficients {
    pub visceral: f64,
    pub emotional: f64,
    pub tactile: f64,
    pub auditory: f64,
}

impl Default for PbmColdStartCoefficients {
    fn default() -> Self {
        PbmColdStartCoefficients {
            visceral: 0.75,
            emotional: 0.40,
            tactile: 0.80,
            auditory: 0.85,
        }
    }
}

// ── Sigmoidal 非线性缩放 ─────────────────────────────────────

/// 强度 sigmoidal 非线性缩放。
///
/// ADR 003 §6.2:
///   - Low intensity (0-30%): ≈ linear
///   - Mid intensity (30-70%): response slows
///   - High intensity (70-100%): near saturation
///
/// 公式（详见 docs/design/009-math-and-constraints.md §一）:
///   x = original / cap
///   σ(x) = 1/(1+e^{-k(x-x0)}), k=6, x0=0.5
///   compression(x) = 1 − α×σ(x), α=0.5
///   applied = original × baseline_coeff × compression(x)
///
/// 低强度 compression≈0.97 → ≈linear; 中compression=0.75 → 压缩;
/// 高强度 compression≈0.52 → 饱和。
pub fn sigmoidal_scale(
    original: u32,
    baseline_coeff: f64,
    cap: u32,
    cfg: &crate::config::SigmoidalConfig,
) -> u32 {
    if cap == 0 {
        return 0;
    }
    let x = original as f64 / cap as f64;
    let k = cfg.k;
    let x0 = cfg.x0;
    let alpha = cfg.compression_alpha;
    let sigmoid = 1.0 / (1.0 + (-k * (x - x0)).exp());
    let compression = 1.0 - alpha * sigmoid;
    let scaled = original as f64 * baseline_coeff * compression;
    let applied = scaled.round() as u32;
    applied.min(cap)
}

// ── PBM 状态输入（收拢8个参数的复数 param） ───────────────────

/// PBM 状态——用于 `personalize()` 将 8 个参数收拢为 1 个。
pub struct PbmState<'a> {
    pub guard: &'a ColdStartGuard,
    /// 当前帧的四维传感器梯度（None = 传感器信号缺失）。
    pub damping_gradients: Option<&'a [(PbmDimension, f64); 4]>,
    /// 上一帧的冻结状态——当 damping_gradients=None 时使用 (Damping Hold)。
    /// None = 无历史状态（首帧或长时间无信号→安全降级为零阻尼）。
    pub previous_frozen: Option<&'a HashMap<PbmDimension, StepState>>,
    /// 冷启动后阻尼淡入窗长度（Session 数）。10 + window 后全额阻尼。
    /// 0 = 无窗口（立即全额阻尼）。默认 5。
    pub cold_start_window: u32,
    pub session_label: SessionLabel,
    pub coeffs: PbmColdStartCoefficients,
    pub user_cap: u32,
    pub pbm_updated_at: &'a str,

    // ── v0.4: Core 下行的运行时决策字段 ──
    //
    // 当前由 Anim 在 Session 启动时从 PBM 档案（或默认值）读取——
    // 长期由 Feelings-Core 在设备本地提供。
    /// 防御激活层级——来自 PBM 内部状态机。
    /// None = 普通用户——无防御激活。
    /// D1/D2/D3 = 等控器检测到对应层级的防御需求——
    ///   非叙事驱动——只由等控器 VSA 相变触发。
    pub defence_level: Option<crate::pbm::DefenceLevel>,

    /// 锚点置信度 R (0.0~1.0)——来自 Core PBM。
    /// R < 0.3 → cap 硬上限 20（low_anchor_cap 保护）。
    /// None = 无数据——不触发低锚保护（向后兼容 v0.3）。
    pub anchor_confidence: Option<f64>,

    // ── v0.4: 阻尼实时梯度 ──
    //
    /// 阻尼状态——持有四维步长乘数和上一帧 PBM 快照。
    /// 冷启动期后——Anim 用此状态计算真实梯度——不再使用硬编码 1.0。
    /// None = 冷启动期——阻尼由 freeze_factor 完全关闭。
    pub damping_state: Option<&'a DampingState>,

    /// 当前帧的四维 PBM 偏移值——[Visceral, Emotional, Tactile, Auditory]。
    /// 用于 DampingState 的梯度计算。
    /// None = 无实时 PBM 数据（离线编译或冷启动）——降级为 Damping Hold。
    pub current_pbm_values: Option<&'a [f64; 4]>,

    // ── v1.1: ADR 011/012 漏桶 + 脱敏 ──
    //
    /// 用户安全档案——四维 LeakRate + CriticalThreshold。
    /// Session 启动时由 Core 快照注入。None = 标准健康默认。
    pub safety_profile: Option<&'a crate::safety::UserSafetyProfile>,

    /// 上一帧四维强度快照——用于信号变异度检测（ADR 012 脱敏看门狗）。
    /// None = 首帧——无历史——不做变异度检测。
    pub previous_intensities: Option<&'a [u32; 4]>,

    /// 脱敏检测连续 N 帧计数器——按维度。
    pub monotony_counters: Option<&'a [u32; 4]>,

    /// 信号节律模板——ADR 012。Core 强度调度器下发。
    /// None = 无节律限制（默认——当前 Core 零代码）。
    pub rhythm_template: Option<crate::safety::RhythmTemplate>,

    /// 运行时配置——所有可调参数。Session 启动时由 main.rs 注入。
    pub config: &'a crate::config::AnimConfig,
}

// ── 主函数：personalize ──────────────────────────────────────

/// Pass 6 Personalize：FSIR × PBM → PSIR。
///
/// 参数已按结构体收拢——`PbmState` 携带全部 PBM 状态入参。
pub fn personalize(
    fsir: &FsirDoc,
    registry: &Registry,
    pbm: &PbmState,
    tracker: Option<&mut crate::safety::NeuroEnergyTracker>,
) -> Result<PsirDoc, AnimiError> {
    // ── 1. 冷启动判定 ──────────────────────────────────────
    let cold_start = pbm.guard.is_cold_start();
    let session_count = pbm.guard.session_count;

    // ── 1a. 低锚点置信度 cap 硬上限（v0.4 P1 #28） ────────────
    // 从 Core PBM 的 anchor_confidence (R) 判定是否需要 low_anchor_cap 保护。
    // R < 0.3 → effective_cap = min(20, user_cap)。不是惩罚——是"你还没准备好——我先替你守着"。
    let effective_cap =
        crate::safety::low_anchor_cap(pbm.user_cap, pbm.anchor_confidence, pbm.config);

    // ── 2. 阻尼矩阵判定 ──────────────────────────────────────
    // v0.4: 步长来源——DampingState → 硬编码降级。
    //       梯度来源——(1) 外部传感器注入 (2) DampingState 实时计算 (3) Damping Hold。
    //       冷启动期——阻尼由 is_frozen 的 cold_start 门控完全关闭。

    fn default_steps() -> [(PbmDimension, f64); 4] {
        [
            (PbmDimension::Visceral, 1.0),
            (PbmDimension::Emotional, 1.0),
            (PbmDimension::Tactile, 1.0),
            (PbmDimension::Auditory, 1.0),
        ]
    }

    let current_steps = pbm
        .damping_state
        .map(|ds| ds.current_steps())
        .unwrap_or_else(default_steps);

    let frozen: HashMap<PbmDimension, StepState> = match pbm.damping_gradients {
        // 优先级一：外部注入梯度（传感器直通）
        Some(gradients) => DampingMatrix::apply(gradients, &current_steps, &pbm.config.damping),

        // 优先级二：DampingState 实时计算 → 降级 Damping Hold
        None => match (pbm.damping_state, pbm.current_pbm_values) {
            (Some(ds), Some(vals)) => match ds.compute_gradients(vals) {
                Some(gradients) => DampingMatrix::apply(&gradients, &current_steps, &pbm.config.damping),
                None => pbm.previous_frozen.cloned().unwrap_or_default(),
            },
            _ => pbm.previous_frozen.cloned().unwrap_or_default(),
        },
    };
    let is_frozen = |dim: PbmDimension| -> bool {
        !cold_start
            && frozen
                .get(&dim)
                .map(|s| matches!(s, StepState::Frozen { .. }))
                .unwrap_or(false)
    };

    // ── 2b. 冷启动阻尼淡入窗（P1 #29） ─────────────────────────
    // v0.4: 冷启动结束后（Session 10+），阻尼从 0% 线性过渡到 100%。
    //   α = min(1.0, (session_count - 10) / window)
    //   damping_reduction = 0.5 × α
    //   freeze_factor = 1.0 - damping_reduction (1.0 → 0.5)
    let damping_window_alpha = if pbm.cold_start_window > 0 {
        let s = session_count.saturating_sub(10);
        let w = pbm.cold_start_window as f64;
        (s as f64 / w).min(1.0)
    } else {
        1.0
    };
    let freeze_factor = |is_damped: bool| -> f64 {
        if is_damped {
            1.0 - 0.5 * damping_window_alpha
        } else {
            1.0
        }
    };

    // ── 3. 四维偏移——按原子主维度选择系数 ──────────────────────
    let atom_dimension = |atom_name: &str| -> PbmDimension {
        registry
            .lookup(atom_name)
            .map(|e| e.dimension)
            .unwrap_or(PbmDimension::Emotional)
    };

    // ── 4. 主旋律——按自身维度判定阻尼 ────────────────────────
    let main_dim = atom_dimension(&fsir.mix.main);
    let damped_main = is_frozen(main_dim);
    let baseline_coeff = match main_dim {
        PbmDimension::Visceral => pbm.coeffs.visceral,
        PbmDimension::Emotional => pbm.coeffs.emotional,
        PbmDimension::Tactile => pbm.coeffs.tactile,
        PbmDimension::Auditory => pbm.coeffs.auditory,
    };

    // ── 5. 主旋律——FSIR 原子名 → 个人偏移 → 个人参数 ──────────
    let main = PersonalizedFeeling {
        atom: fsir.mix.main.clone(),
        baseline_offset: baseline_coeff,
        damped: damped_main,
    };

    // ── 6. 点缀——遍历FSIR点缀 + 比例帽校验 + 缩放 ─────────────
    let mut accents = Vec::new();
    for acc in &fsir.mix.accents {
        // v0.4: 按点缀自身的维度判定阻尼 (P1 #27 fix)
        let accent_dim = atom_dimension(&acc.atom);
        let damped = is_frozen(accent_dim);
        // v0.3: 从 Registry 读取每原子独立比例帽，fallback 0.30
        let ratio_cap = registry.max_ratio(&acc.atom).unwrap_or(0.30);
        let applied_ratio = (acc.ratio.min(ratio_cap)) * freeze_factor(damped);
        accents.push(PersonalizedAccent {
            atom: acc.atom.clone(),
            original_ratio: acc.ratio,
            applied_ratio,
            ratio_cap,
            damped,
        });
    }

    // ── 7. 强度 sigmoidal 缩放 ─────────────────────────────
    let applied_min = sigmoidal_scale(
        fsir.intensity.min,
        baseline_coeff,
        effective_cap,
        &pbm.config.sigmoidal,
    );
    let applied_max = sigmoidal_scale(
        fsir.intensity.max,
        baseline_coeff,
        effective_cap,
        &pbm.config.sigmoidal,
    );

    // 强度上限二次校验
    if applied_max > effective_cap {
        return Err(AnimiError::UserStateSafetyError { severity: Severity::Deny,
            file_name: String::new(),
            cap: format!("{}", effective_cap),
            atom_name: fsir.name.clone(),
            reason: format!(
                "个人校准后强度 {} 超过用户上限 {}（FSIR 强度范围 {}-{}）",
                applied_max, effective_cap, fsir.intensity.min, fsir.intensity.max,
            ),
        });
    }

    // ── 7. 防御激活判定 ─────── v0.4: PBM 状态机驱动 ──────
    // 从 PBM 的 defence_level 推导是否激活防御缩放路径。
    // None = 等控器当前不需要防御缩放。
    // D1/D2/D3 = 对应层级的信号安全约束。
    let defence_activated = pbm.defence_level.is_some();

    // ── 8. 平滑过渡（pass-through）────────────────────
    let smoothing = fsir.smoothing.as_ref().map(|s| PsirSmoothing {
        steps: s
            .steps
            .iter()
            .map(|step| PsirDecayStep {
                multiplier: step.multiplier,
            })
            .collect(),
    });

    // ── 8b. ADR 011 漏桶——时域能量追踪 ──────────────────
    // 在 PSIR 生成后、return Ok 前——调用 tracker.intake_and_verify。
    // 通过 → 继续。不通过 → Err(SafetyBreach) 向上传播。
    let profile = pbm
        .safety_profile
        .copied()
        .unwrap_or_else(crate::safety::UserSafetyProfile::standard);
    if let Some(t) = tracker {
        let now_ns = crate::safety::monotonic_ns();
        t.intake_and_verify(applied_max, main_dim, &profile, now_ns)?;
    }

    // ── 8c. ADR 012 信号变异度检测（脱敏看门狗）────────────
    // 比较当前帧强度与上一帧同维度强度——滑动平均后连续 N 帧变化 < δ → 置 degraded。
    let mut degraded = false;
    if let (Some(prev_ints), Some(counters)) = (pbm.previous_intensities, pbm.monotony_counters) {
        let idx = crate::safety::dim_index(main_dim);
        let prev = prev_ints[idx];
        let curr = applied_max;
        let delta = (curr as i64 - prev as i64).unsigned_abs();
        // δ_threshold = 2（默认——变化幅度 < 2 强度分视为停滞）
        let delta_threshold = pbm.config.monotony.delta_threshold;
        // N 帧窗口（默认 500 帧 = 500ms）
        let monotony_n = pbm.config.monotony.detection_frames;

        if delta < delta_threshold.into() {
            let new_count = counters[idx].saturating_add(1);
            if new_count >= monotony_n {
                degraded = true;
            }
        }
        // 注：到达稳态阈值前 counters 值需要被写入以追踪跨帧状态。
        // v1.1——degraded 标志写入 PSIR，不上报 Core（Core 零代码）。
        // v1.2+——Core 读取 degraded → 在下一帧强度调度时插入恢复帧。
    }

    // ── 9. 组装 PSIR ─────────────────────────────────────
    Ok(PsirDoc::new(
        PsirFeelingInput {
            name: fsir.name.clone(),
            main,
            accents,
            shape_name: fsir.shape.name.clone(),
        },
        PsirIntensityInput {
            original_min: fsir.intensity.min,
            original_max: fsir.intensity.max,
            applied_min,
            applied_max,
            cap: effective_cap,
        },
        PsirMetaInput {
            smoothing,
            cold_start,
            defence_activated,
            defence_level: pbm.defence_level,
            degraded,
            pbm_updated_at: pbm.pbm_updated_at.to_string(),
            session_count,
        },
    ))
}

// ── tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsir::{FsirAccent, FsirIntensity, FsirMeta, FsirMix, FsirShape};

    fn make_fsir(
        name: &str,
        main: &str,
        accents_data: Vec<(&str, f64)>,
        min: u32,
        max: u32,
    ) -> FsirDoc {
        let accents = accents_data
            .into_iter()
            .map(|(a, r)| FsirAccent {
                atom: a.to_string(),
                ratio: r,
            })
            .collect();
        FsirDoc {
            meta: FsirMeta {
                animi_version: "0.3.0".to_string(),
                compiled_at: "2026-06-09T10:00:00Z".to_string(),
                source_hash: None,
                pattern_registry_hash: None,
                safety_rules_version: None,
            },
            name: name.to_string(),
            mix: FsirMix {
                main: main.to_string(),
                accents,
            },
            shape: FsirShape {
                name: "gradual_rise_fall".to_string(),
            },
            intensity: FsirIntensity { min, max },
            smoothing: None,
        }
    }

    #[test]
    fn sigmoidal_zero_intensity() {
        let scaled = sigmoidal_scale(0, 1.0, 100, &crate::config::SigmoidalConfig::default());
        assert_eq!(scaled, 0);
    }

    #[test]
    fn sigmoidal_low_linear() {
        let cap = 100;
        let baseline = 1.0;
        // 10% → compression≈0.96 → applied≈9.6 → 10
        let applied = sigmoidal_scale(
            10,
            baseline,
            cap,
            &crate::config::SigmoidalConfig::default(),
        );
        assert_eq!(applied, 10);
    }

    #[test]
    fn sigmoidal_mid_compression() {
        let cap = 100;
        let baseline = 1.0;
        // 50% → compression=0.75 → applied≈37.5 → 38
        let applied = sigmoidal_scale(
            cap / 2,
            baseline,
            cap,
            &crate::config::SigmoidalConfig::default(),
        );
        assert_eq!(applied, 38);
    }

    #[test]
    fn sigmoidal_high_saturation() {
        let cap = 100;
        let baseline = 1.0;
        // 100% → compression≈0.52 → applied≈52
        let applied = sigmoidal_scale(
            100,
            baseline,
            cap,
            &crate::config::SigmoidalConfig::default(),
        );
        assert_eq!(applied, 52);
    }

    #[test]
    fn sigmoidal_respects_cap() {
        let applied = sigmoidal_scale(200, 1.0, 100, &crate::config::SigmoidalConfig::default());
        assert!(applied <= 100);
    }

    #[test]
    fn basic_personalize() {
        let fsir = make_fsir("calm", "calm_meditative", vec![("belonging", 0.3)], 15, 45);
        let registry = Registry::default();
        let pbm = PbmState {
            guard: &ColdStartGuard::default(),
            damping_gradients: None,
            previous_frozen: None,
            cold_start_window: 5,
            session_label: SessionLabel::ColdStart,
            coeffs: PbmColdStartCoefficients::default(),
            user_cap: 100,
            pbm_updated_at: "2026-06-09T10:00:00Z",
            defence_level: None,
            anchor_confidence: None,
            damping_state: None,
            current_pbm_values: None,
            safety_profile: None,
            previous_intensities: None,
            monotony_counters: None,
            rhythm_template: None,
            config: &crate::config::AnimConfig::default(),
        };
        let psir = personalize(&fsir, &registry, &pbm, None).expect("personalize failed");
        assert_eq!(psir.name, "calm");
        assert!(psir.cold_start);
        assert!(!psir.defence_activated);
        assert!(psir.defence_level.is_none());
        assert_eq!(psir.main.atom, "calm_meditative");
        assert!((psir.main.baseline_offset - 0.40).abs() < 0.001); // emotional
        assert!(!psir.main.damped); // cold_start → 不冻结
        assert_eq!(psir.intensity.cap, 100);
        assert!(psir.intensity.applied_max <= 100);
        assert_eq!(psir.accents.len(), 1);
        assert_eq!(psir.accents[0].atom, "belonging");
        let accent = &psir.accents[0];
        assert!(accent.applied_ratio <= accent.ratio_cap);
        assert!(!accent.damped); // cold_start → 不冻结
    }
}
