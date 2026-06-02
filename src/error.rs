// src/error.rs — Anim 错误系统（ADR 006）
//
// AnimiError = 交织器的错误枚举。手写 Display——不 derive。
// 每个变体 = 一种交叉被挡的原因。
// 不是 panic。不是 abort。不是"交织失败"。
// 是"这帧的交叉被挡了。下一帧继续。"

use std::cell::RefCell;
use std::fmt;

thread_local! {
    /// 当前正在处理的源文件名——由 CLI 入口设置。
    /// oi! 宏自动从此读取，无需每个调用点传递。
    pub static CURRENT_FILE: RefCell<String> = const { RefCell::new(String::new()) };
}

/// 获取当前正在处理的文件名。
pub fn current_file() -> String {
    CURRENT_FILE.with(|f| f.borrow().clone())
}

/// animi 交织器的错误类型。
#[derive(Debug)]
pub enum AnimiError {
    /// 词法错误——源码里有交织器不认识的字符。
    LexError {
        file_name: String,
        line: usize,
        col: usize,
        msg: String,
    },

    /// 语法错误——token 流不符合 .anim 语法。
    ParseError {
        file_name: String,
        line: usize,
        col: usize,
        expected: String,
        found: String,
    },

    /// 类型检查错误——感受原子不存在、强度越界等。
    TypeCheckError {
        file_name: String,
        atom_name: String,
        reason: String,
    },

    /// 静态安全错误——Pass 2（rule.rs）。对任何人的硬规则。
    StaticSafetyError {
        file_name: String,
        rule: String,
        detail: String,
    },

    /// 用户安全错误——Pass 3（safety.rs）。对这个人的拒绝。
    UserStateSafetyError {
        file_name: String,
        cap: String,
        atom_name: String,
        reason: String,
    },

    /// 交织器内部错误——不是用户源码的问题。
    InternalError { file_name: String, msg: String },
}

impl fmt::Display for AnimiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = match self {
            AnimiError::LexError { file_name, .. } => file_name,
            AnimiError::ParseError { file_name, .. } => file_name,
            AnimiError::TypeCheckError { file_name, .. } => file_name,
            AnimiError::StaticSafetyError { file_name, .. } => file_name,
            AnimiError::UserStateSafetyError { file_name, .. } => file_name,
            AnimiError::InternalError { file_name, .. } => file_name,
        };

        match self {
            AnimiError::LexError { line, col, msg, .. } => {
                write!(f, "[oi] 词法错误 ({}:{}:{}): {}", file, line, col, msg)
            }
            AnimiError::ParseError {
                line,
                col,
                expected,
                found,
                ..
            } => {
                write!(
                    f,
                    "[oi] 语法错误 ({}:{}:{}): 期望 {}，但遇到 {}",
                    file, line, col, expected, found
                )
            }
            AnimiError::TypeCheckError {
                atom_name, reason, ..
            } => {
                write!(f, "[oi] 类型错误 ({}): {}——{}", file, atom_name, reason)
            }
            AnimiError::StaticSafetyError { rule, detail, .. } => {
                write!(f, "[oi] 安全规则 ({}): {}——{}", file, rule, detail)
            }
            AnimiError::UserStateSafetyError {
                cap,
                atom_name,
                reason,
                ..
            } => {
                write!(
                    f,
                    "[oi] 用户安全({}) ({}): {}——{}",
                    cap, file, atom_name, reason
                )
            }
            AnimiError::InternalError { msg, .. } => {
                write!(f, "[oi] 内部错误 ({}): {}", file, msg)
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
            file_name: "test.anim".into(),
            line: 1,
            col: 5,
            msg: "unexpected @".into(),
        };
        let s = e.to_string();
        assert!(s.contains("[oi]"));
        assert!(s.contains("test.anim"));
        assert!(s.contains("1:5"));
        assert!(s.contains("unexpected @"));
    }

    #[test]
    fn parse_error_display() {
        let e = AnimiError::ParseError {
            file_name: "test.anim".into(),
            line: 3,
            col: 12,
            expected: "}".into(),
            found: "EOF".into(),
        };
        let s = e.to_string();
        assert!(s.contains("test.anim"));
        assert!(s.contains("3:12"));
    }
}
