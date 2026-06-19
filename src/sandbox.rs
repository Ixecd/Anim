// src/sandbox.rs — Sandbox routing + Governance engine —— ADR 009 v0.6 §十
//
// 沙箱 = 强度阈值触发的执行环境——与原子分类解耦。
// intensity.max >= 90 → 自动进沙箱 → Governance 接管。
//
// Governance 不是阻断——是引导：
//   Attainment → 安静通过（高峰体验不受限）
//   G1/Steer   → 生成低强度 calm 叠加帧——温和拉回
//   G2/Redirect → 忽略原感受包，输出 deep_rest 替代感受
//   G3/Anchor  → 强制 grounding 锚点帧 + 通知 Core 升级 DefenceLevel
//
// 沙箱响应策略（原子级别）：
//   Attainment — 成就/高峰体验，90+是设计目标，只监控不限制
//   Neutral    — 中性，按强度叠加比例分级响应（Steer/Redirect/Anchor）
//   Caution    — 需谨慎，无条件 Redirect
//   Shield     — 必须防护，直接 Anchor + escalate

use crate::pbm::DefenceLevel;
use crate::registry::SandboxResponse;

pub const SANDBOX_THRESHOLD: u32 = 90;
pub const ACCENT_ABSOLUTE_CAP: u32 = 30;
pub const COMBINED_OVERAGE_G1_RATIO: f64 = 1.10;
pub const COMBINED_OVERAGE_G3_RATIO: f64 = 1.30;
pub const SANDBOX_ACCENT_WEIGHT: f64 = 1.5;

pub const STEER_INTENSITY: u32 = 15;
pub const REDIRECT_INTENSITY: u32 = 20;
pub const ANCHOR_INTENSITY: u32 = 5;

// ── GovernanceAction ────────────────────────────────────────────

/// Governance 的产出——不是错误，是一个感受引导指令。
///
/// Pipeline 层根据此指令修改输出，而非拒绝输出。
#[derive(Debug, Clone, PartialEq)]
pub enum GovernanceAction {
    /// 无干预——原始感受包直通。
    PassThrough,
    /// G1：温和校准——生成低强度 calm 叠加帧，叠在原输出上。
    /// 调用方应混合输出而非替换。
    Steer {
        blend_atom: String,
        blend_intensity: u32,
        blend_shape: String,
    },
    /// G2：重新导向——忽略原感受包，输出 deep_rest 替代。
    Redirect {
        atom: String,
        intensity: u32,
        shape: String,
    },
    /// G3：安全锚点——强制 grounding 帧 + 通知 Core 升级 DefenceLevel。
    Anchor {
        atom: String,
        intensity: u32,
        shape: String,
        escalate_defence: bool,
    },
}

impl GovernanceAction {
    pub fn is_passthrough(&self) -> bool {
        matches!(self, GovernanceAction::PassThrough)
    }

