//! 对话框跳过配置（内部 API）
//!
//! 提供 thread-local storage 来存储对话框的非交互式配置。
//! 主要用于测试场景，通过 `DialogTestGuard` 设置配置。

#![allow(dead_code)] // 这些函数在测试代码中使用，编译器在检查库代码时看不到

use std::cell::RefCell;

thread_local! {
    /// 对话框配置（thread-local storage）
    static DIALOG_CONFIG: RefCell<Option<DialogConfig>> = const { RefCell::new(None) };
}

/// 对话框配置
#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct DialogConfig {
    /// ConfirmDialog 的预设值
    pub confirm_value: Option<bool>,
    /// SelectDialog 的预设索引
    pub select_index: Option<usize>,
    /// InputDialog 的预设值（队列，按顺序使用）
    pub input_value: Option<String>,
    /// InputDialog 的预设值队列（用于多个 InputDialog 按顺序使用）
    pub input_value_queue: Vec<String>,
    /// MultiSelectDialog 的预设索引列表
    pub multi_select_indices: Option<Vec<usize>>,
}

impl DialogConfig {
    #[doc(hidden)]
    pub fn new() -> Self {
        Self {
            confirm_value: None,
            select_index: None,
            input_value: None,
            input_value_queue: Vec::new(),
            multi_select_indices: None,
        }
    }
}

/// 对话框配置管理器（零大小结构体，用于组织 thread-local storage 操作）
///
/// 这是一个零大小结构体，仅用于组织相关的函数，不存储任何状态。
/// 所有状态都存储在 thread-local storage 中。
#[doc(hidden)]
pub struct DialogConfigManager;

impl DialogConfigManager {
    /// 设置对话框配置（仅用于测试）
    ///
    /// 注意：这是内部 API，仅供测试代码使用
    #[doc(hidden)]
    pub fn set_config(config: DialogConfig) {
        DIALOG_CONFIG.with(|c| {
            *c.borrow_mut() = Some(config);
        });
    }

    /// 更新对话框配置（仅用于测试）
    /// 合并新配置到现有配置中，而不是完全替换
    ///
    /// 注意：这是内部 API，仅供测试代码使用
    #[doc(hidden)]
    pub fn update_config<F>(f: F)
    where
        F: FnOnce(&mut DialogConfig),
    {
        DIALOG_CONFIG.with(|c| {
            if let Some(ref mut config) = *c.borrow_mut() {
                f(config);
            }
        });
    }

    /// 清理对话框配置（仅用于测试）
    ///
    /// 注意：这是内部 API，仅供测试代码使用
    #[doc(hidden)]
    pub fn clear_config() {
        DIALOG_CONFIG.with(|c| {
            *c.borrow_mut() = None;
        });
    }

    /// 获取 ConfirmDialog 的预设值（内部 API，供对话框使用）
    #[doc(hidden)]
    pub fn get_confirm_value() -> Option<bool> {
        DIALOG_CONFIG.with(|c| c.borrow().as_ref().and_then(|config| config.confirm_value))
    }

    /// 获取 SelectDialog 的预设索引（内部 API，供对话框使用）
    #[doc(hidden)]
    pub fn get_select_index() -> Option<usize> {
        DIALOG_CONFIG.with(|c| c.borrow().as_ref().and_then(|config| config.select_index))
    }

    /// 获取并弹出 InputDialog 的预设值（内部 API，供对话框使用）
    /// 优先从队列中获取，如果队列为空则使用单个值
    pub(crate) fn pop_input_value() -> Option<String> {
        DIALOG_CONFIG.with(|c| {
            let mut config_ref = c.borrow_mut();
            if let Some(config) = config_ref.as_mut() {
                // 优先从队列中获取
                if !config.input_value_queue.is_empty() {
                    return Some(config.input_value_queue.remove(0));
                }
                // 如果队列为空，使用单个值并清除
                if let Some(value) = config.input_value.take() {
                    return Some(value);
                }
            }
            None
        })
    }

    /// 获取 InputDialog 的预设值（内部 API，供对话框使用，不弹出）
    /// 用于向后兼容，但推荐使用 pop_input_value
    pub(crate) fn get_input_value() -> Option<String> {
        DIALOG_CONFIG.with(|c| {
            let config_ref = c.borrow();
            if let Some(config) = config_ref.as_ref() {
                // 优先从队列中获取（不弹出）
                if !config.input_value_queue.is_empty() {
                    return Some(config.input_value_queue[0].clone());
                }
                // 如果队列为空，使用单个值
                return config.input_value.clone();
            }
            None
        })
    }

    /// 获取 MultiSelectDialog 的预设索引列表（内部 API，供对话框使用）
    pub(crate) fn get_multi_select_indices() -> Option<Vec<usize>> {
        DIALOG_CONFIG
            .with(|c| c.borrow().as_ref().and_then(|config| config.multi_select_indices.clone()))
    }

    /// 检查是否启用了非交互式模式（内部 API，供对话框使用）
    #[doc(hidden)]
    pub fn is_non_interactive() -> bool {
        DIALOG_CONFIG.with(|c| c.borrow().is_some())
    }
}

/// 对话框配置构建器（仅用于测试）
///
/// 注意：这是内部 API，仅供测试代码使用
#[doc(hidden)]
pub struct DialogConfigBuilder {
    config: DialogConfig,
}

impl DialogConfigBuilder {
    #[doc(hidden)]
    pub fn new() -> Self {
        Self {
            config: DialogConfig::new(),
        }
    }

    #[doc(hidden)]
    pub fn with_confirm_value(mut self, value: bool) -> Self {
        self.config.confirm_value = Some(value);
        self
    }

    #[doc(hidden)]
    pub fn with_select_index(mut self, index: usize) -> Self {
        self.config.select_index = Some(index);
        self
    }

    #[doc(hidden)]
    pub fn with_input_value(mut self, value: impl Into<String>) -> Self {
        self.config.input_value = Some(value.into());
        self
    }

    #[doc(hidden)]
    pub fn with_multi_select_indices(mut self, indices: Vec<usize>) -> Self {
        self.config.multi_select_indices = Some(indices);
        self
    }

    #[doc(hidden)]
    pub fn build(self) -> DialogConfig {
        self.config
    }
}
