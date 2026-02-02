//! LLM 服务注册

use std::sync::Arc;

use registry::{bind, Container, Scope};

use crate::llm::{
    LLMClient, LLMClientImpl, LLMConfigContextImpl, LLMRepositoryImpl, LLMService, LLMServiceImpl,
};
use domain::{GlobalConfigRepository, LLMConfigContext, LLMRepository};

/// 注册 LLM 相关服务
pub fn register_llm() -> registry::Result<()> {
    // LLM Config Context
    bind!(dyn LLMConfigContext, |c: &Container| {
        let global_config = c
            .get::<dyn GlobalConfigRepository>()
            .expect("GlobalConfigRepository must be registered before LLMConfigContext");
        Arc::new(LLMConfigContextImpl::new(global_config))
    })
    .in_scope(Scope::Singleton)?;

    // LLM Client
    bind!(dyn LLMClient, |c: &Container| {
        let context = c
            .get::<dyn LLMConfigContext>()
            .expect("LLMConfigContext must be registered before LLMClient");
        Arc::new(LLMClientImpl::new(context))
    })
    .in_scope(Scope::Singleton)?;

    // LLM Service
    bind!(dyn LLMService, |c: &Container| {
        let client = c
            .get::<dyn LLMClient>()
            .expect("LLMClient must be registered before LLMService");
        let context = c
            .get::<dyn LLMConfigContext>()
            .expect("LLMConfigContext must be registered before LLMService");
        Arc::new(LLMServiceImpl::new(client, context))
    })
    .in_scope(Scope::Singleton)?;

    // LLM Repository
    bind!(dyn LLMRepository, |c: &Container| {
        let service = c
            .get::<dyn LLMService>()
            .expect("LLMService must be registered before LLMRepository");
        Arc::new(LLMRepositoryImpl::new(service))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
