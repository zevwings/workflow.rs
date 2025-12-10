//! Indicator 模块
//!
//! 提供统一的进度指示器功能，包括：
//! - Spinner - 用于不确定进度的长时间运行操作
//! - Progress - 用于有明确进度的操作（如下载、上传等）

pub mod progress;
pub mod spinner;

// 重新导出
pub use progress::Progress;
pub use spinner::Spinner;
