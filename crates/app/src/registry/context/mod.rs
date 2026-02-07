mod llm_context;
mod jira_context;
mod github_context;
use llm::LLMConfigContext;

pub use llm_context::LLMConfigContextImpl;
pub use jira_context::JiraConfigContextImpl;
pub use github_context::GitHubContextImpl;

use std::sync::Arc;

use registry::{bind, Container, Scope};

use domain::{GitHubContext, GlobalConfigRepository, JiraConfigContext};

/// 注册 LLM 相关服务
pub fn register_context() -> registry::Result<()> {
    // LLM Config Context
    bind!(dyn LLMConfigContext, |c: &Container| {
        let global_config = c
            .get::<dyn GlobalConfigRepository>()
            .expect("GlobalConfigRepository must be registered before LLMConfigContext");
        Arc::new(LLMConfigContextImpl::new(global_config))
    })
    .in_scope(Scope::Singleton)?;

    // Jira Config Context
    bind!(dyn JiraConfigContext, |c: &Container| {
        let global_config = c
            .get::<dyn GlobalConfigRepository>()
            .expect("GlobalConfigRepository must be registered before JiraConfigContext");
        Arc::new(JiraConfigContextImpl::new(global_config))
    })
    .in_scope(Scope::Singleton)?;

    // GitHub Config Context
    bind!(dyn GitHubContext, |c: &Container| {
        let global_config = c
            .get::<dyn GlobalConfigRepository>()
            .expect("GlobalConfigRepository must be registered before GitHubContext");
        Arc::new(GitHubContextImpl::new(global_config))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}

