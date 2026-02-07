//! Jira 服务注册

use std::sync::Arc;

use registry::{bind, Container, Scope};

use crate::jira::{
    IssueService, IssueServiceImpl, JiraClient, JiraClientImpl, JiraRepositoryImpl,
    JiraWorkHistoryRepositoryImpl, StatusService, StatusServiceImpl, UserService, UserServiceImpl,
    WorkHistoryService, WorkHistoryServiceImpl,
};
use domain::{JiraConfigContext, JiraRepository, JiraWorkHistoryRepository, PathService};

/// 注册 Jira 相关服务
///
/// # 注册顺序和依赖关系
///
/// Factory 闭包中的 `.expect()` 表示程序员错误（注册顺序错误），而非运行时错误。
pub fn register_jira() -> registry::Result<()> {
    // Jira Client
    bind!(dyn JiraClient, |c: &Container| {
        let context = c
            .get::<dyn JiraConfigContext>()
            .expect("PROGRAMMER ERROR:JiraConfigContext must be registered before JiraClient");
        Arc::new(JiraClientImpl::new(context))
    })
    .in_scope(Scope::Singleton)?;

    // Issue Service
    bind!(dyn IssueService, |c: &Container| {
        let jira_client = c
            .get::<dyn JiraClient>()
            .expect("PROGRAMMER ERROR:JiraClient must be registered before IssueService");
        Arc::new(IssueServiceImpl::new(jira_client))
    })
    .in_scope(Scope::Singleton)?;

    // Status Service
    bind!(dyn StatusService, |c: &Container| {
        let jira_client = c
            .get::<dyn JiraClient>()
            .expect("PROGRAMMER ERROR:JiraClient must be registered before StatusService");
        let path_service = c
            .get::<dyn PathService>()
            .expect("PROGRAMMER ERROR:PathService must be registered before StatusService");
        Arc::new(StatusServiceImpl::new(jira_client, path_service))
    })
    .in_scope(Scope::Singleton)?;

    // User Service
    bind!(dyn UserService, |c: &Container| {
        let jira_client = c
            .get::<dyn JiraClient>()
            .expect("PROGRAMMER ERROR:JiraClient must be registered before UserService");
        Arc::new(UserServiceImpl::new(jira_client))
    })
    .in_scope(Scope::Singleton)?;

    // Jira Repository
    bind!(dyn JiraRepository, |c: &Container| {
        let issue_service = c
            .get::<dyn IssueService>()
            .expect("PROGRAMMER ERROR:IssueService must be registered before JiraRepository");
        let status_service = c
            .get::<dyn StatusService>()
            .expect("PROGRAMMER ERROR:StatusService must be registered before JiraRepository");
        let user_service = c
            .get::<dyn UserService>()
            .expect("PROGRAMMER ERROR:UserService must be registered before JiraRepository");
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
            .expect("PROGRAMMER ERROR:PathService must be registered before WorkHistoryService");
        Arc::new(WorkHistoryServiceImpl::new(path_service))
    })
    .in_scope(Scope::Singleton)?;

    // Jira Work History Repository
    bind!(dyn JiraWorkHistoryRepository, |c: &Container| {
        let work_history_service = c
            .get::<dyn WorkHistoryService>()
            .expect("PROGRAMMER ERROR:WorkHistoryService must be registered before JiraWorkHistoryRepository");
        Arc::new(JiraWorkHistoryRepositoryImpl::new(work_history_service))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
