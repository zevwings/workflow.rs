//! 环境检查命令
//!
//! 迁移自 `.go/internal/commands/check.go` 的基础能力：
//! - 显示当前配置文件路径
//! - 调用配置服务执行基础检查（如权限）
//! - 环境检查（Git、网络、配置文件权限）
//! - 配置验证（Log、LLM、Jira、GitHub）

mod command;

pub use command::CheckCommand;
