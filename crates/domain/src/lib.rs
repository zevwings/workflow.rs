//! 领域层（DDD Domain Layer）
//!
//! 包含实体、值对象、仓储接口、领域服务接口和领域异常
//!
//! 按业务域组织，每个业务域包含相关的实体、服务接口、仓储接口和错误类型。

pub(crate) mod alias;
pub(crate) mod branch;
pub(crate) mod commit;
pub(crate) mod completion;
pub(crate) mod config;
pub(crate) mod git;
pub(crate) mod github;
pub(crate) mod jira;
pub(crate) mod path;
pub(crate) mod pr;
pub(crate) mod ssh;
pub(crate) mod summary;
pub(crate) mod template;

#[cfg(any(test, feature = "testing"))]
pub mod testing;

// Re-export public types
// 避免使用 glob 导出以防止与子模块名称冲突（如 `template`）
// Re-export alias types
pub use alias::{
    AliasAddResult, AliasError, AliasInfo, AliasListResult, AliasRemoveResult, AliasService,
};
// Re-export business domain types
pub use branch::{
    sanitize_branch_name, BranchService, BranchServiceError, BranchSyncCallbacks,
    BranchSyncOptions, BranchSyncResult, BranchType, SourceBranchInfo, SyncStrategy,
};
pub use commit::{CommitMessageError, CommitMessageService};
pub use completion::{
    get_all_completion_filenames, get_completion_cache_shell_dir, get_completion_filename,
    get_completion_shell_dir, get_completion_shell_path, get_completion_source_shell_path,
    get_shell_source_path, CompletionCheckResult, CompletionError, CompletionGenerateResult,
    CompletionRemoveResult, CompletionService, ShellCompletionStatus,
};
// Re-export config types
pub use config::{
    BranchConfig,
    BranchTemplates,
    CommitTemplates,
    ConfigError,
    GitHubAccount,
    // Verification types
    GitHubAccountInfo,
    GitHubSettings,
    GitHubVerificationResult,
    GitHubVerificationSummary,
    GlobalConfig,
    GlobalConfigRepository,
    JiraConfigInfo,
    JiraSettings,
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
    SshVerificationResult,
    TemplateConfig,
    UserConfig,
    VerificationService,
};
// Re-export git types
pub use git::{
    BlameLineInfo, BranchFilter, BranchInfo, CodePlatform, CommitChangeType, CommitFileChange,
    CommitInfo, FileStatusInfo, FileStatusType, GitError, GitRepoRepository, GitRepository,
    MergeStrategy, RemoteDirection, RemoteInfo, RepoInfo, StashApplyResult, StashEntry,
    StashPopResult, StashStat, TagCreateInfo, TagCreateScope, TagDeleteInfo, TagDeleteScope,
    WorkingTreeStatus,
};
// Re-export SSH types
pub use ssh::{SshError, SshKeyInfo, SshService};
// Re-export external service types
pub use github::{GitHubError, GitHubRepository, GitHubUser};
pub use jira::{
    extract_jira_project, extract_jira_ticket_id, validate_jira_ticket_format,
    AttachmentDownloadResult, DeleteHistoryResult, JiraAttachment, JiraComment, JiraComponent,
    JiraError, JiraIssue, JiraPriority, JiraRepository, JiraStatusConfig, JiraTransition, JiraUser,
    JiraWorkHistoryRepository, ProgressCallback, ProjectStatusConfig, StatusConfigResult,
    WorkHistoryEntry,
};
pub use path::{
    Dir, PathError, PathService, COMPLETIONS_DIR, COMPLETIONS_FILE, COMPLETION_CACHE_DIR,
    JIRA_CONFIG_FILE, MAIN_DIR, PROJECT_CONFIG_FILE, USER_CONFIG_FILE, WORKFLOW_CONFIG_DIR,
    WORKFLOW_CONFIG_FILE,
};
pub use pr::{
    get_all_change_types, get_change_type_by_index, get_change_type_by_name,
    get_change_type_index_by_branch_type, get_change_types_by_branch_type, ChangeType, PrContent,
    PrStatus, PullRequestError, PullRequestInfo, PullRequestService, PullRequestStatus,
    CHANGE_TYPES,
};
pub use summary::{
    CommitBatchAnalysis, CommitConfigAnalysis, CommitFileClassification, CommitLogicAnalysis,
    CommitSummaryAnalysis, CommitSummaryError, CommitSummaryService, CommitTestAnalysis,
    DirectoryStats, DirectoryStatusDistribution,
};
// Re-export template types
pub use template::{
    BranchTemplateVars, ChangeTypeItem, CommitTemplateVars, PrTitleTemplateVars,
    PullRequestTemplateVars,
};
