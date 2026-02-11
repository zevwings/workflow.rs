//! LLM 服务引导

use std::sync::Arc;

use di::{bind, Container, InjectionError, Scope};

use crate::{
    client::LLMClientImpl, executor::LLMExecutorImpl, LLMClient, LLMConfigContext, LLMExecutor,
};

/// 注册 LLM 相关服务
///
/// # 注册顺序和依赖关系
///
/// Factory 闭包中的 `.expect()` 表示程序员错误（注册顺序错误），而非运行时错误。
pub fn register_llm() -> Result<(), InjectionError> {
    bind!(dyn LLMClient, |c: &Container| {
        let context = c.get::<dyn LLMConfigContext>()?;
        Ok(Arc::new(LLMClientImpl::new(context)))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn LLMExecutor, |c: &Container| {
        let client = c.get::<dyn LLMClient>()?;
        Ok(Arc::new(LLMExecutorImpl::new(client)))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
