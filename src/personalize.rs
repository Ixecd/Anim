// src/personalize.rs — Pass 6：FSIR × PBM → PSIR

use crate::error::AnimiError;
use crate::fsir::FsirDoc;
use crate::pbm::{ColdStartGuard, DampingMatrix, PbmDimension, SessionLabel, StepState};
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

// ── PBM 状态输入（收拢8个参数的复数 param） ───────────────────

/// PBM 状态——用于 `personalize()` 将 8 个参数收拢为 1 个。
pub struct PbmState<'a> {
    pub guard: &'a ColdStartGuard,
    pub damping_gradients: Option<&'a [(PbmDimension, f64); 4]>,
    pub session_label: SessionLabel,
    pub coeffs: PbmColdStartCoefficients,
    pub user_cap: u32,
    pub pbm_updated_at: &'a str,
}

// ── 主函数：personalize ──────────────────────────────────────

/// Pass 6 Personalize：FSIR × PBM → PSIR。
///
/// 参数已按结构体收拢——`PbmState` 携带全部 PBM 状态入参。
pub fn personalize(
    fsir: &FsirDoc,
    registry: &Registry,
    pbm: &PbmState,
) -> Result<PsirDoc, AnimiError> {
    // ── 1. 冷启动判定 ──────────────────────────────────────
    let cold_start = pbm.guard.is_cold_start();
    let session_count = pbm.guard.session_count;

    // ── 2. 阻尼矩阵判定 ──────────────────────────────────────
    // v0.3: 无传感器数据 → damping_gradients = None → 阻尼关闭。
    let frozen: HashMap<PbmDimension, StepState> = match pbm.damping_gradients {
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
    let atom_dimension = |atom_name: &str| -> PbmDimension {
        registry
            .lookup(atom_name)
            .map(|e| e.dimension)
            .unwrap_or(PbmDimension::Emotional)
    };
    let baseline_coeff = match atom_dimension(&fsir.mix.main) {
        PbmDimension::Visceral => pbm.coeffs.visceral,
        PbmDimension::Emotional => pbm.coeffs.emotional,
        PbmDimension::Tactile => pbm.coeffs.tactile,
        PbmDimension::Auditory => pbm.coeffs.auditory,
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
        let ratio_cap = registry.max_ratio(&acc.atom).unwrap_or(0.30);
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
    let applied_min = sigmoidal_scale(fsir.intensity.min, baseline_coeff, pbm.user_cap);
    let applied_max = sigmoidal_scale(fsir.intensity.max, baseline_coeff, pbm.user_cap);

    // 强度上限二次校验
    if applied_max > pbm.user_cap {
        return Err(AnimiError::UserStateSafetyError {
            file_name: String::new(),
            cap: format!("{}", pbm.user_cap),
            atom_name: fsir.name.clone(),
            reason: format!(
                "个人校准后强度 {} 超过用户上限 {}（FSIR 强度范围 {}-{}）",
                applied_max, pbm.user_cap, fsir.intensity.min, fsir.intensity.max,
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
            cap: pbm.user_cap,
        },
        PsirMetaInput {
            smoothing,
            cold_start,
            trauma_rerouted,
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
        let registry = Registry::default();
        let pbm = PbmState {
            guard: &ColdStartGuard::default(),
            damping_gradients: None,
            session_label: SessionLabel::ColdStart,
            coeffs: PbmColdStartCoefficients::default(),
            user_cap: 100,
            pbm_updated_at: "2026-06-09T10:00:00Z",
        };
        let psir = personalize(&fsir, &registry, &pbm).expect("personalize failed");
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
