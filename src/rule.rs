// src/rule.rs — Pass 2：静态安全规则
//
// 对任何人的硬规则。不读用户档案。不依赖个人基准。
// 可在设备离线时运行、Server 端预交织、缓存复用。
//
// v1.1 实现：
//   1. 全局强度上限（config.caps.global_intensity）
//   2. abrupt_stop 形状强度不超过 config.caps.abrupt_stop_max
//   3. 沙箱原子不能做主旋律，点缀配比 ≤ config.caps.sandbox_accent_max

use crate::ast::*;
use crate::config::AnimConfig;
use crate::error::AnimiError;
use crate::oi;
use crate::oi_warn;
use crate::registry::{AtomClass, Registry};

/// 对源码执行静态安全规则检查。
pub fn check(
    source: &FeelingSource,
    registry: &Registry,
    config: &AnimConfig,
) -> Result<(), AnimiError> {
    // 规则 1：全局强度上限
    if source.intensity.max > config.caps.global_intensity {
        oi!(
            StaticSafetyError,
            rule = "全局强度上限".into(),
            detail = format!(
                "源码强度 max={} 超过全局上限 {}. 请在源码中将 max 降到 {} 以下",
                source.intensity.max, config.caps.global_intensity, config.caps.global_intensity
            )
        )
    }

    // 规则 2：abrupt_stop 形状强度——降级为 Warn（设计建议，不是物理硬线）
    if source.shape.name == "abrupt_stop" && source.intensity.max > config.caps.abrupt_stop_max {
        oi_warn!(
            StaticSafetyError,
            rule = "abrupt_stop 强度限制".into(),
            detail = format!(
                "abrupt_stop 形状的强度建议不超过 {}. 当前 max={}",
                config.caps.abrupt_stop_max, source.intensity.max
            )
        )
    }

    // 规则 3：沙箱原子不能作为主旋律
    if let Some(entry) = registry.lookup(&source.mix.main.name) {
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
        if let Some(entry) = registry.lookup(&accent.atom.name) {
            if entry.class == AtomClass::Sandbox && accent.ratio > config.caps.sandbox_accent_max {
                oi_warn!(
                    StaticSafetyError,
                    rule = "沙箱配比上限".into(),
                    detail = format!(
                        "沙箱原子 '{}' 作为点缀的建议配比 {:.2} 超过上限 {:.2}",
                        accent.atom.name, accent.ratio, config.caps.sandbox_accent_max
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
        let reg = Registry::default();
        let cfg = crate::config::AnimConfig::default();
        let checker = TypeChecker::new(&reg);
        checker.check(&ast)?;
        check(&ast, &reg, &cfg)
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
