//! Workflow 库入口
//!
//! 这个模块重新导出了 Workflow CLI 的所有公共 API，方便其他模块使用。
//! 采用三层架构设计：
//! - **CLI 入口层** (`bin/`, `main.rs`): 命令行参数解析和命令分发
//! - **命令封装层** (`commands/`): CLI 命令封装，处理用户交互
//! - **核心业务逻辑层** (`lib/`): 所有业务逻辑实现

// 核心库模块声明
#[path = "lib/base/mod.rs"]
pub mod base;
#[path = "lib/completion/mod.rs"]
pub mod completion;
#[path = "lib/git/mod.rs"]
pub mod git;
#[path = "lib/jira/mod.rs"]
pub mod jira;
#[path = "lib/pr/mod.rs"]
pub mod pr;
#[path = "lib/proxy/mod.rs"]
pub mod proxy;
#[path = "lib/rollback/mod.rs"]
pub mod rollback;

// 命令模块声明
#[path = "commands/mod.rs"]
pub mod commands;

// 重新导出所有公共 API，方便外部使用
// 从 base 模块重新导出基础设施类型，保持向后兼容
pub use base::settings::{LLMSettings, Paths, Settings};
pub use base::util::{
    confirm, mask_sensitive_value, Browser, Checksum, Clipboard, LogLevel, Logger, Unzip,
};
pub use base::{
    Authorization, Detect, HttpClient, HttpResponse, HttpRetry, HttpRetryConfig, Reload,
    ShellConfigManager,
};

// 业务模块导出
pub use completion::{
    generate_all_completions, generate_pr_completion, generate_qk_completion,
    generate_workflow_completion, get_all_completion_files, get_completion_filename,
    get_completion_files_for_shell, Completion,
};
pub use git::{
    GitBranch, GitCommit, GitConfig, GitPreCommit, GitRepo, GitStash, MergeStrategy, RepoType,
};
pub use jira::{
    extract_jira_project, extract_jira_ticket_id, sanitize_email_for_filename,
    validate_jira_ticket_format, Jira, JiraApi, JiraAttachment, JiraClient, JiraComment,
    JiraComments, JiraIssue, JiraIssueFields, JiraStatus, JiraStatusConfig, JiraTransition,
    JiraUser, ProjectStatusConfig, WorkHistoryEntry,
};
pub use jira::{JiraLogs, LogEntry};
pub use pr::{
    detect_repo_type, extract_pull_request_id_from_url, generate_branch_name,
    generate_commit_title, generate_pull_request_body, get_current_branch_pr_id, Codeup, GitHub,
    PlatformProvider, PullRequestContent, PullRequestLLM, TYPES_OF_CHANGES,
};
pub use proxy::{
    ProxyConfigGenerator, ProxyDisableResult, ProxyEnableResult, ProxyInfo, ProxyManager,
    ProxyType, SystemProxyReader,
};
pub use rollback::{BackupInfo, RollbackManager};
