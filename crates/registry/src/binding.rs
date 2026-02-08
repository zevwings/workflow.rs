//! 服务绑定

// 标准库
use std::any::{Any, TypeId};
use std::sync::{Arc, OnceLock};

// 内部导入
use crate::error::Result;
use crate::scope::Scope;

/// 类型化的工厂函数类型
type TypedFactory<T> = Box<dyn for<'a> Fn(&'a crate::container::Container) -> Arc<T> + Send + Sync>;

/// 类型擦除的工厂函数类型
/// 返回 Box 包装的 Arc，以支持 ?Sized 类型
type ErasedFactory = Box<
    dyn for<'a> Fn(&'a crate::container::Container) -> Box<dyn Any + Send + Sync> + Send + Sync,
>;

/// 可失败的类型化工厂函数类型
type FallibleTypedFactory<T> =
    Box<dyn for<'a> Fn(&'a crate::container::Container) -> Result<Arc<T>> + Send + Sync>;

/// 可失败的类型擦除工厂函数类型
type FallibleErasedFactory = Box<
    dyn for<'a> Fn(&'a crate::container::Container) -> Result<Box<dyn Any + Send + Sync>>
        + Send
        + Sync,
>;

/// 可转换为绑定工厂的类型
///
/// 此 trait 允许 `bind` 方法接受不同类型的参数：
/// - 闭包：`|c| Arc::new(ServiceImpl::new())`
/// - Arc 实例：`Arc::new(ServiceImpl::new())`
pub trait IntoFactory<T: ?Sized> {
    fn into_factory(self) -> TypedFactory<T>;
}

// 为闭包实现 IntoFactory
impl<T, F> IntoFactory<T> for F
where
    T: 'static + Send + Sync + ?Sized,
    F: for<'a> Fn(&'a crate::container::Container) -> Arc<T> + Send + Sync + 'static,
{
    fn into_factory(self) -> TypedFactory<T> {
        Box::new(self)
    }
}

// 为 Arc<T> 实现 IntoFactory（用于直接绑定实例）
impl<T> IntoFactory<T> for Arc<T>
where
    T: 'static + Send + Sync + ?Sized,
{
    fn into_factory(self) -> TypedFactory<T> {
        Box::new(move |_| self.clone())
    }
}

/// 可转换为可失败绑定工厂的类型
///
/// 此 trait 允许 `try_bind` 方法接受返回 `Result<Arc<T>>` 的闭包：
/// - 闭包：`|c| { let dep = c.get::<Dep>()?; Ok(Arc::new(ServiceImpl::new(dep))) }`
pub trait IntoFallibleFactory<T: ?Sized> {
    fn into_fallible_factory(self) -> FallibleTypedFactory<T>;
}

// 为返回 Result 的闭包实现 IntoFallibleFactory
impl<T, F> IntoFallibleFactory<T> for F
where
    T: 'static + Send + Sync + ?Sized,
    F: for<'a> Fn(&'a crate::container::Container) -> Result<Arc<T>> + Send + Sync + 'static,
{
    fn into_fallible_factory(self) -> FallibleTypedFactory<T> {
        Box::new(self)
    }
}

/// 服务绑定信息
pub struct Binding {
    pub(crate) identifier: TypeId,
    pub(crate) type_name: &'static str,
    pub(crate) factory: ErasedFactory,
    pub(crate) scope: Scope,
    pub(crate) instance: OnceLock<Box<dyn Any + Send + Sync>>,
}

impl Binding {
    /// 创建新的绑定
    pub fn new(
        identifier: TypeId,
        type_name: &'static str,
        factory: ErasedFactory,
        scope: Scope,
    ) -> Self {
        Self {
            identifier,
            type_name,
            factory,
            scope,
            instance: OnceLock::new(),
        }
    }

