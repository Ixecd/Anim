// src/sandbox.rs — Sandbox routing + Governance engine —— ADR 009 v0.5 §十
//
// 沙箱 = 强度阈值触发的执行环境——与原子分类解耦。
// intensity.max >= 90 → 自动进沙箱 → Governance 接管。
// Governance = 检测 → 告知 → 干预（非静默镇压）。
//
// 沙箱响应策略（原子级别）：
//   Attainment — 成就/高峰体验，90+是设计目标，只监控不限制
//   Neutral    — 中性，按强度叠加比例分级响应（G1/G2/G3）
//   Caution    — 需谨慎，无条件 G2 降级
//   Shield     — 必须防护，直接 G3 熔断

use crate::error::AnimiError;
use crate::error::Severity;
use crate::oi;
use crate::oiw;
use crate::pbm::DefenceLevel;
use crate::registry::SandboxResponse;

pub const SANDBOX_THRESHOLD: u32 = 90;
pub const ACCENT_ABSOLUTE_CAP: u32 = 30;
pub const COMBINED_OVERAGE_G1_RATIO: f64 = 1.10;
pub const COMBINED_OVERAGE_G3_RATIO: f64 = 1.30;
pub const SANDBOX_ACCENT_WEIGHT: f64 = 1.5;

// ── 路由判定 ──────────────────────────────────────────────────

pub fn is_sandbox(intensity_max: u32) -> bool {
    intensity_max >= SANDBOX_THRESHOLD
}

// ── 防御敏感系数 ──────────────────────────────────────────────

pub fn d_sensitivity(defence_level: Option<DefenceLevel>) -> f64 {
    match defence_level {
        None => 1.0,
        Some(DefenceLevel::D1) => 1.3,
        Some(DefenceLevel::D2) => 1.8,
        Some(DefenceLevel::D3) => 2.5,
    }
}

// ── 沙箱响应聚合 ──────────────────────────────────────────────

/// 取最严格的治理响应级别，从一组原子类型中决定实际响应策略。
///
/// Shield > Caution > Neutral > Attainment
pub fn worst_response(responses: &[SandboxResponse]) -> SandboxResponse {
    let mut worst = SandboxResponse::Attainment;
    for &r in responses {
        if r as u8 > worst as u8 {
            worst = r;
        }
    }
    worst
}

// ── Accent 强度贡献权重 ───────────────────────────────────────

pub fn accent_intensity_weight(_atom_name: &str) -> f64 {
    SANDBOX_ACCENT_WEIGHT
}

// ── 核心+点缀叠加总强度校验（P0 #3）───────────────────────────

