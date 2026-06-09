// src/personalize.rs — Pass 6：FSIR × PBM → PSIR

use crate::fsir::FsirDoc;
use crate::pbm::{ColdStartGuard, DampingMatrix, PbmDimension, SessionLabel, StepState};
use crate::psir::{
    PersonalizedAccent, PersonalizedFeeling, PsirDecayStep, PsirDoc,
    PsirSmoothing,
};
use crate::registry::Registry;
use crate::error::AnimiError;
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
pub fn sigmoidal_scale(original: u32, baseline_coeff: f64, cap: u32) -> u32 {
    if cap == 0 {
        return 0;
    }
    let x = original as f64 / cap as f64;
    let k: f64 = 6.0;
    let x0: f64 = 0.5;
    let alpha: f64 = 0.5;
    let sigmoid = 1.0 / (1.0 + (-k * (x - x0)).exp());
    let compression = 1.0 - alpha * sigmoid;
    let scaled = original as f64 * baseline_coeff * compression;
    let applied = scaled.round() as u32;
    applied.min(cap)
}

// ── 主函数：personalize ──────────────────────────────────────

/// Pass 6 Personalize：FSIR × PBM → PSIR。
///
/// # 参数
/// - `fsir`: 待个性化的通用感受结构
/// - `guard`: 冷启动守卫——当前用户的 Session 计数状态
/// - `damping_gradients`: 四维度实测梯度（v0.3: None = 无传感器数据, 阻尼关闭）
/// - `session_label`: 本次 Session 的标记——ColdStart/Normal/Abnormal
/// - `coeffs`: PBM 四维冷启动基线系数
/// - `user_cap`: 用户的强度上限
/// - `pbm_updated_at`: PBM 最后更新时间（RFC 3339）
/// - `registry`: Pattern Registry——查询原子主维度和独立比例帽
///
/// # 返回
/// - 成功：PsirDoc
/// - 失败：AnimiError（强度越界/沙箱违规/创伤路径）
pub fn personalize(
    fsir: &FsirDoc,
    guard: &ColdStartGuard,
    damping_gradients: Option<&[(PbmDimension, f64); 4]>,
    _session_label: SessionLabel,
    coeffs: PbmColdStartCoefficients,
    user_cap: u32,
    pbm_updated_at: &str,
    registry: &Registry,
) -> Result<PsirDoc, AnimiError> {
    // ── 1. 冷启动判定 ──────────────────────────────────────
    let cold_start = guard.is_cold_start();
    let session_count = guard.session_count;

    // ── 2. 阻尼矩阵判定 ──────────────────────────────────────
    // v0.3: 无传感器数据 → damping_gradients = None → 阻尼关闭。
    // v0.4+: 传入四维度实测梯度 → DampingMatrix::apply() 计算冻结维度。
    let frozen: HashMap<PbmDimension, StepState> = match damping_gradients {
        Some(gradients) => {
            let default_steps = [
                (PbmDimension::Visceral, 1.0),
                (PbmDimension::Emotional, 1.0),
                (PbmDimension::Tactile, 1.0),
                (PbmDimension::Auditory, 1.0),
            ];
            DampingMatrix::apply(gradients, &default_steps)
        }
        None => HashMap::new(),
    };
    let is_frozen = |dim: PbmDimension| -> bool {
        !cold_start
            && frozen
                .get(&dim)
                .map(|s| matches!(s, StepState::Frozen { .. }))
                .unwrap_or(false)
    };
    let damped_main = is_frozen(PbmDimension::Emotional);
    let damped_accent = is_frozen(PbmDimension::Tactile);

    // ── 3. 四维偏移——按原子主维度选择系数 ──────────────────────
    // v0.3: 从 Registry 查询原子的主维度，对号选用四维系数。
    // 未被 Registry 收录的原子——default Emotional (0.40)。
    let atom_dimension = |atom_name: &str| -> PbmDimension {
        registry
            .lookup(atom_name)
            .map(|e| e.dimension)
            .unwrap_or(PbmDimension::Emotional)
    };
    let baseline_coeff = match atom_dimension(&fsir.mix.main) {
        PbmDimension::Visceral => coeffs.visceral,
        PbmDimension::Emotional => coeffs.emotional,
        PbmDimension::Tactile => coeffs.tactile,
        PbmDimension::Auditory => coeffs.auditory,
    };

    // ── 4. 主旋律——FSIR 原子名 → 个人偏移 → 个人参数 ──────────
    let main = PersonalizedFeeling {
        atom: fsir.mix.main.clone(),
        baseline_offset: baseline_coeff,
        damped: damped_main,
    };

    // ── 5. 点缀——遍历FSIR点缀 + 比例帽校验 + 缩放 ─────────────
    let mut accents = Vec::new();
    for acc in &fsir.mix.accents {
        // v0.3: 从 Registry 读取每原子独立比例帽，fallback 0.30
        let ratio_cap = registry
            .max_ratio(&acc.atom)
            .unwrap_or(0.30);
        let applied_ratio = if damped_accent {
            (acc.ratio / 2.0).min(ratio_cap)
        } else {
            acc.ratio.min(ratio_cap)
        };
        accents.push(PersonalizedAccent {
            atom: acc.atom.clone(),
            original_ratio: acc.ratio,
            applied_ratio,
            ratio_cap,
            damped: damped_accent,
        });
    }

    // ── 6. 强度 sigmoidal 缩放 ─────────────────────────────
    let applied_min = sigmoidal_scale(fsir.intensity.min, baseline_coeff, user_cap);
    let applied_max = sigmoidal_scale(fsir.intensity.max, baseline_coeff, user_cap);

    // 强度上限二次校验
    if applied_max > user_cap {
        return Err(AnimiError::UserStateSafetyError {
            file_name: String::new(),
            cap: format!("{}", user_cap),
            atom_name: fsir.name.clone(),
            reason: format!(
                "个人校准后强度 {} 超过用户上限 {}（FSIR 强度范围 {}-{}）",
                applied_max,
                user_cap,
                fsir.intensity.min,
                fsir.intensity.max,
            ),
        });
    }

    // ── 7. 创伤路径重定向 ─────── v0.3 暂时不实现 ──────
    let trauma_rerouted = false;

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

    // ── 9. 组装 PSIR ─────────────────────────────────────
    Ok(PsirDoc::new(
        fsir.name.clone(),
        main,
        accents,
        fsir.shape.name.clone(),
        fsir.intensity.min,
        fsir.intensity.max,
        applied_min,
        applied_max,
        user_cap,
        smoothing,
        cold_start,
        trauma_rerouted,
        pbm_updated_at.to_string(),
        session_count,
    ))
}

// ── tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsir::{
        FsirAccent, FsirIntensity, FsirMeta, FsirMix, FsirShape,
    };

    fn make_fsir(name: &str, main: &str, accents_data: Vec<(&str, f64)>, min: u32, max: u32) -> FsirDoc {
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
        let scaled = sigmoidal_scale(0, 1.0, 100);
        assert_eq!(scaled, 0);
    }

    #[test]
    fn sigmoidal_low_linear() {
        let cap = 100;
        let baseline = 1.0;
        // 10% → compression≈0.96 → applied≈9.6 → 10
        let applied = sigmoidal_scale(10, baseline, cap);
        assert_eq!(applied, 10);
    }

    #[test]
    fn sigmoidal_mid_compression() {
        let cap = 100;
        let baseline = 1.0;
        // 50% → compression=0.75 → applied≈37.5 → 38
        let applied = sigmoidal_scale(cap / 2, baseline, cap);
        assert_eq!(applied, 38);
    }

    #[test]
    fn sigmoidal_high_saturation() {
        let cap = 100;
        let baseline = 1.0;
        // 100% → compression≈0.52 → applied≈52
        let applied = sigmoidal_scale(100, baseline, cap);
        assert_eq!(applied, 52);
    }

    #[test]
    fn sigmoidal_respects_cap() {
        let applied = sigmoidal_scale(200, 1.0, 100);
        assert!(applied <= 100);
    }

    #[test]
    fn basic_personalize() {
        let fsir = make_fsir("calm", "calm_meditative", vec![("belonging", 0.3)], 15, 45);
        let guard = ColdStartGuard::default(); // session_count = 0
        let coeffs = PbmColdStartCoefficients::default();
        let registry = Registry::default();
        let psir = personalize(
            &fsir,
            &guard,
            None, // v0.3: 无传感器数据
            SessionLabel::ColdStart,
            coeffs,
            100,
            "2026-06-09T10:00:00Z",
            &registry,
        )
        .expect("personalize failed");
        assert_eq!(psir.name, "calm");
        assert!(psir.cold_start);
        assert!(!psir.trauma_rerouted);
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
