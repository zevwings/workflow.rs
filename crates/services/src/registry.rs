//! Services Registry
//!
//! 定义 services crate 的服务注册和依赖注入。

use registry::{bind, Container, Scope};

/// Services 模块标记类型
///
/// 用于标识 services 层服务的注册状态
#[derive(Debug, Clone, Copy)]
pub struct ServicesModule;

/// 构建 Services 模块
///
/// 注册所有 services 层的服务，包括：
/// - PullRequestService
pub fn build_services_module() -> ServicesModule {
    register_services().expect("Failed to register services");
    ServicesModule
}

/// 注册所有 services 服务
fn register_services() -> registry::Result<()> {
    use std::sync::Arc;

    // PullRequestService - 依赖 GitRepository、GitHubRepository 和 LLMRepository
    bind!(dyn domain::PullRequestService, |c: &Container| {
        let git_repo = c
            .get::<dyn domain::GitRepository>()
            .expect("GitRepository not found");
        let github_repo = c
            .get::<dyn domain::GitHubRepository>()
            .expect("GitHubRepository not found");
        let llm_repo = c
            .get::<dyn domain::LLMRepository>()
            .expect("LLMRepository not found");

        Arc::new(crate::PullRequestServiceImpl::new(
            git_repo,
            github_repo,
            llm_repo,
        ))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