    /// 解析服务实例，返回 `Arc<T>`
    pub fn resolve<T: 'static + Send + Sync + ?Sized>(
        &self,
        container: &crate::container::Container,
    ) -> Result<Arc<T>> {
        match self.scope {
            Scope::Singleton => {
                // 使用 OnceLock 确保只初始化一次
                let cached = self.instance.get_or_init(|| (self.factory)(container));
                Self::try_unbox_arc::<T>(cached, self.identifier)
            }
            Scope::Transient => {
                let boxed = (self.factory)(container);
                Self::try_unbox_arc::<T>(&boxed, self.identifier)
            }
        }
    }

    /// 将 Arc<T> 包装到 Box<dyn Any> 进行类型擦除
    ///
    /// Arc<T> 本身是 Sized 的（即使 T 是 ?Sized），所以可以安全地放入 Box<dyn Any>
    pub(crate) fn box_arc<T: 'static + Send + Sync + ?Sized>(
        arc: Arc<T>,
    ) -> Box<dyn Any + Send + Sync> {
        // Arc<T> 本身是 Sized 的，可以直接放入 Box
        Box::new(arc)
    }

    /// 尝试从 Box<dyn Any> 中取出 Arc<T>
    ///
    /// 这是 `box_arc()` 的逆操作，从类型擦除的 Box 中恢复原始 Arc<T>
    pub(crate) fn try_unbox_arc<T: 'static + Send + Sync + ?Sized>(
        boxed: &Box<dyn Any + Send + Sync>,
        expected_type_id: TypeId,
    ) -> Result<Arc<T>> {
        // 运行时类型验证：检查期望的 TypeId 是否匹配
        let actual_type_id = TypeId::of::<Arc<T>>();
        if actual_type_id != expected_type_id {
            return Err(crate::error::RegistryError::TypeCast(format!(
                "Type mismatch: expected {:?}, got {:?}. Expected type: {}",
                expected_type_id,
                actual_type_id,
                std::any::type_name::<Arc<T>>()
            )));
        }

        // 尝试从 Box<dyn Any> 中 downcast 到 Arc<T>
        // Arc<T> 本身是 Sized 的，所以 downcast_ref 可以工作
        boxed.downcast_ref::<Arc<T>>().map(Arc::clone).ok_or_else(|| {
            crate::error::RegistryError::TypeCast(format!(
                "Failed to downcast to Arc<{}>",
                std::any::type_name::<T>()
            ))
        })
    }
}

/// 辅助函数：将类型化的 factory 转换为类型擦除的 factory
///
/// 使用 HRTB (higher-ranked trait bound) 确保工厂函数可以接受任意生命周期的 Container 引用
fn erase_factory_type<T: 'static + Send + Sync + ?Sized>(
    factory: TypedFactory<T>,
) -> ErasedFactory {
    Box::new(move |c| {
        let arc = factory(c);
        Binding::box_arc(arc)
    })
}

/// 辅助函数：将可失败的类型化 factory 转换为类型擦除的 factory
fn erase_fallible_factory_type<T: 'static + Send + Sync + ?Sized>(
    factory: FallibleTypedFactory<T>,
) -> FallibleErasedFactory {
    Box::new(move |c| {
        let arc = factory(c)?;
        Ok(Binding::box_arc(arc))
    })
}

/// 绑定构建器
pub struct BindingBuilder<'a> {
    identifier: TypeId,
    type_name: &'static str,
    factory: ErasedFactory,
    container: &'a crate::container::Container,
}

impl<'a> BindingBuilder<'a> {
    /// 创建新的绑定构建器
    pub(crate) fn new<T: 'static + Send + Sync + ?Sized>(
        identifier: TypeId,
        factory: impl IntoFactory<T>,
        container: &'a crate::container::Container,
    ) -> Self {
        let type_name = std::any::type_name::<Arc<T>>();
        let factory = factory.into_factory();
        let factory = erase_factory_type(factory);
        Self {
            identifier,
            type_name,
            factory,
            container,
        }
    }

    /// 设置作用域并完成绑定，自动注册到容器
    pub fn in_scope(self, scope: Scope) -> Result<()> {
        let binding = Binding::new(self.identifier, self.type_name, self.factory, scope);
        self.container.add_binding(binding)
    }
}

