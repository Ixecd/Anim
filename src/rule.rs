// src/rule.rs — Pass 2：静态安全规则
//
// 对任何人的硬规则。不读用户档案。不依赖个人基准。
// 可在设备离线时运行、Server 端预编译、缓存复用。
//
// v1.1 实现基础三条：
//   1. 全局强度上限 100（默认刻度）
//   2. abrupt_stop 形状强度不超过 20
//   3. 沙箱原子独立校验——不允许主旋律，点缀上限 ≤ 0.3
//
// v1.2+ 逐步接入：
//   - 组合爆炸防护
//   - 点缀配比精调（如 grief ≤ 0.20）
//   - 强度等比缩放（需要用户 cap——那是 Pass 3 的事）

use crate::ast::*;
use crate::error::AnimiError;
use crate::oi;

/// 全局强度的推荐上限。
pub const MAX_GLOBAL_INTENSITY: u32 = 100;

/// abrupt_stop 形状的最大允许强度。
const ABRUPT_STOP_MAX: u32 = 20;

/// 沙箱原子作为点缀时的配比上限。
const SANDBOX_ACCENT_MAX: f64 = 0.3;

/// 对源码执行静态安全规则检查。
pub fn check(source: &FeelingSource, is_core_registry: &dyn Fn(&str) -> Option<bool>) -> Result<(), AnimiError> {
    // 规则 1：全局强度上限
    if source.intensity.max > MAX_GLOBAL_INTENSITY {
        oi!(StaticSafetyError,
            rule="全局强度上限".into(),
            detail=format!(
                "源码强度 max={} 超过全局上限 {}. 请等比缩小强度区间",
                source.intensity.max, MAX_GLOBAL_INTENSITY
            )
        )
    }

    // 规则 2：abrupt_stop 形状强度限制
    if source.shape.name == "abrupt_stop" && source.intensity.max > ABRUPT_STOP_MAX {
        oi!(StaticSafetyError,
            rule="abrupt_stop 强度限制".into(),
            detail=format!(
                "abrupt_stop 形状的强度不能超过 {}. 当前 max={}. 强度 > 20 的突停需要知情同意",
                ABRUPT_STOP_MAX, source.intensity.max
            )
        )
    }

    // 规则 3：沙箱原子不能作为主旋律，点缀配比 ≤ 0.3
    let main_is_core = is_core_registry(&source.mix.main.name).unwrap_or(false);
    if !main_is_core {
        oi!(StaticSafetyError,
            rule="沙箱原子限制".into(),
            detail=format!(
                "沙箱原子 '{}' 不能作为主旋律。只能作为点缀使用",
                source.mix.main.name
            )
        )
    }

    for accent in &source.mix.accents {
        let is_core = is_core_registry(&accent.atom.name).unwrap_or(true);
        if !is_core && accent.ratio > SANDBOX_ACCENT_MAX {
            oi!(StaticSafetyError,
                rule="沙箱配比上限".into(),
                detail=format!(
                    "沙箱原子 '{}' 作为点缀的配比 {:.2} 超过上限 {:.2}",
                    accent.atom.name, accent.ratio, SANDBOX_ACCENT_MAX
                )
            )
        }
    }

    Ok(())
}

/// 强度等比缩放——当 user_cap < source max 时，不等比截断，而是等比缩放整个区间。
///
/// 例：源码 [15, 60]，用户 cap 45
///     ratio = 45/60 = 0.75
///     输出 [11, 45]
///
/// 这是 Pass 3（UserStateSafety）的辅助函数，放在这里是因为它属于规则层逻辑。
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
    use std::sync::LazyLock;

    /// v1.1 测试用核心 Registry——所有原子都是核心原子
    static CORE_REG: LazyLock<fn(&str) -> Option<bool>> = LazyLock::new(|| {
        |name: &str| -> Option<bool> {
            let core_atoms = [
                "calm_meditative", "belonging", "clarity", "safety",
                "post_achievement", "gentle_focus", "deep_rest", "warmth",
            ];
            Some(core_atoms.contains(&name))
        }
    });

    fn check_src(src: &str) -> Result<(), AnimiError> {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        let checker = TypeChecker::new();
        checker.check(&ast)?;
        check(&ast, &*CORE_REG)
    }

    #[test]
    fn rule_stub_passes() {
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
    fn global_intensity_cap_rejected() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: []
    }
    shape: steady
    intensity: [10, 150]
}
"#;
        let result = check_src(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("150"));
    }

    #[test]
    fn abrupt_stop_over_20_rejected() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: []
    }
    shape: abrupt_stop
    intensity: [10, 35]
}
"#;
        let result = check_src(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("abrupt_stop"));
    }

    #[test]
    fn scale_intensity_no_change() {
        let original = Intensity { min: 10, max: 20 };
        let scaled = scale_intensity(&original, 50);
        assert_eq!(scaled.min, 10);
        assert_eq!(scaled.max, 20); // cap > max → 不变
    }

    #[test]
    fn scale_intensity_proportional() {
        let original = Intensity { min: 15, max: 60 };
        let scaled = scale_intensity(&original, 45);
        // ratio = 45/60 = 0.75
        // min = 15 * 0.75 = 11.25 → 11
        assert_eq!(scaled.min, 11);
        assert_eq!(scaled.max, 45);
    }
}
