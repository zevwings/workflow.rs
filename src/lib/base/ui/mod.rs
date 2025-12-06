//! UI 组件模块
//!
//! 提供基于 ratatui 的 UI 组件，包括对话框、进度条、日志查看器等。

pub mod components;
pub mod dialogs;
pub mod file_log_viewer;
pub mod layout;
pub mod log_viewer;
pub mod progress;
pub mod theme;

// 重新导出常用组件
pub use dialogs::{ConfirmDialog, InputDialog, MultiSelectDialog, SelectDialog};
pub use file_log_viewer::FileLogViewer;
pub use log_viewer::LogViewer;
pub use progress::ProgressBar;
