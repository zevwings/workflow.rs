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
pub struct DialogTestGuard {
    _private: (), // 仅用于 RAII，不存储任何数据
}

impl DialogTestGuard {
    /// 创建新的对话框测试配置守卫
    ///
    /// 启用非交互式模式，对话框将使用预设值而不是显示交互式界面。
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
}