/// 校验主旋律 + 点缀叠加总强度是否超过 effective_cap。
///
/// `response`: 所有参与原子的最严格 SandboxResponse（主旋律 + 点缀的最坏情况）。
///
/// 公式：combined = main_max × (1 + Σ accent_ratio × accent_weight)
///
/// 分层治理：
///   Shield     → G3 无条件熔断（物理矛盾——e.g. safety/deep_rest 到 90+）
///   Caution    → G2 无条件降级（不应该需要 90+——e.g. gentle_focus）
///   Attainment → 安静通过——只监控不限制（e.g. post_achievement）
///   Neutral    → 按强度叠加比例：G1告知 / G2降级 / G3熔断
///                + DefenceLevel 升级（D1→G2, D2/D3→G3）
pub fn check_combined(
    applied_main_max: u32,
    accent_ratios: &[(String, f64)],
    effective_cap: u32,
    response: SandboxResponse,
    defence_level: Option<DefenceLevel>,
) -> Result<(), AnimiError> {
    let cap = effective_cap as f64;
    let sum_ratios: f64 = accent_ratios
        .iter()
        .map(|(name, ratio)| ratio * accent_intensity_weight(name))
        .sum();
    let combined = applied_main_max as f64 * (1.0 + sum_ratios);

    // D2+ 无条件熔断（无论原子响应类型）
    if matches!(defence_level, Some(DefenceLevel::D2 | DefenceLevel::D3)) {
        return Err(AnimiError::SafetyBreach {
            severity: Severity::Deny,
            file_name: crate::error::current_file(),
            dimension: crate::pbm::PbmDimension::Emotional,
            current_energy: combined,
            threshold: cap,
        });
    }

    // 原子级响应策略
    match response {
        SandboxResponse::Shield => Err(AnimiError::SafetyBreach {
            severity: Severity::Deny,
            file_name: crate::error::current_file(),
            dimension: crate::pbm::PbmDimension::Emotional,
            current_energy: combined,
            threshold: cap,
        }),
        SandboxResponse::Caution => {
            oi!(
                StaticSafetyError,
                rule = "沙箱谨慎原子(G2降级)".into(),
                detail = format!(
                    "本感受包含需谨慎对待的原子——90+ 强度触发自动降级。已降级至 cap {}。",
                    effective_cap
                )
            )
        }
        SandboxResponse::Attainment => {
            // 成就/高峰体验——只记录，不限
            Ok(())
        }
        SandboxResponse::Neutral => {
            // 按强度叠加比例分级
            if combined <= cap && !matches!(defence_level, Some(DefenceLevel::D1)) {
                return Ok(());
            }

            let overage = combined / cap;

            if overage > COMBINED_OVERAGE_G3_RATIO {
                return Err(AnimiError::SafetyBreach {
                    severity: Severity::Deny,
                    file_name: crate::error::current_file(),
                    dimension: crate::pbm::PbmDimension::Emotional,
                    current_energy: combined,
                    threshold: cap * COMBINED_OVERAGE_G3_RATIO,
                });
            }

            // D1 升级 → G2
            if matches!(defence_level, Some(DefenceLevel::D1))
                || overage > COMBINED_OVERAGE_G1_RATIO
            {
                oi!(
                    StaticSafetyError,
                    rule = "沙箱强度叠加(G2降级)".into(),
                    detail = format!(
                        "强度叠加超限：主旋律 {} + 点缀累积 {} > cap {}。已自动降级至 cap。",
                        applied_main_max,
                        (combined - applied_main_max as f64).round() as u32,
                        effective_cap
                    )
                )
            }

            // G1: 超限 ≤ 10% 且无 D1
            oiw!(
                StaticSafetyError,
                rule = "沙箱强度叠加(G1告知)".into(),
                detail = format!(
                    "强度叠加超限：主旋律 {} + 点缀累积 {} > cap {}（超限 {:.0}%，未触发降级）。",
                    applied_main_max,
                    (combined - applied_main_max as f64).round() as u32,
                    effective_cap,
                    (overage - 1.0) * 100.0
                )
            )
        }
    }
}

// ── 点缀配比绝对强度校验（P0 #4）───────────────────────────────