/// 可失败的服务绑定信息
pub struct FallibleBinding {
    pub(crate) identifier: TypeId,
    pub(crate) type_name: &'static str,
    pub(crate) factory: FallibleErasedFactory,
    pub(crate) scope: Scope,
    pub(crate) instance: OnceLock<Box<dyn Any + Send + Sync>>,
}

impl FallibleBinding {
    /// 创建新的可失败绑定
    pub fn new(
        identifier: TypeId,
        type_name: &'static str,
        factory: FallibleErasedFactory,
        scope: Scope,
    ) -> Self {
        Self {
            identifier,
            type_name,
            factory,
            scope,
            instance: OnceLock::new(),
        }
    }

    /// 解析服务实例，返回 `Result<Arc<T>>`
    pub fn resolve<T: 'static + Send + Sync + ?Sized>(
        &self,
        container: &crate::container::Container,
    ) -> Result<Arc<T>> {
        match self.scope {
            Scope::Singleton => {
                // 对于 Singleton，需要特殊处理：如果还没有缓存，调用 factory 并缓存结果
                if let Some(cached) = self.instance.get() {
                    return Binding::try_unbox_arc::<T>(cached, self.identifier);
                }

                // 调用 factory（可能失败）
                let boxed = (self.factory)(container)?;

                // 尝试缓存（可能已被其他线程设置）
                let _ = self.instance.set(boxed);

                // 从缓存中获取（确保使用同一个实例）
                let cached = self.instance.get().expect("OnceLock should be set");
                Binding::try_unbox_arc::<T>(cached, self.identifier)
            }
            Scope::Transient => {
                let boxed = (self.factory)(container)?;
                Binding::try_unbox_arc::<T>(&boxed, self.identifier)
            }
        }
    }
}

/// 可失败的绑定构建器
pub struct FallibleBindingBuilder<'a> {
    identifier: TypeId,
    type_name: &'static str,
    factory: FallibleErasedFactory,
    container: &'a crate::container::Container,
}

impl<'a> FallibleBindingBuilder<'a> {
    /// 创建新的可失败绑定构建器
    pub(crate) fn new<T: 'static + Send + Sync + ?Sized>(
        identifier: TypeId,
        factory: impl IntoFallibleFactory<T>,
        container: &'a crate::container::Container,
    ) -> Self {
        let type_name = std::any::type_name::<Arc<T>>();
        let factory = factory.into_fallible_factory();
        let factory = erase_fallible_factory_type(factory);
        Self {
            identifier,
            type_name,
            factory,
            container,
        }
    }

    /// 设置作用域并完成绑定，自动注册到容器
    pub fn in_scope(self, scope: Scope) -> Result<()> {
        let binding = FallibleBinding::new(self.identifier, self.type_name, self.factory, scope);
        self.container.add_fallible_binding(binding)
    }
}

#[cfg(test)]
mod tests {
    // 标准库
    use std::sync::Arc;

    // 第三方库
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    // 内部导入
    use super::*;
    use crate::scope::Scope;

    // 测试工具
    pub(crate) trait TestService: Send + Sync {
        fn value(&self) -> i32;
        fn id(&self) -> usize;
    }

    pub(crate) struct TestServiceImpl {
        pub value: i32,
        pub id: usize,
    }

    impl TestService for TestServiceImpl {
        fn value(&self) -> i32 {
            self.value
        }
        fn id(&self) -> usize {
            self.id
        }
    }

    // 1. 核心功能：参数化测试 Singleton 和 Transient
    #[rstest]
    #[case(Scope::Singleton, true)]
    #[case(Scope::Transient, false)]
    fn test_binding_resolve_scope(#[case] scope: Scope, #[case] should_be_same: bool) -> Result<()> {
        let container = crate::container::Container::new();
        let identifier = TypeId::of::<Arc<dyn TestService>>();
        let type_name = std::any::type_name::<Arc<dyn TestService>>();
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(1));
        let counter_clone = counter.clone();
        let factory: ErasedFactory = Box::new(move |_| {
            let id = counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let arc: Arc<dyn TestService> = Arc::new(TestServiceImpl { value: 42, id });
            Binding::box_arc(arc)
        });
        let binding = Binding::new(identifier, type_name, factory, scope);

