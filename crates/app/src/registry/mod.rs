//! 应用层服务注册
//!
//! 负责组合各个 crate 的依赖注入容器，统一管理所有服务。

mod app;
mod context;

use std::sync::{Arc, LazyLock};

/// 应用程序初始化标记
///
/// 确保所有模块都已注册
static APP_INITIALIZED: LazyLock<()> = LazyLock::new(|| {
    // 按依赖顺序初始化模块
    // 注意：这些是启动时的关键初始化，失败时程序无法继续运行

    // 0. 首先注册 llm 层服务（基础仓储实现）
    if let Err(e) = context::register_context() {
        panic!("Failed to register llm module: {e}");
    }

    // 1. 首先注册 storage 层服务（基础仓储实现）
    if let Err(e) = storage::register_storage() {
        panic!("Failed to register storage module: {e}");
    }

    // 2. 然后注册 services 层服务（应用服务，依赖 storage）
    if let Err(e) = services::register_services() {
        panic!("Failed to register services module: {e}");
    }

    // 3. 最后注册 app 层服务（应用层特有服务，可依赖 storage 和 services）
    if let Err(e) = app::register_app() {
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
/// let repo = get_service::<dyn domain::GlobalConfigRepository>();
/// ```
pub fn get_service<T: 'static + Send + Sync + ?Sized>() -> Arc<T> {
    ensure_initialized();
    registry::resolve::<T>().unwrap_or_else(|e| {
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
pub fn get_alias_service() -> Arc<dyn domain::alias::AliasService> {
    get_service::<dyn domain::alias::AliasService>()
}

pub fn get_branch_service() -> Arc<dyn domain::branch::BranchService> {
    get_service::<dyn domain::branch::BranchService>()
}

/// 获取 GlobalConfigRepository
pub fn get_global_config_repository() -> Arc<dyn domain::GlobalConfigRepository> {
    get_service::<dyn domain::GlobalConfigRepository>()
}

/// 获取 RepoConfigRepository
pub fn get_repo_config_repository() -> Arc<dyn domain::RepoConfigRepository> {
    get_service::<dyn domain::RepoConfigRepository>()
}

/// 获取 VerificationService
pub fn get_verification_service() -> Arc<dyn domain::VerificationService> {
    get_service::<dyn domain::VerificationService>()
}

/// 获取 GitRepository（完整 Git 操作）
pub fn get_git_repository() -> Arc<dyn domain::GitRepository> {
    get_service::<dyn domain::GitRepository>()
}

/// 获取 CommitSummaryService（三阶段提交分析）
pub fn get_commit_summary_service() -> Arc<dyn domain::CommitSummaryService> {
    get_service::<dyn domain::CommitSummaryService>()
}

/// 获取 GitHubRepository
pub fn get_github_repository() -> Arc<dyn domain::GitHubRepository> {
    get_service::<dyn domain::GitHubRepository>()
}

/// 获取 JiraRepository
pub fn get_jira_repository() -> Arc<dyn domain::JiraRepository> {
    get_service::<dyn domain::JiraRepository>()
}

/// 获取 JiraWorkHistoryRepository
pub fn get_jira_work_history_repository() -> Arc<dyn domain::JiraWorkHistoryRepository> {
    get_service::<dyn domain::JiraWorkHistoryRepository>()
}

/// 获取 PullRequestService
pub fn get_pull_request_service() -> Arc<dyn domain::pr::service::PullRequestService> {
    get_service::<dyn domain::pr::service::PullRequestService>()
}

/// 获取 CompletionService
pub fn get_completion_service() -> Arc<dyn domain::completion::service::CompletionService> {
    get_service::<dyn domain::completion::service::CompletionService>()
}

/// 获取 PathService
pub fn get_path_service() -> Arc<dyn domain::path::service::PathService> {
    get_service::<dyn domain::path::service::PathService>()
}
