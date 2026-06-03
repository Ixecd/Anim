// src/lib.rs — animi 交织器库入口

pub mod ast;
pub mod error;
pub mod fsir;
pub mod guard;
pub mod lexer;
pub mod log;
pub mod oi;
pub mod parser;
pub mod pbm;
pub mod registry;
pub mod rule;
pub mod safety;
pub mod typeck;

pub use log::{set_log_level, Level, A};
