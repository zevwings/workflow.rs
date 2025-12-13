//! 别名管理模块
//!
//! 提供别名系统的核心功能，包括别名加载、展开和管理。

mod config;
mod manager;

pub use config::CommandsConfig;
pub use manager::AliasManager;
