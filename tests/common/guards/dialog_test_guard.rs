#![allow(clippy::test_attr_in_doctest)]

//! 对话框测试配置守卫
//!
//! 管理测试期间的对话框非交互式配置，自动恢复原始状态。
//!
//! # 使用示例
//!
//! ```rust
//! use tests::common::guards::DialogTestGuard;
//!
//! #[test]
//! fn test_with_dialog_config() -> color_eyre::Result<()> {
//!     let _guard = DialogTestGuard::new()
//!         .with_select_index(0)
//!         .with_confirm_value(false);
//!
//!     // 测试代码，对话框会自动使用 guard 的配置
//!     // BranchSyncCommand::sync_in(...)?;
//!
//!     // Drop时自动清理配置
//!     Ok(())
//! }
//! ```

use workflow::base::dialog::skip_config::{DialogConfigBuilder, DialogConfigManager};

/// 对话框测试配置守卫
///
/// 管理测试期间的对话框非交互式配置，使用 RAII 模式确保配置在作用域结束时自动清理。
/// 使用 thread-local storage 存储配置，保证线程安全。
///
/// # 并行测试安全性
///
/// 此 guard 完全支持并行测试：
/// - 使用 `thread_local!` 宏，每个线程有独立的配置副本
/// - 在创建时自动清理可能存在的旧配置（防御性编程）
/// - 在 drop 时自动清理配置（RAII 模式）
/// - 可以安全地使用 `cargo test -- --test-threads=N` 并行运行测试
///
/// # 线程池重用安全性
///
/// 即使测试框架重用线程（线程池），此 guard 也能保证安全：
/// - 创建时先清理旧配置，确保每个测试有干净的状态
/// - Drop 时清理配置，确保测试结束后不留残留
/// - Thread-local storage 在同一个线程的不同测试之间是隔离的
pub struct DialogTestGuard {
    _private: (), // 仅用于 RAII，不存储任何数据
}

impl DialogTestGuard {
    /// 创建新的对话框测试配置守卫
    ///
    /// 启用非交互式模式，对话框将使用预设值而不是显示交互式界面。
    ///
    /// # 并行测试安全性
    ///
    /// 此方法使用 thread-local storage，完全支持并行测试。每个测试线程有独立的配置副本，
    /// 互不干扰。可以安全地使用 `cargo test -- --test-threads=N` 并行运行测试。
    ///
    /// 即使测试框架重用线程（线程池），此方法也会先清理可能存在的旧配置，确保每个测试
    /// 开始时都有干净的状态。
    ///
    /// # 返回
    ///
    /// 返回新的 `DialogTestGuard` 实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::guards::DialogTestGuard;
    ///
    /// let _guard = DialogTestGuard::new();
    /// ```
    pub fn new() -> Self {
        // 防御性清理：确保即使线程被重用，也先清理可能存在的旧配置
        // 这保证了即使测试框架重用线程（线程池），每个测试也有干净的状态
        DialogConfigManager::clear_config();

        let config = DialogConfigBuilder::new().build();
        DialogConfigManager::set_config(config);
        Self { _private: () }
    }

    /// 设置 ConfirmDialog 的预设值
    ///
    /// # 参数
    ///
    /// * `value` - ConfirmDialog 的返回值
    ///
    /// # 返回
    ///
    /// 返回 `Self` 以支持链式调用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::guards::DialogTestGuard;
    ///
    /// let _guard = DialogTestGuard::new()
    ///     .with_confirm_value(true);
    /// ```
    pub fn with_confirm_value(self, value: bool) -> Self {
        DialogConfigManager::update_config(|config| {
            config.confirm_value = Some(value);
        });
        self
    }

    /// 设置 SelectDialog 的预设索引
    ///
    /// # 参数
    ///
    /// * `index` - SelectDialog 的选项索引
    ///
    /// # 返回
    ///
    /// 返回 `Self` 以支持链式调用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::guards::DialogTestGuard;
    ///
    /// let _guard = DialogTestGuard::new()
    ///     .with_select_index(0);
    /// ```
    pub fn with_select_index(self, index: usize) -> Self {
        DialogConfigManager::update_config(|config| {
            config.select_index = Some(index);
        });
        self
    }

    /// 设置 InputDialog 的预设值
    ///
    /// # 参数
    ///
    /// * `value` - InputDialog 的返回值
    ///
    /// # 返回
    ///
    /// 返回 `Self` 以支持链式调用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::guards::DialogTestGuard;
    ///
    /// let _guard = DialogTestGuard::new()
    ///     .with_input_value("test@example.com");
    /// ```
    #[allow(dead_code)] // 保留以备将来使用
    pub fn with_input_value(self, value: impl Into<String>) -> Self {
        DialogConfigManager::update_config(|config| {
            config.input_value = Some(value.into());
        });
        self
    }

    /// 添加 InputDialog 的预设值到队列（支持多个 InputDialog 按顺序使用）
    ///
    /// # 参数
    ///
    /// * `values` - InputDialog 的返回值列表，会按顺序添加到队列
    ///
    /// # 返回
    ///
    /// 返回 `Self` 以支持链式调用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::guards::DialogTestGuard;
    ///
    /// let _guard = DialogTestGuard::new()
    ///     .with_input_value_queue(vec!["value1", "value2", "value3"]);
    /// ```
    #[allow(dead_code)] // 在 E2E 测试中使用
    pub fn with_input_value_queue(self, values: Vec<impl Into<String>>) -> Self {
        DialogConfigManager::update_config(|config| {
            config.input_value_queue.extend(values.into_iter().map(|v| v.into()));
        });
        self
    }

