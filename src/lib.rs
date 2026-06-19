// src/lib.rs — animi 交织器库入口

pub mod ast;
pub mod codegen;
pub mod config;
pub mod device_map;
pub mod dsir;
pub mod error;
pub mod esir;
pub mod fsir;
pub mod guard;
pub mod lexer;
pub mod log;
pub mod macros;
pub mod oi;
pub mod parser;
pub mod pbm;
pub mod personalize;
pub mod pipeline;
pub mod psir;
pub mod registry;
pub mod rule;
pub mod safety;
pub mod sandbox;
pub mod typeck;

pub use log::{set_log_level, Level, A};

// 日志便捷宏由 #[macro_export] 自动导出到 crate root。
// 库用户可直接 `use animi::A_info;` 后调用 `A_info!("hello")` 等。
