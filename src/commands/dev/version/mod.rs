//! 版本号生成模块
//!
//! 根据 Conventional Commits 规范生成版本号。

mod generate;

pub use generate::{VersionGenerateCommand, VersionInfo};

