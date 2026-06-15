// src/oi.rs — Anim 的"交叉被挡"宏（ADR 006 + ADR 014）
//
// oi 不是 Err。不是 Error。不是"错了"。
// oi 是"交叉被挡。"轻。短。不 panic。不跳闸。
// 这帧不生成。下一帧继续。和 Anim 的安全哲学一致——
// "宁可误杀一帧。绝不污染一条经线。"
//
// ADR 014 — 三层严重度：
//   oi!   Deny  — 编译期硬拒绝。"这帧交叉被挡了。"
//   oi_warn!  Warn  — 打印到 stderr，继续。"你确定？这里很深。"
//   oi_note!  Note  — 仅 --verbose 打印。"你这不是在冥想，是在装睡。"

/// oi! — 交叉被挡。不 panic。返回 Err(AnimiError::Variant { severity: Deny, fields })。
///
/// # 示例
///
/// ```ignore
/// use animi::oi;
/// oi!(LexError, line=12_usize, col=5_usize, msg="unexpected @".to_string());
#[macro_export]
macro_rules! oi {
    ($variant:ident, $($field:ident = $value:expr),* $(,)?) => {
        {
            let _file = $crate::error::CURRENT_FILE.with(|f| f.borrow().clone());
            return Err($crate::error::AnimiError::$variant {
                file_name: _file,
                severity: $crate::error::Severity::Deny,
                $($field: $value),*
            })
        }
    };
}

/// oi_warn! — 警告。返回 Err(Warn) 给调用方自行处理。
///
/// 调用方（main.rs 或管线）按 --strict 决定是否升级为 Deny。
#[macro_export]
macro_rules! oi_warn {
    ($variant:ident, $($field:ident = $value:expr),* $(,)?) => {
        {
            let _file = $crate::error::CURRENT_FILE.with(|f| f.borrow().clone());
            return Err($crate::error::AnimiError::$variant {
                file_name: _file,
                severity: $crate::error::Severity::Warn,
                $($field: $value),*
            })
        }
    };
}

/// oi_note! — 调侃。返回 Err(Note) 给调用方自行处理。
///
/// 调用方仅 --verbose 模式打印。
#[macro_export]
macro_rules! oi_note {
    ($variant:ident, $($field:ident = $value:expr),* $(,)?) => {
        {
            let _file = $crate::error::CURRENT_FILE.with(|f| f.borrow().clone());
            return Err($crate::error::AnimiError::$variant {
                file_name: _file,
                severity: $crate::error::Severity::Note,
                $($field: $value),*
            })
        }
    };
}

/// 无 `return` 封装——可用于 match 表达式臂等。默认 Deny。
#[macro_export]
macro_rules! oi_err {
    ($variant:ident, $($field:ident = $value:expr),* $(,)?) => {
        {
            let _file = $crate::error::CURRENT_FILE.with(|f| f.borrow().clone());
            Err($crate::error::AnimiError::$variant {
                file_name: _file,
                severity: $crate::error::Severity::Deny,
                $($field: $value),*
            })
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::error::{AnimiError, Severity};

    #[test]
    fn oi_macro_returns_error() {
        fn lex() -> Result<(), AnimiError> {
            oi!(LexError, line=42_usize, col=7_usize, msg="test oi".to_string());
        }
        let e = lex().unwrap_err();
        assert_eq!(e.severity(), Severity::Deny);
        assert!(e.to_string().contains("[oi]"));
    }

    #[test]
    fn oi_warn_returns_warn_severity() {
        fn warn() -> Result<(), AnimiError> {
            oi_warn!(StaticSafetyError, rule="abrupt_stop".into(), detail="建议不超过 20".into());
        }
        let e = warn().unwrap_err();
        assert_eq!(e.severity(), Severity::Warn);
        assert!(e.to_string().contains("[oi?]"));
    }

    #[test]
    fn oi_note_returns_note_severity() {
        fn note() -> Result<(), AnimiError> {
            oi_note!(TypeCheckError, atom_name="calm_zero".into(), reason="强度为零的冥想包？".into());
        }
        let e = note().unwrap_err();
        assert_eq!(e.severity(), Severity::Note);
        assert!(e.to_string().contains("[oi~]"));
    }
}
