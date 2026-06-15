// src/psir.rs — PSIR 类型定义

use crate::pbm::DefenceLevel;
use serde::{Deserialize, Serialize};

// ── PsirDoc ──────────────────────────────────────────────────

/// PSIR 文档——个人感受结构。
///
/// Pass 6 (Personalize) 的输出: FSIR × PBM → PSIR。
/// FSIR 是感受的"通用本体"——不绑定任何个人生理参数。
/// PSIR 是同一个感受结构在具体一个人身上的参数化产物——
/// 包含了这个人的四维基线偏移、强度 sigmoidal 缩放、
/// 点缀配比个人帽等全部个人校准结果。
///
/// PSIR 仅存在于设备本地——永不离设备。
/// 这是 Feelings 隐私硬约束——不是优化，是硬边界。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsirDoc {
    /// 感受包名称——来自 FSIR。
    pub name: String,

    /// 主旋律感受原子——经过 PBM 个人参数校准。
    pub main: PersonalizedFeeling,

    /// 点缀列表——经过比例帽校验 + 个人缩放。
    pub accents: Vec<PersonalizedAccent>,

    /// 形状曲线——暂沿用 FSIR 的形状名。
    /// 未来可嵌入个人时序偏移——v0.3 不支持。
    pub shape: PsirShape,

    /// 强度区间——经过 sigmoidal 个人缩放 + 用户上限二次校验。
    pub intensity: PsirIntensity,

    /// oi 帧平滑过渡策略——沿用 FSIR，无个人系数。
    pub smoothing: Option<PsirSmoothing>,

    /// 冷启动——基于 ColdStartGuard 判定。
    pub cold_start: bool,

    /// 创伤路径重定向——本帧是否触发了创伤安全路径。
    pub defence_activated: bool,

    /// 防御激活层级——D1/D2/D3——供下游 Pass 7-8 按防御需求差异化执行。
    /// None = 无防御激活——等控器当前不需要信号缩放保护。
    pub defence_level: Option<DefenceLevel>,

    /// 脱敏降级标志——ADR 012 信号变异度检测。
    /// true = 本帧同维度强度连续多帧停滞——下游强度调度器应插入恢复帧。
    pub degraded: bool,

    /// PBM 更新戳记——记录本次个性化使用的 PBM 版本/状态。
    pub pbm_stamp: PsirPbmStamp,
}

// ── 主旋律原子（个人化）──────────────────────────────────────

/// 经过 PBM 个人校准后的感受原子。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizedFeeling {
    /// 原子名称。
    pub atom: String,

    /// PBM 基线偏移量。
    /// 正值 = 此人此维度的基线高于通用模板。
    /// 负值 = 此人此维度的基线低于通用模板。
    pub baseline_offset: f64,

    /// 阻尼状态——本维度是否在本次 Session 被冻结。
    pub damped: bool,
}

// ── 点缀条目（个人化）────────────────────────────────────────

/// 经过比例帽校验 + PBM 个人缩放后的点缀原子。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizedAccent {
    /// 原子名称。
    pub atom: String,

    /// 原始配比（来自 FSIR）。
    pub original_ratio: f64,

    /// 个人校准后的配比。
    pub applied_ratio: f64,

    /// 比例帽上限——此原子在该用户下的最高允许配比。
    pub ratio_cap: f64,

    /// 阻尼状态。
    pub damped: bool,
}

// ── 形状（暂时 pass-through）────────────────────────────────

/// PSIR 形状曲线——v0.3 沿用 FSIR 形状名。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsirShape {
    pub name: String,
}

// ── 强度（个人化）────────────────────────────────────────────

/// 经过 sigmoidal 缩放 + 用户上限二次校验的强度区间。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsirIntensity {
    /// 原始最小强度（来自 FSIR）。
    pub original_min: u32,

    /// 原始最大强度（来自 FSIR）。
    pub original_max: u32,

    /// 个人校准后的最小强度。
    pub applied_min: u32,

    /// 个人校准后的最大强度。
    pub applied_max: u32,

    /// 用户强度上限。
    pub cap: u32,
}

// ── 平滑过渡（pass-through）───────────────────────────────────

/// PSIR 侧的 oi 帧平滑过渡策略——沿用 FSIR。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsirSmoothing {
    pub steps: Vec<PsirDecayStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsirDecayStep {
    pub multiplier: f64,
}

// ── PBM 更新戳记 ────────────────────────────────────────────

/// PBM 更新戳记——记录本次个性化使用的 PBM 版本和状态。
/// 用于后续 PSIR 的回溯审计和版本兼容校验。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsirPbmStamp {
    /// PBM 最后更新时间（RFC 3339）。
    pub pbm_updated_at: String,

    /// 本次使用的 Session 计数（来自 ColdStartGuard）。
    pub session_count: u32,

    /// 本次是否为冷启动。
    pub cold_start: bool,
}

// ── 构造函数 ──────────────────────────────────────────────────

/// PSIR 感受输入——用于 `PsirDoc::new()` 将参数收拢。
pub struct PsirFeelingInput {
    pub name: String,
    pub main: PersonalizedFeeling,
    pub accents: Vec<PersonalizedAccent>,
    pub shape_name: String,
}

/// PSIR 强度输入——用于 `PsirDoc::new()` 将参数收拢。
pub struct PsirIntensityInput {
    pub original_min: u32,
    pub original_max: u32,
    pub applied_min: u32,
    pub applied_max: u32,
    pub cap: u32,
}

/// PSIR 元数据输入——用于 `PsirDoc::new()` 将参数收拢。
pub struct PsirMetaInput {
    pub smoothing: Option<PsirSmoothing>,
    pub cold_start: bool,
    pub defence_activated: bool,
    pub defence_level: Option<DefenceLevel>,
    /// 脱敏降级标志——ADR 012 信号变异度检测。
    pub degraded: bool,
    pub pbm_updated_at: String,
    pub session_count: u32,
}

impl PsirDoc {
    /// 从三组结构体构造一份 PsirDoc。
    pub fn new(
        feeling: PsirFeelingInput,
        intensity: PsirIntensityInput,
        meta: PsirMetaInput,
    ) -> Self {
        PsirDoc {
            name: feeling.name,
            main: feeling.main,
            accents: feeling.accents,
            shape: PsirShape {
                name: feeling.shape_name,
            },
            intensity: PsirIntensity {
                original_min: intensity.original_min,
                original_max: intensity.original_max,
                applied_min: intensity.applied_min,
                applied_max: intensity.applied_max,
                cap: intensity.cap,
            },
            smoothing: meta.smoothing,
            cold_start: meta.cold_start,
            defence_activated: meta.defence_activated,
            defence_level: meta.defence_level,
            degraded: meta.degraded,
            pbm_stamp: PsirPbmStamp {
                pbm_updated_at: meta.pbm_updated_at,
                session_count: meta.session_count,
                cold_start: meta.cold_start,
            },
        }
    }
}
