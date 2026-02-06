//! 领域层（DDD Domain Layer）
//!
//! 包含实体、值对象、仓储接口、领域服务接口和领域异常
//!
//! 按业务域组织，每个业务域包含相关的实体、服务接口、仓储接口和错误类型。

pub mod alias;
pub mod branch;
pub mod completion;
pub mod config;
pub mod errors;
pub mod git;
pub mod github;
pub mod jira;
pub mod llm;
pub mod path;
pub mod pr;
pub mod template;

// Re-export public types
// 避免使用 glob 导出以防止与子模块名称冲突（如 `template`）
// Re-export alias types
pub use alias::{AliasAddResult, AliasInfo, AliasListResult, AliasRemoveResult, AliasService};

// Re-export config types
pub use config::{
    BranchConfig,
    BranchTemplates,
    CommitTemplates,
    GitHubAccount,
    // Verification types
    GitHubAccountInfo,
    GitHubSettings,
    GitHubVerificationResult,
    GitHubVerificationSummary,
    GlobalConfig,
    GlobalConfigRepository,
    JiraConfigInfo,
    JiraVerificationResult,
    JiraVerificationStatus,
    LLMConfig,
    LLMProviderSettings,
    LLMSettings,
    LLMVerificationResult,
    LLMVerificationStatus,
    LogConfigInfo,
    LogSettings,
    LogVerificationResult,
    MCPConfig,
    MCPServerConfig,
    ProjectConfig,
    PullRequestsTemplates,
    RepoConfig,
    RepoConfigRepository,
    TemplateConfig,
    UserConfig,
    VerificationService,
};

// Re-export business domain types
pub use branch::{
    sanitize_branch_name, BranchService, BranchSyncCallbacks, BranchSyncOptions, BranchSyncResult,
    BranchType, SourceBranchInfo, SyncStrategy,
};
pub use completion::{
    get_all_completion_filenames, get_completion_filename, get_shell_source_path,
    CompletionCheckResult, CompletionGenerateResult, CompletionRemoveResult, CompletionService,
    ShellCompletionStatus,
};
pub use path::{Dir, PathError, PathService};
pub use path::{
    COMPLETIONS_DIR, COMPLETIONS_FILE, COMPLETION_CACHE_DIR, JIRA_CONFIG_FILE, MAIN_DIR,
    PROJECT_CONFIG_FILE, USER_CONFIG_FILE, WORKFLOW_CONFIG_DIR, WORKFLOW_CONFIG_FILE,
};
pub use pr::{
    get_all_change_types, get_change_type_by_index, get_change_type_by_name, ChangeType, PrContent,
    PrStatus, PullRequestInfo, PullRequestService, PullRequestStatus, CHANGE_TYPES,
};

// Re-export template types
pub use template::{
    BranchTemplateVars, ChangeTypeItem, CommitTemplateVars, PullRequestTemplateVars,
};

// Re-export error types
pub use errors::ServiceError;

// Re-export git types
pub use git::{
    BlameLineInfo, BranchFilter, BranchInfo, CodePlatform, CommitInfo, FileStatusInfo,
    FileStatusType, GitError, GitRepoRepository, GitRepository, MergeStrategy, RemoteInfo,
    RepoInfo, TagCreateInfo, TagCreateScope, TagDeleteInfo, TagDeleteScope, WorkingTreeStatus,
};

// Re-export external service types
pub use github::{GitHubContext, GitHubError, GitHubRepository, GitHubUser};
pub use jira::{
    extract_jira_project, extract_jira_ticket_id, validate_jira_ticket_format,
    AttachmentDownloadResult, DeleteHistoryResult, JiraAttachment, JiraComment, JiraComponent,
    JiraConfigContext, JiraError, JiraIssue, JiraPriority, JiraRepository, JiraStatusConfig,
    JiraTransition, JiraUser, JiraWorkHistoryRepository, ProjectStatusConfig, StatusConfigResult,
    WorkHistoryEntry,
};
pub use llm::{
    LLMConfigContext, LLMError, LLMRepository, PullRequestContent, PullRequestReword,
    PullRequestSummary, SupportedLanguage,
};
