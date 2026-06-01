pub mod cli;
pub mod core;
pub mod error;

pub use cli::{args_without_binary_name, run};
pub use core::format_markdown;
pub use error::Error;
