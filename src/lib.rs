//! Workflow 库入口
//!
//! 这个模块重新导出了 Workflow CLI 的所有公共 API，方便其他模块使用。
//! 采用三层架构设计：
//! - **CLI 入口层** (`bin/`, `main.rs`): 命令行参数解析和命令分发
//! - **命令封装层** (`commands/`): CLI 命令封装，处理用户交互
//! - **核心业务逻辑层** (`lib/`): 所有业务逻辑实现

// 核心库模块声明
// 核心基础层 (core/)
#[path = "lib/core/mod.rs"]
#[allow(clippy::module_inception)]
pub mod core;
// 重新导出 core 层的模块
pub use core::constants;
pub use core::http;
pub use core::logger;
pub use core::prompt;
pub use core::shell;
pub use core::util;

// 服务层 (services/)
#[path = "lib/services/mod.rs"]
#[allow(clippy::module_inception)]
pub mod services;
// 重新导出 services 层的模块
pub use services::git;
pub use services::github;
pub use services::jira;
pub use services::llm;

// 配置层 (config/)
#[path = "lib/config/mod.rs"]
#[allow(clippy::module_inception)]
pub mod config;
// 重新导出 config 层的模块
pub use config::mcp;
pub use config::settings;
pub use config::template;

// 业务领域层 (domain/)
#[path = "lib/domain/mod.rs"]
#[allow(clippy::module_inception)]
pub mod domain;
// 重新导出 domain 层的模块
pub use domain::alias;
pub use domain::branch;
pub use domain::cli;
pub use domain::commit;
pub use domain::completion;
pub use domain::pr;
pub use domain::repo;
pub use domain::rollback;

// 适配器层 (infra/)
#[path = "lib/infra/mod.rs"]
pub mod infra;

// 命令模块声明
#[path = "commands/mod.rs"]
pub mod commands;

// 重新导出所有公共 API，方便外部使用
// 重新导出基础设施类型
pub use alias::{AliasManager, CommandsConfig};
pub use constants::*;
pub use http::{Authorization, HttpClient, HttpResponse, HttpRetry, HttpRetryConfig};
pub use logger::LogLevel;
// 语言相关的函数从 llm::client 导出
// (prompt 模块已改为内部模块，不再对外暴露)
pub use llm::{
    find_language, get_language_instruction, get_supported_language_codes,
    get_supported_language_display_names, SupportedLanguage, SUPPORTED_LANGUAGES,
};
pub use settings::{LLMSettings, Paths, Settings};
pub use shell::detect::Detect;
pub use shell::env::{load_env_vars, remove_env_vars, save_env_vars};
pub use shell::paths::{config_file, get_config_path};
pub use shell::reload::Reload;
pub use shell::source::{add_source_for_shell, has_source_for_shell, remove_source_for_shell};
pub use util::browser::Browser;
pub use util::checksum::Checksum;
pub use util::clipboard::Clipboard;
pub use util::concurrent::{ConcurrentExecutor, TaskResult};
pub use util::format::{
    error, key_value, list_item, operation, progress, PathDisplay, Sensitive, SizeDisplay,
};
pub use util::unzip::Unzip;
// 从 llm 重新导出语言增强 API
pub use llm::get_language_requirement;

// 业务模块导出
pub use branch::BranchNaming;
pub use completion::{
    get_all_completion_files, get_completion_filename, get_completion_files_for_shell, Completion,
    CompletionGenerator,
};
pub use git::{
    GitBranch, GitCommit, GitConfig, GitPreCommit, GitRepo, GitStash, MergeStrategy, RepoType,
};
pub use github::{GitHub, GitHubUser};
pub use jira::{
    extract_jira_project, extract_jira_ticket_id, sanitize_email_for_filename,
    validate_jira_ticket_format, Jira, JiraApi, JiraAttachment, JiraClient, JiraComment,
    JiraComments, JiraIssue, JiraIssueFields, JiraStatus, JiraStatusConfig, JiraTransition,
    JiraUser, ProjectStatusConfig, WorkHistoryEntry,
};
pub use jira::{JiraLogs, LogEntry};
pub use pr::{
    extract_pull_request_id_from_url, get_all_change_types, get_change_type_by_index,
    get_change_type_by_name, get_current_branch_pr_id, resolve_pull_request_id, ChangeType,
    PlatformProvider, CHANGE_TYPES, TYPES_OF_CHANGES,
};
// LLM-related PR types are now in llm module
pub use llm::{
    CreateGenerator, FileSummaryGenerator, PullRequestContent, PullRequestSummary, RewordGenerator,
    SummaryGenerator,
};
pub use rollback::{BackupInfo, RollbackManager};
pub use template::{
    BranchTemplateVars, ChangeTypeItem, CommitTemplateVars, PullRequestTemplateVars,
    TemplateEngine, TemplateEngineType,
};
