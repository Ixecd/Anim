// src/lexer.rs — Pass 0a：词法分析
//
// 把 .anim 源码切成 Token 流。
// 不关心语法。不关心语义。只关心"这一块是什么东西"。

use crate::error::AnimiError;
use crate::oi;

/// 源码中的一个词法单元。
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// Token 类型。
    pub kind: TokenKind,

    /// 在源码中的行号（1-indexed）。
    pub line: usize,

    /// 在源码中的列号（1-indexed）。
    pub col: usize,

    /// 字面值。仅 Identifier/Int/Float 有值。
    pub literal: String,
}

/// Token 的种类。
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // 关键字
    Feeling,   // feeling
    Mix,       // mix
    Main,      // main
    Accents,   // accents
    Shape,     // shape
    Intensity, // intensity

    // 字面量
    Identifier, // 感受原子名、shape 名等
    Int,        // 整数（强度值）
    Float,      // 浮点数（点缀配比）

    // 标点
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]
    Colon,    // :
    Comma,    // ,

    // 特殊
    Eof, // 文件结束
}

/// 词法分析器。
pub struct Lexer {
    /// 源码字符数组。
    chars: Vec<char>,

    /// 当前位置。
    pos: usize,

    /// 当前行号。
    line: usize,

    /// 当前列号。
    col: usize,
}

impl Lexer {
    /// 从源码字符串创建词法分析器。
    pub fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    /// 把全部源码切成 Token 流。
    ///
    /// 返回 Token 列表或词法错误。
    pub fn tokenize(&mut self) -> Result<Vec<Token>, AnimiError> {
        let mut tokens = Vec::new();

        while !self.is_eof() {
            self.skip_whitespace();
            if self.is_eof() {
                break;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            line: self.line,
            col: self.col,
            literal: String::new(),
        });

        Ok(tokens)
    }

    /// 读取下一个 Token。
    fn next_token(&mut self) -> Result<Token, AnimiError> {
        let ch = self.peek();

        match ch {
            '{' => self.simple(TokenKind::LBrace),
            '}' => self.simple(TokenKind::RBrace),
            '[' => self.simple(TokenKind::LBracket),
            ']' => self.simple(TokenKind::RBracket),
            ':' => self.simple(TokenKind::Colon),
            ',' => self.simple(TokenKind::Comma),
            ch if ch.is_alphabetic() || ch == '_' => self.ident_or_keyword(),
            ch if ch.is_ascii_digit() || ch == '.' => self.number(),
            _ => {
                let line = self.line;
                let col = self.col;
                oi!(
                    LexError,
                    line = line,
                    col = col,
                    msg = format!("unexpected character: '{}'", ch)
                )
            }
        }
    }

    /// 单字符 Token——直接前进，不需要字面值。
    fn simple(&mut self, kind: TokenKind) -> Result<Token, AnimiError> {
        let line = self.line;
        let col = self.col;
        self.advance();
        Ok(Token {
            kind,
            line,
            col,
            literal: String::new(),
        })
    }

    /// 标识符或关键字。
    fn ident_or_keyword(&mut self) -> Result<Token, AnimiError> {
        let line = self.line;
        let col = self.col;
        let name = self.read_while(|ch| ch.is_alphanumeric() || ch == '_');

        let kind = match name.as_str() {
            "feeling" => TokenKind::Feeling,
            "mix" => TokenKind::Mix,
            "main" => TokenKind::Main,
            "accents" => TokenKind::Accents,
            "shape" => TokenKind::Shape,
            "intensity" => TokenKind::Intensity,
            _ => TokenKind::Identifier,
        };

        Ok(Token {
            kind,
            line,
            col,
            literal: name,
        })
    }

    /// 整数或浮点数。
    fn number(&mut self) -> Result<Token, AnimiError> {
        let line = self.line;
        let col = self.col;

        let int_part = self.read_while(|ch| ch.is_ascii_digit());

        // 遇到小数点 → 浮点数
        if self.peek() == '.' {
            self.advance(); // 吞掉 '.'
            let frac_part = self.read_while(|ch| ch.is_ascii_digit());

            // 孤立小数点——既没有整数部分也没有小数部分 → 非法字符
            if int_part.is_empty() && frac_part.is_empty() {
                oi!(
                    LexError,
                    line = line,
                    col = col,
                    msg = "isolated '.' is not a valid number".to_string()
                )
            }

            let literal = format!("{}.{}", int_part, frac_part);
            return Ok(Token {
                kind: TokenKind::Float,
                line,
                col,
                literal,
            });
        }

        Ok(Token {
            kind: TokenKind::Int,
            line,
            col,
            literal: int_part,
        })
    }

    // ── 辅助方法 ──────────────────────────────────────────────

