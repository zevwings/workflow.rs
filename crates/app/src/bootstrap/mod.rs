//! 应用层服务引导
//!
//! 负责组合各个 crate 的依赖注入容器，统一管理所有服务。

mod app;
mod context;

use std::sync::{Arc, LazyLock};

use domain::{
    AliasService, BranchService, CommitMessageService, CommitSummaryService, CompletionService,
    GitHubRepository, GitRepository, GlobalConfigRepository, JiraRepository,
    JiraWorkHistoryRepository, PathService, PullRequestService, RepoConfigRepository,
    VerificationService,
};
use llm::register_llm;
use services::register_services;
use storage::register_storage;

use crate::logger::LoggerManager;

/// 应用程序初始化标记
///
/// 确保所有模块都已注册。
///
/// # Panic
///
/// 如果任何模块注册失败，将 panic 并终止程序。
/// 由于 `LazyLock` 无法返回 `Result`，且模块注册失败意味着
/// 应用程序无法正常运行，因此使用 panic 是合理的。
static APP_INITIALIZED: LazyLock<()> = LazyLock::new(|| {
    // 按依赖顺序初始化模块
    // 注意：这些是启动时的关键初始化，失败时程序无法继续运行

    // 0. 首先注册配置上下文服务（LLMConfigContext, JiraConfigContext, GitHubContext）
    if let Err(e) = context::register_context() {
        eprintln!("Fatal: Failed to register context module: {e}");
        panic!("Failed to register context module: {e}");
    }

    // 1. 注册 LLM 层服务（LLMClient, LLMExecutor）
    if let Err(e) = register_llm() {
        eprintln!("Fatal: Failed to register llm module: {e}");
        panic!("Failed to register llm module: {e}");
    }

    // 2. 注册 storage 层服务（基础仓储实现）
    if let Err(e) = register_storage() {
        eprintln!("Fatal: Failed to register storage module: {e}");
        panic!("Failed to register storage module: {e}");
    }

    // 3. 然后注册 services 层服务（应用服务，依赖 storage 和 llm）
    if let Err(e) = register_services() {
        eprintln!("Fatal: Failed to register services module: {e}");
        panic!("Failed to register services module: {e}");
    }

    // 4. 最后注册 app 层服务（应用层特有服务，可依赖 storage 和 services）
    if let Err(e) = app::register_app() {
        eprintln!("Fatal: Failed to register app module: {e}");
        panic!("Failed to register app module: {e}");
    }
});

/// 确保应用已初始化
fn ensure_initialized() {
    LazyLock::force(&APP_INITIALIZED);
}

/// 从全局容器获取服务
///
/// 这是依赖注入的核心函数，用于获取已注册的服务实例。
/// 首次调用时会自动初始化所有模块（storage, services, app）。
///
/// # 类型参数
///
/// * `T` - 服务 trait 类型，必须实现 `Send + Sync + 'static`
///
/// # 返回
///
/// 返回服务的 `Arc<T>` 智能指针
///
/// # Panic
///
/// 如果服务未注册或初始化失败，将 panic 并显示详细错误信息。
///
/// # 示例
///
/// ```rust,ignore
/// let repo = get_service::<dyn GlobalConfigRepository>();
/// ```
pub fn get_service<T: 'static + Send + Sync + ?Sized>() -> Arc<T> {
    ensure_initialized();
    di::resolve::<T>().unwrap_or_else(|e| {
        panic!(
            "Failed to resolve service {}: {}",
            std::any::type_name::<T>(),
            e
        )
    })
}

// ============================================================================
// 便捷的服务获取函数
// ============================================================================

/// 获取 AliasService
pub fn get_alias_service() -> Arc<dyn AliasService> {
    get_service::<dyn AliasService>()
}

/// 获取 BranchService
pub fn get_branch_service() -> Arc<dyn BranchService> {
    get_service::<dyn BranchService>()
}

/// 获取 GlobalConfigRepository
pub fn get_global_config_repository() -> Arc<dyn GlobalConfigRepository> {
    get_service::<dyn GlobalConfigRepository>()
}

/// 获取 RepoConfigRepository
pub fn get_repo_config_repository() -> Arc<dyn RepoConfigRepository> {
    get_service::<dyn RepoConfigRepository>()
}

/// 获取 VerificationService
pub fn get_verification_service() -> Arc<dyn VerificationService> {
    get_service::<dyn VerificationService>()
}

/// 获取 GitRepository（完整 Git 操作）
pub fn get_git_repository() -> Arc<dyn GitRepository> {
    get_service::<dyn GitRepository>()
}

/// 获取 CommitSummaryService（三阶段提交分析）
pub fn get_commit_summary_service() -> Arc<dyn CommitSummaryService> {
    get_service::<dyn CommitSummaryService>()
}

/// 获取 CommitMessageService（单次提交 message 生成）
pub fn get_commit_message_service() -> Arc<dyn CommitMessageService> {
    get_service::<dyn CommitMessageService>()
}

/// 获取 GitHubRepository
pub fn get_github_repository() -> Arc<dyn GitHubRepository> {
    get_service::<dyn GitHubRepository>()
}

/// 获取 JiraRepository
pub fn get_jira_repository() -> Arc<dyn JiraRepository> {
    get_service::<dyn JiraRepository>()
}

/// 获取 JiraWorkHistoryRepository
pub fn get_jira_work_history_repository() -> Arc<dyn JiraWorkHistoryRepository> {
    get_service::<dyn JiraWorkHistoryRepository>()
}

/// 获取 PullRequestService
pub fn get_pull_request_service() -> Arc<dyn PullRequestService> {
    get_service::<dyn PullRequestService>()
}

/// 获取 CompletionService
pub fn get_completion_service() -> Arc<dyn CompletionService> {
    get_service::<dyn CompletionService>()
}

/// 获取 PathService
pub fn get_path_service() -> Arc<dyn PathService> {
    get_service::<dyn PathService>()
}

/// 获取 LoggerManager
pub fn get_logger_manager() -> Arc<dyn LoggerManager> {
    get_service::<dyn LoggerManager>()
}
