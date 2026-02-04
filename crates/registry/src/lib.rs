//! 依赖注入容器
//!
//! 提供轻量级的依赖注入功能，支持 Singleton 和 Transient 生命周期。
//!
//! # 特性
//!
//! - **全局容器**：使用 `DashMap` 实现，支持多线程高性能并发访问
//! - **Singleton 缓存**：使用 `OnceLock` 实现线程安全的单例缓存
//! - **循环依赖检测**：使用线程局部变量跟踪解析栈
//! - **灵活绑定**：支持闭包、Arc 实例、具体类型三种绑定方式
//! - **性能优化**：使用 `DashMap` 的分片锁实现细粒度并发控制
//!
//! # 绑定方式
//!
//! ## 方式 1：使用 `bind!` 宏（推荐 - 最简洁）
//!
//! ```rust,ignore
//! use registry::bind;
//!
//! // 使用宏：自动处理类型转换，自动使用 Container::global()
//! bind!(dyn ConfigService, |_| {
//!     Arc::new(ConfigServiceImpl::new())
//! })
//! .in_scope(Scope::Singleton)?;
//! ```
//!
//! ## 方式 2：使用 Container::global().bind()（显式类型转换）
//!
//! ```rust,ignore
//! Container::global().bind::<dyn ConfigService>(|c| {
//!     let dep = c.get::<dyn OtherService>()?;
//!     Arc::new(ConfigServiceImpl::new(dep)) as Arc<dyn ConfigService>
//! })
//! .in_scope(Scope::Singleton)?;
//! ```
//!
//! ## 方式 3：Arc 实例绑定 trait 对象
//!
//! ```rust,ignore
//! let instance: Arc<dyn ConfigService> = Arc::new(ConfigServiceImpl::new());
//! Container::global().bind::<dyn ConfigService>(instance)
//!     .in_scope(Scope::Singleton)?;
//! ```
//!
//! ## 方式 4：直接绑定具体类型（无需指定泛型）
//!
//! ```rust,ignore
//! // 绑定时类型自动推断
//! Container::global().bind_instance(Arc::new(ConfigServiceImpl::new()))
//!     .in_scope(Scope::Singleton)?;
//!
//! // 获取时使用具体类型
//! let service: Arc<ConfigServiceImpl> = Container::global().get::<ConfigServiceImpl>()?;
//! ```
//!
//! # 类型擦除与安全性
//!
//! 使用类型擦除技术实现泛型容器：
//! - 通过 `Box<dyn Any>` 实现类型擦除和存储
//! - 使用 `TypeId` 在运行时确保类型安全
//! - 使用 `downcast_ref` 安全地恢复原始类型
//! - 通过类型约束和测试确保正确性

use std::sync::Arc;

pub mod binding;
mod container;
mod error;
mod scope;

// ============================================================================
// 简化绑定的宏
// ============================================================================

/// 简化 trait object 绑定的宏
///
/// 自动处理从具体类型到 trait object 的类型转换，使用全局容器。
///
/// # 示例
///
/// ```rust,ignore
/// use registry::bind;
///
/// // 无依赖的服务
/// bind!(dyn Service, |_| {
///     Arc::new(ServiceImpl::new())
/// })
/// .in_scope(Scope::Singleton)?;
///
/// // 有依赖的服务
/// bind!(dyn OtherService, |c| {
///     let dep = c.get::<dyn Service>().expect("Service not found");
///     Arc::new(OtherServiceImpl::new(dep))
/// })
/// .in_scope(Scope::Singleton)?;
/// ```
#[macro_export]
macro_rules! bind {
    ($trait_type:ty, $factory:expr) => {{
        let factory_fn = $factory;
        let wrapped: ::std::boxed::Box<
            dyn for<'a> ::std::ops::Fn(&'a $crate::Container) -> ::std::sync::Arc<$trait_type>
                + ::std::marker::Send
                + ::std::marker::Sync,
        > = ::std::boxed::Box::new(
            move |c: &$crate::Container| -> ::std::sync::Arc<$trait_type> {
                factory_fn(c) as ::std::sync::Arc<$trait_type>
            },
        );
        $crate::Container::global().bind::<$trait_type>(wrapped)
    }};
}

