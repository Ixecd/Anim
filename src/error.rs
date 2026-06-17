// src/error.rs — Anim 错误系统（ADR 006 + ADR 014）
//
// AnimiError = 交织器的错误枚举。手写 Display——不 derive。
// 每个变体 = 一种交叉被挡的原因。
// 不是 panic。不是 abort。不是"交织失败"。
// 是"这帧的交叉被挡了。下一帧继续。"
//
// ADR 014: 每个变体带 severity 字段——Deny 编译期拒绝，Warn 打印继续，Note 调试专用。

use std::cell::RefCell;
use std::fmt;

/// 错误严重度——ADR 014。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// 编译期硬拒绝——return Err。和当前 oi! 行为一致。
    Deny,
    /// 打印到 stderr——return Ok 继续。--strict 下升级为 Deny。
    Warn,
    /// 仅 --verbose 模式打印——return Ok 继续。
    Note,
}

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
    LexError {
        file_name: String,
        line: usize,
        col: usize,
        msg: String,
        severity: Severity,
    },
    ParseError {
        file_name: String,
        line: usize,
        col: usize,
        expected: String,
        found: String,
        severity: Severity,
    },
    TypeCheckError {
        file_name: String,
        atom_name: String,
        reason: String,
        severity: Severity,
    },
    /// 静态安全——对任何人的硬规则（Pass 2 / rule.rs）。
    StaticSafetyError {
        file_name: String,
        rule: String,
        detail: String,
        severity: Severity,
    },
    /// 用户安全——对这个人的拒绝（Pass 3 / safety.rs）。
    UserStateSafetyError {
        file_name: String,
        cap: String,
        atom_name: String,
        reason: String,
        severity: Severity,
    },
    /// 时域能量累积熔断——ADR 011 Neuro-Leaky Bucket。
    SafetyBreach {
        file_name: String,
        dimension: crate::pbm::PbmDimension,
        current_energy: f64,
        threshold: f64,
        severity: Severity,
    },
    /// 交织器内部错误。
    InternalError {
        file_name: String,
        msg: String,
        severity: Severity,
    },
}

impl AnimiError {
    /// 错误严重度。
    pub fn severity(&self) -> Severity {
        match self {
            AnimiError::LexError { severity, .. } => *severity,
            AnimiError::ParseError { severity, .. } => *severity,
            AnimiError::TypeCheckError { severity, .. } => *severity,
            AnimiError::StaticSafetyError { severity, .. } => *severity,
            AnimiError::UserStateSafetyError { severity, .. } => *severity,
            AnimiError::SafetyBreach { severity, .. } => *severity,
            AnimiError::InternalError { severity, .. } => *severity,
        }
    }
}

impl fmt::Display for AnimiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = match self {
            AnimiError::LexError { file_name, .. } => file_name,
            AnimiError::ParseError { file_name, .. } => file_name,
            AnimiError::TypeCheckError { file_name, .. } => file_name,
            AnimiError::StaticSafetyError { file_name, .. } => file_name,
            AnimiError::UserStateSafetyError { file_name, .. } => file_name,
            AnimiError::SafetyBreach { file_name, .. } => file_name,
            AnimiError::InternalError { file_name, .. } => file_name,
        };

        let tag = match self.severity() {
            Severity::Deny => "[oi]",
            Severity::Warn => "[oi?]",
            Severity::Note => "[oi~]",
        };

        match self {
            AnimiError::LexError { line, col, msg, .. } => {
                write!(f, "{} 词法错误 ({}:{}:{}): {}", tag, file, line, col, msg)
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
                    "{} 语法错误 ({}:{}:{}): 期望 {}，但遇到 {}",
                    tag, file, line, col, expected, found
                )
            }
            AnimiError::TypeCheckError {
                atom_name, reason, ..
            } => {
                write!(f, "{} 类型错误 ({}): {}——{}", tag, file, atom_name, reason)
            }
            AnimiError::StaticSafetyError { rule, detail, .. } => {
                write!(f, "{} 安全规则 ({}): {}——{}", tag, file, rule, detail)
            }
            AnimiError::UserStateSafetyError {
                cap,
                atom_name,
                reason,
                ..
            } => {
                write!(
                    f,
                    "{} 用户安全({}) ({}): {}——{}",
                    tag, cap, file, atom_name, reason
                )
            }
            AnimiError::SafetyBreach {
                dimension,
                current_energy,
                threshold,
                ..
            } => {
                write!(f, "{} 时域能量熔断 ({}): 维度 {:?} 累积能量 {:.1} 超过临界阈值 {:.1}——触发强制保护帧",
                    tag, file, dimension, current_energy, threshold)
            }
            AnimiError::InternalError { msg, .. } => {
                write!(f, "{} 内部错误 ({}): {}", tag, file, msg)
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
            severity: Severity::Deny,
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
            severity: Severity::Deny,
        };
        let s = e.to_string();
        assert!(s.contains("test.anim"));
        assert!(s.contains("3:12"));
    }

    #[test]
    fn warn_shows_oi_question() {
        let e = AnimiError::StaticSafetyError {
            file_name: "test.anim".into(),
            rule: "abrupt_stop".into(),
            detail: "建议不超过 20".into(),
            severity: Severity::Warn,
        };
        let s = e.to_string();
        assert!(s.contains("[oi?]"));
        assert!(!s.contains("[oi] "));
    }

    #[test]
    fn note_shows_oi_tilde() {
        let e = AnimiError::TypeCheckError {
            file_name: "test.anim".into(),
            atom_name: "calm_zero".into(),
            reason: "强度为零的冥想包？".into(),
            severity: Severity::Note,
        };
        let s = e.to_string();
        assert!(s.contains("[oi~]"));
    }
}
