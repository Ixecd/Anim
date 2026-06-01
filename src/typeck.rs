// src/typeck.rs — Pass 1：类型检查
//
// 验证 AST 中的感受原子名是否在 Pattern Registry 中存在。
// 验证 shape 名称是否合法。
// v1.1 临时——硬编码几个注册原子名和 shape。
// 不存在的原子 → oi! 拒绝。这帧不生成任何 IR。

use crate::ast::*;
use crate::error::AnimiError;
use crate::oi;

/// 类型检查器。
///
/// 当前 v1.1 使用内建 Registry。
/// 未来 v1.2+ 将从 Feelings-Pattern 仓库加载 Registry。
pub struct TypeChecker {
    /// 注册的感受原子名集合。
    registry: Vec<String>,

    /// 注册的 shape 名集合。
    shapes: Vec<String>,
}

impl TypeChecker {
    /// 创建 v1.1 内建 Registry 的类型检查器。
    pub fn new() -> Self {
        TypeChecker {
            registry: vec![
                "calm_meditative".into(),
                "belonging".into(),
                "clarity".into(),
                "safety".into(),
                "post_achievement".into(),
                "gentle_focus".into(),
                "deep_rest".into(),
                "warmth".into(),
            ],
            shapes: vec![
                "gradual_rise_fall".into(),
                "sharp_peak".into(),
                "steady".into(),
                "slow_decay".into(),
                "wave".into(),
            ],
        }
    }

    /// 对一份 FeelingSource AST 执行类型检查。
    ///
    /// 验证内容：
    /// - 主旋律感受原子名是否在 Registry 中
    /// - 点缀感受原子名是否在 Registry 中
    /// - shape 名称是否合法
    /// - 强度区间是否合法（max >= min）
    ///
    /// 全部通过 → Ok(())。
    /// 任何不通过 → oi!(TypeCheckError, ...)。这帧不生成。
    pub fn check(&self, source: &FeelingSource) -> Result<(), AnimiError> {
        // 主旋律
        self.check_atom(&source.mix.main, "主旋律")?;

        // 点缀
        for accent in &source.mix.accents {
            self.check_atom(&accent.atom, "点缀")?;
        }

        // shape
        if !self.shapes.contains(&source.shape.name) {
            oi!(TypeCheckError,
                atom_name=source.shape.name.clone(),
                reason=format!(
                    "未注册的 shape 名称。可用: {}",
                    self.shapes.join(", ")
                )
            )
        }

        // 强度区间
        source.intensity.validate().map_err(|msg| {
            AnimiError::TypeCheckError {
                atom_name: source.name.clone(),
                reason: msg,
            }
        })?;

        Ok(())
    }

    /// 检查单个感受原子名是否在 Registry 中。
    fn check_atom(&self, atom: &FeelingAtom, role: &str) -> Result<(), AnimiError> {
        if !self.registry.contains(&atom.name) {
            // 找编辑距离最近的候选项（简单版：找包含相同前缀的）
            let suggestions: Vec<&String> = self
                .registry
                .iter()
                .filter(|r| {
                    // 简单的相似度：有共同前缀或至少 3 个字符重叠
                    let common = r.chars().zip(atom.name.chars()).take_while(|(a, b)| a == b).count();
                    common >= 3
                })
                .collect();

            let hint = if suggestions.is_empty() {
                String::new()
            } else {
                format!(" 你是不是想说: {}", suggestions.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "))
            };

            oi!(TypeCheckError,
                atom_name=atom.name.clone(),
                reason=format!("未注册的感受原子（{}）。{}", role, hint)
            )
        }
        Ok(())
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn check_source(src: &str) -> Result<(), AnimiError> {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        let checker = TypeChecker::new();
        checker.check(&ast)
    }

    #[test]
    fn valid_atoms_pass() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [belonging 0.3, clarity 0.2]
    }
    shape: gradual_rise_fall
    intensity: [15, 45]
}
"#;
        assert!(check_source(src).is_ok());
    }

    #[test]
    fn unregistered_main_rejected() {
        let src = r#"
feeling calm {
    mix {
        main: explosion
        accents: [belonging 0.3]
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        let result = check_source(src);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("explosion"));
        assert!(msg.contains("未注册"));
    }

    #[test]
    fn unregistered_accent_rejected() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [rage 0.5]
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        let result = check_source(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("rage"));
    }

    #[test]
    fn unregistered_shape_rejected() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: []
    }
    shape: nuclear_explosion
    intensity: [10, 20]
}
"#;
        let result = check_source(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("nuclear_explosion"));
    }

    #[test]
    fn invalid_intensity_range_rejected() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: []
    }
    shape: steady
    intensity: [100, 10]
}
"#;
        let result = check_source(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("100"));
    }
}
