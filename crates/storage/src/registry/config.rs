//! Config 服务注册

use std::sync::Arc;

use registry::{bind, Container, Scope};

use crate::config::{GlobalConfigRepositoryImpl, RepoConfigRepositoryImpl};
use domain::{GlobalConfigRepository, PathService, RepoConfigRepository};

/// 注册 Config 相关服务
pub fn register_config() -> registry::Result<()> {
    bind!(dyn GlobalConfigRepository, |c: &Container| {
        let path_service = c
            .get::<dyn PathService>()
            .expect("PathService must be registered before GlobalConfigRepository");
        Arc::new(GlobalConfigRepositoryImpl::new(path_service))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn RepoConfigRepository, |_: &Container| {
        Arc::new(RepoConfigRepositoryImpl::new())
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
