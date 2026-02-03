//! Services Registry
//!
//! 定义 services crate 的服务注册和依赖注入。

use std::sync::Arc;

use registry::{bind, try_bind, Container, Scope};

/// Services 模块标记类型
///
/// 用于标识 services 层服务的注册状态
#[derive(Debug, Clone, Copy)]
pub struct ServicesModule;

/// 构建 Services 模块
///
/// 注册所有 services 层的服务，包括：
/// - AliasService
/// - PullRequestService
/// - CompletionService
///
/// # 错误
///
/// 如果服务注册失败，返回 `registry::Error`。
pub fn register_services() -> registry::Result<ServicesModule> {
    // AliasService - 依赖 GlobalConfigRepository
    try_bind!(dyn domain::AliasService, |c: &Container| {
        let config_repo = c.get::<dyn domain::GlobalConfigRepository>()?;
        Ok(Arc::new(crate::AliasServiceImpl::new(config_repo)))
    })
    .in_scope(Scope::Singleton)?;

    // PullRequestService - 依赖 GitRepository、GitHubRepository 和 LLMRepository
    try_bind!(dyn domain::PullRequestService, |c: &Container| {
        let git_repo = c.get::<dyn domain::GitRepository>()?;
        let github_repo = c.get::<dyn domain::GitHubRepository>()?;
        let llm_repo = c.get::<dyn domain::LLMRepository>()?;

        Ok(Arc::new(crate::PullRequestServiceImpl::new(
            git_repo,
            github_repo,
            llm_repo,
        )))
    })
    .in_scope(Scope::Singleton)?;

    // CompletionService - 无外部依赖
    bind!(dyn domain::CompletionService, |_c: &Container| {
        Arc::new(crate::CompletionServiceImpl::new())
    })
    .in_scope(Scope::Singleton)?;

    Ok(ServicesModule)
}
