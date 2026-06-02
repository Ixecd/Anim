// src/oi.rs — Anim 的轻量拒绝宏（ADR 006）
//
// oi 不是 Err。不是 Error。不是"错了"。
// oi 是"交叉被挡。"轻。短。不 panic。不跳闸。
// 这帧不生成。下一帧继续。和 Anim 的安全哲学一致——
// "宁可误杀一帧。绝不污染一条经线。"
//
// v1.2+ 扩展：FPGA 实时 Session 需带 frame_id / user_id / 生理数据上下文。
// 方案：oi! 加可选 context 参数，或 thread-local context 注入。

/// oi! — 交叉被挡。不 panic。返回 Err(AnimiError::Variant { fields })。
///
/// # 示例
///
/// ```ignore
/// use animi::oi;
/// // 注意：字符串字段需要显式 .to_string()——oi! 不做隐式转换。
/// oi!(LexError, line=12_usize, col=5_usize, msg="unexpected @".to_string());
#[macro_export]
macro_rules! oi {
    ($variant:ident, $($field:ident = $value:expr),* $(,)?) => {
        {
            let _file = $crate::error::CURRENT_FILE.with(|f| f.borrow().clone());
            return Err($crate::error::AnimiError::$variant {
                file_name: _file,
                $($field: $value),*
            })
        }
    };
}

/// 无 `return` 封装——可用于 match 表达式臂等。
#[macro_export]
macro_rules! oi_err {
    ($variant:ident, $($field:ident = $value:expr),* $(,)?) => {
        {
            let _file = $crate::error::CURRENT_FILE.with(|f| f.borrow().clone());
            Err($crate::error::AnimiError::$variant {
                file_name: _file,
                $($field: $value),*
            })
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::error::AnimiError;

    #[test]
    fn oi_macro_returns_error() {
        fn lex() -> Result<(), AnimiError> {
            oi!(
                LexError,
                line = 42_usize,
                col = 7_usize,
                msg = "test oi".to_string()
            );
        }
        let result = lex();
        assert!(result.is_err());
        let e = result.unwrap_err();
        match e {
            AnimiError::LexError { line, col, msg, .. } => {
                assert_eq!(line, 42);
                assert_eq!(col, 7);
                assert_eq!(msg, "test oi");
            }
            _ => panic!("expected LexError"),
        }
    }

    #[test]
    fn oi_macro_typecheck_error() {
        fn check() -> Result<(), AnimiError> {
            oi!(
                TypeCheckError,
                atom_name = "explosion".to_string(),
                reason = "未注册的感受原子".to_string()
            );
        }
        let result = check();
        assert!(result.is_err());
        let e = result.unwrap_err();
        assert!(e.to_string().contains("explosion"));
        assert!(e.to_string().contains("未注册"));
    }
}
