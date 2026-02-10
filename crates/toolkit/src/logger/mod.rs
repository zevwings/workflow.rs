//! Logger 模块
//!
//! 提供结构化日志记录功能，基于 tracing crate。
//!
//! ## 模块组织
//!
//! - [`config`] - 日志配置结构体
//! - [`subscriber`] - Tracing subscriber 配置实现
//! - [`path`] - 日志文件路径管理
//! - [`macros`] - 日志宏定义
//!
//! ## 使用示例
//!
//! ### 初始化
//!
//! 使用 [`LoggerConfig`] 结构体来配置日志：
//!
//! ```rust,no_run
//! use toolkit::logger::{self, LoggerConfig};
//! use toolkit::logger::LoggerError;
//! use std::path::PathBuf;
//!
//! # fn main() -> std::result::Result<(), LoggerError> {
//! let config = LoggerConfig::new(
//!     Some("info".to_string()),  // 日志级别
//!     Some("text".to_string()),   // 日志格式
//!     true,                        // 启用控制台输出
//!     PathBuf::from("/tmp/logs"),  // 日志目录
//! );
//! logger::init(Some("my-app"), &config)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### 记录日志
//!
//! ```rust
//! use toolkit::{log_debug, log_info, log_warn, log_error};
//!
//! log_info!("Operation completed");
//! let error = "some error";
//! log_error!("Operation failed: {}", error);
//! ```

pub(crate) mod cleanup;
pub(crate) mod config;
pub(crate) mod error;
pub(crate) mod path;
pub(crate) mod subscriber;

// 重新导出主要类型和函数
pub use config::LoggerConfig;
pub use error::LoggerError;
pub use subscriber::init;

/// 从模块路径中提取模块名
///
/// 提取逻辑：
/// - 对于 `workflow::xxx::yyy::zzz` 格式，提取 `yyy`（索引2）
/// - 对于 `toolkit::xxx::yyy::zzz` 格式，提取 `xxx`（索引1）
/// - 对于其他格式，提取最后一个部分
/// - 如果路径为空，返回 "unknown"
#[doc(hidden)]
#[macro_export]
macro_rules! __extract_module_name {
    () => {{
        let path = module_path!();
        let parts: Vec<&str> = path.split("::").collect();
        match parts.as_slice() {
            ["workflow", _, module, ..] => (*module).to_string(),
            ["toolkit", module, ..] => (*module).to_string(),
            [] => "unknown".to_string(),
            _ => parts.last().copied().unwrap_or("unknown").to_string(),
        }
    }};
}

/// 格式化并记录调试级别的日志
///
/// 用于 lib 层的内部调试信息记录，不直接输出到控制台。
/// 默认情况下不输出，可以通过环境变量 `RUST_LOG=debug` 启用。
/// 自动包含模块信息作为日志字段（`module={module_name}`）。
///
/// # Examples
///
/// ```
/// use toolkit::log_debug;
///
/// log_debug!("Processing data");
/// let count = 5;
/// log_debug!("Found {} attachment(s)", count);
/// ```
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        {
            let module = $crate::__extract_module_name!();
            $crate::__tracing::debug!(module = %module, $($arg)*);
        }
    };
}

/// 格式化并记录信息级别的日志
///
/// 用于 lib 层的内部操作记录，不直接输出到控制台。
/// 默认情况下不输出，可以通过环境变量 `RUST_LOG=info` 启用。
/// 自动包含模块信息作为日志字段（`module={module_name}`）。
///
/// # Examples
///
/// ```
/// use toolkit::log_info;
///
/// log_info!("Starting download");
/// let count = 10;
/// log_info!("Downloaded {} files", count);
/// ```
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        {
            let module = $crate::__extract_module_name!();
            $crate::__tracing::info!(module = %module, $($arg)*);
        }
    };
}

/// 格式化并记录警告级别的日志
///
/// 用于 lib 层的内部警告记录，不直接输出到控制台。
/// 默认情况下不输出，可以通过环境变量 `RUST_LOG=warn` 启用。
/// 自动包含模块信息作为日志字段（`module={module_name}`）。
///
/// # Examples
///
/// ```
/// use toolkit::log_warn;
///
/// log_warn!("Retrying operation");
/// let filename = "file.txt";
/// let error = "network error";
/// log_warn!("Failed to download {}: {}", filename, error);
/// ```
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        {
            let module = $crate::__extract_module_name!();
            $crate::__tracing::warn!(module = %module, $($arg)*);
        }
    };
}

