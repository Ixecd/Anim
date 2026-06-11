// src/safety.rs — Pass 3：用户安全
//
// 对这个人的拒绝。读用户档案——创伤分型、强度 cap、锚点置信度。
// 必须在设备本地运行——用户穿戴并开始 Session 时。
//
// v1.1 桩——全部通过。后续版本接入：
//   - 强度等比缩放（scale_intensity）
//   - 创伤分型交叉判定（v1/v2/v3 × 社交/情绪/躯体）
//   - 强度上限 cap（如低锚点置信度 max 20）
//   - 创伤原子黑名单（禁主不禁点）
//   - 触觉维度锁定
//
// v0.4: 未成年不等于年龄。成年 = 锚在里面（R > 0.3），不是法律上的 18 岁。
//   `anchor_confidence` 由 Feelings-Core PBM 提供——Anim 只做硬上限。
//   见 Feelings docs/society/adulthood-as-anchor.md。

use crate::ast::*;
use crate::error::AnimiError;

/// 对源码执行用户安全规则检查。
///
/// v1.1——全部通过。后续版本需要加载用户档案。
pub fn check(_source: &FeelingSource) -> Result<(), AnimiError> {
    Ok(())
}

/// 检查 + 强度缩放——返回缩放后的 Interval。
/// v1.1 `user_cap` 从环境变量 `ANIMI_USER_CAP` 取，默认 100。
pub fn check_with_scale(source: &FeelingSource, user_cap: u32) -> Result<Intensity, AnimiError> {
    check(source)?;
    Ok(scale_intensity(&source.intensity, user_cap))
}

/// 低锚点置信度 cap —— 硬上限 20。
///
/// 不是"未成年"——不是年龄。是锚点在外面的人——还没学会自己管住自己。
/// anchor_confidence = Feelings-Core PBM 提供的内部锚点成熟度置信度 R (0.0~1.0)。
/// R < 0.3 → cap = min(20, user_cap)。不是惩罚——是"你还没准备好——我先替你守着"。
///
/// v0.4: Anim 侧只做硬上限。R 由 Core 提供——Anim 不计算。
///       当前默认 None（无数据→不拦截）。v0.4 接 Core 后改为必传。
pub fn low_anchor_cap(user_cap: u32, anchor_confidence: Option<f64>) -> u32 {
    match anchor_confidence {
        Some(r) if r < 0.3 => user_cap.min(20),
        _ => user_cap,
    }
}

/// 强度等比缩放——当 user_cap < source max 时，等比缩放整个区间。
///
/// v1.1 未接入：Pass 3 是桩，没有用户档案，拿不到 user_cap。
/// v1.2+ 接入后——Pass 3 check() 在验证完成时调用此函数缩放强度。
///
/// 例：源码 [15, 60]，用户 cap 45
///     ratio = 45/60 = 0.75
///     输出 [11, 45]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::typeck::TypeChecker;

    fn check_src(src: &str) -> Result<(), AnimiError> {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        let reg = crate::registry::Registry::default();
        let checker = TypeChecker::new(&reg);
        checker.check(&ast)?;
        check(&ast)
    }

    #[test]
    fn safety_stub_passes() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: []
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        assert!(check_src(src).is_ok());
    }

    #[test]
    fn low_anchor_confidence_caps_at_20() {
        // R < 0.3 → cap 20。即使 user_cap = 100。
        assert_eq!(low_anchor_cap(100, Some(0.1)), 20);
        // user_cap 本身就低于 20 → 不变。
        assert_eq!(low_anchor_cap(15, Some(0.1)), 15);
    }

    #[test]
    fn normal_anchor_confidence_no_cap() {
        // R >= 0.3 → 不触发低锚保护。
        assert_eq!(low_anchor_cap(100, Some(0.5)), 100);
        assert_eq!(low_anchor_cap(100, Some(0.9)), 100);
    }

    #[test]
    fn no_anchor_data_no_cap() {
        // None = 无数据 → 不拦截（向后兼容 v0.3）。
        assert_eq!(low_anchor_cap(100, None), 100);
    }

    #[test]
    fn low_anchor_boundary() {
        // R = 0.3 刚好在线上 → 不触发。
        assert_eq!(low_anchor_cap(100, Some(0.3)), 100);
        // R = 0.2999 → 触发。
        assert_eq!(low_anchor_cap(100, Some(0.29999)), 20);
    }

    #[test]
    fn scale_intensity_no_change() {
        let original = Intensity { min: 10, max: 20 };
        let scaled = scale_intensity(&original, 50);
        assert_eq!(scaled.min, 10);
        assert_eq!(scaled.max, 20);
    }

    #[test]
    fn scale_intensity_proportional() {
        let original = Intensity { min: 15, max: 60 };
        let scaled = scale_intensity(&original, 45);
        assert_eq!(scaled.min, 11);
        assert_eq!(scaled.max, 45);
    }

    #[test]
    fn check_with_scale_applies_cap() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: []
    }
    shape: steady
    intensity: [15, 60]
}
"#;
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let scaled = check_with_scale(&ast, 45).unwrap();
        assert_eq!(scaled.min, 11);
        assert_eq!(scaled.max, 45);
    }
}
