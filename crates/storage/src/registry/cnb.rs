//! CNB 服务注册

use std::sync::Arc;

use domain::{CNBContext, CNBRepository, GitRepository, GlobalConfigRepository};
use registry::{bind, Container, Scope};

use crate::cnb::{
    CNBClient, CNBClientImpl, CNBContextImpl, CNBRepositoryImpl, PullRequestDiffService,
    PullRequestDiffServiceImpl, PullRequestMutationService, PullRequestMutationServiceImpl,
    PullRequestQueryService, PullRequestQueryServiceImpl, PullRequestReviewService,
    PullRequestReviewServiceImpl, ServiceContext, ServiceContextImpl,
};

/// 注册 CNB 相关服务
pub fn register_cnb() -> registry::Result<()> {
    // CNB Context
    bind!(dyn CNBContext, |c: &Container| {
        let config_repo = c
            .get::<dyn GlobalConfigRepository>()
            .expect("GlobalConfigRepository must be registered before CNBContext");
        let git_repo = c
            .get::<dyn GitRepository>()
            .expect("GitRepository must be registered before CNBContext");
        Arc::new(CNBContextImpl::new(git_repo, config_repo))
    })
    .in_scope(Scope::Singleton)?;

    // CNB Client
    bind!(dyn CNBClient, |c: &Container| {
        let context = c
            .get::<dyn CNBContext>()
            .expect("CNBContext must be registered before CNBClient");
        Arc::new(CNBClientImpl::new(context))
    })
    .in_scope(Scope::Singleton)?;

    // Service Context
    bind!(dyn ServiceContext, |c: &Container| {
        let cnb_context = c
            .get::<dyn CNBContext>()
            .expect("CNBContext must be registered before ServiceContext");
        Arc::new(ServiceContextImpl::new(cnb_context))
    })
    .in_scope(Scope::Singleton)?;

    // Pull Request Query Service
    bind!(dyn PullRequestQueryService, |c: &Container| {
        let client = c
            .get::<dyn CNBClient>()
            .expect("CNBClient must be registered before PullRequestQueryService");
        let context = c
            .get::<dyn ServiceContext>()
            .expect("ServiceContext must be registered before PullRequestQueryService");
        Arc::new(PullRequestQueryServiceImpl::new(client, context))
    })
    .in_scope(Scope::Singleton)?;

    // Pull Request Mutation Service
    bind!(dyn PullRequestMutationService, |c: &Container| {
        let client = c
            .get::<dyn CNBClient>()
            .expect("CNBClient must be registered before PullRequestMutationService");
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
            .get::<dyn CNBClient>()
            .expect("CNBClient must be registered before PullRequestReviewService");
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
            .get::<dyn CNBClient>()
            .expect("CNBClient must be registered before PullRequestDiffService");
        let context = c
            .get::<dyn ServiceContext>()
            .expect("ServiceContext must be registered before PullRequestDiffService");
        Arc::new(PullRequestDiffServiceImpl::new(client, context))
    })
    .in_scope(Scope::Singleton)?;

    // CNB Repository
    bind!(dyn CNBRepository, |c: &Container| {
        let query_service = c
            .get::<dyn PullRequestQueryService>()
            .expect("PullRequestQueryService must be registered before CNBRepository");
        let mutation_service = c
            .get::<dyn PullRequestMutationService>()
            .expect("PullRequestMutationService must be registered before CNBRepository");
        let review_service = c
            .get::<dyn PullRequestReviewService>()
            .expect("PullRequestReviewService must be registered before CNBRepository");
        let diff_service = c
            .get::<dyn PullRequestDiffService>()
            .expect("PullRequestDiffService must be registered before CNBRepository");
        Arc::new(CNBRepositoryImpl::new(
            query_service,
            mutation_service,
            review_service,
            diff_service,
        ))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
