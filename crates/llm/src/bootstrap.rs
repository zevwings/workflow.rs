use std::sync::Arc;

use di::{bind, Container, InjectionError, Scope};

use crate::{
    client::{LLMClientImpl, DEFAULT_TIMEOUT},
    LLMClient, LLMConfigContext,
};

pub fn register_llm() -> Result<(), InjectionError> {
    bind!(dyn LLMClient, |c: &Container| {
        let context = c.get::<dyn LLMConfigContext>()?;
        let timeout = DEFAULT_TIMEOUT;
        Ok(Arc::new(LLMClientImpl::new(context, timeout)))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
