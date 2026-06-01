// src/guard.rs — Pass 4：运行期插桩预埋
//
// 交织期预埋——运行期激活。不在这里判断。
// 真正的判断在 FPGA 端由寄存器比较完成。
// 这一层做的事——生成栅栏代码——不是执行栅栏。
//
// v1.1 桩——全部通过。后续版本接入：
//   - ESIR 层插入 safety_pin 字段
//   - 硬件衰减状态机预配置
//   - 实时闭环偏差阈值预埋
//   - 看门狗复位条件注入

use crate::ast::*;
use crate::error::AnimiError;

/// 向 AST 注入运行期安全栅栏。
///
/// v1.1——全部通过。后续版本需要生成 ESIR 插桩代码。
pub fn inject(_source: &FeelingSource) -> Result<(), AnimiError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::typeck::TypeChecker;

    fn inject_src(src: &str) -> Result<(), AnimiError> {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        let reg = crate::registry::Registry::default();
        let checker = TypeChecker::new(&reg);
        checker.check(&ast)?;
        inject(&ast)
    }

    #[test]
    fn guard_stub_passes() {
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
        assert!(inject_src(src).is_ok());
    }
}
