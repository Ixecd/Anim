// src/rule.rs — Pass 2：静态安全规则
//
// 对任何人的硬规则。不读用户档案。不依赖个人基准。
// 可在设备离线时运行、Server 端预编译、缓存复用。
//
// v1.1 实现：
//   1. 全局强度上限 100（默认刻度）
//   2. abrupt_stop 形状强度不超过 20
//   3. 沙箱原子不能做主旋律，点缀配比 ≤ 0.3

use crate::ast::*;
use crate::error::AnimiError;
use crate::oi;
use crate::registry::{self, AtomClass};

/// 全局强度的推荐上限。
pub const MAX_GLOBAL_INTENSITY: u32 = 100;

/// abrupt_stop 形状的最大允许强度。
const ABRUPT_STOP_MAX: u32 = 20;

/// 沙箱原子作为点缀时的配比上限。
const SANDBOX_ACCENT_MAX: f64 = 0.3;

/// 对源码执行静态安全规则检查。
pub fn check(source: &FeelingSource) -> Result<(), AnimiError> {
    // 规则 1：全局强度上限
    if source.intensity.max > MAX_GLOBAL_INTENSITY {
        oi!(
            StaticSafetyError,
            rule = "全局强度上限".into(),
            detail = format!(
                "源码强度 max={} 超过全局上限 {}. 请等比缩小强度区间",
                source.intensity.max, MAX_GLOBAL_INTENSITY
            )
        )
    }

    // 规则 2：abrupt_stop 形状强度限制
    if source.shape.name == "abrupt_stop" && source.intensity.max > ABRUPT_STOP_MAX {
        oi!(
            StaticSafetyError,
            rule = "abrupt_stop 强度限制".into(),
            detail = format!(
                "abrupt_stop 形状的强度不能超过 {}. 当前 max={}",
                ABRUPT_STOP_MAX, source.intensity.max
            )
        )
    }

    // 规则 3：沙箱原子不能作为主旋律
    if let Some(entry) = registry::lookup_atom(&source.mix.main.name) {
        if entry.class == AtomClass::Sandbox {
            oi!(
                StaticSafetyError,
                rule = "沙箱原子限制".into(),
                detail = format!(
                    "沙箱原子 '{}' 不能作为主旋律。只能作为点缀使用",
                    source.mix.main.name
                )
            )
        }
    }

    // 规则 4：沙箱原子点缀配比上限
    for accent in &source.mix.accents {
        if let Some(entry) = registry::lookup_atom(&accent.atom.name) {
            if entry.class == AtomClass::Sandbox && accent.ratio > SANDBOX_ACCENT_MAX {
                oi!(
                    StaticSafetyError,
                    rule = "沙箱配比上限".into(),
                    detail = format!(
                        "沙箱原子 '{}' 作为点缀的配比 {:.2} 超过上限 {:.2}",
                        accent.atom.name, accent.ratio, SANDBOX_ACCENT_MAX
                    )
                )
            }
        }
    }

    Ok(())
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
        let checker = TypeChecker::new();
        checker.check(&ast)?;
        check(&ast)
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
}
