//! 客户端相关错误消息
//!
//! 包含 HTTP 客户端、生成器和输入读取相关的错误消息。

// HTTP 客户端错误消息

/// 创建 HTTP 客户端失败
pub const HTTP_CLIENT_CREATE_CLIENT_FAILED: &str = "Failed to create HTTP client";

// 生成器创建错误消息

/// 创建生成器失败（带格式化参数）
pub const GENERATOR_CREATE_GENERATOR_FAILED_FORMAT: &str = "Failed to create generator for {}";

/// 创建 zsh 生成器失败
pub const GENERATOR_CREATE_ZSH_GENERATOR_FAILED: &str = "Failed to create zsh generator";

// 输入读取错误消息

/// 读取 Jira 票据 ID 失败
pub const INPUT_READ_JIRA_TICKET_ID_FAILED: &str = "Failed to read Jira ticket ID";

/// 读取分支名称失败
pub const INPUT_READ_BRANCH_NAME_FAILED: &str = "Failed to read branch name";
