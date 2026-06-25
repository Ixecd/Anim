// src/lib.rs — animi 交织器库入口
//
// Anim 职责收敛：.anim → FSIR only (Pass 0-5)
// Pass 6+ 归属 Feelings-Core

pub mod ast;
pub mod config;
pub mod error;
pub mod fsir;
pub mod guard;
pub mod lexer;
pub mod log;
pub mod macros;
pub mod oi;
pub mod parser;
pub mod pbm;
pub mod registry;
pub mod rule;
pub mod safety;
pub mod typeck;

pub use log::{set_log_level, Level, A};
