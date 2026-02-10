//! GitHub 服务注册

use std::sync::Arc;

use domain::{GitHubContext, GitHubRepository, GitRepoRepository};
use registry::{try_bind, Container, RegistryError, Scope};

use crate::github::{
    GitHubClient, GitHubClientImpl, GitHubRepositoryImpl, PullRequestDiffService,
    PullRequestDiffServiceImpl, PullRequestMutationService, PullRequestMutationServiceImpl,
    PullRequestQueryService, PullRequestQueryServiceImpl, PullRequestReviewService,
    PullRequestReviewServiceImpl, ServiceContext, ServiceContextImpl,
};

/// 注册 GitHub 相关服务
///
/// # 注册顺序和依赖关系
///
/// 服务注册顺序：
/// 1. **GitHubContext** (外部注册) - 必须在调用此函数前注册
/// 2. **GitRepoRepository** (外部注册) - 必须在调用此函数前注册
/// 3. **GitHubClient** (依赖 GitHubContext)
/// 4. **ServiceContext** (依赖 GitRepoRepository)
/// 5. **PullRequestQueryService** (依赖 GitHubClient, ServiceContext)
/// 6. **其他服务** (依赖上述服务)
///
/// Factory 闭包中的 `.expect()` 表示程序员错误（注册顺序错误），而非运行时错误。
pub fn register_github() -> Result<(), RegistryError> {
    // 注册 GitHub Client (依赖外部的 GitHubContext)
    try_bind!(dyn GitHubClient, |c: &Container| {
        let context = c.get::<dyn GitHubContext>()?;
        Ok(Arc::new(GitHubClientImpl::new(context)))
    })
    .in_scope(Scope::Singleton)?;

    // Service Context
    try_bind!(dyn ServiceContext, |c: &Container| {
        let repo_repository = c.get::<dyn GitRepoRepository>()?;
        Ok(Arc::new(ServiceContextImpl::new(repo_repository)))
    })
    .in_scope(Scope::Singleton)?;

    // Pull Request Query Service
    try_bind!(dyn PullRequestQueryService, |c: &Container| {
        let client = c.get::<dyn GitHubClient>()?;
        let context = c.get::<dyn ServiceContext>()?;
        Ok(Arc::new(PullRequestQueryServiceImpl::new(client, context)))
    })
    .in_scope(Scope::Singleton)?;

    // Pull Request Mutation Service
    try_bind!(dyn PullRequestMutationService, |c: &Container| {
        let client = c.get::<dyn GitHubClient>()?;
        let query_service = c.get::<dyn PullRequestQueryService>()?;
        let context = c.get::<dyn ServiceContext>()?;
        Ok(Arc::new(PullRequestMutationServiceImpl::new(
            client,
            query_service,
            context,
        )))
    })
    .in_scope(Scope::Singleton)?;

    // Pull Request Review Service
    try_bind!(dyn PullRequestReviewService, |c: &Container| {
        let client = c.get::<dyn GitHubClient>()?;
        let query_service = c.get::<dyn PullRequestQueryService>()?;
        let context = c.get::<dyn ServiceContext>()?;
        Ok(Arc::new(PullRequestReviewServiceImpl::new(
            client,
            query_service,
            context,
        )))
    })
    .in_scope(Scope::Singleton)?;

    // Pull Request Diff Service
    try_bind!(dyn PullRequestDiffService, |c: &Container| {
        let client = c.get::<dyn GitHubClient>()?;
        let context = c.get::<dyn ServiceContext>()?;
        Ok(Arc::new(PullRequestDiffServiceImpl::new(client, context)))
    })
    .in_scope(Scope::Singleton)?;

    // GitHub Repository
    try_bind!(dyn GitHubRepository, |c: &Container| {
        let query_service = c.get::<dyn PullRequestQueryService>()?;
        let mutation_service = c.get::<dyn PullRequestMutationService>()?;
        let review_service = c.get::<dyn PullRequestReviewService>()?;
        let diff_service = c.get::<dyn PullRequestDiffService>()?;
        Ok(Arc::new(GitHubRepositoryImpl::new(
            query_service,
            mutation_service,
            review_service,
            diff_service,
        )))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
