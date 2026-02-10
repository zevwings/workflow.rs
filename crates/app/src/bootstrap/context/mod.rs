mod github_context;
mod jira_context;
mod llm_context;

pub use github_context::GitHubContextImpl;
pub use jira_context::JiraConfigContextImpl;
pub use llm_context::LLMConfigContextImpl;

use std::sync::Arc;

use di::{bind, Container, InjectionError, Scope};
use domain::{GitHubContext, GlobalConfigRepository, JiraConfigContext, PathService};
use llm::LLMConfigContext;

/// 注册配置上下文服务
///
/// # 注册顺序和依赖关系
///
/// 配置上下文依赖 `GlobalConfigRepository`，须在调用本函数前已注册。
/// Factory 闭包内通过 `?` 传播依赖解析错误，由本函数返回 `Err`，由调用方统一处理。
pub fn register_context() -> Result<(), InjectionError> {
    // LLM Config Context
    bind!(dyn LLMConfigContext, |c: &Container| {
        let global_config = c.get::<dyn GlobalConfigRepository>()?;
        Ok(Arc::new(LLMConfigContextImpl::new(global_config)))
    })
    .in_scope(Scope::Singleton)?;

    // Jira Config Context
    bind!(dyn JiraConfigContext, |c: &Container| {
        let global_config = c.get::<dyn GlobalConfigRepository>()?;
        let path_service = c.get::<dyn PathService>()?;
        Ok(Arc::new(JiraConfigContextImpl::new(
            global_config,
            path_service,
        )))
    })
    .in_scope(Scope::Singleton)?;

    // GitHub Config Context
    bind!(dyn GitHubContext, |c: &Container| {
        let global_config = c.get::<dyn GlobalConfigRepository>()?;
        Ok(Arc::new(GitHubContextImpl::new(global_config)))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
