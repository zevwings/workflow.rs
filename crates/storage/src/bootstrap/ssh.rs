//! SSH 服务注册

use std::sync::Arc;

use di::{bind, Container, InjectionError, Scope};
use domain::SshService;

use crate::ssh::SshServiceImpl;

/// 注册 SSH 相关服务
pub(super) fn register_ssh() -> Result<(), InjectionError> {
    bind!(dyn SshService, |_: &Container| {
        Ok(Arc::new(SshServiceImpl::new()))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
