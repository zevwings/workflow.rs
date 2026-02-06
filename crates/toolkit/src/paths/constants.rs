//! 路径常量定义
//!
//! 定义所有路径相关的常量，包括目录名称和文件名。

/// 工作流目录名称
pub const WORKFLOW_DIR: &str = ".workflow";

/// 配置目录名称
pub const CONFIG_DIR: &str = "config";

// /// 主配置文件名称
// pub const WORKFLOW_CONFIG_FILE: &str = "workflow.toml";

/// Jira 配置文件名称
pub const JIRA_CONFIG_FILE: &str = "jira.toml";

/// LLM 配置文件名称
pub const LLM_CONFIG_FILE: &str = "llm.toml";

/// 补全配置文件名称（用于 zsh/bash 的 source 配置）
pub const COMPLETIONS_FILE: &str = ".completions";

/// 项目配置文件名称（仓库级别）
pub const PROJECT_CONFIG_FILE: &str = "config.toml";

/// 用户配置文件名称（仓库级别）
pub const USER_CONFIG_FILE: &str = "user.toml";

/// Completion 缓存目录名称
pub const COMPLETION_CACHE_DIR: &str = ".completion_cache";

/// Completions 目录名称
pub const COMPLETIONS_DIR: &str = "completions";
