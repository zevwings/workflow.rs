//! 配置初始化命令
//!
//! 迁移自 `.go/internal/commands/setup.go` 的交互式版本（简化实现）：
//! - 读取现有 `workflow.toml`（如果不存在则从默认值开始）
//! - 交互式配置 GitHub / Jira / LLM / 日志
//! - 保存配置并给出提示

mod command;

pub use command::SetupCommand;
