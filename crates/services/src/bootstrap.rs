//! Services Bootstrap
//!
//! 定义 services crate 的服务引导和依赖注入。

use std::sync::Arc;

use di::{bind, Container, InjectionError, Scope};
use domain::{
    AliasService, BranchService, CommitMessageService, CommitSummaryService, CompletionService,
    GitHubRepository, GitRepository, GlobalConfigRepository, PathService, PullRequestService,
};
use llm::{LLMConfigContext, LLMExecutor};

use crate::{
    alias::AliasServiceImpl, branch::BranchServiceImpl, commit::CommitMessageServiceImpl,
    completion::CompletionServiceImpl, path::PathServiceImpl, pull_request::PullRequestServiceImpl,
    summary::CommitSummaryServiceImpl,
};

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
pub fn register_services() -> Result<(), InjectionError> {
    // AliasService - 依赖 GlobalConfigRepository
    bind!(dyn AliasService, |c: &Container| {
        let config_repo = c.get::<dyn GlobalConfigRepository>()?;
        Ok(Arc::new(AliasServiceImpl::new(config_repo)))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn BranchService, |c: &Container| {
        let llm_executor = c.get::<dyn LLMExecutor>()?;
        let llm_context = c.get::<dyn LLMConfigContext>()?;
        Ok(Arc::new(BranchServiceImpl::new(llm_executor, llm_context)))
    })
    .in_scope(Scope::Singleton)?;

    // PullRequestService - 依赖 GitRepository、GitHubRepository、CommitSummaryService
    bind!(dyn PullRequestService, |c: &Container| {
        let git_repo = c.get::<dyn GitRepository>()?;
        let github_repo = c.get::<dyn GitHubRepository>()?;
        let commit_summary_service = c.get::<dyn CommitSummaryService>()?;

        Ok(Arc::new(PullRequestServiceImpl::new(
            git_repo,
            github_repo,
            commit_summary_service,
        )))
    })
    .in_scope(Scope::Singleton)?;

    // CommitSummaryService - 依赖 GitRepository、LLMExecutor、LLMConfigContext
    bind!(dyn CommitSummaryService, |c: &Container| {
        let git_repo = c.get::<dyn GitRepository>()?;
        let llm_executor = c.get::<dyn LLMExecutor>()?;
        let llm_context = c.get::<dyn LLMConfigContext>()?;
        Ok(Arc::new(CommitSummaryServiceImpl::new(
            git_repo,
            llm_executor,
            llm_context,
        )))
    })
    .in_scope(Scope::Singleton)?;

    // CommitMessageService - 依赖 GitRepository、LLMExecutor、LLMConfigContext
    bind!(dyn CommitMessageService, |c: &Container| {
        let git_repo = c.get::<dyn GitRepository>()?;
        let llm_executor = c.get::<dyn LLMExecutor>()?;
        let llm_context = c.get::<dyn LLMConfigContext>()?;
        Ok(Arc::new(CommitMessageServiceImpl::new(
            git_repo,
            llm_executor,
            llm_context,
        )))
    })
    .in_scope(Scope::Singleton)?;

    // CompletionService - 依赖 PathService
    bind!(dyn CompletionService, |c: &Container| {
        let path_service = c.get::<dyn PathService>()?;
        Ok(Arc::new(CompletionServiceImpl::new(path_service)))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn PathService, |_c: &Container| {
        Ok(Arc::new(PathServiceImpl::new()))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
