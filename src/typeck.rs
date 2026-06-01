// src/typeck.rs — Pass 1：类型检查
//
// 验证 AST 中的感受原子名是否在 Pattern Registry 中存在。
// 验证 shape 名称是否合法。
// v1.1 临时——硬编码几个注册原子名和 shape。
// 不存在的原子 → oi! 拒绝。这帧不生成任何 IR。

use crate::ast::*;
use crate::error::AnimiError;
use crate::oi;

/// Registry 中的一个感受原子条目。
struct AtomEntry {
    name: String,
    /// 作为点缀时的最大配比。0.0-1.0。
    /// 主旋律不受此限制。
    max_ratio: f64,
}

/// 类型检查器。
///
/// 当前 v1.1 使用内建 Registry。
/// 未来 v1.2+ 将从 Feelings-Pattern 仓库加载 Registry。
pub struct TypeChecker {
    /// 注册的感受原子列表。
    registry: Vec<AtomEntry>,

    /// 注册的 shape 名集合。
    shapes: Vec<String>,
}

impl TypeChecker {
    /// 创建 v1.1 内建 Registry 的类型检查器。
    pub fn new() -> Self {
        TypeChecker {
            registry: vec![
                AtomEntry { name: "calm_meditative".into(), max_ratio: 1.0 },
                AtomEntry { name: "belonging".into(), max_ratio: 0.5 },
                AtomEntry { name: "clarity".into(), max_ratio: 0.5 },
                AtomEntry { name: "safety".into(), max_ratio: 0.5 },
                AtomEntry { name: "post_achievement".into(), max_ratio: 0.3 },
                AtomEntry { name: "gentle_focus".into(), max_ratio: 0.5 },
                AtomEntry { name: "deep_rest".into(), max_ratio: 0.5 },
                AtomEntry { name: "warmth".into(), max_ratio: 0.5 },
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
    /// - 点缀配比是否 ≤ 该原子的 max_ratio
    /// - shape 名称是否合法
    /// - 强度区间是否合法（max >= min）
    ///
    /// 全部通过 → Ok(())。
    /// 任何不通过 → oi!(TypeCheckError, ...)。这帧不生成。
    pub fn check(&self, source: &FeelingSource) -> Result<(), AnimiError> {
        // 主旋律
        self.check_atom(&source.mix.main, "主旋律")?;

        // 点缀——附加 max_ratio 检查
        for accent in &source.mix.accents {
            let entry = self.check_atom(&accent.atom, "点缀")?;

            if accent.ratio > entry.max_ratio {
                oi!(TypeCheckError,
                    atom_name=accent.atom.name.clone(),
                    reason=format!(
                        "点缀配比 {:.2} 超过了该原子的上限 {:.2}",
                        accent.ratio, entry.max_ratio
                    )
                )
            }
        }

        // shape——用同一个 lookup 逻辑，给"你是不是想说 X"建议
        self.lookup_name(&source.shape.name, self.shapes.iter().map(|s| s.as_str()), "shape")?;

        // 强度区间——parser 已校验 max >= min，这里只做语义检查
        if source.intensity.max == 0 && source.intensity.min == 0 {
            oi!(TypeCheckError,
                atom_name=source.name.clone(),
                reason="强度不能为零——信号没有强度等于没生成".to_string()
            )
        }
        if source.intensity.max > 10000 {
            oi!(TypeCheckError,
                atom_name=source.name.clone(),
                reason=format!(
                    "强度 max={} 过大——默认刻度 0-100。如确认无误请使用等比例缩小或将数值单位改为 0-10000。上限 10000。DSIR 阶段无法映射这么高的值",
                    source.intensity.max
                )
            )
        }

        Ok(())
    }

    /// 泛型名称查找——对原子和 shape 通用。
    /// 不在 values 中 → oi!(TypeCheckError)，带编辑距离建议。
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

        // 前缀匹配 ≥ 3 或子串包含
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

        oi!(TypeCheckError,
            atom_name=name.to_string(),
            reason=format!("未注册的 {}。{}", role, hint)
        )
    }

    /// 检查单个感受原子名是否在 Registry 中。
    /// 返回对应的 AtomEntry 引用——供调用方做额外检查（如 max_ratio）。
    fn check_atom(&self, atom: &FeelingAtom, role: &str) -> Result<&AtomEntry, AnimiError> {
        for entry in &self.registry {
            if entry.name == atom.name {
                return Ok(entry);
            }
        }

        // 未找到——走泛型 lookup 报错（带"你是不是想说 X"建议）
        self.lookup_name(&atom.name, self.registry.iter().map(|e| e.name.as_str()), role)?;

        // Rust 看不到 oi! 的 return——此行为不可达
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
    fn uppercase_keyword_rejected() {
        // FEELING 不是关键字——lexer 当 Identifier，parser 期望 feeling 关键字 → 报错
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
    fn unicode_identifier_passes() {
        // calm_平静——is_alphanumeric 接受中文
        let src = r#"
feeling calm_平静 {
    mix {
        main: calm_meditative
        accents: []
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        assert!(check_source(src).is_ok());
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
        // main: mix——mix 是关键字，parser 期望 Identifier 但 TokenKind 是 Mix
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
        // parser 打印的是 TokenKind::Mix 的 Debug 格式 = "Mix"
        assert!(result.unwrap_err().to_string().contains("Mix"));
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
    fn ratio_exceeds_max_ratio_rejected() {
        // belonging max_ratio = 0.5，0.8 超了
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
        // belonging max_ratio = 0.5，正好 0.5 → 通过
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
    fn ratio_negative_rejected_by_lexer() {
        // - 不是 .anim 的合法字符——词法分析器直接拒绝
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
        // calm_meditative.max_ratio=1.0，1.0 刚好卡上限 → 通过
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
