// src/log.rs — Anim 日志系统
//
// KubePivot 用 P，Anim 用 A。
// A_info! / A_warn! / A_error!——带时间戳 + 颜色。
// 后续 v1.2+ 可用 tracing 替换底层——宏接口不变。

use std::sync::atomic::{AtomicU8, Ordering};

/// 全局日志级别——0=Debug, 1=Info, 2=Warn, 3=Error。
static LOG_LEVEL: AtomicU8 = AtomicU8::new(0); // 默认 Debug

/// 设置全局日志级别。
pub fn set_log_level(level: Level) {
    LOG_LEVEL.store(level.as_u8(), Ordering::Relaxed);
}

/// 日志级别——Debug < Info < Warn < Error。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    fn as_u8(&self) -> u8 {
        match self {
            Level::Debug => 0,
            Level::Info => 1,
            Level::Warn => 2,
            Level::Error => 3,
        }
    }
}

/// 彩色标签——TTY 用 ANSI，非 TTY 用纯文本。
impl Level {
    fn label(&self) -> &str {
        use std::io::IsTerminal;
        if std::io::stderr().is_terminal() {
            match self {
                Level::Debug => "\x1b[90mdebug\x1b[0m",
                Level::Info => "\x1b[36minfo\x1b[0m",
                Level::Warn => "\x1b[33mwarn\x1b[0m",
                Level::Error => "\x1b[31merror\x1b[0m",
            }
        } else {
            match self {
                Level::Debug => "debug",
                Level::Info => "info",
                Level::Warn => "warn",
                Level::Error => "error",
            }
        }
    }
}

/// Anim 全局日志器。对齐 KubePivot 的 `P`。
pub static A: Logger = Logger;

/// 日志器——KubePivot 用 P，Anim 用 A。
pub struct Logger;

impl Logger {
    pub fn debug(&self, args: std::fmt::Arguments) {
        emit(Level::Debug, args);
    }
    pub fn info(&self, args: std::fmt::Arguments) {
        emit(Level::Info, args);
    }
    pub fn warn(&self, args: std::fmt::Arguments) {
        emit(Level::Warn, args);
    }
    pub fn error(&self, args: std::fmt::Arguments) {
        emit(Level::Error, args);
    }
}

/// 内部写到 stderr，带时间戳和颜色。
#[allow(dead_code)]
fn emit(level: Level, args: std::fmt::Arguments) {
    if level.as_u8() < LOG_LEVEL.load(Ordering::Relaxed) {
        return;
    }
    let ts = chrono::Utc::now().format("%H:%M:%S");
    eprintln!("[{}] {} {}", ts, level.label(), args);
}

/// `A.debug(format_args!(...))` 的语法糖。
#[macro_export]
macro_rules! A_debug {
    ($($arg:tt)*) => { $crate::log::A.debug(format_args!($($arg)*)) };
}

/// `A.info(format_args!(...))` 的语法糖。
#[macro_export]
macro_rules! A_info {
    ($($arg:tt)*) => { $crate::log::A.info(format_args!($($arg)*)) };
}

/// `A.warn(format_args!(...))` 的语法糖。
#[macro_export]
macro_rules! A_warn {
    ($($arg:tt)*) => { $crate::log::A.warn(format_args!($($arg)*)) };
}

/// `A.error(format_args!(...))` 的语法糖。
#[macro_export]
macro_rules! A_error {
    ($($arg:tt)*) => { $crate::log::A.error(format_args!($($arg)*)) };
}

#[cfg(test)]
mod tests {
    // emit 直接写 stderr——此处只验证宏可编译和 Level label 正确
    use super::*;

    #[test]
    fn level_labels_contain_keywords() {
        // TTY 或非 TTY 环境都可能——只验证包含关键词
        assert!(Level::Info.label().contains("info"));
        assert!(Level::Warn.label().contains("warn"));
        assert!(Level::Error.label().contains("error"));
    }

    #[test]
    fn log_methods_compile() {
        A.info(format_args!("用户发起了交织请求"));
        A.warn(format_args!("Registry 加载失败，使用内建列表"));
        A.error(format_args!("无法读取文件 {}", "test.anim"));
        A.debug(format_args!("详细调试信息"));
    }
}
