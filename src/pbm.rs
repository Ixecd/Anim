// src/pbm.rs — 个人基线矩阵（PBM）地基
//
// PBM 是 animi 交织管线的核心数据结构——一个四维超维向量，
// 记录用户在每个感受维度上的个人生理基线偏移。
//
// 本文件实现 PBM 的三个架构地基（P0 #6/#7/#8）：
//   - SessionLabel / DataConfidence — 异常终止 Session 不污染基线 (#6)
//   - ColdStartGuard — 前 10 次 Session 强制关闭预测器 (#7)
//   - DampingMatrix — 跨维度阻尼矩阵的具体参数表 (#8)
//
// 完整 PBM（四维差异化冷启动 + sigmoidal 收敛 + 因子三实时置信度）
// 待 v0.3 实现。本文件是那一切的底座。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Session 标签与数据置信度 (P0 #6) ──────────────────────────

/// Session 终止类型——决定 PBM 更新策略。
///
/// 异常终止 Session 的数据不能被当作"正常的基线漂移"写入 PBM。
/// 安全插桩强制终止的 Session ≠ 用户自然结束的 Session——
/// 前者是突发噪声，后者是真实基线信号。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionLabel {
    /// 正常 Session——用户自然结束。PBM 全量更新。
    Normal,

    /// 异常终止——安全插桩强制终止（心率超标 / 皮电骤升 / RuntimeGuard 帧拦截）。
    /// PBM 只更新安全阈值（`safety_bounds`），不更新基线。
    Abnormal,

    /// 冷启动 Session——用户前 N 次使用。预测器关闭。数据置信度固定为 High。
    /// 不因冷启动数据不成熟而降级——因为没有预测器，就没有"预测错"。
    ColdStart,
}

/// 数据置信度——本次 Session 的生理数据在多大程度上可信。
///
/// 异常 Session 的数据置信度为 Contaminated——PBM 基线不应由此更新。
/// 正常 Session 的数据置信度由因子三实时判定：High/Low。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataConfidence {
    /// 高置信度——生理信号稳定，因子三未触发降级。PBM 全量更新。
    High,

    /// 低置信度——因子三触发过降级（HRV 紊乱 / 皮电飙升），但未达到 RuntimeGuard 帧拦截。
    /// PBM 更新步长缩小（默认 ×0.15），但不冻结。
    Low,

    /// 污染——安全插桩强制终止。PBM 基线不更新。只更新安全阈值。
    Contaminated,
}

impl SessionLabel {
    /// Session 标签 → PBM 更新策略。
    pub fn update_strategy(&self) -> PbmUpdateStrategy {
        match self {
            SessionLabel::Normal => PbmUpdateStrategy::Full,
            SessionLabel::Abnormal => PbmUpdateStrategy::SafetyOnly,
            SessionLabel::ColdStart => PbmUpdateStrategy::Full, // 冷启动数据可信——只是没有预测器
        }
    }
}

impl DataConfidence {
    /// 置信度 → PBM 步长乘数。
    ///
    /// High   = ×1.0（全步长）
    /// Low    = ×0.15（因子三降级——ADR 007 §7.6）
    /// Contaminated = ×0.0（冻结——不更新基线）
    pub fn step_multiplier(&self) -> f64 {
        match self {
            DataConfidence::High => 1.0,
            DataConfidence::Low => 0.15,
            DataConfidence::Contaminated => 0.0,
        }
    }
}

/// PBM 更新策略——对应 FORGET P0 #6 的"需数据置信度分级"。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PbmUpdateStrategy {
    /// 全量更新——基线和安全阈值都更新。
    Full,

    /// 仅安全阈值——基线不动。异常 Session 专用。
    SafetyOnly,
}

// ── 冷启动守护 (P0 #7) ──────────────────────────────────────

/// 冷启动守护——新用户前 N 次 Session 强制关闭预测器。
///
/// ADR 007 §7.9：无历史数据 → 预测准确率 < 50% → 频繁安全插桩 → 数据污染。
/// 前 10 次 Session 不开启预测——只做信号注入和生理采集，PBM 用固定冷启动系数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColdStartGuard {
    /// 已完成的 Session 数。
    pub session_count: u32,

    /// 冷启动阈值——默认 10。低于此阈值时预测器关闭。
    pub threshold: u32,
}

impl Default for ColdStartGuard {
    fn default() -> Self {
        ColdStartGuard {
            session_count: 0,
            threshold: 10,
        }
    }
}

impl ColdStartGuard {
    /// 本次 Session 是否处于冷启动期。
    pub fn is_cold_start(&self) -> bool {
        self.session_count < self.threshold
    }

