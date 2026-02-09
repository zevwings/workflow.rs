//! 应用层服务注册
//!
//! 注册应用层特有的服务。应用层可以注册自己的服务，这些服务通常：
//! - 依赖于 domain、storage、services 层的服务
//! - 提供应用层特有的功能（如命令编排、工作流管理等）
//! - 封装应用层的业务逻辑

// 这些导入在注册服务时会用到，保留以便后续使用
#[allow(unused_imports)]
use std::sync::Arc;

#[allow(unused_imports)]
use registry::{bind, try_bind, Container, Scope};

/// 注册应用层服务
///
/// 注册所有应用层特有的服务。应用层服务通常依赖于：
/// - domain 层的 trait 定义
/// - storage 层的仓储实现
/// - services 层的应用服务
///
/// # 错误
///
/// 如果服务注册失败，返回 `registry::Error`。
///
/// # 使用方式
///
/// ## 方式 1：注册 trait 对象服务（推荐）
///
/// ```rust,ignore
/// // 定义 trait（通常在 domain 层）
/// pub trait AppService: Send + Sync {
///     fn do_something(&self) -> Result<()>;
/// }
///
/// // 实现服务（在 app 层）
/// pub struct AppServiceImpl {
///     repo: Arc<dyn domain::SomeRepository>,
/// }
///
/// impl AppService for AppServiceImpl {
///     fn do_something(&self) -> Result<()> {
///         // 实现逻辑
///         Ok(())
///     }
/// }
///
/// // 在 register_app() 中注册
/// try_bind!(dyn AppService, |c: &Container| {
///     let repo = c.get::<dyn domain::SomeRepository>()?;
///     Ok(Arc::new(AppServiceImpl::new(repo)))
/// })
/// .in_scope(Scope::Singleton)?;
/// ```
///
/// ## 方式 2：注册具体类型服务
///
/// ```rust,ignore
/// // 直接注册具体类型（无需 trait）
/// bind_instance!(|c: &Container| {
///     let dependency = c.get::<dyn SomeService>()?;
///     Arc::new(MyServiceImpl::new(dependency))
/// })
/// .in_scope(Scope::Singleton)?;
/// ```
///
/// ## 方式 3：无依赖的服务
///
/// ```rust,ignore
/// // 无依赖的服务可以使用 bind! 宏
/// bind!(dyn SimpleService, |_c: &Container| {
///     Arc::new(SimpleServiceImpl::new())
/// })
/// .in_scope(Scope::Singleton)?;
/// ```
pub fn register_app() -> registry::Result<()> {
    // TODO: 在这里注册应用层特有的服务
    //
    // 示例：注册一个应用层服务
    //
    // try_bind!(dyn SomeAppService, |c: &Container| {
    //     let repo = c.get::<dyn domain::SomeRepository>()?;
    //     let service = c.get::<dyn domain::SomeService>()?;
    //     Ok(Arc::new(SomeAppServiceImpl::new(repo, service)))
    // })
    // .in_scope(Scope::Singleton)?;

    Ok(())
}