/// 格式化并记录错误级别的日志
///
/// 用于 lib 层的内部错误记录，不直接输出到控制台。
/// 默认情况下不输出，可以通过环境变量 `RUST_LOG=error` 启用。
/// 自动包含模块信息作为日志字段（`module={module_name}`）。
///
/// # Examples
///
/// ```
/// use toolkit::log_error;
///
/// log_error!("Operation failed");
/// let code = 500;
/// let message = "Internal Server Error";
/// log_error!("Error: {} - {}", code, message);
/// ```
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        {
            let module = $crate::__extract_module_name!();
            $crate::__tracing::error!(module = %module, $($arg)*);
        }
    };
}

/// 记录带结构化字段的调试级别日志
///
/// 支持添加额外的结构化字段到日志中。
/// 自动包含模块信息作为日志字段。
///
/// # Examples
///
/// ```
/// use toolkit::log_debug_with_fields;
///
/// log_debug_with_fields!(
///     user_id = 123,
///     request_id = "abc-123";
///     "Processing request"
/// );
/// ```
#[macro_export]
macro_rules! log_debug_with_fields {
    ($($key:ident = $value:tt),+ $(,)?; $($arg:tt)*) => {
        {
            let module = $crate::__extract_module_name!();
            $crate::__tracing::debug!(module = %module, $($key = $value,)* $($arg)*);
        }
    };
    ($($arg:tt)*) => {
        {
            let module = $crate::__extract_module_name!();
            $crate::__tracing::debug!(module = %module, $($arg)*);
        }
    };
}

/// 记录带结构化字段的信息级别日志
///
/// 支持添加额外的结构化字段到日志中。
/// 自动包含模块信息作为日志字段。
///
/// # Examples
///
/// ```
/// use toolkit::log_info_with_fields;
///
/// log_info_with_fields!(
///     user_id = 123,
///     request_id = "abc-123";
///     "Operation completed"
/// );
/// ```
#[macro_export]
macro_rules! log_info_with_fields {
    ($($key:ident = $value:tt),+ $(,)?; $($arg:tt)*) => {
        {
            let module = $crate::__extract_module_name!();
            $crate::__tracing::info!(module = %module, $($key = $value,)* $($arg)*);
        }
    };
    ($($arg:tt)*) => {
        {
            let module = $crate::__extract_module_name!();
            $crate::__tracing::info!(module = %module, $($arg)*);
        }
    };
}

/// 记录带结构化字段的警告级别日志
///
/// 支持添加额外的结构化字段到日志中。
/// 自动包含模块信息作为日志字段。
///
/// # Examples
///
/// ```
/// use toolkit::log_warn_with_fields;
///
/// log_warn_with_fields!(
///     retry_count = 3,
///     max_retries = 5;
///     "Retrying operation"
/// );
/// ```
#[macro_export]
macro_rules! log_warn_with_fields {
    ($($key:ident = $value:tt),+ $(,)?; $($arg:tt)*) => {
        {
            let module = $crate::__extract_module_name!();
            $crate::__tracing::warn!(module = %module, $($key = $value,)* $($arg)*);
        }
    };
    ($($arg:tt)*) => {
        {
            let module = $crate::__extract_module_name!();
            $crate::__tracing::warn!(module = %module, $($arg)*);
        }
    };
}

/// 记录带结构化字段和错误的错误级别日志
///
/// 支持添加额外的结构化字段和错误信息到日志中。
/// 自动包含模块信息作为日志字段。
///
/// # Examples
///
/// ```
/// use toolkit::log_error_with_fields;
///
/// let error_msg = "File not found";
/// log_error_with_fields!(
///     file_path = "/path/to/file",
///     error = error_msg;
///     "Failed to open file"
/// );
/// ```
#[macro_export]
macro_rules! log_error_with_fields {
    ($($key:ident = $value:tt),+ $(,)?; $($arg:tt)*) => {
        {
            let module = $crate::__extract_module_name!();
            $crate::__tracing::error!(module = %module, $($key = $value,)* $($arg)*);
        }
    };
    ($($arg:tt)*) => {
        {
            let module = $crate::__extract_module_name!();
            $crate::__tracing::error!(module = %module, $($arg)*);
        }
    };
}
