// src/pbm.rs — 共享类型定义
//
// Anim 职责收敛：PbmDimension + DefenceLevel 为 Anim 和 Core 共享枚举。
// PBM 状态类型（ColdStartGuard/DampingState 等）已迁移至 Feelings-Core。

use serde::{Deserialize, Serialize};

/// PBM 四维——Anim 和 Core 共享的维度枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PbmDimension {
    Visceral,
    Emotional,
    Tactile,
    Auditory,
}

impl PbmDimension {
    pub fn dim_index(dim: PbmDimension) -> usize {
        match dim {
            PbmDimension::Visceral => 0,
            PbmDimension::Emotional => 1,
            PbmDimension::Tactile => 2,
            PbmDimension::Auditory => 3,
        }
    }
}

/// 防御激活层级——Anim 和 Core 共享。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefenceLevel {
    D1,
    D2,
    D3,
}
