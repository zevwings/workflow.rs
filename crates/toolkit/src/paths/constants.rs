//! 路径常量定义
//!
//! 定义所有路径相关的常量，包括目录名称和文件名。

/// 工作流目录名称
pub const WORKFLOW_DIR: &str = ".workflow";

/// 配置目录名称
pub const CONFIG_DIR: &str = "config";

/// 主配置文件名称
pub const WORKFLOW_CONFIG_FILE: &str = "workflow.toml";

/// Jira 配置文件名称
pub const JIRA_CONFIG_FILE: &str = "jira.toml";

/// LLM 配置文件名称
pub const LLM_CONFIG_FILE: &str = "llm.toml";

/// 补全文件名称
pub const COMPLETIONS_FILE: &str = ".completions";
