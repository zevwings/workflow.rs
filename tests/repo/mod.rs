//! Repo 模块测试
//!
//! 包含 Repo 模块的所有测试文件。
//!
//! 注意：config 的单元测试已迁移到 src/lib/repo/config/types.rs 中

pub mod config_integration; // 仓库配置集成测试
pub mod config_private; // 私有仓库配置测试
pub mod config_public; // 公共仓库配置测试（包含文件系统集成测试）
pub mod config_repo; // 统一仓库配置接口测试
