//! 配置管理命令
//! 包括应用配置管理（初始化、查看、GitHub 账号、日志级别）和系统环境管理（代理、Completion、环境检查）

// 应用配置管理
pub mod show;
pub mod github;
pub mod log;
pub mod setup;

// 系统环境管理
pub mod check;
pub mod completion;
pub mod proxy;

