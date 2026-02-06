//! Jira 服务注册

use std::sync::Arc;

use registry::{bind, Container, Scope};

use crate::jira::{
    IssueService, IssueServiceImpl, JiraClient, JiraClientImpl, JiraConfigContextImpl,
    JiraRepositoryImpl, JiraWorkHistoryRepositoryImpl, StatusService, StatusServiceImpl,
    UserService, UserServiceImpl, WorkHistoryService, WorkHistoryServiceImpl,
};
use domain::{
    GlobalConfigRepository, JiraConfigContext, JiraRepository, JiraWorkHistoryRepository,
    PathService,
};

/// 注册 Jira 相关服务
pub fn register_jira() -> registry::Result<()> {
    // Jira Config Context
    bind!(dyn JiraConfigContext, |c: &Container| {
        let global_config = c
            .get::<dyn GlobalConfigRepository>()
            .expect("GlobalConfigRepository must be registered before JiraConfigContext");
        Arc::new(JiraConfigContextImpl::new(global_config))
    })
    .in_scope(Scope::Singleton)?;

    // Jira Client
    bind!(dyn JiraClient, |c: &Container| {
        let context = c
            .get::<dyn JiraConfigContext>()
            .expect("JiraConfigContext must be registered before JiraClient");
        Arc::new(JiraClientImpl::new(context))
    })
    .in_scope(Scope::Singleton)?;

    // Issue Service
    bind!(dyn IssueService, |c: &Container| {
        let jira_client = c
            .get::<dyn JiraClient>()
            .expect("JiraClient must be registered before IssueService");
        Arc::new(IssueServiceImpl::new(jira_client))
    })
    .in_scope(Scope::Singleton)?;

    // Status Service
    bind!(dyn StatusService, |c: &Container| {
        let jira_client = c
            .get::<dyn JiraClient>()
            .expect("JiraClient must be registered before StatusService");
        let path_service = c
            .get::<dyn PathService>()
            .expect("PathService must be registered before StatusService");
        Arc::new(StatusServiceImpl::new(jira_client, path_service))
    })
    .in_scope(Scope::Singleton)?;

    // User Service
    bind!(dyn UserService, |c: &Container| {
        let jira_client = c
            .get::<dyn JiraClient>()
            .expect("JiraClient must be registered before UserService");
        Arc::new(UserServiceImpl::new(jira_client))
    })
    .in_scope(Scope::Singleton)?;

    // Jira Repository
    bind!(dyn JiraRepository, |c: &Container| {
        let issue_service = c
            .get::<dyn IssueService>()
            .expect("IssueService must be registered before JiraRepository");
        let status_service = c
            .get::<dyn StatusService>()
            .expect("StatusService must be registered before JiraRepository");
        let user_service = c
            .get::<dyn UserService>()
            .expect("UserService must be registered before JiraRepository");
        Arc::new(JiraRepositoryImpl::new(
            issue_service,
            status_service,
            user_service,
        ))
    })
    .in_scope(Scope::Singleton)?;

    // Work History Service（不依赖 JiraClient，独立运行）
    bind!(dyn WorkHistoryService, |c: &Container| {
        let path_service = c
            .get::<dyn PathService>()
            .expect("PathService must be registered before WorkHistoryService");
        Arc::new(WorkHistoryServiceImpl::new(path_service))
    })
    .in_scope(Scope::Singleton)?;

    // Jira Work History Repository
    bind!(dyn JiraWorkHistoryRepository, |c: &Container| {
        let work_history_service = c
            .get::<dyn WorkHistoryService>()
            .expect("WorkHistoryService must be registered before JiraWorkHistoryRepository");
        Arc::new(JiraWorkHistoryRepositoryImpl::new(work_history_service))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