/// 简化具体类型绑定的宏
///
/// 自动处理具体类型的绑定，使用全局容器，无需显式类型转换。
///
/// # 示例
///
/// ```rust,ignore
/// use registry::bind_instance;
///
/// // 无依赖的具体类型
/// bind_instance!(|_| {
///     Arc::new(ConfigServiceImpl::new())
/// })
/// .in_scope(Scope::Singleton)?;
///
/// // 有依赖的具体类型
/// bind_instance!(|c| {
///     let dep = c.get::<OtherService>().expect("OtherService not found");
///     Arc::new(MyServiceImpl::new(dep))
/// })
/// .in_scope(Scope::Singleton)?;
/// ```
#[macro_export]
macro_rules! bind_instance {
    ($factory:expr) => {{
        $crate::Container::global().bind_instance($factory)
    }};
}

/// 简化可失败 trait object 绑定的宏
///
/// 支持返回 `Result<Arc<T>>` 的工厂函数，允许工厂函数在创建服务时返回错误，
/// 从而消除 `expect()` 导致的 panic 风险。
///
/// # 示例
///
/// ```rust,ignore
/// use registry::try_bind;
///
/// // 有依赖的服务（可失败）
/// try_bind!(dyn OtherService, |c: &Container| {
///     let dep = c.get::<dyn Service>()?;
///     Ok(Arc::new(OtherServiceImpl::new(dep)))
/// })
/// .in_scope(Scope::Singleton)?;
/// ```
#[macro_export]
macro_rules! try_bind {
    ($trait_type:ty, $factory:expr) => {{
        let factory_fn = $factory;
        let wrapped: ::std::boxed::Box<
            dyn for<'a> ::std::ops::Fn(
                    &'a $crate::Container,
                )
                    -> $crate::Result<::std::sync::Arc<$trait_type>>
                + ::std::marker::Send
                + ::std::marker::Sync,
        > = ::std::boxed::Box::new(
            move |c: &$crate::Container| -> $crate::Result<::std::sync::Arc<$trait_type>> {
                factory_fn(c).map(|v| v as ::std::sync::Arc<$trait_type>)
            },
        );
        $crate::Container::global().try_bind::<$trait_type>(wrapped)
    }};
}

// 重新导出所有公共类型和函数
pub use binding::{
    Binding, BindingBuilder, FallibleBinding, FallibleBindingBuilder, IntoFactory,
    IntoFallibleFactory,
};
pub use container::Container;
pub use error::{RegistryError, Result};
pub use scope::Scope;

// ============================================================================
// 从全局容器获取服务
// ============================================================================

/// 从全局容器解析并获取服务实例
///
/// 使用 DashMap 的内部细粒度锁，允许多个线程高性能并发访问。
///
/// # 示例
///
/// ```rust,ignore
/// use registry::resolve;
/// use std::sync::Arc;
///
/// // 获取 trait 对象
/// let service: Arc<dyn ConfigService> = resolve::<dyn ConfigService>()?;
///
/// // 或获取具体类型
/// let service: Arc<ConfigServiceImpl> = resolve::<ConfigServiceImpl>()?;
/// ```
pub fn resolve<T: 'static + Send + Sync + ?Sized>() -> Result<Arc<T>> {
    let container = Container::global();
    container.get::<T>()
}

/// 从全局容器获取服务的简化宏
///
/// # 示例
///
/// ```rust,ignore
/// use registry::get_it;
/// use std::sync::Arc;
///
/// let service: Arc<dyn ConfigService> = get_it!(dyn ConfigService)?;
/// ```
#[macro_export]
macro_rules! get_it {
    ($ty:ty) => {
        $crate::resolve::<$ty>()
    };
}

/// 注册服务并初始化全局容器（宏）
///
/// # 示例
///
/// ```rust,ignore
/// use registry::{bind, Scope};
/// use std::sync::Arc;
///
/// // 方式 1：使用 bind! 宏 - 无依赖（推荐 - 最简洁）
/// bind!(dyn ConfigService, |_| {
///     Arc::new(ConfigServiceImpl::new())
/// })
/// .in_scope(Scope::Singleton)?;
///
/// // 方式 2：使用 bind! 宏 - 有依赖
/// bind!(dyn OtherService, |c| {
///     let dep = c.get::<dyn ConfigService>().expect("ConfigService not found");
///     Arc::new(OtherServiceImpl::new(dep))
/// })
/// .in_scope(Scope::Singleton)?;
///
/// // 方式 3：直接绑定具体类型（类型自动推断）
/// Container::global()
///     .bind_instance(Arc::new(LogServiceImpl::new()))
///     .in_scope(Scope::Singleton)?;
/// ```
#[macro_export]
macro_rules! registry {
    (|$container:ident| { $($body:tt)* }) => {
        $crate::Container::register(|$container| -> $crate::Result<()> {
            $($body)*
            Ok(())
        })
    };
}
