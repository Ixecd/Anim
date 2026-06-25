// src/safety.rs — Pass 3：用户安全
//
// Anim 职责收敛。漏桶/脱敏/阻尼已迁移至 Feelings-Core。
// 本文件仅保留 Pass 3 的强度缩放 + 低锚点 cap。

use crate::ast::*;
use crate::config::AnimConfig;
use crate::error::AnimiError;

/// 对源码执行用户安全规则检查（v1.1 桩——全部通过）。
pub fn check(_source: &FeelingSource) -> Result<(), AnimiError> {
    Ok(())
}

/// 检查 + 强度缩放——返回缩放后的 Interval。
pub fn check_with_scale(
    source: &FeelingSource,
    user_cap: u32,
    _config: &AnimConfig,
) -> Result<Intensity, AnimiError> {
    check(source)?;
    Ok(scale_intensity(&source.intensity, user_cap))
}

/// 低锚点置信度硬上限——R < threshold → cap = low_anchor_cap。
/// anchor_confidence 由 Feelings-Core PBM 提供。
pub fn low_anchor_cap(user_cap: u32, anchor_confidence: Option<f64>, config: &AnimConfig) -> u32 {
    match anchor_confidence {
        Some(r) if r < config.caps.low_anchor_boundary => user_cap.min(config.caps.low_anchor_cap),
        _ => user_cap,
    }
}

/// 强度等比缩放——当 user_cap < source max 时等比缩放。
pub fn scale_intensity(intensity: &Intensity, user_cap: u32) -> Intensity {
    if intensity.max <= user_cap {
        return intensity.clone();
    }
    let ratio = user_cap as f64 / intensity.max as f64;
    Intensity {
        min: (intensity.min as f64 * ratio).round() as u32,
        max: user_cap,
    }
}
