//! VerificationService 注册

use std::sync::Arc;

use llm::LLMExecutor;
use registry::{try_bind, Container, RegistryError, Scope};

use crate::config::VerificationServiceImpl;
use domain::{GitHubRepository, GlobalConfigRepository, JiraRepository, VerificationService};

/// 注册 VerificationService
///
/// # 注册顺序和依赖关系
///
/// Factory 闭包中的 `.expect()` 表示程序员错误（注册顺序错误），而非运行时错误。
pub fn register_verify() -> Result<(), RegistryError> {
    try_bind!(dyn VerificationService, |c: &Container| {
        let llm_executor = c.get::<dyn LLMExecutor>()?;
        let config_repository = c.get::<dyn GlobalConfigRepository>()?;
        let jira_repository = c.get::<dyn JiraRepository>()?;
        let github_repository = c.get::<dyn GitHubRepository>()?;
        Ok(Arc::new(VerificationServiceImpl::new(
            llm_executor,
            config_repository,
            jira_repository,
            github_repository,
        )))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
