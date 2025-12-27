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
#[path = "lib/branch/mod.rs"]
pub mod branch;
#[path = "lib/cli/mod.rs"]
pub mod cli;
#[path = "lib/commit/mod.rs"]
pub mod commit;
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
#[path = "lib/repo/mod.rs"]
pub mod repo;
#[path = "lib/rollback/mod.rs"]
pub mod rollback;
#[path = "lib/template/mod.rs"]
pub mod template;

// 命令模块声明
#[path = "commands/mod.rs"]
pub mod commands;

// 重新导出所有公共 API，方便外部使用
// 从 base 模块重新导出基础设施类型，保持向后兼容
pub use base::checksum::Checksum;
pub use base::format::DisplayFormatter;
pub use base::settings::{LLMSettings, Paths, Settings};
pub use base::system::{Browser, Clipboard};
pub use base::zip::Unzip;
pub use base::{
    Authorization, Detect, HttpClient, HttpResponse, HttpRetry, HttpRetryConfig, LogLevel, Logger,
    Reload, ShellConfigManager, Tracer,
};
// 从 base::prompt 重新导出 Prompt 相关 API
pub use base::prompt::{
    find_language, generate_summarize_pr_system_prompt, get_language_instruction,
    get_supported_language_codes, get_supported_language_display_names, SupportedLanguage,
    GENERATE_BRANCH_SYSTEM_PROMPT, SUPPORTED_LANGUAGES,
};
// 从 base::llm 重新导出语言增强 API
pub use base::llm::get_language_requirement;

// 业务模块导出
pub use branch::BranchNaming;
pub use completion::{
    get_all_completion_files, get_completion_filename, get_completion_files_for_shell, Completion,
    CompletionGenerator,
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
    extract_pull_request_id_from_url, get_all_change_types, get_change_type_by_index,
    get_change_type_by_name, get_current_branch_pr_id, resolve_pull_request_id, ChangeType,
    CreateGenerator, FileSummaryGenerator, GitHub, GitHubUser, PlatformProvider,
    PullRequestContent, PullRequestSummary, RewordGenerator, SummaryGenerator, CHANGE_TYPES,
    TYPES_OF_CHANGES,
};
pub use proxy::{
    ProxyConfigGenerator, ProxyDisableResult, ProxyEnableResult, ProxyInfo, ProxyManager,
    ProxyType, SystemProxyReader,
};
pub use rollback::{BackupInfo, RollbackManager};
pub use template::{
    BranchTemplateVars, ChangeTypeItem, CommitTemplateVars, PullRequestTemplateVars,
    TemplateEngine, TemplateEngineType,
};