    /// 当前字符，不前进。
    fn peek(&self) -> char {
        self.chars.get(self.pos).copied().unwrap_or('\0')
    }

    /// 前进一个字符。
    fn advance(&mut self) {
        if self.pos < self.chars.len() {
            if self.chars[self.pos] == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            self.pos += 1;
        }
    }

    /// 是否到达文件末尾。
    fn is_eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    /// 读取满足条件的连续字符。
    fn read_while(&mut self, pred: fn(char) -> bool) -> String {
        let mut s = String::new();
        while !self.is_eof() && pred(self.peek()) {
            s.push(self.peek());
            self.advance();
        }
        s
    }

    /// 跳过空白和注释（-- 到行尾）。
    fn skip_whitespace(&mut self) {
        while !self.is_eof() {
            let ch = self.peek();
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '-' && self.peek_next() == Some('-') {
                // 行注释：-- 到行尾
                while !self.is_eof() && self.peek() != '\n' {
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    /// 看下一个字符但不前进。
    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_keywords() {
        let src = "feeling mix main accents shape intensity";
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();

        let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Feeling,
                TokenKind::Mix,
                TokenKind::Main,
                TokenKind::Accents,
                TokenKind::Shape,
                TokenKind::Intensity,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn tokenize_identifiers() {
        let src = "calm_meditative belonging clarity";
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[0].literal, "calm_meditative");
        assert_eq!(tokens[1].literal, "belonging");
        assert_eq!(tokens[2].literal, "clarity");
    }

    #[test]
    fn tokenize_numbers() {
        let src = "15 45 0.3";
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Int);
        assert_eq!(tokens[0].literal, "15");
        assert_eq!(tokens[1].kind, TokenKind::Int);
        assert_eq!(tokens[1].literal, "45");
        assert_eq!(tokens[2].kind, TokenKind::Float);
        assert_eq!(tokens[2].literal, "0.3");
    }

    #[test]
    fn tokenize_punctuation() {
        let src = "{ } [ ] : ,";
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();

        let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::LBrace,
                TokenKind::RBrace,
                TokenKind::LBracket,
                TokenKind::RBracket,
                TokenKind::Colon,
                TokenKind::Comma,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn tokenize_full_example() {
        let src = r#"
feeling calm_meditative {
    mix {
        main: calm_meditative
        accents: [belonging 0.3, clarity 0.2]
    }
    shape: gradual_rise_fall
    intensity: [15, 45]
}
"#;
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();

        // 验证 token 种类序列
        let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Feeling,
                TokenKind::Identifier, // calm_meditative
                TokenKind::LBrace,
                TokenKind::Mix,
                TokenKind::LBrace,
                TokenKind::Main,
                TokenKind::Colon,
                TokenKind::Identifier, // calm_meditative
                TokenKind::Accents,
                TokenKind::Colon,
                TokenKind::LBracket,
                TokenKind::Identifier, // belonging
                TokenKind::Float,      // 0.3
                TokenKind::Comma,
                TokenKind::Identifier, // clarity
                TokenKind::Float,      // 0.2
                TokenKind::RBracket,
                TokenKind::RBrace,
                TokenKind::Shape,
                TokenKind::Colon,
                TokenKind::Identifier, // gradual_rise_fall
                TokenKind::Intensity,
                TokenKind::Colon,
                TokenKind::LBracket,
                TokenKind::Int, // 15
                TokenKind::Comma,
                TokenKind::Int, // 45
                TokenKind::RBracket,
                TokenKind::RBrace,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_with_comments() {
        let src = r#"
-- this is a comment
feeling calm_meditative { -- inline comment
    shape: gradual_rise_fall
}
"#;
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();

        let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Feeling,
                TokenKind::Identifier, // calm_meditative
                TokenKind::LBrace,
                TokenKind::Shape,
                TokenKind::Colon,
                TokenKind::Identifier, // gradual_rise_fall
                TokenKind::RBrace,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_unexpected_char_error() {
        let src = "feeling @";
        let mut lexer = Lexer::new(src);
        let result = lexer.tokenize();
        assert!(result.is_err());
        let e = result.unwrap_err();
        let msg = e.to_string();
        assert!(msg.contains("@"));
    }

    #[test]
    fn tokenize_isolated_dot_is_error() {
        let src = ".";
        let mut lexer = Lexer::new(src);
        let result = lexer.tokenize();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("isolated"));
    }

    #[test]
    fn tokenize_line_column() {
        let src = "feeling\ncalm";
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();

        // "feeling" 在第 1 行第 1 列
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].col, 1);

        // "calm" 在第 2 行第 1 列
        assert_eq!(tokens[1].line, 2);
        assert_eq!(tokens[1].col, 1);
    }
}
