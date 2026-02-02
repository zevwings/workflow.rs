//! 工具包（Toolkit）
//!
//! 通用工具、HTTP 客户端、日志、配置等
//! 可以被所有层级使用
//!
//! ## 使用方式
//!
//! 所有 API 都通过 `toolkit::{xx}` 直接访问：
//! ```rust
//! use toolkit::{HttpClient, TemplateEngine, Platform, log_info};
//! ```

pub mod http;
pub mod logger;
pub mod paths;
pub mod template;
pub mod util;

// Private module to re-export tracing for use in exported macros
// This allows macros to use $crate::__tracing::debug! etc.
#[doc(hidden)]
pub mod __tracing {
    pub use tracing::{debug, error, info, warn};
}

// ============================================================================
// 统一重新导出主要公共 API
// ============================================================================

// HTTP 客户端模块
pub use http::{
    Authorization, HttpClient, HttpClientConfig, HttpError, HttpMethod, HttpMethodError,
    HttpResponse, HttpRetry, HttpRetryConfig, HttpRetryError, IntoHeaderMap,
    MultipartRequestConfig, RequestConfig, RetryResult,
};

// Logger 模块
pub use logger::init as logger_init;
pub use logger::{LoggerConfig, LoggerError};

// Paths 模块
pub use paths::{PathError, Paths};

// Template 模块
pub use template::{TemplateEngine, TemplateEngineType, TemplateError};

// Util 模块 - 导出所有工具类型和函数
pub use util::{
    Browser, BrowserError, BrowserExt, ClipboardError, ClipboardExt, DirectoryWalker, FileReader,
    FileWriter, PathExt, Platform, PlatformError, Sensitive, SizeExt, Truncate,
};
