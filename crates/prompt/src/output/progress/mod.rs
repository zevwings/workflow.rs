//! 进度条模块
//!
//! 提供统一的进度条功能，用于显示有明确进度的操作（如下载、上传等）。
//!
//! # 示例
//!
//! ```rust,no_run
//! use prompt::{progress_bar, Progress};
//!
//! // 方式 1：已知总数
//! let pb = progress_bar("Downloading files...")
//!     .with_total(100)
//!     .start();
//! for i in 0..100 {
//!     pb.inc(1);
//!     std::thread::sleep(std::time::Duration::from_millis(10));
//! }
//! pb.finish_with_message("Download completed!");
//!
//! // 方式 2：下载模式（显示速度和 ETA）
//! let pb = Progress::new_download(1024 * 1024, "Downloading...");
//! pb.set_position(512 * 1024);
//! pb.finish_with_message("Download completed!");
//!
//! // 方式 3：未知总数（使用 spinner 模式）
//! let pb = Progress::new_unknown("Downloading...");
//! pb.inc(1);
//! pb.finish_with_message("Download completed!");
//! ```

mod bar;
mod builder;
mod format;
#[allow(clippy::module_inception)]
mod progress;
mod render;
mod terminal;

pub use bar::ProgressBar;
pub use builder::ProgressBarBuilder;
pub use progress::Progress;

/// 便捷函数
pub fn progress_bar(message: impl Into<String>) -> ProgressBarBuilder {
    ProgressBarBuilder::new(message)
}

/// 格式化进度条宏
///
/// 提供格式化字符串的便捷方式，避免手动使用 `format!`。
/// 使用 `progress!` 作为宏名，与 `spinner!` 保持一致。
///
/// # Examples
///
/// ```rust,no_run
/// use toolkit::progress;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let pb = progress!("Downloading {}...", "file.zip")
///     .with_total(100)
///     .start();
/// // 使用进度条
/// pb.finish_with_message("Download completed!");
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! progress {
    ($($arg:tt)*) => {
        $crate::progress_bar(format!($($arg)*))
    };
}