/// 校验单个点缀的绝对强度是否超过上限。
///
/// 公式：absolute = ratio × source_intensity_max × d_sensitivity(defence_level)
///
/// Attainment → 安静通过。Shield → 无条件 G3。
pub fn check_accent_absolute(
    ratio: f64,
    source_intensity_max: u32,
    response: SandboxResponse,
    defence_level: Option<DefenceLevel>,
) -> Result<(), AnimiError> {
    if matches!(response, SandboxResponse::Attainment) {
        return Ok(());
    }

    let sensitivity = d_sensitivity(defence_level);
    let absolute = ratio * source_intensity_max as f64 * sensitivity;

    if matches!(response, SandboxResponse::Shield) {
        return Err(AnimiError::SafetyBreach {
            severity: Severity::Deny,
            file_name: crate::error::current_file(),
            dimension: crate::pbm::PbmDimension::Emotional,
            current_energy: absolute,
            threshold: 0.0,
        });
    }

    if absolute > ACCENT_ABSOLUTE_CAP as f64 {
        let clamped = ACCENT_ABSOLUTE_CAP as f64 / (source_intensity_max as f64 * sensitivity);
        oi!(
            StaticSafetyError,
            rule = "沙箱点缀绝对强度(G2降级)".into(),
            detail = format!(
                "点缀配比 {:.2} × 强度 {} × 敏感系数 {:.1} = {:.1} > cap {}。已降级至 {:.2}。",
                ratio, source_intensity_max, sensitivity, absolute, ACCENT_ABSOLUTE_CAP, clamped
            )
        )
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 路由 ─────────────────────────────────────────────

    #[test]
    fn sandbox_threshold_89_not_triggered() {
        assert!(!is_sandbox(89));
    }

    #[test]
    fn sandbox_threshold_90_triggered() {
        assert!(is_sandbox(90));
    }

    #[test]
    fn sandbox_threshold_100_triggered() {
        assert!(is_sandbox(100));
    }

    // ── d_sensitivity ────────────────────────────────────

    #[test]
    fn d_sensitivity_none_is_1() {
        assert!((d_sensitivity(None) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn d_sensitivity_d1_is_1_3() {
        assert!((d_sensitivity(Some(DefenceLevel::D1)) - 1.3).abs() < 1e-10);
    }

    #[test]
    fn d_sensitivity_d2_is_1_8() {
        assert!((d_sensitivity(Some(DefenceLevel::D2)) - 1.8).abs() < 1e-10);
    }

    #[test]
    fn d_sensitivity_d3_is_2_5() {
        assert!((d_sensitivity(Some(DefenceLevel::D3)) - 2.5).abs() < 1e-10);
    }

    // ── worst_response ───────────────────────────────────

    #[test]
    fn worst_response_single() {
        assert_eq!(
            worst_response(&[SandboxResponse::Caution]),
            SandboxResponse::Caution
        );
    }

    #[test]
    fn worst_response_picks_highest() {
        assert_eq!(
            worst_response(&[SandboxResponse::Neutral, SandboxResponse::Shield]),
            SandboxResponse::Shield
        );
    }

    #[test]
    fn worst_response_attainment_at_bottom() {
        assert_eq!(
            worst_response(&[SandboxResponse::Attainment, SandboxResponse::Neutral]),
            SandboxResponse::Neutral
        );
    }

    #[test]
    fn worst_response_empty_is_attainment() {
        assert_eq!(worst_response(&[]), SandboxResponse::Attainment);
    }

    // ── check_combined ───────────────────────────────────

    fn nr() -> SandboxResponse {
        SandboxResponse::Neutral
    }

    #[test]
    fn combined_under_cap_passes() {
        let result = check_combined(50, &[("belonging".into(), 0.3)], 100, nr(), None);
        assert!(result.is_ok());
    }

    #[test]
    fn combined_g1_notify() {
        let result = check_combined(90, &[("belonging".into(), 0.11)], 100, nr(), None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().severity(), Severity::Warn);
    }

    #[test]
    fn combined_g2_degrade() {
        let result = check_combined(90, &[("belonging".into(), 0.3)], 100, nr(), None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().severity(), Severity::Deny);
    }

    #[test]
    fn combined_g2_d1_triggers_degrade() {
        let result = check_combined(
            90,
            &[("belonging".into(), 0.05)],
            100,
            nr(),
            Some(DefenceLevel::D1),
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().severity(), Severity::Deny);
    }

    #[test]
    fn combined_g3_fuse_by_overage() {
        let result = check_combined(90, &[("belonging".into(), 0.5)], 100, nr(), None);
        assert!(result.is_err());
    }

    #[test]
    fn combined_shield_fuses() {
        let result = check_combined(90, &[], 100, SandboxResponse::Shield, None);
        assert!(result.is_err());
    }

    #[test]
    fn combined_caution_degrade_under_cap() {
        let result = check_combined(90, &[], 100, SandboxResponse::Caution, None);
        assert!(result.is_err());
    }

    #[test]
    fn combined_attainment_passes_at_high_intensity() {
        let result = check_combined(
            95,
            &[("belonging".into(), 0.3)],
            100,
            SandboxResponse::Attainment,
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn combined_d2_overrides_attainment() {
        let result = check_combined(
            95,
            &[],
            100,
            SandboxResponse::Attainment,
            Some(DefenceLevel::D2),
        );
        assert!(result.is_err());
    }

    #[test]
    fn combined_high_intensity_pass_with_no_accents_neutral_under_cap() {
        let result = check_combined(95, &[], 100, nr(), None);
        assert!(result.is_ok());
    }

    // ── check_accent_absolute ────────────────────────────

    #[test]
    fn accent_absolute_under_cap_passes() {
        let result = check_accent_absolute(0.3, 90, nr(), None);
        assert!(result.is_ok());
    }

    #[test]
    fn accent_absolute_over_cap_fails() {
        let result = check_accent_absolute(0.5, 90, nr(), None);
        assert!(result.is_err());
    }

    #[test]
    fn accent_absolute_d1_lowers_tolerance() {
        let result = check_accent_absolute(0.3, 90, nr(), Some(DefenceLevel::D1));
        assert!(result.is_err());
    }

    #[test]
    fn accent_absolute_low_intensity_passes() {
        let result = check_accent_absolute(0.5, 50, nr(), None);
        assert!(result.is_ok());
    }

    #[test]
    fn accent_absolute_attainment_passes_even_at_high() {
        let result = check_accent_absolute(0.5, 90, SandboxResponse::Attainment, None);
        assert!(result.is_ok());
    }

    #[test]
    fn accent_absolute_shield_fuses_at_any_intensity() {
        let result = check_accent_absolute(0.01, 50, SandboxResponse::Shield, None);
        assert!(result.is_err());
    }
}