    pub fn description(&self) -> &str {
        match self {
            GovernanceAction::PassThrough => "直通",
            GovernanceAction::Steer { .. } => "G1 温和拉回",
            GovernanceAction::Redirect { .. } => "G2 重新导向",
            GovernanceAction::Anchor { .. } => "G3 安全锚点",
        }
    }
}

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
/// 返回 GovernanceAction 而非 Error——Governance 是引导，不是拒绝。
///
/// 公式：combined = main_max × (1 + Σ accent_ratio × accent_weight)
pub fn check_combined(
    applied_main_max: u32,
    accent_ratios: &[(String, f64)],
    effective_cap: u32,
    response: SandboxResponse,
    defence_level: Option<DefenceLevel>,
) -> GovernanceAction {
    let cap = effective_cap as f64;
    let sum_ratios: f64 = accent_ratios
        .iter()
        .map(|(name, ratio)| ratio * accent_intensity_weight(name))
        .sum();
    let combined = applied_main_max as f64 * (1.0 + sum_ratios);

    // D2+ 无条件安全锚点
    if matches!(defence_level, Some(DefenceLevel::D2 | DefenceLevel::D3)) {
        return GovernanceAction::Anchor {
            atom: "deep_rest".into(),
            intensity: ANCHOR_INTENSITY,
            shape: "steady".into(),
            escalate_defence: matches!(defence_level, Some(DefenceLevel::D3)),
        };
    }

    match response {
        SandboxResponse::Shield => GovernanceAction::Anchor {
            atom: "deep_rest".into(),
            intensity: ANCHOR_INTENSITY,
            shape: "steady".into(),
            escalate_defence: true,
        },
        SandboxResponse::Caution => GovernanceAction::Redirect {
            atom: "deep_rest".into(),
            intensity: effective_cap.min(REDIRECT_INTENSITY),
            shape: "slow_decay".into(),
        },
        SandboxResponse::Attainment => {
            // 成就/高峰体验——安静通过
            GovernanceAction::PassThrough
        }
        SandboxResponse::Neutral => {
            if combined <= cap && !matches!(defence_level, Some(DefenceLevel::D1)) {
                return GovernanceAction::PassThrough;
            }

            let overage = combined / cap;

            if overage > COMBINED_OVERAGE_G3_RATIO {
                return GovernanceAction::Anchor {
                    atom: "deep_rest".into(),
                    intensity: ANCHOR_INTENSITY,
                    shape: "steady".into(),
                    escalate_defence: false,
                };
            }

            // D1 或超限 > 10% → Redirect
            if matches!(defence_level, Some(DefenceLevel::D1))
                || overage > COMBINED_OVERAGE_G1_RATIO
            {
                GovernanceAction::Redirect {
                    atom: "deep_rest".into(),
                    intensity: effective_cap.min(REDIRECT_INTENSITY),
                    shape: "slow_decay".into(),
                }
            } else {
                // G1: 超限 ≤ 10% 且无 D1 → Steer
                GovernanceAction::Steer {
                    blend_atom: "calm_meditative".into(),
                    blend_intensity: STEER_INTENSITY,
                    blend_shape: "slow_decay".into(),
                }
            }
        }
    }
}

// ── 点缀配比绝对强度校验（P0 #4）───────────────────────────────

/// 校验单个点缀的绝对强度是否超过上限，返回 Governance 引导指令。
///
/// 公式：absolute = ratio × source_intensity_max × d_sensitivity(defence_level)
pub fn check_accent_absolute(
    ratio: f64,
    source_intensity_max: u32,
    response: SandboxResponse,
    defence_level: Option<DefenceLevel>,
) -> GovernanceAction {
    if matches!(response, SandboxResponse::Attainment) {
        return GovernanceAction::PassThrough;
    }

    let sensitivity = d_sensitivity(defence_level);
    let absolute = ratio * source_intensity_max as f64 * sensitivity;

    if matches!(response, SandboxResponse::Shield)
        || matches!(defence_level, Some(DefenceLevel::D2 | DefenceLevel::D3))
    {
        return GovernanceAction::Anchor {
            atom: "deep_rest".into(),
            intensity: ANCHOR_INTENSITY,
            shape: "steady".into(),
            escalate_defence: true,
        };
    }

    if absolute > ACCENT_ABSOLUTE_CAP as f64 {
        GovernanceAction::Steer {
            blend_atom: "calm_meditative".into(),
            blend_intensity: STEER_INTENSITY,
            blend_shape: "slow_decay".into(),
        }
    } else {
        GovernanceAction::PassThrough
    }
}

// ── 聚合多个 GovernanceAction ─────────────────────────────────

/// 返回最严格的 GovernanceAction。用于多个 check 结果合并。
pub fn strictest(actions: &[GovernanceAction]) -> GovernanceAction {
    let mut worst = GovernanceAction::PassThrough;
    for a in actions {
        let rank = action_rank(a);
        if rank > action_rank(&worst) {
            worst = a.clone();
        }
    }
    worst
}