        let service1: Arc<dyn TestService> = binding.resolve(&container)?;
        let service2: Arc<dyn TestService> = binding.resolve(&container)?;

        assert_eq!(service1.value(), 42);
        assert_eq!(service2.value(), 42);

        if should_be_same {
            assert_eq!(service1.id(), service2.id());
            assert!(Arc::ptr_eq(&service1, &service2));
        } else {
            assert_ne!(service1.id(), service2.id());
            assert!(!Arc::ptr_eq(&service1, &service2));
        }

        Ok(())
    }

    // 2. BindingBuilder 链式调用和作用域
    #[test]
    fn test_binding_builder_in_scope() -> Result<()> {
        let container = crate::container::Container::new();
        let identifier = TypeId::of::<Arc<dyn TestService>>();

        let factory = |_c: &crate::container::Container| -> Arc<dyn TestService> {
            Arc::new(TestServiceImpl { value: 42, id: 1 })
        };

        let result =
            BindingBuilder::new(identifier, factory, &container).in_scope(Scope::Transient);

        assert!(result.is_ok());
        assert!(container.is_bound::<dyn TestService>());

        // 验证 Transient 作用域
        let service1: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        let service2: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        assert!(!Arc::ptr_eq(&service1, &service2));

        Ok(())
    }

    // 3. BindingBuilder in_scope 方法（Singleton 作用域）
    #[test]
    fn test_binding_builder_singleton() -> Result<()> {
        let container = crate::container::Container::new();
        let identifier = TypeId::of::<Arc<dyn TestService>>();

        let factory = |_c: &crate::container::Container| -> Arc<dyn TestService> {
            Arc::new(TestServiceImpl { value: 42, id: 1 })
        };

        let result =
            BindingBuilder::new(identifier, factory, &container).in_scope(Scope::Singleton);

        assert!(result.is_ok());
        assert!(container.is_bound::<dyn TestService>());

        // 验证 Singleton 作用域
        let service1: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        let service2: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        assert!(Arc::ptr_eq(&service1, &service2));
        Ok(())
    }

    // 4. 测试 IntoFactory trait - 使用 Arc<T> 直接绑定
    #[test]
    fn test_into_factory_with_arc_instance() -> Result<()> {
        let container = crate::container::Container::new();
        let identifier = TypeId::of::<Arc<dyn TestService>>();

        // 直接传递 Arc 实例
        let instance: Arc<dyn TestService> = Arc::new(TestServiceImpl { value: 99, id: 42 });
        let result =
            BindingBuilder::new(identifier, instance, &container).in_scope(Scope::Singleton);

        assert!(result.is_ok());
        assert!(container.is_bound::<dyn TestService>());

        // 验证服务值
        let service: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        assert_eq!(service.value(), 99);
        assert_eq!(service.id(), 42);
        Ok(())
    }

    // 5. 测试 IntoFactory trait - Arc 实例的 Singleton 行为
    #[test]
    fn test_into_factory_arc_singleton_behavior() -> Result<()> {
        let container = crate::container::Container::new();
        let identifier = TypeId::of::<Arc<dyn TestService>>();

        // 使用 Arc 实例绑定
        let instance: Arc<dyn TestService> = Arc::new(TestServiceImpl { value: 100, id: 1 });
        BindingBuilder::new(identifier, instance, &container)
            .in_scope(Scope::Singleton)?;

        // 多次获取应该返回同一个实例（Singleton）
        let service1: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        let service2: Arc<dyn TestService> = container.get::<dyn TestService>()?;

        assert_eq!(service1.value(), 100);
        assert_eq!(service2.value(), 100);
        assert!(Arc::ptr_eq(&service1, &service2));

        Ok(())
    }

    // 6. 测试 IntoFactory trait - 闭包方式仍然有效
    #[test]
    fn test_into_factory_with_closure_still_works() -> Result<()> {
        let container = crate::container::Container::new();
        let identifier = TypeId::of::<Arc<dyn TestService>>();

        // 使用闭包绑定
        let factory = |_c: &crate::container::Container| -> Arc<dyn TestService> {
            Arc::new(TestServiceImpl { value: 200, id: 2 })
        };

        let result =
            BindingBuilder::new(identifier, factory, &container).in_scope(Scope::Transient);

        assert!(result.is_ok());

        let service1: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        let service2: Arc<dyn TestService> = container.get::<dyn TestService>()?;

        // Transient 应该创建不同实例
        assert_eq!(service1.value(), 200);
        assert_eq!(service2.value(), 200);
        assert!(!Arc::ptr_eq(&service1, &service2));

        Ok(())
    }

    // ============================================================================
    // FallibleBinding 测试
    // ============================================================================

    #[rstest]
    #[case(Scope::Singleton, true)]
    #[case(Scope::Transient, false)]
    fn test_fallible_binding_resolve_scope(#[case] scope: Scope, #[case] should_be_same: bool) -> Result<()> {
        let container = crate::container::Container::new();
        let identifier = TypeId::of::<Arc<dyn TestService>>();
        let type_name = std::any::type_name::<Arc<dyn TestService>>();
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(1));
        let counter_clone = counter.clone();

        let factory: FallibleErasedFactory = Box::new(move |_| {
            let id = counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let arc: Arc<dyn TestService> = Arc::new(TestServiceImpl { value: 42, id });
            Ok(Binding::box_arc(arc))
        });

        let binding = FallibleBinding::new(identifier, type_name, factory, scope);

        let service1: Arc<dyn TestService> = binding.resolve(&container)?;
        let service2: Arc<dyn TestService> = binding.resolve(&container)?;

        assert_eq!(service1.value(), 42);
        assert_eq!(service2.value(), 42);

        if should_be_same {
            assert_eq!(service1.id(), service2.id());
            assert!(Arc::ptr_eq(&service1, &service2));
        } else {
            assert_ne!(service1.id(), service2.id());
            assert!(!Arc::ptr_eq(&service1, &service2));
        }

        Ok(())
    }

    #[test]
    fn test_fallible_binding_resolve_error() -> Result<()> {
        let container = crate::container::Container::new();
        let identifier = TypeId::of::<Arc<dyn TestService>>();
        let type_name = std::any::type_name::<Arc<dyn TestService>>();

        let factory: FallibleErasedFactory = Box::new(|_| {
            Err(crate::error::RegistryError::NotBound(
                "Simulated error".to_string(),
            ))
        });

        let binding = FallibleBinding::new(identifier, type_name, factory, Scope::Singleton);

        let result: Result<Arc<dyn TestService>> = binding.resolve(&container);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(crate::error::RegistryError::NotBound(_))
        ));

        Ok(())
    }

    #[test]
    fn test_fallible_binding_singleton_caches_on_success() -> Result<()> {
        let container = crate::container::Container::new();
        let identifier = TypeId::of::<Arc<dyn TestService>>();
        let type_name = std::any::type_name::<Arc<dyn TestService>>();
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let factory: FallibleErasedFactory = Box::new(move |_| {
            call_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let arc: Arc<dyn TestService> = Arc::new(TestServiceImpl { value: 42, id: 1 });
            Ok(Binding::box_arc(arc))
        });

        let binding = FallibleBinding::new(identifier, type_name, factory, Scope::Singleton);

        // 多次调用 resolve
        let _ = binding.resolve::<dyn TestService>(&container)?;
        let _ = binding.resolve::<dyn TestService>(&container)?;
        let _ = binding.resolve::<dyn TestService>(&container)?;

        // Singleton 应该只调用一次 factory
        assert_eq!(
            call_count.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "Singleton fallible factory should be called only once"
        );

        Ok(())
    }

    // ============================================================================
    // FallibleBindingBuilder 测试
    // ============================================================================

    #[test]
    fn test_fallible_binding_builder_in_scope_singleton() -> Result<()> {
        let container = crate::container::Container::new();
        let identifier = TypeId::of::<Arc<dyn TestService>>();

        let factory =
            |_c: &crate::container::Container| -> crate::error::Result<Arc<dyn TestService>> {
                Ok(Arc::new(TestServiceImpl { value: 42, id: 1 }))
            };

        let result =
            FallibleBindingBuilder::new(identifier, factory, &container).in_scope(Scope::Singleton);

        assert!(result.is_ok());
        assert!(container.is_bound::<dyn TestService>());

        // 验证 Singleton 作用域
        let service1: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        let service2: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        assert!(Arc::ptr_eq(&service1, &service2));

        Ok(())
    }

    #[test]
    fn test_fallible_binding_builder_in_scope_transient() -> Result<()> {
        let container = crate::container::Container::new();
        let identifier = TypeId::of::<Arc<dyn TestService>>();

        let factory =
            |_c: &crate::container::Container| -> crate::error::Result<Arc<dyn TestService>> {
                Ok(Arc::new(TestServiceImpl { value: 42, id: 1 }))
            };

        let result =
            FallibleBindingBuilder::new(identifier, factory, &container).in_scope(Scope::Transient);

        assert!(result.is_ok());
        assert!(container.is_bound::<dyn TestService>());

        // 验证 Transient 作用域
        let service1: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        let service2: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        assert!(!Arc::ptr_eq(&service1, &service2));

        Ok(())
    }

    // ============================================================================
    // IntoFallibleFactory trait 测试
    // ============================================================================

    #[test]
    fn test_into_fallible_factory_with_closure() -> Result<()> {
        let container = crate::container::Container::new();
        let identifier = TypeId::of::<Arc<dyn TestService>>();

        // 使用返回 Result 的闭包
        let factory =
            |_c: &crate::container::Container| -> crate::error::Result<Arc<dyn TestService>> {
                Ok(Arc::new(TestServiceImpl { value: 99, id: 42 }))
            };

        let result =
            FallibleBindingBuilder::new(identifier, factory, &container).in_scope(Scope::Singleton);

        assert!(result.is_ok());

        let service: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        assert_eq!(service.value(), 99);
        assert_eq!(service.id(), 42);

        Ok(())
    }

    #[test]
    fn test_into_fallible_factory_error_propagation() -> Result<()> {
        let container = crate::container::Container::new();
        let identifier = TypeId::of::<Arc<dyn TestService>>();

        // 使用返回错误的闭包
        let factory =
            |_c: &crate::container::Container| -> crate::error::Result<Arc<dyn TestService>> {
                Err(crate::error::RegistryError::NotBound(
                    "Dependency missing".to_string(),
                ))
            };

        let result =
            FallibleBindingBuilder::new(identifier, factory, &container).in_scope(Scope::Singleton);

        assert!(result.is_ok()); // 绑定本身成功

        // 获取时返回错误
        let get_result: Result<Arc<dyn TestService>> = container.get();
        assert!(get_result.is_err());

        Ok(())
    }

    // ============================================================================
    // try_unbox_arc 类型不匹配测试
    // ============================================================================

    #[test]
    fn test_try_unbox_arc_type_mismatch() -> Result<()> {
        // 创建一个 Arc<TestServiceImpl>
        let arc: Arc<TestServiceImpl> = Arc::new(TestServiceImpl { value: 42, id: 1 });
        let boxed = Binding::box_arc(arc);

        // 使用错误的 TypeId 进行 unbox
        trait OtherService: Send + Sync {}
        let wrong_identifier = TypeId::of::<Arc<dyn OtherService>>();

        let result = Binding::try_unbox_arc::<TestServiceImpl>(&boxed, wrong_identifier);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(crate::error::RegistryError::TypeCast(_))
        ));

        Ok(())
    }

    #[test]
    fn test_try_unbox_arc_correct_type() -> Result<()> {
        // 创建一个 Arc<dyn TestService>
        let arc: Arc<dyn TestService> = Arc::new(TestServiceImpl { value: 42, id: 1 });
        let identifier = TypeId::of::<Arc<dyn TestService>>();
        let boxed = Binding::box_arc(arc);

        let result = Binding::try_unbox_arc::<dyn TestService>(&boxed, identifier);

        assert!(result.is_ok());
        let service = result?;
        assert_eq!(service.value(), 42);

        Ok(())
    }
}