    /// Session 标签——冷启动期内返回 ColdStart，否则 Normal。
    pub fn label(&self) -> SessionLabel {
        if self.is_cold_start() {
            SessionLabel::ColdStart
        } else {
            SessionLabel::Normal
        }
    }

    /// 是否应启用预测器。
    pub fn predictor_enabled(&self) -> bool {
        !self.is_cold_start()
    }

    /// 完成一次 Session——计数 +1。
    pub fn complete_session(&mut self) {
        self.session_count = self.session_count.saturating_add(1);
    }

    /// 距冷启动结束还需多少次 Session。
    pub fn remaining(&self) -> u32 {
        self.threshold.saturating_sub(self.session_count)
    }
}

// ── 跨维度阻尼矩阵 (P0 #8) ──────────────────────────────────

/// PBM 四维标识。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PbmDimension {
    /// 内脏维度——心率、HRV、迷走神经张力。
    Visceral,
    /// 情绪维度——皮电、杏仁核激活、前额叶-岛叶共振。
    Emotional,
    /// 触觉维度——CT 纤维通路、皮肤感受器响应。
    Tactile,
    /// 听觉维度——骨传导通路、听觉皮层响应。
    Auditory,
}

/// 创伤分级——来自 `trauma-protocol.md`，ADR 009 §十一。
///
/// Feelings-Core PBM 在 Session 启动时从用户档案读取并下发给 Anim。
/// Anim 不判断——不诊断——不存档。只执行分级的安全约束。
///
/// 分级：
///   - V1：禁主不禁点缀（安全类原子除外）——强度上限不受影响
///   - V2：禁主 + 点缀配比减半 + 强度上限同等缩减
///   - V3：全禁 + LeakRate→~0 + CriticalThreshold→极低 + P0 抢占就绪
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraumaTier {
    /// 创伤一级——禁主不禁点。
    V1,
    /// 创伤二级——禁主 + 点缀配比减半。
    V2,
    /// 创伤三级——全禁。激活主动麻痹 + P0 抢占的物理前提。
    V3,
}

/// 单个维度的步长状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    /// 活跃——本维度步长正常更新。
    Active,
    /// 冻结——被其他维度的瞬时震荡污染，本维度步长暂停更新。
    Frozen { frozen_by: PbmDimension },
}

/// 跨维度阻尼矩阵 —— 阻止一个维度的瞬时噪声污染其他维度的 PBM 更新。
///
/// ADR 007 §7.7：情绪维度的瞬时梯度过大时——
///   - 冻结内脏维度（心率剧变来自情绪，不是内脏基线变了）
///   - 冻结触觉维度（皮电飙升来自情绪，不是触觉基线变了）
///   - 听觉维度保持（听觉通路受情绪干扰最小）
///
/// 冻结不是永久的。情绪梯度回落 → 自动解冻。
///
/// # 参数表
///
/// | 触发维度   | 阈值 × 步长 | 冻结 Visceral | 冻结 Emotional | 冻结 Tactile | 冻结 Auditory |
/// |-----------|-------------|---------------|----------------|--------------|---------------|
/// | Emotional | > 2.0×step  | ✅ 冻结        | — (自身)       | ✅ 冻结       | ❌ 保持       |
/// | Visceral  | > 1.5×step  | — (自身)      | ✅ 冻结        | ❌ 保持       | ❌ 保持       |
/// | Tactile   | > 3.0×step  | ❌ 保持       | ❌ 保持        | — (自身)     | ❌ 保持       |
/// | Auditory  | > 2.0×step  | ❌ 保持       | ❌ 保持        | ❌ 保持       | — (自身)     |
///
/// 梯度 = |本帧 PBM 偏移 - 上帧 PBM 偏移|。步长 = 当前维度衰减后的步长乘数。
#[derive(Debug, Clone)]
pub struct DampingMatrix;

impl DampingMatrix {
    /// 梯度阈值——梯度超过 threshold × 当前步长 → 触发跨维度冻结。
    pub fn gradient_threshold(dim: PbmDimension) -> f64 {
        match dim {
            PbmDimension::Emotional => 2.0,
            PbmDimension::Visceral => 1.5,
            PbmDimension::Tactile => 3.0,
            PbmDimension::Auditory => 2.0,
        }
    }

