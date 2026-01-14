//! Logger 宏定义模块
//!
//! 本模块包含所有 `log_*!` 宏的定义，用于 lib 层的结构化日志记录。
//! 这些宏自动包含模块信息作为日志字段（`module={module_name}`）。

/// 从模块路径中提取模块名
///
/// 提取逻辑：
/// - 对于 `workflow::xxx::yyy::zzz` 格式，提取 `yyy`（索引2）
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
/// use workflow::log_debug;
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
            tracing::debug!(module = %module, $($arg)*);
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
/// use workflow::log_info;
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
            tracing::info!(module = %module, $($arg)*);
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
/// use workflow::log_warn;
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
            tracing::warn!(module = %module, $($arg)*);
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
/// use workflow::log_error;
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
            tracing::error!(module = %module, $($arg)*);
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
/// use workflow::log_debug_with_fields;
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
            tracing::debug!(module = %module, $($key = $value,)* $($arg)*);
        }
    };
    ($($arg:tt)*) => {
        {
            let module = $crate::__extract_module_name!();
            tracing::debug!(module = %module, $($arg)*);
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
/// use workflow::log_info_with_fields;
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
            tracing::info!(module = %module, $($key = $value,)* $($arg)*);
        }
    };
    ($($arg:tt)*) => {
        {
            let module = $crate::__extract_module_name!();
            tracing::info!(module = %module, $($arg)*);
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
/// use workflow::log_warn_with_fields;
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
            tracing::warn!(module = %module, $($key = $value,)* $($arg)*);
        }
    };
    ($($arg:tt)*) => {
        {
            let module = $crate::__extract_module_name!();
            tracing::warn!(module = %module, $($arg)*);
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
/// use workflow::log_error_with_fields;
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
            tracing::error!(module = %module, $($key = $value,)* $($arg)*);
        }
    };
    ($($arg:tt)*) => {
        {
            let module = $crate::__extract_module_name!();
            tracing::error!(module = %module, $($arg)*);
        }
    };
}
