// src/parser.rs — Pass 0b：语法分析
//
// Token 流 → AST（FeelingSource）。
// 递归下降解析。语法错误 → oi! 拒绝。
// 不检查类型。不检查安全。只检查"能不能按语法读出来"。

use crate::ast::*;
use crate::error::AnimiError;
use crate::lexer::{Token, TokenKind};
use crate::oi;

/// 语法分析器。
pub struct Parser {
    /// Token 流。
    tokens: Vec<Token>,

    /// 当前正在处理的 Token 位置。
    pos: usize,
}

impl Parser {
    /// 从 Token 流创建语法分析器。
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// 解析 .anim 源码，返回一棵完整的 FeelingSource AST。
    pub fn parse(&mut self) -> Result<FeelingSource, AnimiError> {
        // feeling <name> { ... }
        self.expect_keyword(TokenKind::Feeling, "feeling")?;
        let name = self.expect_identifier("感受包名称")?;

        self.expect(TokenKind::LBrace, "{")?;

        // mix { ... }
        let mix = self.parse_mix()?;

        // shape: <name>
        let shape = self.parse_shape()?;

        // intensity: [<min>, <max>]
        let intensity = self.parse_intensity()?;

        self.expect(TokenKind::RBrace, "}")?;

        // 后面只能有 Eof
        let peek = self.peek();
        if peek.kind != TokenKind::Eof {
            oi!(ParseError, line=peek.line, col=peek.col,
                expected="EOF".to_string(), found=format!("unexpected token {:?}", peek.kind))
        }

        Ok(FeelingSource {
            name,
            mix,
            shape,
            intensity,
        })
    }

    /// 解析 mix { main: <atom> accents: [...] }
    fn parse_mix(&mut self) -> Result<Mix, AnimiError> {
        self.expect_keyword(TokenKind::Mix, "mix")?;
        self.expect(TokenKind::LBrace, "{")?;

        // main: <atom>
        self.expect_keyword(TokenKind::Main, "main")?;
        self.expect(TokenKind::Colon, ":")?;
        let main_name = self.expect_identifier("主旋律感受原子名")?;
        let main = FeelingAtom { name: main_name };

        // accents: [<atom> <ratio>, ...]
        self.expect_keyword(TokenKind::Accents, "accents")?;
        self.expect(TokenKind::Colon, ":")?;
        let accents = self.parse_accent_list()?;

        self.expect(TokenKind::RBrace, "}")?;

        Ok(Mix { main, accents })
    }

    /// 解析点缀列表：accents: [<atom> <ratio>, ...]
    fn parse_accent_list(&mut self) -> Result<Vec<Accent>, AnimiError> {
        self.expect(TokenKind::LBracket, "[")?;

        let mut accents = Vec::new();

        // 空列表 → 直接闭合
        if self.peek().kind == TokenKind::RBracket {
            self.advance(); // 吞掉 ]
            return Ok(accents);
        }

        loop {
            let atom_name = self.expect_identifier("点缀感受原子名")?;
            let ratio_literal = self.expect_number("点缀配比")?;

            let ratio: f64 = ratio_literal
                .parse()
                .map_err(|_| AnimiError::ParseError {
                    line: self.peek().line,
                    col: self.peek().col,
                    expected: "有效的配比数字".to_string(),
                    found: ratio_literal.clone(),
                })?;

            accents.push(Accent {
                atom: FeelingAtom { name: atom_name },
                ratio,
            });

            let peek = self.peek();
            match peek.kind {
                TokenKind::Comma => {
                    self.advance(); // 吞掉 ,
                    // 处理尾逗号：逗号后紧跟 ] → 列表结束
                    if self.peek().kind == TokenKind::RBracket {
                        self.advance(); // 吞掉 ]
                        return Ok(accents);
                    }
                }
                TokenKind::RBracket => {
                    self.advance(); // 吞掉 ]
                    return Ok(accents);
                }
                _ => {
                    oi!(ParseError, line=peek.line, col=peek.col,
                        expected="逗号或 ]".to_string(),
                        found=format!("{:?}", peek.kind))
                }
            }
        }
    }

    /// 解析 shape: <name>
    fn parse_shape(&mut self) -> Result<Shape, AnimiError> {
        self.expect_keyword(TokenKind::Shape, "shape")?;
        self.expect(TokenKind::Colon, ":")?;
        let name = self.expect_identifier("shape 名称")?;
        Ok(Shape { name })
    }

    /// 解析 intensity: [<min>, <max>]
    fn parse_intensity(&mut self) -> Result<Intensity, AnimiError> {
        self.expect_keyword(TokenKind::Intensity, "intensity")?;
        self.expect(TokenKind::Colon, ":")?;
        self.expect(TokenKind::LBracket, "[")?;

        let min = self.parse_int("强度最小值")?;
        self.expect(TokenKind::Comma, ",")?;
        let max = self.parse_int("强度最大值")?;

        self.expect(TokenKind::RBracket, "]")?;

        let intensity = Intensity { min, max };
        intensity
            .validate()
            .map_err(|msg| AnimiError::ParseError {
                line: self.peek().line,
                col: self.peek().col,
                expected: "合法的强度区间（max >= min）".to_string(),
                found: msg,
            })?;

        Ok(intensity)
    }

    // ── 辅助方法 ──────────────────────────────────────────────

