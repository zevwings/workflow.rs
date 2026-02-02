//! GitHub 服务注册

use std::sync::Arc;

use domain::{GitHubContext, GitHubRepository, GitRepoRepository, GlobalConfigRepository};
use registry::{bind, Container, Scope};

use crate::github::{
    GitHubClient, GitHubClientImpl, GitHubContextImpl, GitHubRepositoryImpl,
    PullRequestDiffService, PullRequestDiffServiceImpl, PullRequestMutationService,
    PullRequestMutationServiceImpl, PullRequestQueryService, PullRequestQueryServiceImpl,
    PullRequestReviewService, PullRequestReviewServiceImpl, ServiceContext, ServiceContextImpl,
};

/// 注册 GitHub 相关服务
pub fn register_github() -> registry::Result<()> {
    // GitHub Context
    bind!(dyn GitHubContext, |c: &Container| {
        let global_config = c
            .get::<dyn GlobalConfigRepository>()
            .expect("GlobalConfigRepository must be registered before GitHubContext");
        Arc::new(GitHubContextImpl::new(global_config))
    })
    .in_scope(Scope::Singleton)?;

    // GitHub Client
    bind!(dyn GitHubClient, |c: &Container| {
        let context = c
            .get::<dyn GitHubContext>()
            .expect("GitHubContext must be registered before GitHubClient");
        Arc::new(GitHubClientImpl::new(context))
    })
    .in_scope(Scope::Singleton)?;

    // Service Context
    bind!(dyn ServiceContext, |c: &Container| {
        let repo_repository = c
            .get::<dyn GitRepoRepository>()
            .expect("GitRepoRepository must be registered before ServiceContext");
        Arc::new(ServiceContextImpl::new(repo_repository))
    })
    .in_scope(Scope::Singleton)?;

    // Pull Request Query Service
    bind!(dyn PullRequestQueryService, |c: &Container| {
        let client = c
            .get::<dyn GitHubClient>()
            .expect("GitHubClient must be registered before PullRequestQueryService");
        let context = c
            .get::<dyn ServiceContext>()
            .expect("ServiceContext must be registered before PullRequestQueryService");
        Arc::new(PullRequestQueryServiceImpl::new(client, context))
    })
    .in_scope(Scope::Singleton)?;

    // Pull Request Mutation Service
    bind!(dyn PullRequestMutationService, |c: &Container| {
        let client = c
            .get::<dyn GitHubClient>()
            .expect("GitHubClient must be registered before PullRequestMutationService");
        let query_service = c
            .get::<dyn PullRequestQueryService>()
            .expect("PullRequestQueryService must be registered before PullRequestMutationService");
        let context = c
            .get::<dyn ServiceContext>()
            .expect("ServiceContext must be registered before PullRequestMutationService");
        Arc::new(PullRequestMutationServiceImpl::new(
            client,
            query_service,
            context,
        ))
    })
    .in_scope(Scope::Singleton)?;

    // Pull Request Review Service
    bind!(dyn PullRequestReviewService, |c: &Container| {
        let client = c
            .get::<dyn GitHubClient>()
            .expect("GitHubClient must be registered before PullRequestReviewService");
        let query_service = c
            .get::<dyn PullRequestQueryService>()
            .expect("PullRequestQueryService must be registered before PullRequestReviewService");
        let context = c
            .get::<dyn ServiceContext>()
            .expect("ServiceContext must be registered before PullRequestReviewService");
        Arc::new(PullRequestReviewServiceImpl::new(
            client,
            query_service,
            context,
        ))
    })
    .in_scope(Scope::Singleton)?;

    // Pull Request Diff Service
    bind!(dyn PullRequestDiffService, |c: &Container| {
        let client = c
            .get::<dyn GitHubClient>()
            .expect("GitHubClient must be registered before PullRequestDiffService");
        let context = c
            .get::<dyn ServiceContext>()
            .expect("ServiceContext must be registered before PullRequestDiffService");
        Arc::new(PullRequestDiffServiceImpl::new(client, context))
    })
    .in_scope(Scope::Singleton)?;

    // GitHub Repository
    bind!(dyn GitHubRepository, |c: &Container| {
        let query_service = c
            .get::<dyn PullRequestQueryService>()
            .expect("PullRequestQueryService must be registered before GitHubRepository");
        let mutation_service = c
            .get::<dyn PullRequestMutationService>()
            .expect("PullRequestMutationService must be registered before GitHubRepository");
        let review_service = c
            .get::<dyn PullRequestReviewService>()
            .expect("PullRequestReviewService must be registered before GitHubRepository");
        let diff_service = c
            .get::<dyn PullRequestDiffService>()
            .expect("PullRequestDiffService must be registered before GitHubRepository");
        Arc::new(GitHubRepositoryImpl::new(
            query_service,
            mutation_service,
            review_service,
            diff_service,
        ))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
