//! VerificationService 注册

use std::sync::Arc;

use registry::{bind, Container, Scope};

use crate::config::VerificationServiceImpl;
use domain::{
    CNBRepository, GitHubRepository, GlobalConfigRepository, JiraRepository, LLMRepository,
    VerificationService,
};

/// 注册 VerificationService
pub fn register_verify() -> registry::Result<()> {
    bind!(dyn VerificationService, |c: &Container| {
        let config_repository = c
            .get::<dyn GlobalConfigRepository>()
            .expect("GlobalConfigRepository must be registered before VerificationService");
        let llm_repository = c
            .get::<dyn LLMRepository>()
            .expect("LLMRepository must be registered before VerificationService");
        let jira_repository = c
            .get::<dyn JiraRepository>()
            .expect("JiraRepository must be registered before VerificationService");
        let github_repository = c
            .get::<dyn GitHubRepository>()
            .expect("GitHubRepository must be registered before VerificationService");
        let cnb_repository = c
            .get::<dyn CNBRepository>()
            .expect("CNBRepository must be registered before VerificationService");

        Arc::new(VerificationServiceImpl::new(
            config_repository,
            llm_repository,
            jira_repository,
            github_repository,
            cnb_repository,
        ))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
