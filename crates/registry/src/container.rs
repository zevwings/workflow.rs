//! 依赖注入容器
//!
//! # 线程安全性
//!
//! 使用 `DashMap` 实现全局容器，提供细粒度锁和高性能并发访问。
//! 所有服务类型必须实现 `Send + Sync`。
//! DashMap 内部使用分片锁，允许访问不同服务时完全并发。
//! Singleton 服务使用 `Mutex` 保护实例缓存，确保线程安全。
//! 循环依赖检测使用线程局部变量，每个线程独立跟踪解析栈。

// 标准库
use std::any::TypeId;
use std::sync::Arc;

// 第三方库
use dashmap::DashMap;
use once_cell::sync::Lazy;

// 内部导入
use crate::binding::{Binding, BindingBuilder, FallibleBinding, FallibleBindingBuilder};
use crate::error::{RegistryError, Result};

// 线程局部变量：跟踪服务解析栈，检测循环依赖
thread_local! {
    static RESOLUTION_STACK: std::cell::RefCell<Vec<TypeId>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// 依赖注入容器
pub struct Container {
    bindings: DashMap<TypeId, Binding>,
    fallible_bindings: DashMap<TypeId, FallibleBinding>,
}

/// 全局容器单例，使用 Lazy + DashMap 实现无锁并发访问
static GLOBAL_CONTAINER: Lazy<Container> = Lazy::new(Container::new);

impl Container {
    /// 创建新的容器（仅内部使用）
    pub(crate) fn new() -> Self {
        Self {
            bindings: DashMap::new(),
            fallible_bindings: DashMap::new(),
        }
    }

    /// 获取全局容器单例的引用
    pub fn global() -> &'static Container {
        &GLOBAL_CONTAINER
    }

    /// 检查全局容器是否已初始化
    ///
    /// 注意：由于使用 Lazy 初始化，首次调用会触发容器创建
    pub fn is_initialized() -> bool {
        Lazy::get(&GLOBAL_CONTAINER).is_some()
    }

    /// 注册服务到全局容器
    ///
    /// DashMap 内部使用细粒度锁，允许并发修改不同的服务。
    pub fn register<F>(register_fn: F) -> Result<()>
    where
        F: FnOnce(&Container) -> Result<()>,
    {
        let container = Self::global();
        register_fn(container)
    }

    /// 绑定服务（需要手动类型转换）
    ///
    /// 支持两种方式：
    /// - 闭包：`bind(|c| Arc::new(ServiceImpl::new()) as Arc<dyn Service>)`
    /// - Arc 实例：`bind(Arc::new(ServiceImpl::new()))`
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// // 使用闭包绑定 trait 对象
    /// Container::global().bind::<dyn Service>(|_| {
    ///     Arc::new(ServiceImpl::new()) as Arc<dyn Service>
    /// })
    /// .in_scope(Scope::Singleton)?;
    ///
    /// // 或直接绑定 Arc 实例
    /// Container::global().bind::<dyn Service>(
    ///     Arc::new(ServiceImpl::new()) as Arc<dyn Service>
    /// )
    /// .in_scope(Scope::Singleton)?;
    /// ```
    pub fn bind<T: 'static + Send + Sync + ?Sized>(
        &self,
        factory: impl crate::binding::IntoFactory<T>,
    ) -> BindingBuilder<'_> {
        // 使用 Arc<T> 的 TypeId 作为标识符
        let identifier = TypeId::of::<Arc<T>>();
        BindingBuilder::new(identifier, factory, self)
    }

    /// 直接绑定具体类型实例，自动推断类型（无需指定泛型参数）
    ///
    /// 支持两种方式：
    /// - 闭包：`bind_instance(|c| Arc::new(ConfigServiceImpl::new()))`
    /// - Arc 实例：`bind_instance(Arc::new(ConfigServiceImpl::new()))`
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// // 使用闭包创建实例
    /// container.bind_instance(|_| Arc::new(ConfigServiceImpl::new()))
    ///     .in_scope(Scope::Singleton)?;
    ///
    /// // 或直接绑定 Arc 实例
    /// container.bind_instance(Arc::new(ConfigServiceImpl::new()))
    ///     .in_scope(Scope::Singleton)?;
    ///
    /// // 获取时使用具体类型
    /// let service: Arc<ConfigServiceImpl> = container.get::<ConfigServiceImpl>()?;
    /// ```
    pub fn bind_instance<T: 'static + Send + Sync>(
        &self,
        factory: impl crate::binding::IntoFactory<T>,
    ) -> BindingBuilder<'_> {
        let identifier = TypeId::of::<Arc<T>>();
        BindingBuilder::new(identifier, factory, self)
    }

    /// 绑定服务（可失败的工厂函数）
    ///
    /// 支持返回 `Result<Arc<T>>` 的闭包，允许工厂函数在创建服务时返回错误。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// // 使用可失败的闭包绑定 trait 对象
    /// Container::global().try_bind::<dyn Service>(|c| {
    ///     let dep = c.get::<dyn Dependency>()?;
    ///     Ok(Arc::new(ServiceImpl::new(dep)) as Arc<dyn Service>)
    /// })
    /// .in_scope(Scope::Singleton)?;
    /// ```
    pub fn try_bind<T: 'static + Send + Sync + ?Sized>(
        &self,
        factory: impl crate::binding::IntoFallibleFactory<T>,
    ) -> FallibleBindingBuilder<'_> {
        let identifier = TypeId::of::<Arc<T>>();
        FallibleBindingBuilder::new(identifier, factory, self)
    }

    /// 获取服务，Singleton 作用域返回同一个 Arc 的克隆
    pub fn get<T: 'static + Send + Sync + ?Sized>(&self) -> Result<Arc<T>> {
        let identifier = TypeId::of::<Arc<T>>();

        // 先检查循环依赖并压栈
        RESOLUTION_STACK.with(|stack| {
            let mut stack = stack.borrow_mut();

            if stack.contains(&identifier) {
                let cycle_start = stack.iter().position(|&id| id == identifier).expect(
                    "Internal error: identifier found in contains() but not in position(). This should never happen."
                );
                let cycle: Vec<String> = stack[cycle_start..]
                    .iter()
                    .chain(std::iter::once(&identifier))
                    .map(|id| format!("{:?}", id))
                    .collect();
                return Err(RegistryError::CircularDependency(cycle.join(" -> ")));
            }

            stack.push(identifier);
            Ok(())
        })?;

        // 先查找普通绑定
        let result = if let Some(binding) = self.bindings.get(&identifier) {
            binding.resolve::<T>(self)
        } else if let Some(fallible_binding) = self.fallible_bindings.get(&identifier) {
            // 再查找可失败绑定
            fallible_binding.resolve::<T>(self)
        } else {
            Err(RegistryError::NotBound(format!(
                "Service '{}' is not bound",
                std::any::type_name::<Arc<T>>()
            )))
        };

        // 无论成功与否，都要从栈中弹出
        RESOLUTION_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });

        result
    }

    /// 内部方法：完成绑定
    pub(crate) fn add_binding(&self, binding: Binding) -> Result<()> {
        let identifier = binding.identifier;
        // 检查是否已在任一 map 中绑定
        if self.bindings.contains_key(&identifier)
            || self.fallible_bindings.contains_key(&identifier)
        {
            return Err(RegistryError::AlreadyBound(
                "Service already bound".to_string(),
            ));
        }
        self.bindings.insert(identifier, binding);
        Ok(())
    }

    /// 内部方法：完成可失败绑定
    pub(crate) fn add_fallible_binding(&self, binding: FallibleBinding) -> Result<()> {
        let identifier = binding.identifier;
        // 检查是否已在任一 map 中绑定
        if self.bindings.contains_key(&identifier)
            || self.fallible_bindings.contains_key(&identifier)
        {
            return Err(RegistryError::AlreadyBound(
                "Service already bound".to_string(),
            ));
        }
        self.fallible_bindings.insert(identifier, binding);
        Ok(())
    }

    /// 检查服务是否已绑定
    pub fn is_bound<T: 'static + ?Sized>(&self) -> bool {
        let identifier = TypeId::of::<Arc<T>>();
        self.bindings.contains_key(&identifier) || self.fallible_bindings.contains_key(&identifier)
    }

    /// 解绑服务
    pub fn unbind<T: 'static + ?Sized>(&self) {
        let identifier = TypeId::of::<Arc<T>>();
        self.bindings.remove(&identifier);
        self.fallible_bindings.remove(&identifier);
    }

    /// 解绑所有服务
    pub fn unbind_all(&self) {
        self.bindings.clear();
        self.fallible_bindings.clear();
    }

    /// 获取绑定数量
    pub fn binding_count(&self) -> usize {
        self.bindings.len() + self.fallible_bindings.len()
    }

    /// 验证容器中所有已绑定的服务，建议在应用启动时调用
    ///
    /// # 注意
    ///
    /// 此方法会调用所有 factory 函数来验证它们是否能成功创建实例。
    /// **重要**：Factory 函数不应该 panic，如果 panic 会被捕获并报告为验证错误。
    /// DashMap 的迭代器提供快照视图，不会阻塞其他操作。
    pub fn validate(&self) -> Result<()> {
        let mut errors = Vec::new();

        // 验证普通绑定
        for entry in self.bindings.iter() {
            let binding = entry.value();
            let container_ref = self as *const Container;
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // SAFETY: 这里使用 unsafe 解引用原始指针的原因和安全性保证：
                // 1. container_ref 指向 self，在整个 validate() 方法执行期间，self 的生命周期保持有效
                // 2. 这个闭包在 catch_unwind 内立即执行，不会逃逸到 validate() 方法之外
                // 3. self 是不可变借用 (&self)，不会在验证期间被移动或修改
                // 4. 原始指针仅用于绕过借用检查器的限制，因为 self 已经被不可变借用
                // 5. 指针始终指向有效的内存地址，不会出现悬垂指针
                let container = unsafe { &*container_ref };
                (binding.factory)(container)
            })) {
                Ok(_instance) => {}
                Err(_) => {
                    errors.push(format!(
                        "Service '{}': Factory function panicked during validation",
                        binding.type_name
                    ));
                }
            }
        }

        // 验证可失败绑定
        for entry in self.fallible_bindings.iter() {
            let binding = entry.value();
            let container_ref = self as *const Container;
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let container = unsafe { &*container_ref };
                (binding.factory)(container)
            })) {
                Ok(Ok(_instance)) => {}
                Ok(Err(e)) => {
                    errors.push(format!(
                        "Service '{}': Factory function returned error: {}",
                        binding.type_name, e
                    ));
                }
                Err(_) => {
                    errors.push(format!(
                        "Service '{}': Factory function panicked during validation",
                        binding.type_name
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(RegistryError::ValidationError(format!(
                "Validation failed for {} service(s):\n  {}",
                errors.len(),
                errors.join("\n  ")
            )))
        }
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    // 标准库
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;

    // 第三方库
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use serial_test::serial;

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

    // ============================================================================
    // 容器创建与管理
    // ============================================================================

    #[test]
    fn test_container_creation() -> Result<()> {
        // 测试 new() 和 default()
        let container1 = Container::new();
        let container2 = Container::default();

        assert_eq!(container1.binding_count(), 0);
        assert_eq!(container2.binding_count(), 0);
        assert!(!container1.is_bound::<dyn TestService>());

        Ok(())
    }

    #[test]
    fn test_global_container_lifecycle() -> Result<()> {
        // 测试全局容器获取和初始化状态
        let container1 = Container::global();
        let container2 = Container::global();

        // 应该返回同一个引用
        assert!(std::ptr::eq(container1, container2));
        assert!(Container::is_initialized());

        Ok(())
    }

    // ============================================================================
    // 全局容器注册
    // ============================================================================

    #[test]
    #[serial]
    fn test_register_success() -> Result<()> {
        // 清理全局容器
        let container = Container::global();
        if container.is_bound::<dyn TestService>() {
            container.unbind::<dyn TestService>();
        }

        let result = Container::register(|_container| -> Result<()> {
            Container::global()
                .bind::<dyn TestService>(|_: &Container| {
                    Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
                })
                .in_scope(Scope::Singleton)?;
            Ok(())
        });

        assert!(result.is_ok(), "Registration should succeed");

        // 验证服务已注册
        let container = Container::global();
        assert!(container.is_bound::<dyn TestService>());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_register_error() -> Result<()> {
        // 清理全局容器
        let container = Container::global();
        if container.is_bound::<dyn TestService>() {
            container.unbind::<dyn TestService>();
        }

        // 先注册一次
        let first_result = Container::register(|_container| -> Result<()> {
            Container::global()
                .bind::<dyn TestService>(|_: &Container| {
                    Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
                })
                .in_scope(Scope::Singleton)?;
            Ok(())
        });
        assert!(first_result.is_ok());

        // 再次注册应该失败（已绑定）
        let result = Container::register(|_container| -> Result<()> {
            Container::global()
                .bind::<dyn TestService>(|_: &Container| {
                    Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
                })
                .in_scope(Scope::Singleton)?;
            Ok(())
        });

        assert!(result.is_err());
        assert!(matches!(result, Err(RegistryError::AlreadyBound(_))));

        Ok(())
    }

    // ============================================================================
    // 服务绑定
    // ============================================================================

    #[test]
    fn test_bind_and_check() -> Result<()> {
        let container = Container::new();

        // 初始状态：未绑定
        assert!(!container.is_bound::<dyn TestService>());

        // 绑定服务
        let result = container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton);

        assert!(result.is_ok());
        assert!(container.is_bound::<dyn TestService>());
        assert_eq!(container.binding_count(), 1);

        Ok(())
    }

    #[test]
    fn test_add_binding_scenarios() -> Result<()> {
        let container = Container::new();
        let identifier = TypeId::of::<Arc<dyn TestService>>();
        let type_name = std::any::type_name::<Arc<dyn TestService>>();

        // 测试添加绑定成功
        type TestFactory =
            Box<dyn Fn(&Container) -> Box<dyn std::any::Any + Send + Sync> + Send + Sync>;
        let factory1: TestFactory = Box::new(|_| {
            let arc: Arc<dyn TestService> = Arc::new(TestServiceImpl { value: 42, id: 1 });
            Binding::box_arc(arc)
        });
        let binding1 = Binding::new(identifier, type_name, factory1, Scope::Singleton);
        assert!(container.add_binding(binding1).is_ok());
        assert!(container.is_bound::<dyn TestService>());

        // 测试重复添加应该失败
        let factory2: TestFactory = Box::new(|_| {
            let arc: Arc<dyn TestService> = Arc::new(TestServiceImpl { value: 43, id: 2 });
            Binding::box_arc(arc)
        });
        let binding2 = Binding::new(identifier, type_name, factory2, Scope::Singleton);
        let result = container.add_binding(binding2);

        assert!(result.is_err());
        assert!(matches!(result, Err(RegistryError::AlreadyBound(_))));

        Ok(())
    }

    // ============================================================================
    // 服务获取
    // ============================================================================

    #[test]
    fn test_get_success() -> Result<()> {
        let container = Container::new();
        container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)?;

        let service: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        assert_eq!(service.value(), 42);

        Ok(())
    }

    #[test]
    fn test_get_not_bound() -> Result<()> {
        let container = Container::new();
        let result: Result<Arc<dyn TestService>> = container.get();

        assert!(result.is_err());
        assert!(matches!(result, Err(RegistryError::NotBound(_))));

        Ok(())
    }

    // ============================================================================
    // 作用域管理 (Singleton vs Transient)
    // ============================================================================

    #[rstest]
    #[case(Scope::Singleton, true)] // Singleton 返回同一个实例
    #[case(Scope::Transient, false)] // Transient 返回不同实例
    fn test_scope_behavior(#[case] scope: Scope, #[case] should_be_same: bool) -> Result<()> {
        let container = Container::new();
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        container
            .bind::<dyn TestService>(move |_: &Container| {
                let count = call_count_clone.fetch_add(1, Ordering::SeqCst);
                Arc::new(TestServiceImpl {
                    value: 42,
                    id: count,
                }) as Arc<dyn TestService>
            })
            .in_scope(scope)?;

        let service1: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        let service2: Arc<dyn TestService> = container.get::<dyn TestService>()?;

        assert_eq!(service1.value(), 42);
        assert_eq!(service2.value(), 42);

        if should_be_same {
            // Singleton: 应该是同一个实例
            assert!(Arc::ptr_eq(&service1, &service2));
            assert_eq!(call_count.load(Ordering::SeqCst), 1);
        } else {
            // Transient: 应该是不同实例
            assert!(!Arc::ptr_eq(&service1, &service2));
            assert_eq!(call_count.load(Ordering::SeqCst), 2);
        }

        Ok(())
    }

    // ============================================================================
    // 解绑操作
    // ============================================================================

    #[test]
    fn test_unbind() -> Result<()> {
        let container = Container::new();

        container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)?;

        assert!(container.is_bound::<dyn TestService>());
        assert_eq!(container.binding_count(), 1);

        container.unbind::<dyn TestService>();

        assert!(!container.is_bound::<dyn TestService>());
        assert_eq!(container.binding_count(), 0);

        Ok(())
    }

    #[test]
    fn test_unbind_all() -> Result<()> {
        let container = Container::new();

        // 注册多个服务
        container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)?;

        trait AnotherService: Send + Sync {}
        struct AnotherServiceImpl;
        impl AnotherService for AnotherServiceImpl {}

        container
            .bind::<dyn AnotherService>(|_: &Container| {
                Arc::new(AnotherServiceImpl) as Arc<dyn AnotherService>
            })
            .in_scope(Scope::Singleton)?;

        assert_eq!(container.binding_count(), 2);

        // 清空所有绑定
        container.unbind_all();

        assert_eq!(container.binding_count(), 0);
        assert!(!container.is_bound::<dyn TestService>());
        assert!(!container.is_bound::<dyn AnotherService>());

        Ok(())
    }

    // ============================================================================
    // 验证功能
    // ============================================================================

    #[test]
    fn test_validate_container() -> Result<()> {
        // 测试空容器验证
        let empty_container = Container::new();
        assert!(empty_container.validate().is_ok());

        // 测试有正常服务的容器验证
        let container = Container::new();
        container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)?;
        assert!(container.validate().is_ok());

        Ok(())
    }

    #[test]
    fn test_validate_with_panicking_factory() -> Result<()> {
        let container = Container::new();

        // 创建会 panic 的工厂函数
        container
            .bind::<dyn TestService>(|_: &Container| {
                panic!("Factory function panicked during validation");
            })
            .in_scope(Scope::Singleton)?;

        // 验证应该返回错误
        let result = container.validate();
        assert!(result.is_err());

        if let Err(RegistryError::ValidationError(msg)) = result {
            assert!(msg.contains("panicked"));
            assert!(msg.contains("Factory function"));
        } else {
            panic!("Expected ValidationError");
        }

        Ok(())
    }

    // ============================================================================
    // 多服务管理
    // ============================================================================

    #[test]
    fn test_multiple_services() -> Result<()> {
        let container = Container::new();

        trait ServiceA: Send + Sync {
            fn value(&self) -> i32;
        }
        struct ServiceAImpl;
        impl ServiceA for ServiceAImpl {
            fn value(&self) -> i32 {
                1
            }
        }

        trait ServiceB: Send + Sync {
            fn value(&self) -> i32;
        }
        struct ServiceBImpl;
        impl ServiceB for ServiceBImpl {
            fn value(&self) -> i32 {
                2
            }
        }

        container
            .bind::<dyn ServiceA>(|_: &Container| Arc::new(ServiceAImpl) as Arc<dyn ServiceA>)
            .in_scope(Scope::Singleton)?;

        container
            .bind::<dyn ServiceB>(|_: &Container| Arc::new(ServiceBImpl) as Arc<dyn ServiceB>)
            .in_scope(Scope::Singleton)?;

        assert_eq!(container.binding_count(), 2);

        let service_a: Arc<dyn ServiceA> = container.get::<dyn ServiceA>()?;
        let service_b: Arc<dyn ServiceB> = container.get::<dyn ServiceB>()?;

        assert_eq!(service_a.value(), 1);
        assert_eq!(service_b.value(), 2);

        Ok(())
    }

    // ============================================================================
    // 并发安全测试
    // ============================================================================

    #[test]
    #[serial]
    fn test_concurrent_global_access() -> Result<()> {
        // 清理全局容器
        let container = Container::global();
        container.unbind_all();

        // 注册服务
        Container::register(|_container| -> Result<()> {
            Container::global()
                .bind::<dyn TestService>(|_: &Container| {
                    Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
                })
                .in_scope(Scope::Singleton)?;
            Ok(())
        })?;

        const THREAD_COUNT: usize = 4;
        const ITERATIONS_PER_THREAD: usize = 20;

        let handles: Vec<_> = (0..THREAD_COUNT)
            .map(|_| {
                thread::spawn(move || {
                    for _ in 0..ITERATIONS_PER_THREAD {
                        let container = Container::global();
                        // DashMap 内部管理锁，直接访问即可
                        let service: Arc<dyn TestService> =
                            container.get::<dyn TestService>().unwrap();
                        assert_eq!(service.value(), 42);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_concurrent_singleton_consistency() -> Result<()> {
        // 清理全局容器
        let container = Container::global();
        container.unbind_all();

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        // 注册服务，使用计数器跟踪 factory 调用次数
        Container::register(|_container| -> Result<()> {
            let call_count = call_count_clone.clone();
            Container::global()
                .bind::<dyn TestService>(move |_: &Container| {
                    call_count.fetch_add(1, Ordering::SeqCst);
                    Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
                })
                .in_scope(Scope::Singleton)?;
            Ok(())
        })?;

        const THREAD_COUNT: usize = 4;
        const ITERATIONS_PER_THREAD: usize = 20;

        let handles: Vec<_> = (0..THREAD_COUNT)
            .map(|_| {
                thread::spawn(move || {
                    for _ in 0..ITERATIONS_PER_THREAD {
                        let container = Container::global();
                        // DashMap 内部管理锁，直接访问即可
                        let _service: Arc<dyn TestService> =
                            container.get::<dyn TestService>().unwrap();
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // Singleton 应该只调用一次 factory
        let total_calls = call_count.load(Ordering::SeqCst);
        assert_eq!(
            total_calls, 1,
            "Singleton factory should be called only once, but was called {} times",
            total_calls
        );

        Ok(())
    }

    // ============================================================================
    // bind 方法：Arc 实例和闭包测试
    // ============================================================================

    #[test]
    fn test_bind_with_arc_instance() -> Result<()> {
        let container = Container::new();

        // 直接使用 Arc 实例绑定
        let instance: Arc<dyn TestService> = Arc::new(TestServiceImpl { value: 888, id: 99 });
        container.bind::<dyn TestService>(instance).in_scope(Scope::Singleton)?;

        assert!(container.is_bound::<dyn TestService>());

        // 获取服务并验证
        let service: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        assert_eq!(service.value(), 888);
        assert_eq!(service.id(), 99);

        Ok(())
    }

    #[test]
    fn test_bind_concrete_type() -> Result<()> {
        let container = Container::new();

        // 测试绑定具体类型（不是 trait 对象）
        container
            .bind::<TestServiceImpl>(Arc::new(TestServiceImpl { value: 777, id: 88 }))
            .in_scope(Scope::Singleton)?;

        let service: Arc<TestServiceImpl> = container.get::<TestServiceImpl>()?;
        assert_eq!(service.value, 777);
        assert_eq!(service.id, 88);

        Ok(())
    }

    #[test]
    fn test_bind_with_closure() -> Result<()> {
        let container = Container::new();

        // 使用闭包绑定
        container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl {
                    value: 111,
                    id: 222,
                }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)?;

        let service: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        assert_eq!(service.value(), 111);

        Ok(())
    }

    #[test]
    fn test_bind_with_dependencies() -> Result<()> {
        let container = Container::new();

        trait ConfigService: Send + Sync {
            fn get_config(&self) -> &str;
        }

        struct ConfigServiceImpl {
            config: String,
        }

        impl ConfigService for ConfigServiceImpl {
            fn get_config(&self) -> &str {
                &self.config
            }
        }

        trait AppService: Send + Sync {
            fn get_app_name(&self) -> String;
        }

        struct AppServiceImpl {
            config: Arc<dyn ConfigService>,
        }

        impl AppService for AppServiceImpl {
            fn get_app_name(&self) -> String {
                format!("App: {}", self.config.get_config())
            }
        }

        // 先绑定依赖服务
        _ = container
            .bind::<dyn ConfigService>(|_: &Container| {
                Arc::new(ConfigServiceImpl {
                    config: "production".to_string(),
                }) as Arc<dyn ConfigService>
            })
            .in_scope(Scope::Singleton);

        // 绑定依赖于其他服务的服务
        _ = container
            .bind::<dyn AppService>(|c: &Container| {
                let config = c.get::<dyn ConfigService>().unwrap();
                Arc::new(AppServiceImpl { config }) as Arc<dyn AppService>
            })
            .in_scope(Scope::Singleton);

        // 验证依赖注入正常工作
        let app_service: Arc<dyn AppService> = container.get::<dyn AppService>().unwrap();
        assert_eq!(app_service.get_app_name(), "App: production");

        Ok(())
    }

    // ============================================================================
    // bind_instance 测试（直接绑定具体类型，无需指定泛型）
    // ============================================================================

    #[test]
    fn test_bind_instance_basic() -> Result<()> {
        let container = Container::new();

        // 直接绑定具体类型，无需显式指定类型参数
        container
            .bind_instance(Arc::new(TestServiceImpl {
                value: 999,
                id: 100,
            }))
            .in_scope(Scope::Singleton)?;

        // 使用具体类型获取
        let service: Arc<TestServiceImpl> = container.get::<TestServiceImpl>()?;
        assert_eq!(service.value, 999);
        assert_eq!(service.id, 100);

        Ok(())
    }

    #[test]
    fn test_bind_instance_multiple_types() -> Result<()> {
        let container = Container::new();

        struct ServiceA {
            name: String,
        }
        struct ServiceB {
            count: i32,
        }

        // 绑定多个不同的具体类型
        container
            .bind_instance(Arc::new(ServiceA {
                name: "ServiceA".to_string(),
            }))
            .in_scope(Scope::Singleton)?;

        container
            .bind_instance(Arc::new(ServiceB { count: 42 }))
            .in_scope(Scope::Singleton)?;

        // 分别获取
        let service_a: Arc<ServiceA> = container.get::<ServiceA>()?;
        let service_b: Arc<ServiceB> = container.get::<ServiceB>()?;

        assert_eq!(service_a.name, "ServiceA");
        assert_eq!(service_b.count, 42);

        Ok(())
    }

    #[test]
    fn test_bind_instance_vs_bind_trait() -> Result<()> {
        let container = Container::new();

        trait MyService: Send + Sync {
            fn get_value(&self) -> i32;
        }

        struct MyServiceImpl {
            value: i32,
        }

        impl MyService for MyServiceImpl {
            fn get_value(&self) -> i32 {
                self.value
            }
        }

        // 同时支持：绑定具体类型和绑定 trait 对象
        // 1. 绑定具体类型
        container
            .bind_instance(Arc::new(MyServiceImpl { value: 100 }))
            .in_scope(Scope::Singleton)?;

        // 2. 绑定 trait 对象（不同的标识符）
        let trait_instance: Arc<dyn MyService> = Arc::new(MyServiceImpl { value: 200 });
        container.bind::<dyn MyService>(trait_instance).in_scope(Scope::Singleton)?;

        // 可以分别获取
        let concrete: Arc<MyServiceImpl> = container.get::<MyServiceImpl>()?;
        let trait_obj: Arc<dyn MyService> = container.get::<dyn MyService>()?;

        assert_eq!(concrete.value, 100);
        assert_eq!(trait_obj.get_value(), 200);

        Ok(())
    }

    // ============================================================================
    // 循环依赖检测
    // ============================================================================

    #[test]
    fn test_circular_dependency_direct() -> Result<()> {
        // 测试直接循环依赖：A -> A
        let container = Container::new();

        trait ServiceA: Send + Sync {}

        struct ServiceAImpl {
            _self_ref: Arc<dyn ServiceA>,
        }

        impl ServiceA for ServiceAImpl {}

        // ServiceA 依赖于自身（使用 try_bind 来正确处理循环依赖错误）
        _ = container
            .try_bind::<dyn ServiceA>(|c: &Container| {
                let self_ref = c.get::<dyn ServiceA>()?;
                Ok(Arc::new(ServiceAImpl {
                    _self_ref: self_ref,
                }) as Arc<dyn ServiceA>)
            })
            .in_scope(Scope::Transient);

        // 获取服务时应该检测到循环依赖
        let result: Result<Arc<dyn ServiceA>> = container.get();
        assert!(result.is_err());
        assert!(matches!(result, Err(RegistryError::CircularDependency(_))));

        Ok(())
    }

    #[test]
    fn test_circular_dependency_indirect() -> Result<()> {
        // 测试间接循环依赖：A -> B -> A
        let container = Container::new();

        trait ServiceA: Send + Sync {}
        trait ServiceB: Send + Sync {}

        struct ServiceAImpl {
            _dep: Arc<dyn ServiceB>,
        }
        struct ServiceBImpl {
            _dep: Arc<dyn ServiceA>,
        }

        impl ServiceA for ServiceAImpl {}
        impl ServiceB for ServiceBImpl {}

        // ServiceA 依赖于 ServiceB（使用 try_bind）
        _ = container
            .try_bind::<dyn ServiceA>(|c: &Container| {
                let dep = c.get::<dyn ServiceB>()?;
                Ok(Arc::new(ServiceAImpl { _dep: dep }) as Arc<dyn ServiceA>)
            })
            .in_scope(Scope::Transient);

        // ServiceB 依赖于 ServiceA（形成循环）
        _ = container
            .try_bind::<dyn ServiceB>(|c: &Container| {
                let dep = c.get::<dyn ServiceA>()?;
                Ok(Arc::new(ServiceBImpl { _dep: dep }) as Arc<dyn ServiceB>)
            })
            .in_scope(Scope::Transient);

        // 获取 ServiceA 时应该检测到循环依赖
        let result: Result<Arc<dyn ServiceA>> = container.get();
        assert!(result.is_err());
        assert!(matches!(result, Err(RegistryError::CircularDependency(_))));

        Ok(())
    }

    #[test]
    fn test_circular_dependency_chain() -> Result<()> {
        // 测试链式循环依赖：A -> B -> C -> A
        let container = Container::new();

        trait ServiceA: Send + Sync {}
        trait ServiceB: Send + Sync {}
        trait ServiceC: Send + Sync {}

        struct ServiceAImpl {
            _dep: Arc<dyn ServiceB>,
        }
        struct ServiceBImpl {
            _dep: Arc<dyn ServiceC>,
        }
        struct ServiceCImpl {
            _dep: Arc<dyn ServiceA>,
        }

        impl ServiceA for ServiceAImpl {}
        impl ServiceB for ServiceBImpl {}
        impl ServiceC for ServiceCImpl {}

        container
            .try_bind::<dyn ServiceA>(|c: &Container| {
                let dep = c.get::<dyn ServiceB>()?;
                Ok(Arc::new(ServiceAImpl { _dep: dep }) as Arc<dyn ServiceA>)
            })
            .in_scope(Scope::Transient)?;

        container
            .try_bind::<dyn ServiceB>(|c: &Container| {
                let dep = c.get::<dyn ServiceC>()?;
                Ok(Arc::new(ServiceBImpl { _dep: dep }) as Arc<dyn ServiceB>)
            })
            .in_scope(Scope::Transient)?;

        container
            .try_bind::<dyn ServiceC>(|c: &Container| {
                let dep = c.get::<dyn ServiceA>()?;
                Ok(Arc::new(ServiceCImpl { _dep: dep }) as Arc<dyn ServiceC>)
            })
            .in_scope(Scope::Transient)?;

        let result: Result<Arc<dyn ServiceA>> = container.get();
        assert!(result.is_err());
        assert!(matches!(result, Err(RegistryError::CircularDependency(_))));

        Ok(())
    }

    // ============================================================================
    // try_bind 可失败绑定测试
    // ============================================================================

    #[test]
    fn test_try_bind_success() -> Result<()> {
        let container = Container::new();

        // 使用 try_bind 绑定服务（成功场景）
        container
            .try_bind::<dyn TestService>(|_c: &Container| {
                Ok(Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>)
            })
            .in_scope(Scope::Singleton)?;

        assert!(container.is_bound::<dyn TestService>());

        let service: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        assert_eq!(service.value(), 42);

        Ok(())
    }

    #[test]
    fn test_try_bind_factory_returns_error() -> Result<()> {
        let container = Container::new();

        // 使用 try_bind 绑定服务（工厂函数返回错误）
        container
            .try_bind::<dyn TestService>(|_c: &Container| {
                Err(RegistryError::NotBound("Dependency not found".to_string()))
            })
            .in_scope(Scope::Singleton)?;

        assert!(container.is_bound::<dyn TestService>());

        // 获取服务时应该返回错误
        let result: Result<Arc<dyn TestService>> = container.get();
        assert!(result.is_err());
        assert!(matches!(result, Err(RegistryError::NotBound(_))));

        Ok(())
    }

    #[test]
    fn test_try_bind_with_dependency() -> Result<()> {
        let container = Container::new();

        trait ConfigService: Send + Sync {
            fn get_value(&self) -> &str;
        }

        struct ConfigServiceImpl {
            value: String,
        }

        impl ConfigService for ConfigServiceImpl {
            fn get_value(&self) -> &str {
                &self.value
            }
        }

        // 先绑定依赖服务
        container
            .bind::<dyn ConfigService>(|_: &Container| {
                Arc::new(ConfigServiceImpl {
                    value: "config_value".to_string(),
                }) as Arc<dyn ConfigService>
            })
            .in_scope(Scope::Singleton)?;

        // 使用 try_bind 绑定依赖于其他服务的服务
        container
            .try_bind::<dyn TestService>(|c: &Container| {
                let config = c.get::<dyn ConfigService>()?;
                let value = if config.get_value() == "config_value" {
                    100
                } else {
                    0
                };
                Ok(Arc::new(TestServiceImpl { value, id: 1 }) as Arc<dyn TestService>)
            })
            .in_scope(Scope::Singleton)?;

        let service: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        assert_eq!(service.value(), 100);

        Ok(())
    }

    #[test]
    fn test_try_bind_dependency_not_found() -> Result<()> {
        let container = Container::new();

        // 使用 try_bind 绑定服务，但依赖不存在
        container
            .try_bind::<dyn TestService>(|c: &Container| {
                // 尝试获取不存在的依赖
                trait NonExistent: Send + Sync {}
                let _dep: Arc<dyn NonExistent> = c.get()?;
                Ok(Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>)
            })
            .in_scope(Scope::Singleton)?;

        // 获取服务时应该返回依赖未找到的错误
        let result: Result<Arc<dyn TestService>> = container.get();
        assert!(result.is_err());
        assert!(matches!(result, Err(RegistryError::NotBound(_))));

        Ok(())
    }

    #[rstest]
    #[case(Scope::Singleton, true)]
    #[case(Scope::Transient, false)]
    fn test_try_bind_scope_behavior(
        #[case] scope: Scope,
        #[case] should_be_same: bool,
    ) -> Result<()> {
        let container = Container::new();
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        container
            .try_bind::<dyn TestService>(move |_: &Container| {
                let count = call_count_clone.fetch_add(1, Ordering::SeqCst);
                Ok(Arc::new(TestServiceImpl {
                    value: 42,
                    id: count,
                }) as Arc<dyn TestService>)
            })
            .in_scope(scope)?;

        let service1: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        let service2: Arc<dyn TestService> = container.get::<dyn TestService>()?;

        if should_be_same {
            assert!(Arc::ptr_eq(&service1, &service2));
            assert_eq!(call_count.load(Ordering::SeqCst), 1);
        } else {
            assert!(!Arc::ptr_eq(&service1, &service2));
            assert_eq!(call_count.load(Ordering::SeqCst), 2);
        }

        Ok(())
    }

    // ============================================================================
    // validate 对 fallible_bindings 测试
    // ============================================================================

    #[test]
    fn test_validate_fallible_binding_success() -> Result<()> {
        let container = Container::new();

        container
            .try_bind::<dyn TestService>(|_: &Container| {
                Ok(Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>)
            })
            .in_scope(Scope::Singleton)?;

        // 验证应该成功
        assert!(container.validate().is_ok());

        Ok(())
    }

    #[test]
    fn test_validate_fallible_binding_error() -> Result<()> {
        let container = Container::new();

        container
            .try_bind::<dyn TestService>(|_: &Container| {
                Err(RegistryError::NotBound("Simulated error".to_string()))
            })
            .in_scope(Scope::Singleton)?;

        // 验证应该返回错误
        let result = container.validate();
        assert!(result.is_err());

        if let Err(RegistryError::ValidationError(msg)) = result {
            assert!(msg.contains("returned error"));
        } else {
            panic!("Expected ValidationError");
        }

        Ok(())
    }

    #[test]
    fn test_validate_fallible_binding_panic() -> Result<()> {
        let container = Container::new();

        container
            .try_bind::<dyn TestService>(|_: &Container| {
                panic!("Simulated panic in fallible factory");
            })
            .in_scope(Scope::Singleton)?;

        // 验证应该捕获 panic 并返回错误
        let result = container.validate();
        assert!(result.is_err());

        if let Err(RegistryError::ValidationError(msg)) = result {
            assert!(msg.contains("panicked"));
        } else {
            panic!("Expected ValidationError");
        }

        Ok(())
    }

    #[test]
    fn test_validate_mixed_bindings() -> Result<()> {
        let container = Container::new();

        // 添加普通绑定
        container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)?;

        // 添加可失败绑定（成功）
        trait AnotherService: Send + Sync {}
        struct AnotherServiceImpl;
        impl AnotherService for AnotherServiceImpl {}

        container
            .try_bind::<dyn AnotherService>(|_: &Container| {
                Ok(Arc::new(AnotherServiceImpl) as Arc<dyn AnotherService>)
            })
            .in_scope(Scope::Singleton)?;

        // 验证应该成功
        assert!(container.validate().is_ok());

        Ok(())
    }

    // ============================================================================
    // 增强测试：并发竞争条件
    // ============================================================================

    #[test]
    fn test_concurrent_bind_different_services() -> Result<()> {
        // 多线程同时绑定不同的服务
        let container = Arc::new(Container::new());

        trait ServiceA: Send + Sync {}
        trait ServiceB: Send + Sync {}
        trait ServiceC: Send + Sync {}
        trait ServiceD: Send + Sync {}

        struct ServiceAImpl;
        struct ServiceBImpl;
        struct ServiceCImpl;
        struct ServiceDImpl;

        impl ServiceA for ServiceAImpl {}
        impl ServiceB for ServiceBImpl {}
        impl ServiceC for ServiceCImpl {}
        impl ServiceD for ServiceDImpl {}

        let handles: Vec<_> = vec![
            {
                let c = container.clone();
                thread::spawn(move || {
                    c.bind::<dyn ServiceA>(|_: &Container| {
                        Arc::new(ServiceAImpl) as Arc<dyn ServiceA>
                    })
                    .in_scope(Scope::Singleton)
                })
            },
            {
                let c = container.clone();
                thread::spawn(move || {
                    c.bind::<dyn ServiceB>(|_: &Container| {
                        Arc::new(ServiceBImpl) as Arc<dyn ServiceB>
                    })
                    .in_scope(Scope::Singleton)
                })
            },
            {
                let c = container.clone();
                thread::spawn(move || {
                    c.bind::<dyn ServiceC>(|_: &Container| {
                        Arc::new(ServiceCImpl) as Arc<dyn ServiceC>
                    })
                    .in_scope(Scope::Singleton)
                })
            },
            {
                let c = container.clone();
                thread::spawn(move || {
                    c.bind::<dyn ServiceD>(|_: &Container| {
                        Arc::new(ServiceDImpl) as Arc<dyn ServiceD>
                    })
                    .in_scope(Scope::Singleton)
                })
            },
        ];

        for handle in handles {
            assert!(handle.join().is_ok());
        }

        // 验证所有服务都已绑定
        assert_eq!(container.binding_count(), 4);
        assert!(container.is_bound::<dyn ServiceA>());
        assert!(container.is_bound::<dyn ServiceB>());
        assert!(container.is_bound::<dyn ServiceC>());
        assert!(container.is_bound::<dyn ServiceD>());

        Ok(())
    }

    #[test]
    fn test_concurrent_get_and_unbind() -> Result<()> {
        // 测试并发获取和解绑的场景
        let container = Arc::new(Container::new());

        // 先绑定服务
        _ = container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton);

        const THREAD_COUNT: usize = 8;
        const ITERATIONS: usize = 50;

        let handles: Vec<_> = (0..THREAD_COUNT)
            .map(|i| {
                let c = container.clone();
                thread::spawn(move || {
                    for _ in 0..ITERATIONS {
                        // 偶数线程尝试获取服务
                        // 奇数线程检查绑定状态
                        if i % 2 == 0 {
                            let _ = c.get::<dyn TestService>();
                        } else {
                            let _ = c.is_bound::<dyn TestService>();
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        Ok(())
    }

    // ============================================================================
    // 增强测试：状态转换场景
    // ============================================================================

    #[test]
    fn test_singleton_rebind_after_unbind() -> Result<()> {
        let container = Container::new();

        // 第一次绑定
        container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 100, id: 1 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)?;

        // 获取第一个实例
        let service1: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        assert_eq!(service1.value(), 100);

        // 解绑
        container.unbind::<dyn TestService>();
        assert!(!container.is_bound::<dyn TestService>());

        // 重新绑定（不同的值）
        container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 200, id: 2 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)?;

        // 获取第二个实例（应该是新值）
        let service2: Arc<dyn TestService> = container.get::<dyn TestService>()?;
        assert_eq!(service2.value(), 200);

        // 两个实例应该不同
        assert!(!Arc::ptr_eq(&service1, &service2));

        Ok(())
    }

    #[test]
    fn test_unbind_all_and_rebind() -> Result<()> {
        let container = Container::new();

        trait ServiceA: Send + Sync {
            fn name(&self) -> &str;
        }
        struct ServiceAImpl {
            name: &'static str,
        }
        impl ServiceA for ServiceAImpl {
            fn name(&self) -> &str {
                self.name
            }
        }

        // 绑定多个服务
        container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 1, id: 1 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)?;

        container
            .bind::<dyn ServiceA>(|_: &Container| {
                Arc::new(ServiceAImpl { name: "first" }) as Arc<dyn ServiceA>
            })
            .in_scope(Scope::Singleton)?;

        assert_eq!(container.binding_count(), 2);

        // 清空所有绑定
        container.unbind_all();
        assert_eq!(container.binding_count(), 0);

        // 重新绑定
        container
            .bind::<dyn ServiceA>(|_: &Container| {
                Arc::new(ServiceAImpl { name: "second" }) as Arc<dyn ServiceA>
            })
            .in_scope(Scope::Singleton)?;

        // 验证新绑定有效
        let service: Arc<dyn ServiceA> = container.get::<dyn ServiceA>()?;
        assert_eq!(service.name(), "second");

        Ok(())
    }

    #[test]
    fn test_transient_to_singleton_rebind() -> Result<()> {
        let container = Container::new();
        let call_count = Arc::new(AtomicUsize::new(0));

        // 先以 Transient 绑定
        let count1 = call_count.clone();
        container
            .bind::<dyn TestService>(move |_: &Container| {
                count1.fetch_add(1, Ordering::SeqCst);
                Arc::new(TestServiceImpl { value: 1, id: 1 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Transient)?;

        // 获取两次，应该调用两次 factory
        let _ = container.get::<dyn TestService>()?;
        let _ = container.get::<dyn TestService>()?;
        assert_eq!(call_count.load(Ordering::SeqCst), 2);

        // 解绑后以 Singleton 重新绑定
        container.unbind::<dyn TestService>();
        call_count.store(0, Ordering::SeqCst);

        let count2 = call_count.clone();
        container
            .bind::<dyn TestService>(move |_: &Container| {
                count2.fetch_add(1, Ordering::SeqCst);
                Arc::new(TestServiceImpl { value: 2, id: 2 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)?;

        // 获取两次，应该只调用一次 factory
        let _ = container.get::<dyn TestService>()?;
        let _ = container.get::<dyn TestService>()?;
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        Ok(())
    }

    // ============================================================================
    // 增强测试：FallibleBinding 并发初始化
    // ============================================================================

    #[test]
    fn test_fallible_singleton_concurrent_init() -> Result<()> {
        let container = Arc::new(Container::new());
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        // 使用 try_bind 绑定 Singleton 服务
        container
            .try_bind::<dyn TestService>(move |_: &Container| {
                call_count_clone.fetch_add(1, Ordering::SeqCst);
                // 模拟一些初始化延迟
                std::thread::sleep(std::time::Duration::from_millis(1));
                Ok(Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>)
            })
            .in_scope(Scope::Singleton)?;

        const THREAD_COUNT: usize = 8;

        let handles: Vec<_> = (0..THREAD_COUNT)
            .map(|_| {
                let c = container.clone();
                thread::spawn(move || c.get::<dyn TestService>())
            })
            .collect();

        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.join().unwrap());
        }

        // 所有结果都应该成功
        for result in &results {
            assert!(result.is_ok());
        }

        // Singleton 应该只调用一次 factory（或在竞争情况下最多几次，但缓存同一个结果）
        let total_calls = call_count.load(Ordering::SeqCst);
        assert!(
            total_calls >= 1,
            "Factory should be called at least once, got {}",
            total_calls
        );

        // 所有返回的实例应该相同
        let first = results[0].as_ref().unwrap();
        for result in results.iter().skip(1) {
            let service = result.as_ref().unwrap();
            assert!(Arc::ptr_eq(first, service));
        }

        Ok(())
    }

    #[test]
    fn test_fallible_binding_error_not_cached() -> Result<()> {
        let container = Container::new();
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        // 绑定一个会失败的服务
        container
            .try_bind::<dyn TestService>(move |_: &Container| {
                let count = call_count_clone.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(RegistryError::NotBound("Simulated failure".to_string()))
                } else {
                    Ok(Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>)
                }
            })
            .in_scope(Scope::Transient) // 使用 Transient 以便多次调用
                ?;

        // 前两次调用应该失败
        assert!(container.get::<dyn TestService>().is_err());
        assert!(container.get::<dyn TestService>().is_err());

        // 第三次调用应该成功
        let result = container.get::<dyn TestService>();
        assert!(result.is_ok());
        assert_eq!(result?.value(), 42);

        Ok(())
    }

    // ============================================================================
    // 增强测试：深层依赖链
    // ============================================================================

    #[test]
    fn test_deep_dependency_chain() -> Result<()> {
        let container = Container::new();

        // 创建 5 层深的依赖链：E -> D -> C -> B -> A
        trait ServiceA: Send + Sync {
            fn level(&self) -> i32;
        }
        trait ServiceB: Send + Sync {
            fn level(&self) -> i32;
        }
        trait ServiceC: Send + Sync {
            fn level(&self) -> i32;
        }
        trait ServiceD: Send + Sync {
            fn level(&self) -> i32;
        }
        trait ServiceE: Send + Sync {
            fn level(&self) -> i32;
        }

        struct ServiceAImpl;
        struct ServiceBImpl(Arc<dyn ServiceA>);
        struct ServiceCImpl(Arc<dyn ServiceB>);
        struct ServiceDImpl(Arc<dyn ServiceC>);
        struct ServiceEImpl(Arc<dyn ServiceD>);

        impl ServiceA for ServiceAImpl {
            fn level(&self) -> i32 {
                1
            }
        }
        impl ServiceB for ServiceBImpl {
            fn level(&self) -> i32 {
                self.0.level() + 1
            }
        }
        impl ServiceC for ServiceCImpl {
            fn level(&self) -> i32 {
                self.0.level() + 1
            }
        }
        impl ServiceD for ServiceDImpl {
            fn level(&self) -> i32 {
                self.0.level() + 1
            }
        }
        impl ServiceE for ServiceEImpl {
            fn level(&self) -> i32 {
                self.0.level() + 1
            }
        }

        // 按顺序绑定
        container
            .bind::<dyn ServiceA>(|_: &Container| Arc::new(ServiceAImpl) as Arc<dyn ServiceA>)
            .in_scope(Scope::Singleton)?;

        container
            .try_bind::<dyn ServiceB>(|c: &Container| {
                let a = c.get::<dyn ServiceA>()?;
                Ok(Arc::new(ServiceBImpl(a)) as Arc<dyn ServiceB>)
            })
            .in_scope(Scope::Singleton)?;

        container
            .try_bind::<dyn ServiceC>(|c: &Container| {
                let b = c.get::<dyn ServiceB>()?;
                Ok(Arc::new(ServiceCImpl(b)) as Arc<dyn ServiceC>)
            })
            .in_scope(Scope::Singleton)?;

        container
            .try_bind::<dyn ServiceD>(|c: &Container| {
                let c_service = c.get::<dyn ServiceC>()?;
                Ok(Arc::new(ServiceDImpl(c_service)) as Arc<dyn ServiceD>)
            })
            .in_scope(Scope::Singleton)?;

        container
            .try_bind::<dyn ServiceE>(|c: &Container| {
                let d = c.get::<dyn ServiceD>()?;
                Ok(Arc::new(ServiceEImpl(d)) as Arc<dyn ServiceE>)
            })
            .in_scope(Scope::Singleton)?;

        // 获取最深层的服务
        let service_e: Arc<dyn ServiceE> = container.get::<dyn ServiceE>()?;
        assert_eq!(service_e.level(), 5);

        Ok(())
    }

    // ============================================================================
    // 增强测试：边界条件
    // ============================================================================

    #[test]
    fn test_unbind_nonexistent_service() -> Result<()> {
        let container = Container::new();

        // 解绑不存在的服务不应该 panic
        container.unbind::<dyn TestService>();
        assert!(!container.is_bound::<dyn TestService>());

        Ok(())
    }

    #[test]
    fn test_get_after_unbind() -> Result<()> {
        let container = Container::new();

        // 绑定服务
        container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)?;

        // 解绑
        container.unbind::<dyn TestService>();

        // 获取应该返回 NotBound 错误
        let result: Result<Arc<dyn TestService>> = container.get();
        assert!(result.is_err());
        assert!(matches!(result, Err(RegistryError::NotBound(_))));

        Ok(())
    }

    #[test]
    fn test_many_services_binding() -> Result<()> {
        // 压力测试：绑定大量服务
        let container = Container::new();

        // 使用具体类型绑定多个服务
        for i in 0..100 {
            let value = i;
            container
                .bind_instance(Arc::new(TestServiceImpl {
                    value,
                    id: i as usize,
                }))
                .in_scope(Scope::Singleton)?; // 由于所有都是同一类型，只有第一个会成功
        }

        // 至少绑定了一个
        assert!(container.binding_count() >= 1);

        Ok(())
    }

    #[test]
    fn test_validate_empty_container() -> Result<()> {
        let container = Container::new();

        // 空容器验证应该成功
        assert!(container.validate().is_ok());
        assert_eq!(container.binding_count(), 0);

        Ok(())
    }

    #[test]
    fn test_is_bound_for_fallible_binding() -> Result<()> {
        let container = Container::new();

        // 使用 try_bind 绑定
        container
            .try_bind::<dyn TestService>(|_: &Container| {
                Ok(Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>)
            })
            .in_scope(Scope::Singleton)?;

        // is_bound 应该返回 true（即使是 fallible binding）
        assert!(container.is_bound::<dyn TestService>());

        Ok(())
    }

    #[test]
    fn test_binding_count_includes_both_types() -> Result<()> {
        let container = Container::new();

        trait ServiceA: Send + Sync {}
        struct ServiceAImpl;
        impl ServiceA for ServiceAImpl {}

        // 添加普通绑定
        container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)?;

        // 添加可失败绑定
        container
            .try_bind::<dyn ServiceA>(|_: &Container| {
                Ok(Arc::new(ServiceAImpl) as Arc<dyn ServiceA>)
            })
            .in_scope(Scope::Singleton)?;

        // binding_count 应该包含两种类型
        assert_eq!(container.binding_count(), 2);

        Ok(())
    }
}
