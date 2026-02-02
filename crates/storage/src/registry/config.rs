//! Config 服务注册

use std::sync::Arc;

use registry::{bind, Container, Scope};

use crate::config::{GlobalConfigRepositoryImpl, RepoConfigRepositoryImpl};
use domain::{GlobalConfigRepository, RepoConfigRepository};

/// 注册 Config 相关服务
pub fn register_config() -> registry::Result<()> {
    bind!(dyn GlobalConfigRepository, |_: &Container| {
        Arc::new(GlobalConfigRepositoryImpl::new())
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn RepoConfigRepository, |_: &Container| {
        Arc::new(RepoConfigRepositoryImpl::new())
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
