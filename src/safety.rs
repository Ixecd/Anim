// src/safety.rs — Pass 3：用户安全
//
// 对这个人的拒绝。读用户档案——创伤分型、强度 cap、未成年标记。
// 必须在设备本地运行——用户穿戴并开始 Session 时。
//
// v1.1 桩——全部通过。后续版本接入：
//   - 强度等比缩放（scale_intensity）
//   - 创伤分型交叉判定（v1/v2/v3 × 社交/情绪/躯体）
//   - 强度上限 cap（如未成年 max 35）
//   - 创伤原子黑名单（禁主不禁点）
//   - 触觉维度锁定

use crate::ast::*;
use crate::error::AnimiError;

/// 对源码执行用户安全规则检查。
///
/// v1.1——全部通过。后续版本需要加载用户档案。
pub fn check(_source: &FeelingSource) -> Result<(), AnimiError> {
    Ok(())
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
}
