//! 命令实现模块
//!
//! 这个模块包含所有 CLI 命令的实现，作为命令封装层，负责：
//! - 解析命令参数
//! - 处理用户交互
//! - 格式化输出
//! - 调用核心业务逻辑层 (`lib/`) 的功能
//!
//! ## 模块分类
//!
//! ### 生命周期管理 (`lifecycle/`)
//! - `install` - 安装命令实现
//! - `uninstall` - 卸载命令实现
//! - `update` - 更新命令实现（重新构建、更新二进制文件、更新 completion）
//!
//! ### 配置管理
//! - `config/` - 配置管理命令（setup, show, log, completion）
//! - `github/` - GitHub 账号管理命令（list, add, remove, switch, update, current）
//! - `check/` - 环境检查命令（git_status, network）
//! - `proxy/` - 代理管理命令（on, off, check）
//!
//! ### 业务功能
//! - `pr/` - PR 相关命令（create, merge, close, status, list, update, integrate）
//! - `log/` - 日志操作命令（download, find, search）
//! - `jira/` - Jira 操作命令（info, attachments, clean）
//! - `branch/` - 分支管理命令（clean, ignore）

// 生命周期管理
pub mod lifecycle;

// 配置管理
pub mod check;
pub mod config;
pub mod github;
pub mod llm;
pub mod proxy;

// 业务功能
pub mod branch;
pub mod jira;
pub mod log;
pub mod pr;