    /// 给定触发维度和被检查维度——返回是否应冻结。
    ///
    /// 返回 `Some(frozen_by)` = 冻结；`None` = 保持活跃。
    pub fn should_freeze(trigger: PbmDimension, target: PbmDimension) -> Option<PbmDimension> {
        if trigger == target {
            return None; // 自身不冻结自身
        }

        match trigger {
            PbmDimension::Emotional => match target {
                PbmDimension::Visceral => Some(trigger), // 冻结
                PbmDimension::Tactile => Some(trigger),  // 冻结
                PbmDimension::Auditory => None,          // 保持
                PbmDimension::Emotional => None,         // unreachable
            },
            PbmDimension::Visceral => match target {
                PbmDimension::Emotional => Some(trigger), // 冻结
                PbmDimension::Tactile => None,            // 保持
                PbmDimension::Auditory => None,           // 保持
                PbmDimension::Visceral => None,
            },
            PbmDimension::Tactile => {
                // 触觉的瞬时震荡几乎不污染其他维度
                None
            }
            PbmDimension::Auditory => {
                // 听觉的瞬时震荡几乎不污染其他维度
                None
            }
        }
    }

    /// 对一个完整 Session 的四个维度做阻尼判定。
    ///
    /// `gradients`: 每个维度的梯度值（|本帧偏移 - 上帧偏移|）
    /// `current_steps`: 每个维度的当前步长乘数
    ///
    /// 返回每个维度 → 步长状态的映射。调用方通过维度名查询，无需关心顺序。
    ///
    /// # 多维度同时触发阻尼时的覆盖规则
    ///
    /// 当多个维度同时触发阻尼时，所有受影响的维度**都会被冻结**——
    /// 不管处理顺序如何，最终结果一致：只要存在任一触发维度指向目标维度，
    /// 目标维度即被冻结。不会出现"后处理的维度覆盖先处理的维度导致结果不同"的问题。
    ///
    /// 例：Emotional 和 Visceral 同时超阈值 →
    ///   - Emotional 触发 → 冻结 Visceral、冻结 Tactile
    ///   - Visceral 触发 → 冻结 Emotional
    ///   - 最终：Visceral 被冻结、Tactile 被冻结、Emotional 被冻结、Auditory 保持 Active
    ///   - 无论先处理哪个维度，结果相同——冻结集合的并集，无覆盖歧义。
    pub fn apply(
        gradients: &[(PbmDimension, f64); 4],
        current_steps: &[(PbmDimension, f64); 4],
    ) -> HashMap<PbmDimension, StepState> {
        let step_map = |dim: PbmDimension| -> f64 {
            current_steps
                .iter()
                .find(|(d, _)| *d == dim)
                .map(|(_, s)| *s)
                .unwrap_or(1.0)
        };

        // 初始化所有维度为 Active
        let mut states: HashMap<_, _> = gradients
            .iter()
            .map(|(d, _)| (*d, StepState::Active))
            .collect();

        // 检查哪些维度触发了阻尼
        for (dim, grad) in gradients.iter() {
            let threshold = Self::gradient_threshold(*dim) * step_map(*dim);
            if *grad > threshold {
                // 此维度触发了阻尼——冻结所有受影响的维度
                for (target_dim, _) in gradients.iter() {
                    if let Some(frozen_by) = Self::should_freeze(*dim, *target_dim) {
                        states.insert(*target_dim, StepState::Frozen { frozen_by });
                    }
                }
            }
        }

        states
    }
}

