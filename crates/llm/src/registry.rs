//! LLM 服务注册

use std::sync::Arc;

use crate::{
    client::LLMClientImpl, executor::LLMExecutorImpl, LLMClient, LLMConfigContext, LLMExecutor,
};
use registry::{try_bind, Container, Scope};

/// 注册 LLM 相关服务
///
/// # 注册顺序和依赖关系
///
/// Factory 闭包中的 `.expect()` 表示程序员错误（注册顺序错误），而非运行时错误。
pub fn register_llm() -> registry::Result<()> {
    try_bind!(dyn LLMClient, |c: &Container| {
        let context = c.get::<dyn LLMConfigContext>()?;
        Ok(Arc::new(LLMClientImpl::new(context)))
    })
    .in_scope(Scope::Singleton)?;

    try_bind!(dyn LLMExecutor, |c: &Container| {
        let client = c.get::<dyn LLMClient>()?;
        Ok(Arc::new(LLMExecutorImpl::new(client)))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