    /// 设置 MultiSelectDialog 的预设索引列表
    ///
    /// # 参数
    ///
    /// * `indices` - MultiSelectDialog 的选项索引列表
    ///
    /// # 返回
    ///
    /// 返回 `Self` 以支持链式调用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::guards::DialogTestGuard;
    ///
    /// let _guard = DialogTestGuard::new()
    ///     .with_multi_select_indices(vec![0, 2]);
    /// ```
    #[allow(dead_code)] // 保留以备将来使用
    pub fn with_multi_select_indices(self, indices: Vec<usize>) -> Self {
        DialogConfigManager::update_config(|config| {
            config.multi_select_indices = Some(indices);
        });
        self
    }
}

impl Drop for DialogTestGuard {
    fn drop(&mut self) {
        // 清理配置
        DialogConfigManager::clear_config();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use workflow::base::dialog::skip_config;

    /// 测试 DialogTestGuard 设置和恢复配置
    ///
    /// ## 测试目的
    /// 验证 `DialogTestGuard` 能够设置对话框配置，并在 drop 时自动清理。
    ///
    /// ## 测试场景
    /// 1. 创建 DialogTestGuard 并设置配置
    /// 2. 验证配置已设置
    /// 3. Drop guard
    /// 4. 验证配置已清理
    ///
    /// ## 预期结果
    /// - 设置时，配置被正确设置
    /// - Drop 后，配置被清理
    #[test]
    fn test_dialog_test_guard_set_and_clear() {
        {
            let _guard = DialogTestGuard::new().with_confirm_value(true).with_select_index(0);

            // 验证配置已设置
            assert!(skip_config::DialogConfigManager::is_non_interactive());
            assert_eq!(
                skip_config::DialogConfigManager::get_confirm_value(),
                Some(true)
            );
            assert_eq!(
                skip_config::DialogConfigManager::get_select_index(),
                Some(0)
            );
        }

        // 验证配置已清理
        assert!(!skip_config::DialogConfigManager::is_non_interactive());
        assert_eq!(skip_config::DialogConfigManager::get_confirm_value(), None);
        assert_eq!(skip_config::DialogConfigManager::get_select_index(), None);
    }

    /// 测试 DialogTestGuard 的线程隔离性
    ///
    /// ## 测试目的
    /// 验证 `DialogTestGuard` 在不同线程之间的配置是隔离的，确保并行测试的安全性。
    ///
    /// ## 测试场景
    /// 1. 在主线程设置配置
    /// 2. 在另一个线程检查配置（应该是 None）
    /// 3. 验证线程隔离性
    ///
    /// ## 预期结果
    /// - 不同线程之间的配置完全隔离
    /// - 一个线程的配置不会影响另一个线程
    #[test]
    fn test_thread_isolation() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use std::thread;

        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        // 在主线程设置配置
        let _guard = DialogTestGuard::new().with_confirm_value(true);

        // 验证主线程能看到配置
        assert!(skip_config::DialogConfigManager::is_non_interactive());
        assert_eq!(
            skip_config::DialogConfigManager::get_confirm_value(),
            Some(true)
        );

        // 在另一个线程检查配置（应该是 None，因为 thread-local storage 是线程隔离的）
        let handle = thread::spawn(move || {
            let is_set = skip_config::DialogConfigManager::is_non_interactive();
            flag_clone.store(is_set, Ordering::Relaxed);
        });

        handle.join().unwrap();

        // 另一个线程应该看不到主线程的配置
        assert!(
            !flag.load(Ordering::Relaxed),
            "Thread-local storage should be isolated between threads"
        );
    }

    /// 测试 DialogTestGuard 的防御性清理
    ///
    /// ## 测试目的
    /// 验证 `DialogTestGuard::new()` 会先清理可能存在的旧配置，确保线程重用时的安全性。
    ///
    /// ## 测试场景
    /// 1. 手动设置一个配置
    /// 2. 创建 DialogTestGuard（应该先清理旧配置）
    /// 3. 验证配置被正确设置（而不是保留旧配置）
    ///
    /// ## 预期结果
    /// - `new()` 会先清理旧配置
    /// - 然后设置新配置
    #[test]
    fn test_defensive_cleanup() {
        // 手动设置一个配置（模拟线程重用场景）
        let old_config = DialogConfigBuilder::new()
            .with_confirm_value(false)
            .with_select_index(99)
            .build();
        DialogConfigManager::set_config(old_config);

        // 验证旧配置存在
        assert!(skip_config::DialogConfigManager::is_non_interactive());
        assert_eq!(
            skip_config::DialogConfigManager::get_confirm_value(),
            Some(false)
        );
        assert_eq!(
            skip_config::DialogConfigManager::get_select_index(),
            Some(99)
        );

        // 创建新的 guard（应该先清理旧配置，然后设置新配置）
        let _guard = DialogTestGuard::new().with_confirm_value(true).with_select_index(0);

        // 验证新配置被正确设置（而不是保留旧配置）
        assert!(skip_config::DialogConfigManager::is_non_interactive());
        assert_eq!(
            skip_config::DialogConfigManager::get_confirm_value(),
            Some(true)
        );
        assert_eq!(
            skip_config::DialogConfigManager::get_select_index(),
            Some(0)
        );
    }
}