// ── 单元测试 ────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // SessionLabel / DataConfidence

    #[test]
    fn session_label_normal_full_update() {
        assert_eq!(
            SessionLabel::Normal.update_strategy(),
            PbmUpdateStrategy::Full
        );
    }

    #[test]
    fn session_label_abnormal_safety_only() {
        assert_eq!(
            SessionLabel::Abnormal.update_strategy(),
            PbmUpdateStrategy::SafetyOnly
        );
    }

    #[test]
    fn data_confidence_multipliers() {
        assert!((DataConfidence::High.step_multiplier() - 1.0).abs() < 1e-10);
        assert!((DataConfidence::Low.step_multiplier() - 0.15).abs() < 1e-10);
        assert!((DataConfidence::Contaminated.step_multiplier() - 0.0).abs() < 1e-10);
    }

    // ColdStartGuard

    #[test]
    fn cold_start_session_0_is_cold() {
        let guard = ColdStartGuard::default();
        assert!(guard.is_cold_start());
        assert_eq!(guard.label(), SessionLabel::ColdStart);
        assert!(!guard.predictor_enabled());
        assert_eq!(guard.remaining(), 10);
    }

    #[test]
    fn cold_start_session_9_is_cold() {
        let guard = ColdStartGuard {
            session_count: 9,
            threshold: 10,
        };
        assert!(guard.is_cold_start());
        assert_eq!(guard.remaining(), 1);
    }

    #[test]
    fn cold_start_session_10_is_warm() {
        let guard = ColdStartGuard {
            session_count: 10,
            threshold: 10,
        };
        assert!(!guard.is_cold_start());
        assert_eq!(guard.label(), SessionLabel::Normal);
        assert!(guard.predictor_enabled());
        assert_eq!(guard.remaining(), 0);
    }

    #[test]
    fn cold_start_complete_session_increments() {
        let mut guard = ColdStartGuard::default();
        assert_eq!(guard.session_count, 0);
        guard.complete_session();
        assert_eq!(guard.session_count, 1);
        guard.complete_session();
        assert_eq!(guard.session_count, 2);
    }

    #[test]
    fn cold_start_saturating_no_overflow() {
        let mut guard = ColdStartGuard {
            session_count: u32::MAX,
            threshold: 10,
        };
        guard.complete_session();
        assert_eq!(guard.session_count, u32::MAX); // saturating_add
    }

    // DampingMatrix

    #[test]
    fn damping_emotional_freezes_visceral_and_tactile() {
        assert_eq!(
            DampingMatrix::should_freeze(PbmDimension::Emotional, PbmDimension::Visceral),
            Some(PbmDimension::Emotional)
        );
        assert_eq!(
            DampingMatrix::should_freeze(PbmDimension::Emotional, PbmDimension::Tactile),
            Some(PbmDimension::Emotional)
        );
    }

    #[test]
    fn damping_emotional_keeps_auditory() {
        assert_eq!(
            DampingMatrix::should_freeze(PbmDimension::Emotional, PbmDimension::Auditory),
            None
        );
    }

    #[test]
    fn damping_self_never_frozen() {
        for dim in &[
            PbmDimension::Visceral,
            PbmDimension::Emotional,
            PbmDimension::Tactile,
            PbmDimension::Auditory,
        ] {
            assert_eq!(DampingMatrix::should_freeze(*dim, *dim), None);
        }
    }

    #[test]
    fn damping_tactile_never_contaminates() {
        // 触觉震荡不污染任何维度
        for target in &[
            PbmDimension::Visceral,
            PbmDimension::Emotional,
            PbmDimension::Auditory,
        ] {
            assert_eq!(
                DampingMatrix::should_freeze(PbmDimension::Tactile, *target),
                None
            );
        }
    }

    #[test]
    fn damping_apply_all_active_when_no_trigger() {
        let gradients = [
            (PbmDimension::Visceral, 0.1),
            (PbmDimension::Emotional, 0.2),
            (PbmDimension::Tactile, 0.1),
            (PbmDimension::Auditory, 0.05),
        ];
        let steps = [
            (PbmDimension::Visceral, 0.5),
            (PbmDimension::Emotional, 0.5),
            (PbmDimension::Tactile, 0.5),
            (PbmDimension::Auditory, 0.5),
        ];
        let states = DampingMatrix::apply(&gradients, &steps);
        for (_, s) in &states {
            assert_eq!(*s, StepState::Active);
        }
    }

    #[test]
    fn damping_apply_emotional_trigger_freezes_visceral_and_tactile() {
        let gradients = [
            (PbmDimension::Visceral, 0.1),
            (PbmDimension::Emotional, 1.5), // > 2.0 × 0.5 = 1.0 → 触发
            (PbmDimension::Tactile, 0.1),
            (PbmDimension::Auditory, 0.05),
        ];
        let steps = [
            (PbmDimension::Visceral, 0.5),
            (PbmDimension::Emotional, 0.5),
            (PbmDimension::Tactile, 0.5),
            (PbmDimension::Auditory, 0.5),
        ];
        let states = DampingMatrix::apply(&gradients, &steps);
        // Visceral 应被 Emotional 冻结
        assert_eq!(
            states[&PbmDimension::Visceral],
            StepState::Frozen {
                frozen_by: PbmDimension::Emotional
            }
        );
        // Emotional 自身保持 Active
        assert_eq!(states[&PbmDimension::Emotional], StepState::Active);
        // Tactile 应被 Emotional 冻结
        assert_eq!(
            states[&PbmDimension::Tactile],
            StepState::Frozen {
                frozen_by: PbmDimension::Emotional
            }
        );
        // Auditory 保持
        assert_eq!(states[&PbmDimension::Auditory], StepState::Active);
    }

    #[test]
    fn damping_matrix_json_roundtrip() {
        // 验证阻尼矩阵核心类型可序列化
        let guard = ColdStartGuard::default();
        let json = serde_json::to_string(&guard).unwrap();
        let guard2: ColdStartGuard = serde_json::from_str(&json).unwrap();
        assert_eq!(guard.session_count, guard2.session_count);
        assert_eq!(guard.threshold, guard2.threshold);
    }
}
