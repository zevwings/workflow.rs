//! LLM 服务注册

use std::sync::Arc;

use crate::{
    client::LLMClientImpl, executor::LLMExecutorImpl, LLMClient, LLMConfigContext, LLMExecutor,
};
use registry::{bind, Container, Scope};

/// 注册 LLM 相关服务
pub fn register_llm() -> registry::Result<()> {
    bind!(dyn LLMClient, |c: &Container| {
        let context = c
            .get::<dyn LLMConfigContext>()
            .expect("LLMConfigContext must be registered before LLMClient");
        Arc::new(LLMClientImpl::new(context))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn LLMExecutor, |c: &Container| {
        let client = c
            .get::<dyn LLMClient>()
            .expect("LLMClient must be registered before LLMExecutor");
        Arc::new(LLMExecutorImpl::new(client))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
