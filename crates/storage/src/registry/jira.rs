//! Jira 服务注册

use std::sync::Arc;

use registry::{Container, RegistryError, Scope, try_bind};

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
pub fn register_jira() -> Result<(), RegistryError> {
    // Jira Client
    try_bind!(dyn JiraClient, |c: &Container| {
        let context = c.get::<dyn JiraConfigContext>()?;
        Ok(Arc::new(JiraClientImpl::new(context)))
    })
    .in_scope(Scope::Singleton)?;

    // Issue Service
    try_bind!(dyn IssueService, |c: &Container| {
        let jira_client = c.get::<dyn JiraClient>()?;
        Ok(Arc::new(IssueServiceImpl::new(jira_client)))
    })
    .in_scope(Scope::Singleton)?;

    // Status Service
    try_bind!(dyn StatusService, |c: &Container| {
        let jira_client = c.get::<dyn JiraClient>()?;
        let path_service = c.get::<dyn PathService>()?;
        Ok(Arc::new(StatusServiceImpl::new(jira_client, path_service)))
    })
    .in_scope(Scope::Singleton)?;

    // User Service
    try_bind!(dyn UserService, |c: &Container| {
        let jira_client = c
            .get::<dyn JiraClient>()
            .expect("PROGRAMMER ERROR:JiraClient must be registered before UserService");
        Ok(Arc::new(UserServiceImpl::new(jira_client)))
    })
    .in_scope(Scope::Singleton)?;

    // Jira Repository
    try_bind!(dyn JiraRepository, |c: &Container| {
        let issue_service = c.get::<dyn IssueService>()?;
        let status_service = c.get::<dyn StatusService>()?;
        let user_service = c.get::<dyn UserService>()?;
        Ok(Arc::new(JiraRepositoryImpl::new(
            issue_service,
            status_service,
            user_service,
        )))
    })
    .in_scope(Scope::Singleton)?;

    // Work History Service（不依赖 JiraClient，独立运行）
    try_bind!(dyn WorkHistoryService, |c: &Container| {
        let path_service = c.get::<dyn PathService>()?;
        Ok(Arc::new(WorkHistoryServiceImpl::new(path_service)))
    })
    .in_scope(Scope::Singleton)?;

    // Jira Work History Repository
    try_bind!(dyn JiraWorkHistoryRepository, |c: &Container| {
        let work_history_service = c.get::<dyn WorkHistoryService>()?;
        Ok(Arc::new(JiraWorkHistoryRepositoryImpl::new(
            work_history_service,
        )))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
