//! 领域层（DDD Domain Layer）
//!
//! 包含实体、值对象、仓储接口、领域服务接口和领域异常
//!
//! 按业务域组织，每个业务域包含相关的实体、服务接口、仓储接口和错误类型。

pub mod branch;
pub mod commit;
pub mod completion;
pub mod config;
pub mod errors;
pub mod git;
pub mod github;
pub mod jira;
pub mod llm;
pub mod pr;
pub mod template;

// Re-export public types
// 避免使用 glob 导出以防止与子模块名称冲突（如 `template`）
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
    BranchNaming, BranchService, BranchSync, BranchSyncCallbacks, BranchSyncOptions,
    BranchSyncResult, BranchType, SourceBranchInfo, SyncStrategy,
};
pub use commit::{AmendPreview, CommitAmend, CommitReword, CommitService, CommitSquash};
pub use completion::{
    Completion, CompletionConfigResult, CompletionRemovalResult, CompletionService,
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
    extract_jira_project, validate_jira_ticket_format, AttachmentDownloadResult, JiraAttachment,
    JiraComment, JiraConfigContext, JiraError, JiraIssue, JiraRepository, JiraStatusConfig,
    JiraUser, ProjectStatusConfig, StatusConfigResult,
};
pub use llm::{
    LLMConfigContext, LLMError, LLMRepository, PullRequestContent, PullRequestReword,
    PullRequestSummary, SupportedLanguage,
};
