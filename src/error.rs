// src/error.rs — Anim 错误系统（ADR 006）
//
// AnimiError = 编译器的错误枚举。手写 Display——不 derive。
// 每个变体 = 一种交叉被挡的原因。
// 不是 panic。不是 abort。不是"编译失败"。
// 是"这帧的交叉被挡了。下一帧继续。"

use std::fmt;

/// Anim 编译器的错误类型。
#[derive(Debug)]
pub enum AnimiError {
    /// 词法错误——源码里有编译器不认识的字符。
    LexError {
        line: usize,
        col: usize,
        msg: String,
    },

    /// 语法错误——token 流不符合 .anim 语法。
    ParseError {
        line: usize,
        col: usize,
        expected: String,
        found: String,
    },

    /// 类型检查错误——感受原子不存在、强度越界等。
    TypeCheckError { atom_name: String, reason: String },

    /// 静态安全错误——Pass 2（rule.rs）。对任何人的硬规则。
    StaticSafetyError { rule: String, detail: String },

    /// 用户安全错误——Pass 3（safety.rs）。对这个人的拒绝。
    UserStateSafetyError {
        cap: String,
        atom_name: String,
        reason: String,
    },

    /// 编译器内部错误——不是用户源码的问题。
    InternalError { msg: String },
}

impl fmt::Display for AnimiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnimiError::LexError { line, col, msg } => {
                write!(f, "[oi] 词法错误 ({}:{}): {}", line, col, msg)
            }
            AnimiError::ParseError {
                line,
                col,
                expected,
                found,
            } => {
                write!(
                    f,
                    "[oi] 语法错误 ({}:{}): 期望 {}，但遇到 {}",
                    line, col, expected, found
                )
            }
            AnimiError::TypeCheckError { atom_name, reason } => {
                write!(f, "[oi] 类型错误: {}——{}", atom_name, reason)
            }
            AnimiError::StaticSafetyError { rule, detail } => {
                write!(f, "[oi] 安全规则: {}——{}", rule, detail)
            }
            AnimiError::UserStateSafetyError {
                cap,
                atom_name,
                reason,
            } => {
                write!(f, "[oi] 用户安全({}): {}——{}", cap, atom_name, reason)
            }
            AnimiError::InternalError { msg } => {
                write!(f, "[oi] 内部错误: {}", msg)
            }
        }
    }
}

impl std::error::Error for AnimiError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_error_display() {
        let e = AnimiError::LexError {
            line: 1,
            col: 5,
            msg: "unexpected @".into(),
        };
        let s = e.to_string();
        assert!(s.contains("[oi]"));
        assert!(s.contains("1:5"));
        assert!(s.contains("unexpected @"));
    }

    #[test]
    fn parse_error_display() {
        let e = AnimiError::ParseError {
            line: 3,
            col: 12,
            expected: "}".into(),
            found: "EOF".into(),
        };
        let s = e.to_string();
        assert!(s.contains("3:12"));
        assert!(s.contains("}"));
        assert!(s.contains("EOF"));
    }
}
