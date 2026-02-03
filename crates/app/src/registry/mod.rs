//! 应用层服务注册
//!
//! 负责组合各个 crate 的依赖注入容器，统一管理所有服务。

use once_cell::sync::Lazy;
use std::sync::Arc;

/// 应用程序初始化标记
///
/// 确保所有模块都已注册
static APP_INITIALIZED: Lazy<()> = Lazy::new(|| {
    // 按依赖顺序初始化模块
    storage::register_storage().expect("Failed to register storage module");
    let _services = services::build_services_module();
});

/// 确保应用已初始化
fn ensure_initialized() {
    Lazy::force(&APP_INITIALIZED);
}

/// 从全局容器获取服务
///
/// # 示例
///
/// ```rust,ignore
/// let repo = get_service::<dyn domain::GlobalConfigRepository>();
/// ```
pub fn get_service<T: 'static + Send + Sync + ?Sized>() -> Arc<T> {
    ensure_initialized();
    registry::resolve::<T>().expect("Failed to resolve service")
}

// ============================================================================
// 便捷的服务获取函数
// ============================================================================

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

/// 获取 GitRepoRepository（仅 get_repo_info，供 GitHub 等使用）
pub fn get_git_repo_repository() -> Arc<dyn domain::GitRepoRepository> {
    get_service::<dyn domain::GitRepoRepository>()
}

/// 获取 GitRepository（完整 Git 操作）
pub fn get_git_repository() -> Arc<dyn domain::GitRepository> {
    get_service::<dyn domain::GitRepository>()
}

/// 获取 GitHubRepository
pub fn get_github_repository() -> Arc<dyn domain::GitHubRepository> {
    get_service::<dyn domain::GitHubRepository>()
}

/// 获取 JiraRepository
pub fn get_jira_repository() -> Arc<dyn domain::JiraRepository> {
    get_service::<dyn domain::JiraRepository>()
}

/// 获取 LLMRepository
pub fn get_llm_repository() -> Arc<dyn domain::llm::repository::LLMRepository> {
    get_service::<dyn domain::llm::repository::LLMRepository>()
}

/// 获取 PullRequestService
pub fn get_pull_request_service() -> Arc<dyn domain::pr::service::PullRequestService> {
    get_service::<dyn domain::pr::service::PullRequestService>()
}
