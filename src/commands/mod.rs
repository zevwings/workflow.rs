//! 命令实现模块
//!
//! 这个模块包含所有 CLI 命令的实现，作为命令封装层，负责：
//! - 解析命令参数
//! - 处理用户交互
//! - 格式化输出
//! - 调用核心业务逻辑层 (`lib/`) 的功能
//!
//! ## 模块说明
//!
//! - `check` - 检查工具命令（git_status, network）
//! - `config` - 配置查看命令
//! - `install` - 安装命令实现
//! - `pr` - PR 相关命令（create, merge, status, list, update）
//! - `proxy` - 代理管理命令（on, off, check）
//! - `qk` - 快速日志操作命令（download, find, search）
//! - `setup` - 初始化设置命令
//! - `uninstall` - 卸载命令实现

pub mod check;
pub mod config;
pub mod install;
pub mod pr;
pub mod proxy;
pub mod qk;
pub mod setup;
pub mod uninstall;
