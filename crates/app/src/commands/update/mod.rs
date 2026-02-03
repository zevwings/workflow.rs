//! 更新命令模块
//!
//! 提供从 GitHub Releases 更新 Workflow CLI 的功能。
//!
//! ## 功能
//!
//! - 获取最新版本信息
//! - 下载并验证更新包
//! - 自动备份和回滚
//! - 验证安装结果
//!
//! ## 模块结构
//!
//! - `types` - 类型定义
//! - `version` - 版本管理
//! - `download` - 下载和解压
//! - `verify` - 验证逻辑
//! - `command` - 主命令实现

mod command;
mod download;
mod types;
mod verify;
mod version;

pub use command::UpdateCommand;
