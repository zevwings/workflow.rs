mod github_context;
mod jira_context;
mod llm_context;

pub use github_context::GitHubContextImpl;
pub use jira_context::JiraConfigContextImpl;
pub use llm_context::LLMConfigContextImpl;

use std::sync::Arc;

use domain::{GitHubContext, GlobalConfigRepository, JiraConfigContext};
use llm::LLMConfigContext;
use registry::{Container, RegistryError, Scope, try_bind};

/// 注册配置上下文服务
///
/// # 注册顺序和依赖关系
///
/// Factory 闭包中的 `.expect()` 表示程序员错误（注册顺序错误），而非运行时错误。
pub fn register_context() -> Result<(), RegistryError> {
    // LLM Config Context
    try_bind!(dyn LLMConfigContext, |c: &Container| {
        let global_config = c.get::<dyn GlobalConfigRepository>().expect(
            "PROGRAMMER ERROR:GlobalConfigRepository must be registered before LLMConfigContext",
        );
        Ok(Arc::new(LLMConfigContextImpl::new(global_config)))
    })
    .in_scope(Scope::Singleton)?;

    // Jira Config Context
    try_bind!(dyn JiraConfigContext, |c: &Container| {
        let global_config = c.get::<dyn GlobalConfigRepository>().expect(
            "PROGRAMMER ERROR:GlobalConfigRepository must be registered before JiraConfigContext",
        );
        Ok(Arc::new(JiraConfigContextImpl::new(global_config)))
    })
    .in_scope(Scope::Singleton)?;

    // GitHub Config Context
    try_bind!(dyn GitHubContext, |c: &Container| {
        let global_config = c.get::<dyn GlobalConfigRepository>().expect(
            "PROGRAMMER ERROR:GlobalConfigRepository must be registered before GitHubContext",
        );
        Ok(Arc::new(GitHubContextImpl::new(global_config)))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
