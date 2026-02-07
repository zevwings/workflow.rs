//! Config 服务注册

use std::sync::Arc;

use registry::{bind, Container, Scope};

use crate::config::{GlobalConfigRepositoryImpl, RepoConfigRepositoryImpl};
use domain::{GlobalConfigRepository, PathService, RepoConfigRepository};

/// 注册 Config 相关服务
///
/// # 注册顺序和依赖关系
///
/// Factory 闭包中的 `.expect()` 表示程序员错误（注册顺序错误），而非运行时错误。
pub fn register_config() -> registry::Result<()> {
    bind!(dyn GlobalConfigRepository, |c: &Container| {
        let path_service = c.get::<dyn PathService>().expect(
            "PROGRAMMER ERROR:PathService must be registered before GlobalConfigRepository",
        );
        Arc::new(GlobalConfigRepositoryImpl::new(path_service))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn RepoConfigRepository, |c: &Container| {
        let path_service = c
            .get::<dyn PathService>()
            .expect("PROGRAMMER ERROR:PathService must be registered before RepoConfigRepository");
        Arc::new(RepoConfigRepositoryImpl::new(path_service))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
