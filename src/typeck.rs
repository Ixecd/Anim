// src/typeck.rs — Pass 1：类型检查
//
// 验证 AST 中的感受原子名和 shape 名是否在 Registry 中存在。
// 验证点缀配比是否 ≤ 该原子的 max_ratio。
//
// v1.1 使用内建 Registry（registry.rs）。
// v1.2+ 将从外部 JSON/YAML 加载。

use crate::ast::*;
use crate::error::AnimiError;
use crate::oi;
use crate::registry;

/// 类型检查器。
pub struct TypeChecker;

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker
    }

    /// 对一份 FeelingSource AST 执行类型检查。
    ///
    /// 验证内容：
    /// - 主旋律感受原子名是否在 Registry 中
    /// - 点缀感受原子名是否在 Registry 中
    /// - 点缀配比是否 ≤ 该原子的 max_ratio
    /// - shape 名称是否合法
    /// - 强度区间是否合法（max >= min + 非零）
    pub fn check(&self, source: &FeelingSource) -> Result<(), AnimiError> {
        // 主旋律
        self.check_atom(&source.mix.main, "主旋律")?;

        // 点缀——附加 max_ratio 检查
        for accent in &source.mix.accents {
            let max_r = self.check_atom(&accent.atom, "点缀")?;

            if accent.ratio > max_r {
                oi!(
                    TypeCheckError,
                    atom_name = accent.atom.name.clone(),
                    reason = format!(
                        "点缀配比 {:.2} 超过了该原子的上限 {:.2}",
                        accent.ratio, max_r
                    )
                )
            }
        }

        // shape
        self.lookup_name(&source.shape.name, registry::shape_names(), "shape")?;

        // 强度区间——parser 已校验 max >= min，rule.rs 校验全局上限 100
        if source.intensity.max == 0 && source.intensity.min == 0 {
            oi!(
                TypeCheckError,
                atom_name = source.name.clone(),
                reason = "强度不能为零——信号没有强度等于没生成".to_string()
            )
        }

        Ok(())
    }

    /// 泛型名称查找，带"你是不是想说 X"建议。
    fn lookup_name<'a>(
        &self,
        name: &str,
        values: impl Iterator<Item = &'a str>,
        role: &str,
    ) -> Result<(), AnimiError> {
        let all: Vec<&str> = values.collect();
        if all.contains(&name) {
            return Ok(());
        }

        let suggestions: Vec<&&str> = all
            .iter()
            .filter(|r| {
                let prefix = r
                    .chars()
                    .zip(name.chars())
                    .take_while(|(a, b)| a == b)
                    .count();
                prefix >= 3 || (name.len() >= 3 && r.contains(name))
            })
            .collect();

        let hint = if suggestions.is_empty() {
            String::new()
        } else {
            format!(
                " 你是不是想说: {}",
                suggestions
                    .iter()
                    .map(|s| **s)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        oi!(
            TypeCheckError,
            atom_name = name.to_string(),
            reason = format!("未注册的 {}。{}", role, hint)
        )
    }

    /// 检查感受原子名是否在 Registry 中，返回 max_ratio。
    fn check_atom(&self, atom: &FeelingAtom, role: &str) -> Result<f64, AnimiError> {
        if let Some(entry) = registry::lookup_atom(&atom.name) {
            return Ok(entry.max_ratio);
        }

        self.lookup_name(&atom.name, registry::atom_names(), role)?;
        unreachable!()
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
        assert!(result.unwrap_err().to_string().contains("explosion"));
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("nuclear_explosion"));
    }

    #[test]
    fn ratio_exceeds_max_ratio_rejected() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [belonging 0.8]
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        let result = check_source(src);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("belonging"));
        assert!(msg.contains("0.80"));
        assert!(msg.contains("0.50"));
    }

    #[test]
    fn ratio_at_max_ratio_passes() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [belonging 0.5]
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        assert!(check_source(src).is_ok());
    }

    #[test]
    fn ratio_zero_passes() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [belonging 0.0]
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        assert!(check_source(src).is_ok());
    }

    #[test]
    fn ratio_one_passes() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [calm_meditative 1.0]
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        assert!(check_source(src).is_ok());
    }

    #[test]
    fn ratio_over_one_rejected_by_parser() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [belonging 1.5]
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        let result = check_source(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("1.0"));
    }

    #[test]
    fn ratio_negative_rejected_by_lexer() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [belonging -0.3]
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        let result = check_source(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("-"));
    }

    #[test]
    fn intensity_zero_zero_rejected() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: []
    }
    shape: steady
    intensity: [0, 0]
}
"#;
        let result = check_source(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不能为零"));
    }

    #[test]
    fn intensity_100_100_passes() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: []
    }
    shape: steady
    intensity: [100, 100]
}
"#;
        assert!(check_source(src).is_ok());
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

    #[test]
    fn uppercase_keyword_rejected() {
        let src = r#"
FEELING calm {
    mix {
        main: calm_meditative
        accents: []
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        let result = check_source(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("feeling"));
    }

    #[test]
    fn unicode_rejected_by_lexer() {
        // ASCII-only lexer → 中文字符被拒绝
        let src = "feeling calm_平";
        let result = check_source(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("平"));
    }

    #[test]
    fn empty_mix_block_rejected() {
        let src = r#"
feeling calm {
    mix {
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        let result = check_source(src);
        assert!(result.is_err());
    }

    #[test]
    fn keyword_as_identifier_rejected() {
        let src = r#"
feeling calm {
    mix {
        main: mix
        accents: []
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        let result = check_source(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Mix"));
    }
}
