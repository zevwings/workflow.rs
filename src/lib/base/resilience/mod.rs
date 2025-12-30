//! 操作可靠性工具模块
//!
//! 提供超时和重试机制，用于提高操作的可靠性。
//! 主要用于 release/update 命令中的文件下载、解压、文件系统操作等。
//!
//! ## 模块结构
//!
//! - `timeout` - 超时工具（防止操作卡住）
//! - `retry` - 重试工具（重试临时性错误）

pub mod retry;
pub mod timeout;

pub use retry::{execute_with_retry, execute_with_timeout_and_retry, RetryConfig, RetryResult};
pub use timeout::{
    default_download_timeout, default_extract_timeout, default_filesystem_timeout,
    default_read_timeout, default_script_timeout, execute_with_timeout, TimeoutConfig,
};
