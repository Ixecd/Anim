// src/rule.rs — Pass 2：静态安全规则
//
// 对任何人的硬规则。不读用户档案。不依赖个人基准。
// 可在设备离线时运行、Server 端预编译、缓存复用。
//
// v1.1 桩——全部通过。后续版本接入：
//   - 强度等比缩放上限
//   - 点缀配比硬上限（如 grief 类原子点缀 ≤ 0.20）
//   - 组合爆炸防护（同包内主旋律+点缀总数 ≤ N）
//   - shape 约束（如 sharp_peak 不允许高强度区间）

use crate::ast::*;
use crate::error::AnimiError;

/// 对源码执行静态安全规则检查。
///
/// v1.1——全部通过。后续版本逐步启用。
pub fn check(_source: &FeelingSource) -> Result<(), AnimiError> {
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
}
