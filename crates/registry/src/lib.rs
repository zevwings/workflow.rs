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
use std::result::Result;

pub mod binding;
mod container;
mod error;
mod scope;

// 重新导出所有公共类型和函数
pub use binding::{
    Binding, BindingBuilder, FallibleBinding, FallibleBindingBuilder, IntoFactory,
    IntoFallibleFactory,
};
pub use container::Container;
pub use error::RegistryError;
pub use scope::Scope;

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
                    -> Result<::std::sync::Arc<$trait_type>, $crate::RegistryError>
                + ::std::marker::Send
                + ::std::marker::Sync,
        > = ::std::boxed::Box::new(
            move |c: &$crate::Container| -> Result<::std::sync::Arc<$trait_type>, $crate::RegistryError> {
                factory_fn(c).map(|v| v as ::std::sync::Arc<$trait_type>)
            },
        );
        $crate::Container::global().try_bind::<$trait_type>(wrapped)
    }};
}

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
pub fn resolve<T: 'static + Send + Sync + ?Sized>() -> Result<Arc<T>, RegistryError> {
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
        $crate::Container::register(|$container| -> Result<(), RegistryError> {
            $($body)*
            Ok(())
        })
    };
}

#[cfg(test)]
mod tests {
    // 标准库
    use std::sync::Arc;

    // 第三方库
    use pretty_assertions::assert_eq;
    use serial_test::serial;

    // 内部导入
    use super::*;

    // 测试工具
    trait TestService: Send + Sync {
        fn value(&self) -> i32;
    }

    struct TestServiceImpl {
        value: i32,
    }

    impl TestService for TestServiceImpl {
        fn value(&self) -> i32 {
            self.value
        }
    }

    // ============================================================================
    // resolve() 函数测试
    // ============================================================================

    #[test]
    #[serial]
    fn test_resolve_success() {
        // 清理全局容器
        Container::global().unbind_all();

        // 注册服务
        Container::global()
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 42 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)
            .unwrap();

