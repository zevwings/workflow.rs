//! Services Registry
//!
//! 定义 services crate 的服务注册和依赖注入。

use std::sync::Arc;

use llm::{LLMConfigContext, LLMExecutor};
use registry::{bind, try_bind, Container, Scope};

use crate::branch::BranchServiceImpl;
use crate::alias::AliasServiceImpl;
use crate::completion::CompletionServiceImpl;
use crate::path::PathServiceImpl;
use crate::pull_request::PullRequestServiceImpl;

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
pub fn register_services() -> registry::Result<()> {
    // AliasService - 依赖 GlobalConfigRepository
    try_bind!(dyn domain::AliasService, |c: &Container| {
        let config_repo = c.get::<dyn domain::GlobalConfigRepository>()?;
        Ok(Arc::new(AliasServiceImpl::new(config_repo)))
    })
    .in_scope(Scope::Singleton)?;


    try_bind!(dyn domain::BranchService, |c: &Container| {
        let llm_executor = c.get::<dyn LLMExecutor>()?;
        let llm_context = c.get::<dyn LLMConfigContext>()?;
        Ok(Arc::new(BranchServiceImpl::new(llm_executor, llm_context)))
    })
    .in_scope(Scope::Singleton)?;

    // PullRequestService - 依赖 GitRepository、GitHubRepository 和 LLMRepository
    try_bind!(dyn domain::PullRequestService, |c: &Container| {
        let git_repo = c.get::<dyn domain::GitRepository>()?;
        let github_repo = c.get::<dyn domain::GitHubRepository>()?;

        Ok(Arc::new(PullRequestServiceImpl::new(git_repo, github_repo)))
    })
    .in_scope(Scope::Singleton)?;

    // CompletionService - 无外部依赖
    bind!(dyn domain::CompletionService, |c: &Container| {
        let path_service = c
            .get::<dyn domain::PathService>()
            .expect("PathService must be registered before CompletionService");
        Arc::new(CompletionServiceImpl::new(path_service))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn domain::PathService, |_c: &Container| {
        Arc::new(PathServiceImpl::new())
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
