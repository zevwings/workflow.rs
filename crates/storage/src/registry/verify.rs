//! VerificationService 注册

use std::sync::Arc;

use llm::LLMExecutor;
use registry::{bind, Container, Scope};

use crate::config::VerificationServiceImpl;
use domain::{GitHubRepository, GlobalConfigRepository, JiraRepository, VerificationService};

/// 注册 VerificationService
pub fn register_verify() -> registry::Result<()> {
    bind!(dyn VerificationService, |c: &Container| {
        let llm_executor = c
            .get::<dyn LLMExecutor>()
            .expect("LLMExecutor must be registered before VerificationService");
        let config_repository = c
            .get::<dyn GlobalConfigRepository>()
            .expect("GlobalConfigRepository must be registered before VerificationService");
        let jira_repository = c
            .get::<dyn JiraRepository>()
            .expect("JiraRepository must be registered before VerificationService");
        let github_repository = c
            .get::<dyn GitHubRepository>()
            .expect("GitHubRepository must be registered before VerificationService");

        Arc::new(VerificationServiceImpl::new(
            llm_executor,
            config_repository,
            jira_repository,
            github_repository,
        ))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