        // 使用 resolve 获取服务
        let service: Arc<dyn TestService> = resolve::<dyn TestService>().unwrap();
        assert_eq!(service.value(), 42);
    }

    #[test]
    #[serial]
    fn test_resolve_not_bound() {
        // 清理全局容器
        Container::global().unbind_all();

        // 尝试获取未绑定的服务
        trait NonExistent: Send + Sync {}
        let result: Result<Arc<dyn NonExistent>, RegistryError> = resolve();
        assert!(result.is_err());
        assert!(matches!(result, Err(RegistryError::NotBound(_))));
    }

    // ============================================================================
    // bind! 宏测试
    // ============================================================================

    #[test]
    #[serial]
    fn test_bind_macro_basic() {
        // 清理全局容器
        Container::global().unbind_all();

        // 使用 bind! 宏绑定服务
        bind!(dyn TestService, |_: &Container| {
            Arc::new(TestServiceImpl { value: 100 })
        })
        .in_scope(Scope::Singleton)
        .unwrap();

        let service: Arc<dyn TestService> = Container::global().get().unwrap();
        assert_eq!(service.value(), 100);
    }

    #[test]
    #[serial]
    fn test_bind_macro_with_dependency() {
        // 清理全局容器
        Container::global().unbind_all();

        trait ConfigService: Send + Sync {
            fn get_multiplier(&self) -> i32;
        }

        struct ConfigServiceImpl {
            multiplier: i32,
        }

        impl ConfigService for ConfigServiceImpl {
            fn get_multiplier(&self) -> i32 {
                self.multiplier
            }
        }

        // 先绑定依赖
        Container::global()
            .bind::<dyn ConfigService>(|_: &Container| {
                Arc::new(ConfigServiceImpl { multiplier: 10 }) as Arc<dyn ConfigService>
            })
            .in_scope(Scope::Singleton)
            .unwrap();

        // 使用 bind! 宏绑定依赖于其他服务的服务
        bind!(dyn TestService, |c: &Container| {
            let config = c.get::<dyn ConfigService>().expect("ConfigService not found");
            Arc::new(TestServiceImpl {
                value: config.get_multiplier() * 5,
            })
        })
        .in_scope(Scope::Singleton)
        .unwrap();

        let service: Arc<dyn TestService> = Container::global().get().unwrap();
        assert_eq!(service.value(), 50);
    }

    // ============================================================================
    // bind_instance! 宏测试
    // ============================================================================

    #[test]
    #[serial]
    fn test_bind_instance_macro_basic() {
        // 清理全局容器
        Container::global().unbind_all();

        // 使用 bind_instance! 宏绑定具体类型
        bind_instance!(|_: &Container| { Arc::new(TestServiceImpl { value: 200 }) })
            .in_scope(Scope::Singleton)
            .unwrap();

        let service: Arc<TestServiceImpl> = Container::global().get().unwrap();
        assert_eq!(service.value, 200);
    }

    // ============================================================================
    // try_bind! 宏测试
    // ============================================================================

    #[test]
    #[serial]
    fn test_try_bind_macro_success() {
        // 清理全局容器
        Container::global().unbind_all();

        // 使用 try_bind! 宏绑定服务（成功场景）
        try_bind!(dyn TestService, |_: &Container| {
            Ok(Arc::new(TestServiceImpl { value: 300 }))
        })
        .in_scope(Scope::Singleton)
        .unwrap();

        let service: Arc<dyn TestService> = Container::global().get().unwrap();
        assert_eq!(service.value(), 300);
    }

    #[test]
    #[serial]
    fn test_try_bind_macro_with_dependency() {
        // 清理全局容器
        Container::global().unbind_all();

        trait ConfigService: Send + Sync {
            fn get_value(&self) -> i32;
        }

        struct ConfigServiceImpl {
            value: i32,
        }

        impl ConfigService for ConfigServiceImpl {
            fn get_value(&self) -> i32 {
                self.value
            }
        }

        // 先绑定依赖
        Container::global()
            .bind::<dyn ConfigService>(|_: &Container| {
                Arc::new(ConfigServiceImpl { value: 42 }) as Arc<dyn ConfigService>
            })
            .in_scope(Scope::Singleton)
            .unwrap();

        // 使用 try_bind! 宏，依赖获取可能失败
        try_bind!(dyn TestService, |c: &Container| {
            let config = c.get::<dyn ConfigService>()?;
            Ok(Arc::new(TestServiceImpl {
                value: config.get_value() * 2,
            }))
        })
        .in_scope(Scope::Singleton)
        .unwrap();

        let service: Arc<dyn TestService> = Container::global().get().unwrap();
        assert_eq!(service.value(), 84);
    }

    #[test]
    #[serial]
    fn test_try_bind_macro_error() {
        // 清理全局容器
        Container::global().unbind_all();

        // 使用 try_bind! 宏绑定服务（工厂返回错误）
        try_bind!(dyn TestService, |_: &Container| {
            Err::<Arc<TestServiceImpl>, _>(RegistryError::NotBound("Simulated error".to_string()))
        })
        .in_scope(Scope::Singleton)
        .unwrap();

        let result: Result<Arc<dyn TestService>, RegistryError> = Container::global().get();
        assert!(result.is_err());
    }

    // ============================================================================
    // get_it! 宏测试
    // ============================================================================

    #[test]
    #[serial]
    fn test_get_it_macro_success() {
        // 清理全局容器
        Container::global().unbind_all();

        // 注册服务
        Container::global()
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 400 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)
            .unwrap();

        // 使用 get_it! 宏获取服务
        let service: Arc<dyn TestService> = get_it!(dyn TestService).unwrap();
        assert_eq!(service.value(), 400);
    }

    #[test]
    #[serial]
    fn test_get_it_macro_not_bound() {
        // 清理全局容器
        Container::global().unbind_all();

        // 使用 get_it! 宏获取未绑定的服务
        trait NonExistent: Send + Sync {}
        let result = get_it!(dyn NonExistent);
        assert!(result.is_err());
    }

    // ============================================================================
    // registry! 宏测试
    // ============================================================================

    #[test]
    #[serial]
    fn test_registry_macro_basic() {
        // 清理全局容器
        Container::global().unbind_all();

        // 使用 registry! 宏注册服务
        let result = registry!(|_container| {
            Container::global()
                .bind::<dyn TestService>(|_: &Container| {
                    Arc::new(TestServiceImpl { value: 500 }) as Arc<dyn TestService>
                })
                .in_scope(Scope::Singleton)?;
        });

        assert!(result.is_ok());

        let service: Arc<dyn TestService> = Container::global().get().unwrap();
        assert_eq!(service.value(), 500);
    }

    #[test]
    #[serial]
    fn test_registry_macro_multiple_services() {
        // 清理全局容器
        Container::global().unbind_all();

        trait ServiceA: Send + Sync {
            fn value(&self) -> i32;
        }
        trait ServiceB: Send + Sync {
            fn value(&self) -> i32;
        }

        struct ServiceAImpl;
        struct ServiceBImpl;

        impl ServiceA for ServiceAImpl {
            fn value(&self) -> i32 {
                1
            }
        }
        impl ServiceB for ServiceBImpl {
            fn value(&self) -> i32 {
                2
            }
        }

        // 使用 registry! 宏注册多个服务
        let result = registry!(|_container| {
            Container::global()
                .bind::<dyn ServiceA>(|_: &Container| Arc::new(ServiceAImpl) as Arc<dyn ServiceA>)
                .in_scope(Scope::Singleton)?;

            Container::global()
                .bind::<dyn ServiceB>(|_: &Container| Arc::new(ServiceBImpl) as Arc<dyn ServiceB>)
                .in_scope(Scope::Singleton)?;
        });

        assert!(result.is_ok());

        let service_a: Arc<dyn ServiceA> = Container::global().get().unwrap();
        let service_b: Arc<dyn ServiceB> = Container::global().get().unwrap();

        assert_eq!(service_a.value(), 1);
        assert_eq!(service_b.value(), 2);
    }

    #[test]
    #[serial]
    fn test_registry_macro_error_propagation() {
        // 清理全局容器
        Container::global().unbind_all();

        // 先注册一个服务
        Container::global()
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 1 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)
            .unwrap();

        // 使用 registry! 宏尝试重复注册（应该失败）
        let result = registry!(|_container| {
            Container::global()
                .bind::<dyn TestService>(|_: &Container| {
                    Arc::new(TestServiceImpl { value: 2 }) as Arc<dyn TestService>
                })
                .in_scope(Scope::Singleton)?;
        });

        assert!(result.is_err());
        assert!(matches!(result, Err(RegistryError::AlreadyBound(_))));
    }
}
