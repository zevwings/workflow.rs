//! Util 模块
//!
//! 本模块提供了各种工具函数和实用工具，包括：
//! - 基础工具（文件、目录、平台检测等）
//! - 格式化工具（显示格式化、消息格式化、字符串处理）
//! - 并发工具（并发任务执行）
//! - 浏览器和剪贴板操作
//! - 文件解压和校验和验证
//!
//! ## 模块结构
//!
//! - `platform` - 平台检测工具（操作系统和架构检测）
//! - `browser` - 浏览器操作（`Browser`）
//! - `clipboard` - 剪贴板操作（`Clipboard`）
//! - `unzip` - 解压工具（tar.gz 文件解压）
//! - `checksum` - 校验和工具（SHA256 计算和验证）
//! - `concurrent` - 并发任务执行工具
//! - `format` - 格式化工具（消息格式化、显示格式化、字符串处理）
//! - `date` - 日期时间工具
//! - `directory` - 目录管理工具
//! - `file` - 文件读写工具

pub mod browser;
pub mod checksum;
pub mod clipboard;
pub mod concurrent;
pub mod date;
pub mod directory;
pub mod file;
pub mod format;
pub mod platform;
pub mod unzip;
