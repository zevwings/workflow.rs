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
use std::sync::Arc;

// 第三方库
use dashmap::DashMap;
use once_cell::sync::Lazy;

// 内部导入
use crate::binding::{Binding, BindingBuilder};
use crate::error::{RegistryError, Result};
use crate::ServiceIdentifier;

// 线程局部变量：跟踪服务解析栈，检测循环依赖
thread_local! {
    static RESOLUTION_STACK: std::cell::RefCell<Vec<ServiceIdentifier>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// 依赖注入容器
pub struct Container {
    bindings: DashMap<ServiceIdentifier, Binding>,
}

/// 全局容器单例，使用 Lazy + DashMap 实现无锁并发访问
static GLOBAL_CONTAINER: Lazy<Container> = Lazy::new(Container::new);

impl Container {
    /// 创建新的容器（仅内部使用）
    pub(crate) fn new() -> Self {
        Self {
            bindings: DashMap::new(),
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
        let identifier = std::any::TypeId::of::<Arc<T>>();
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
        let identifier = std::any::TypeId::of::<Arc<T>>();
        BindingBuilder::new(identifier, factory, self)
    }

    /// 获取服务，Singleton 作用域返回同一个 Arc 的克隆
    pub fn get<T: 'static + Send + Sync + ?Sized>(&self) -> Result<Arc<T>> {
        let identifier = std::any::TypeId::of::<Arc<T>>();

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

        // 查找绑定
        let binding = self.bindings.get(&identifier).ok_or_else(|| {
            RegistryError::NotBound(format!(
                "Service '{}' is not bound",
                std::any::type_name::<Arc<T>>()
            ))
        })?;

        // 在没有持有 stack 借用的情况下调用 resolve
        let result = binding.resolve::<T>(self);

        // 无论成功与否，都要从栈中弹出
        RESOLUTION_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });

        result
    }

    /// 内部方法：完成绑定
    pub(crate) fn add_binding(&self, binding: Binding) -> Result<()> {
        // DashMap 的 insert 返回旧值，如果存在则表示已绑定
        if self.bindings.insert(binding.identifier, binding).is_some() {
            return Err(RegistryError::AlreadyBound(
                "Service already bound".to_string(),
            ));
        }
        Ok(())
    }

    /// 检查服务是否已绑定
    pub fn is_bound<T: 'static + ?Sized>(&self) -> bool {
        let identifier = std::any::TypeId::of::<Arc<T>>();
        self.bindings.contains_key(&identifier)
    }

    /// 解绑服务
    pub fn unbind<T: 'static + ?Sized>(&self) {
        let identifier = std::any::TypeId::of::<Arc<T>>();
        self.bindings.remove(&identifier);
    }

    /// 解绑所有服务
    pub fn unbind_all(&self) {
        self.bindings.clear();
    }

    /// 获取绑定数量
    pub fn binding_count(&self) -> usize {
        self.bindings.len()
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

        // DashMap 的 iter() 提供快照式迭代，不会长时间持有锁
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
    fn test_container_creation() {
        // 测试 new() 和 default()
        let container1 = Container::new();
        let container2 = Container::default();

        assert_eq!(container1.binding_count(), 0);
        assert_eq!(container2.binding_count(), 0);
        assert!(!container1.is_bound::<dyn TestService>());
    }

    #[test]
    fn test_global_container_lifecycle() {
        // 测试全局容器获取和初始化状态
        let container1 = Container::global();
        let container2 = Container::global();

        // 应该返回同一个引用
        assert!(std::ptr::eq(container1, container2));
        assert!(Container::is_initialized());
    }

    // ============================================================================
    // 全局容器注册
    // ============================================================================

    #[test]
    #[serial]
    fn test_register_success() {
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
    }

    #[test]
    #[serial]
    fn test_register_error() {
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
    }

    // ============================================================================
    // 服务绑定
    // ============================================================================

    #[test]
    fn test_bind_and_check() {
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
    }

    #[test]
    fn test_add_binding_scenarios() {
        let container = Container::new();
        let identifier = std::any::TypeId::of::<Arc<dyn TestService>>();
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
    }

    // ============================================================================
    // 服务获取
    // ============================================================================

    #[test]
    fn test_get_success() {
        let container = Container::new();
        container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)
            .unwrap();

        let service: Arc<dyn TestService> = container.get::<dyn TestService>().unwrap();
        assert_eq!(service.value(), 42);
    }

    #[test]
    fn test_get_not_bound() {
        let container = Container::new();
        let result: Result<Arc<dyn TestService>> = container.get();

        assert!(result.is_err());
        assert!(matches!(result, Err(RegistryError::NotBound(_))));
    }

    // ============================================================================
    // 作用域管理 (Singleton vs Transient)
    // ============================================================================

    #[rstest]
    #[case(Scope::Singleton, true)] // Singleton 返回同一个实例
    #[case(Scope::Transient, false)] // Transient 返回不同实例
    fn test_scope_behavior(#[case] scope: Scope, #[case] should_be_same: bool) {
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
            .in_scope(scope)
            .unwrap();

        let service1: Arc<dyn TestService> = container.get::<dyn TestService>().unwrap();
        let service2: Arc<dyn TestService> = container.get::<dyn TestService>().unwrap();

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
    }

    // ============================================================================
    // 解绑操作
    // ============================================================================

    #[test]
    fn test_unbind() {
        let container = Container::new();

        container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)
            .unwrap();

        assert!(container.is_bound::<dyn TestService>());
        assert_eq!(container.binding_count(), 1);

        container.unbind::<dyn TestService>();

        assert!(!container.is_bound::<dyn TestService>());
        assert_eq!(container.binding_count(), 0);
    }

    #[test]
    fn test_unbind_all() {
        let container = Container::new();

        // 注册多个服务
        container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)
            .unwrap();

        trait AnotherService: Send + Sync {}
        struct AnotherServiceImpl;
        impl AnotherService for AnotherServiceImpl {}

        container
            .bind::<dyn AnotherService>(|_: &Container| {
                Arc::new(AnotherServiceImpl) as Arc<dyn AnotherService>
            })
            .in_scope(Scope::Singleton)
            .unwrap();

        assert_eq!(container.binding_count(), 2);

        // 清空所有绑定
        container.unbind_all();

        assert_eq!(container.binding_count(), 0);
        assert!(!container.is_bound::<dyn TestService>());
        assert!(!container.is_bound::<dyn AnotherService>());
    }

    // ============================================================================
    // 验证功能
    // ============================================================================

    #[test]
    fn test_validate_container() {
        // 测试空容器验证
        let empty_container = Container::new();
        assert!(empty_container.validate().is_ok());

        // 测试有正常服务的容器验证
        let container = Container::new();
        container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl { value: 42, id: 1 }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)
            .unwrap();
        assert!(container.validate().is_ok());
    }

    #[test]
    fn test_validate_with_panicking_factory() {
        let container = Container::new();

        // 创建会 panic 的工厂函数
        container
            .bind::<dyn TestService>(|_: &Container| {
                panic!("Factory function panicked during validation");
            })
            .in_scope(Scope::Singleton)
            .unwrap();

        // 验证应该返回错误
        let result = container.validate();
        assert!(result.is_err());

        if let Err(RegistryError::ValidationError(msg)) = result {
            assert!(msg.contains("panicked"));
            assert!(msg.contains("Factory function"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    // ============================================================================
    // 多服务管理
    // ============================================================================

    #[test]
    fn test_multiple_services() {
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
            .in_scope(Scope::Singleton)
            .unwrap();

        container
            .bind::<dyn ServiceB>(|_: &Container| Arc::new(ServiceBImpl) as Arc<dyn ServiceB>)
            .in_scope(Scope::Singleton)
            .unwrap();

        assert_eq!(container.binding_count(), 2);

        let service_a: Arc<dyn ServiceA> = container.get::<dyn ServiceA>().unwrap();
        let service_b: Arc<dyn ServiceB> = container.get::<dyn ServiceB>().unwrap();

        assert_eq!(service_a.value(), 1);
        assert_eq!(service_b.value(), 2);
    }

    // ============================================================================
    // 并发安全测试
    // ============================================================================

    #[test]
    #[serial]
    fn test_concurrent_global_access() {
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
        })
        .unwrap();

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
    }

    #[test]
    #[serial]
    fn test_concurrent_singleton_consistency() {
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
        })
        .unwrap();

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
    }

    // ============================================================================
    // bind 方法：Arc 实例和闭包测试
    // ============================================================================

    #[test]
    fn test_bind_with_arc_instance() {
        let container = Container::new();

        // 直接使用 Arc 实例绑定
        let instance: Arc<dyn TestService> = Arc::new(TestServiceImpl { value: 888, id: 99 });
        container.bind::<dyn TestService>(instance).in_scope(Scope::Singleton).unwrap();

        assert!(container.is_bound::<dyn TestService>());

        // 获取服务并验证
        let service: Arc<dyn TestService> = container.get::<dyn TestService>().unwrap();
        assert_eq!(service.value(), 888);
        assert_eq!(service.id(), 99);
    }

    #[test]
    fn test_bind_concrete_type() {
        let container = Container::new();

        // 测试绑定具体类型（不是 trait 对象）
        container
            .bind::<TestServiceImpl>(Arc::new(TestServiceImpl { value: 777, id: 88 }))
            .in_scope(Scope::Singleton)
            .unwrap();

        let service: Arc<TestServiceImpl> = container.get::<TestServiceImpl>().unwrap();
        assert_eq!(service.value, 777);
        assert_eq!(service.id, 88);
    }

    #[test]
    fn test_bind_with_closure() {
        let container = Container::new();

        // 使用闭包绑定
        container
            .bind::<dyn TestService>(|_: &Container| {
                Arc::new(TestServiceImpl {
                    value: 111,
                    id: 222,
                }) as Arc<dyn TestService>
            })
            .in_scope(Scope::Singleton)
            .unwrap();

        let service: Arc<dyn TestService> = container.get::<dyn TestService>().unwrap();
        assert_eq!(service.value(), 111);
    }

    #[test]
    fn test_bind_with_dependencies() {
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
        container
            .bind::<dyn ConfigService>(|_: &Container| {
                Arc::new(ConfigServiceImpl {
                    config: "production".to_string(),
                }) as Arc<dyn ConfigService>
            })
            .in_scope(Scope::Singleton)
            .unwrap();

        // 绑定依赖于其他服务的服务
        container
            .bind::<dyn AppService>(|c: &Container| {
                let config = c.get::<dyn ConfigService>().unwrap();
                Arc::new(AppServiceImpl { config }) as Arc<dyn AppService>
            })
            .in_scope(Scope::Singleton)
            .unwrap();

        // 验证依赖注入正常工作
        let app_service: Arc<dyn AppService> = container.get::<dyn AppService>().unwrap();
        assert_eq!(app_service.get_app_name(), "App: production");
    }

    // ============================================================================
    // bind_instance 测试（直接绑定具体类型，无需指定泛型）
    // ============================================================================

    #[test]
    fn test_bind_instance_basic() {
        let container = Container::new();

        // 直接绑定具体类型，无需显式指定类型参数
        container
            .bind_instance(Arc::new(TestServiceImpl {
                value: 999,
                id: 100,
            }))
            .in_scope(Scope::Singleton)
            .unwrap();

        // 使用具体类型获取
        let service: Arc<TestServiceImpl> = container.get::<TestServiceImpl>().unwrap();
        assert_eq!(service.value, 999);
        assert_eq!(service.id, 100);
    }

    #[test]
    fn test_bind_instance_multiple_types() {
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
            .in_scope(Scope::Singleton)
            .unwrap();

        container
            .bind_instance(Arc::new(ServiceB { count: 42 }))
            .in_scope(Scope::Singleton)
            .unwrap();

        // 分别获取
        let service_a: Arc<ServiceA> = container.get::<ServiceA>().unwrap();
        let service_b: Arc<ServiceB> = container.get::<ServiceB>().unwrap();

        assert_eq!(service_a.name, "ServiceA");
        assert_eq!(service_b.count, 42);
    }

    #[test]
    fn test_bind_instance_vs_bind_trait() {
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
            .in_scope(Scope::Singleton)
            .unwrap();

        // 2. 绑定 trait 对象（不同的标识符）
        let trait_instance: Arc<dyn MyService> = Arc::new(MyServiceImpl { value: 200 });
        container
            .bind::<dyn MyService>(trait_instance)
            .in_scope(Scope::Singleton)
            .unwrap();

        // 可以分别获取
        let concrete: Arc<MyServiceImpl> = container.get::<MyServiceImpl>().unwrap();
        let trait_obj: Arc<dyn MyService> = container.get::<dyn MyService>().unwrap();

        assert_eq!(concrete.value, 100);
        assert_eq!(trait_obj.get_value(), 200);
    }
}