fn action_rank(action: &GovernanceAction) -> u8 {
    match action {
        GovernanceAction::PassThrough => 0,
        GovernanceAction::Steer { .. } => 1,
        GovernanceAction::Redirect { .. } => 2,
        GovernanceAction::Anchor { .. } => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nr() -> SandboxResponse {
        SandboxResponse::Neutral
    }

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

    #[test]
    fn combined_under_cap_passes() {
        let action = check_combined(50, &[("belonging".into(), 0.3)], 100, nr(), None);
        assert!(action.is_passthrough());
    }

    #[test]
    fn combined_g1_steer() {
        let action = check_combined(90, &[("belonging".into(), 0.11)], 100, nr(), None);
        assert!(matches!(action, GovernanceAction::Steer { .. }));
    }

    #[test]
    fn combined_g2_redirect() {
        let action = check_combined(90, &[("belonging".into(), 0.2)], 100, nr(), None);
        assert!(matches!(action, GovernanceAction::Redirect { .. }));
    }

    #[test]
    fn combined_g2_d1_redirect() {
        let action = check_combined(
            90,
            &[("belonging".into(), 0.05)],
            100,
            nr(),
            Some(DefenceLevel::D1),
        );
        assert!(matches!(action, GovernanceAction::Redirect { .. }));
    }

    #[test]
    fn combined_g3_anchor() {
        let action = check_combined(90, &[("belonging".into(), 0.5)], 100, nr(), None);
        assert!(matches!(action, GovernanceAction::Anchor { .. }));
    }

    #[test]
    fn combined_shield_anchors() {
        let action = check_combined(90, &[], 100, SandboxResponse::Shield, None);
        assert!(matches!(action, GovernanceAction::Anchor { .. }));
    }

    #[test]
    fn combined_caution_redirects_under_cap() {
        let action = check_combined(90, &[], 100, SandboxResponse::Caution, None);
        assert!(matches!(action, GovernanceAction::Redirect { .. }));
    }

    #[test]
    fn combined_attainment_passthrough_at_high_intensity() {
        let action = check_combined(
            95,
            &[("belonging".into(), 0.3)],
            100,
            SandboxResponse::Attainment,
            None,
        );
        assert!(action.is_passthrough());
    }

    #[test]
    fn combined_d2_overrides_attainment_to_anchor() {
        let action = check_combined(
            95,
            &[],
            100,
            SandboxResponse::Attainment,
            Some(DefenceLevel::D2),
        );
        assert!(matches!(action, GovernanceAction::Anchor { .. }));
    }

    #[test]
    fn combined_neutral_under_cap_passthrough() {
        let action = check_combined(95, &[], 100, nr(), None);
        assert!(action.is_passthrough());
    }

    // ── check_accent_absolute ────────────────────────────

    #[test]
    fn accent_absolute_under_cap_passthrough() {
        let action = check_accent_absolute(0.3, 90, nr(), None);
        assert!(action.is_passthrough());
    }

    #[test]
    fn accent_absolute_over_cap_steers() {
        let action = check_accent_absolute(0.5, 90, nr(), None);
        assert!(matches!(action, GovernanceAction::Steer { .. }));
    }

    #[test]
    fn accent_absolute_d1_lowers_tolerance_steers() {
        let action = check_accent_absolute(0.3, 90, nr(), Some(DefenceLevel::D1));
        assert!(matches!(action, GovernanceAction::Steer { .. }));
    }

    #[test]
    fn accent_absolute_low_intensity_passthrough() {
        let action = check_accent_absolute(0.5, 50, nr(), None);
        assert!(action.is_passthrough());
    }

    #[test]
    fn accent_absolute_attainment_passthrough_even_at_high() {
        let action = check_accent_absolute(0.5, 90, SandboxResponse::Attainment, None);
        assert!(action.is_passthrough());
    }

    #[test]
    fn accent_absolute_shield_anchors_at_any_intensity() {
        let action = check_accent_absolute(0.01, 50, SandboxResponse::Shield, None);
        assert!(matches!(action, GovernanceAction::Anchor { .. }));
    }

    // ── strictest ────────────────────────────────────────

    #[test]
    fn strictest_picks_anchor_over_steer() {
        let actions = [
            GovernanceAction::Steer {
                blend_atom: "calm".into(),
                blend_intensity: 15,
                blend_shape: "steady".into(),
            },
            GovernanceAction::Anchor {
                atom: "deep_rest".into(),
                intensity: 5,
                shape: "steady".into(),
                escalate_defence: true,
            },
        ];
        assert!(matches!(
            strictest(&actions),
            GovernanceAction::Anchor { .. }
        ));
    }

    #[test]
    fn strictest_all_passthrough() {
        let actions = [GovernanceAction::PassThrough, GovernanceAction::PassThrough];
        assert!(strictest(&actions).is_passthrough());
    }

    #[test]
    fn strictest_empty_is_passthrough() {
        assert!(strictest(&[]).is_passthrough());
    }
}
