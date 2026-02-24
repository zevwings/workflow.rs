//! Codeup 服务注册

use std::sync::Arc;

use client::CodeupClient;
use di::{bind, Container, InjectionError, Scope};
use domain::{CodeupRepository, GlobalConfigRepository};

use crate::codeup::{
    CodeupRepositoryImpl, PullRequestMutationService, PullRequestMutationServiceImpl,
    PullRequestQueryService, PullRequestQueryServiceImpl, ServiceContext, ServiceContextImpl,
};

/// 注册 Codeup 相关服务
///
/// # 注册顺序和依赖关系
///
/// 服务注册顺序：
/// 1. **CodeupConfigContext** (外部注册) - 必须在调用此函数前注册
/// 2. **CodeupClient** (依赖 CodeupConfigContext)
/// 3. **ServiceContext** (依赖 CodeupSettings)
/// 4. **PullRequestQueryService** (依赖 CodeupClient, ServiceContext)
/// 5. **PullRequestMutationService** (依赖 CodeupClient, ServiceContext)
/// 6. **CodeupRepository** (依赖上述服务)
pub(super) fn register_codeup() -> Result<(), InjectionError> {
    // Service Context
    bind!(dyn ServiceContext, |c: &Container| {
        let global_config = c.get::<dyn GlobalConfigRepository>().map_err(|e| {
            InjectionError::ValidationError(format!("Failed to get config repository: {}", e))
        })?;
        let config = global_config.load().map_err(|e| {
            InjectionError::ValidationError(format!("Failed to load config: {}", e))
        })?;
        Ok(Arc::new(ServiceContextImpl::new(Arc::new(config.codeup))))
    })
    .in_scope(Scope::Singleton)?;

    // Pull Request Query Service
    bind!(dyn PullRequestQueryService, |c: &Container| {
        let client = c.get::<dyn CodeupClient>()?;
        let context = c.get::<dyn ServiceContext>()?;
        Ok(Arc::new(PullRequestQueryServiceImpl::new(client, context)))
    })
    .in_scope(Scope::Singleton)?;

    // Pull Request Mutation Service
    bind!(dyn PullRequestMutationService, |c: &Container| {
        let client = c.get::<dyn CodeupClient>()?;
        let context = c.get::<dyn ServiceContext>()?;
        Ok(Arc::new(PullRequestMutationServiceImpl::new(
            client, context,
        )))
    })
    .in_scope(Scope::Singleton)?;

    // Codeup Repository
    bind!(dyn CodeupRepository, |c: &Container| {
        let query_service = c.get::<dyn PullRequestQueryService>()?;
        let mutation_service = c.get::<dyn PullRequestMutationService>()?;
        let context = c.get::<dyn ServiceContext>()?;
        Ok(Arc::new(CodeupRepositoryImpl::new(
            mutation_service,
            query_service,
            context,
        )))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
