// src/log.rs — Anim 日志系统
//
// KubePivot 用 P，Anim 用 A。
// A_info! / A_warn! / A_error!——带时间戳 + 颜色。
// 后续 v1.2+ 可用 tracing 替换底层——宏接口不变。

use std::fmt;

/// 日志级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Level {
    Info,
    Warn,
    Error,
}

/// 彩色标签。
impl Level {
    fn label(&self) -> &str {
        match self {
            Level::Info => "\x1b[36minfo\x1b[0m",
            Level::Warn => "\x1b[33mwarn\x1b[0m",
            Level::Error => "\x1b[31merror\x1b[0m",
        }
    }
}

/// 写到 stderr，带时间戳和颜色。
#[allow(dead_code)]
pub fn emit(level: Level, args: fmt::Arguments) {
    let ts = chrono::Local::now().format("%H:%M:%S");
    eprintln!("[{}] {} {}", ts, level.label(), args);
}

/// 信息日志。
#[macro_export]
macro_rules! A_info {
    ($($arg:tt)*) => {
        $crate::log::emit($crate::log::Level::Info, format_args!($($arg)*))
    };
}

/// 警告日志。
#[macro_export]
macro_rules! A_warn {
    ($($arg:tt)*) => {
        $crate::log::emit($crate::log::Level::Warn, format_args!($($arg)*))
    };
}

/// 错误日志。
#[macro_export]
macro_rules! A_error {
    ($($arg:tt)*) => {
        $crate::log::emit($crate::log::Level::Error, format_args!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    // emit 直接写 stderr——此处只验证宏可编译和 Level label 正确
    use super::*;

    #[test]
    fn level_labels() {
        assert_eq!(Level::Info.label(), "\x1b[36minfo\x1b[0m");
        assert_eq!(Level::Warn.label(), "\x1b[33mwarn\x1b[0m");
        assert_eq!(Level::Error.label(), "\x1b[31merror\x1b[0m");
    }

    #[test]
    fn macros_compile() {
        A_info!("用户发起了交织请求");
        A_warn!("Registry 加载失败，使用内建列表");
        A_error!("无法读取文件 {}", "test.anim");
    }
}
