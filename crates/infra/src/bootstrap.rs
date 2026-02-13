use std::sync::Arc;

use di::{bind, Container, InjectionError, Scope};

use client::{HttpClient, LLMClient, LLMConfigContext, LanguageManager};

use crate::http::ReqwestHttpClient;
use crate::llm::{LLMClientImpl, LanguageManagerImpl};

pub fn register_client() -> Result<(), InjectionError> {
    bind!(dyn HttpClient, |_c: &Container| {
        let client = ReqwestHttpClient::new()
            .map_err(|e| InjectionError::CreateInstanceFailed(e.to_string()))?;
        Ok(Arc::new(client))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn LanguageManager, |_c: &Container| {
        Ok(Arc::new(LanguageManagerImpl::new()))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn LLMClient, |c: &Container| {
        let context = c.get::<dyn LLMConfigContext>()?;
        let client = c.get::<dyn HttpClient>()?;
        Ok(Arc::new(LLMClientImpl::new(client, context)))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
