// src/safety.rs — Pass 3：用户安全
//
// 对这个人的拒绝。读用户档案——创伤分型、强度 cap、未成年标记。
// 必须在设备本地运行——用户穿戴并开始 Session 时。
//
// v1.1 桩——全部通过。后续版本接入：
//   - 创伤分型交叉判定（v1/v2/v3 × 社交/情绪/躯体）
//   - 强度上限 cap（如未成年 max 35）
//   - 创伤原子黑名单（禁主不禁点）
//   - 触觉维度锁定（如创伤史禁止高强度触觉）

use crate::ast::*;
use crate::error::AnimiError;

/// 对源码执行用户安全规则检查。
///
/// v1.1——全部通过。后续版本需要加载用户档案。
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
}