    /// 当前 Token，不前进。返回 clone——避免引用临时值。
    fn peek(&self) -> Token {
        self.tokens
            .get(self.pos)
            .cloned()
            .unwrap_or(Token {
                kind: TokenKind::Eof,
                line: 0,
                col: 0,
                literal: String::new(),
            })
    }

    /// 前进一个 Token。返回被吞掉的旧 Token。
    fn advance(&mut self) {
        self.pos += 1;
    }

    /// 当前 Token 必须是 expect 类型，否则 ParseError。
    fn expect(&mut self, kind: TokenKind, expected_name: &str) -> Result<(), AnimiError> {
        let token = self.peek();
        if token.kind == kind {
            self.pos += 1;
            Ok(())
        } else {
            oi!(ParseError, line=token.line, col=token.col,
                expected=expected_name.to_string(),
                found=format!("{:?}", token.kind))
        }
    }

    /// 当前 Token 必须是关键字（TokenKind 匹配），否则 ParseError。
    fn expect_keyword(&mut self, kind: TokenKind, keyword: &str) -> Result<(), AnimiError> {
        let token = self.peek();
        if token.kind == kind {
            self.pos += 1;
            Ok(())
        } else {
            oi!(ParseError, line=token.line, col=token.col,
                expected=format!("keyword `{}`", keyword).to_string(),
                found=format!("{:?}", token.kind))
        }
    }

    /// 当前 Token 必须是 Identifier 并返回其字面值，否则 ParseError。
    fn expect_identifier(&mut self, expected_name: &str) -> Result<String, AnimiError> {
        let token = self.peek().clone();
        if token.kind == TokenKind::Identifier {
            self.pos += 1;
            Ok(token.literal)
        } else {
            oi!(ParseError, line=token.line, col=token.col,
                expected=expected_name.to_string(),
                found=format!("{:?}", token.kind))
        }
    }

    /// 当前 Token 必须是 Int 或 Float 并返回其字面值，否则 ParseError。
    fn expect_number(&mut self, expected_name: &str) -> Result<String, AnimiError> {
        let token = self.peek().clone();
        match token.kind {
            TokenKind::Int | TokenKind::Float => {
                self.pos += 1;
                Ok(token.literal)
            }
            _ => {
                oi!(ParseError, line=token.line, col=token.col,
                    expected=expected_name.to_string(),
                    found=format!("{:?}", token.kind))
            }
        }
    }

    /// 当前 Token 必须是 Int 并返回解析后的 u32 值，否则 ParseError。
    fn parse_int(&mut self, expected_name: &str) -> Result<u32, AnimiError> {
        let token = self.peek().clone();
        if token.kind != TokenKind::Int {
            oi!(ParseError, line=token.line, col=token.col,
                expected=expected_name.to_string(),
                found=format!("{:?}", token.kind))
        }
        self.pos += 1;
        token.literal.parse().map_err(|_| AnimiError::ParseError {
            line: token.line,
            col: token.col,
            expected: expected_name.to_string(),
            found: token.literal,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_source(src: &str) -> Result<FeelingSource, AnimiError> {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn parse_minimal() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [belonging 0.3]
    }
    shape: gradual_rise_fall
    intensity: [15, 45]
}
"#;
        let result = parse_source(src).unwrap();
        assert_eq!(result.name, "calm");
        assert_eq!(result.mix.main.name, "calm_meditative");
        assert_eq!(result.mix.accents.len(), 1);
        assert_eq!(result.mix.accents[0].atom.name, "belonging");
        assert_eq!(result.mix.accents[0].ratio, 0.3);
        assert_eq!(result.shape.name, "gradual_rise_fall");
        assert_eq!(result.intensity.min, 15);
        assert_eq!(result.intensity.max, 45);
    }

    #[test]
    fn parse_multiple_accents() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [belonging 0.3, clarity 0.2, safety 0.1]
    }
    shape: sharp_peak
    intensity: [0, 100]
}
"#;
        let result = parse_source(&src).unwrap();
        assert_eq!(result.mix.accents.len(), 3);
        assert_eq!(result.mix.accents[1].atom.name, "clarity");
        assert_eq!(result.mix.accents[2].ratio, 0.1);
    }

    #[test]
    fn parse_empty_accents() {
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
        let result = parse_source(src).unwrap();
        assert_eq!(result.mix.accents.len(), 0);
    }

    #[test]
    fn parse_trailing_comma() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [belonging 0.3,]
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        let result = parse_source(src).unwrap();
        assert_eq!(result.mix.accents.len(), 1);
    }

    #[test]
    fn parse_missing_feeling_keyword() {
        let src = r#"
calm {
    mix {
        main: calm_meditative
        accents: []
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        let result = parse_source(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("feeling"));
    }

    #[test]
    fn parse_missing_closing_brace() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: []
    }
    shape: steady
    intensity: [10, 20]
"#;
        let result = parse_source(src);
        assert!(result.is_err());
    }

    #[test]
    fn parse_invalid_intensity_order() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: []
    }
    shape: steady
    intensity: [45, 15]
}
"#;
        let result = parse_source(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max"));
    }

    #[test]
    fn parse_accent_without_ratio() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [belonging]
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        let result = parse_source(src);
        assert!(result.is_err());
    }

    #[test]
    fn parse_extra_content_after_close() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: []
    }
    shape: steady
    intensity: [10, 20]
}
extra garbage
"#;
        let result = parse_source(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("EOF"));
    }
}
