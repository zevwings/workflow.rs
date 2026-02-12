//! 内部日志宏
//!
//! 自动带 `target: "http"`，可通过 `RUST_LOG=http=debug` 启用。

#[doc(hidden)]
pub(crate) mod __tracing {
    pub use tracing::{debug, warn};
}

/// 内部调试日志，自动带 `target: "http"`，可通过 `RUST_LOG=http=debug` 启用。
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::logger::__tracing::debug!(target: "http", $($arg)*);
    };
}

/// 内部警告日志，自动带 `target: "http"`。
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::logger::__tracing::warn!(target: "http", $($arg)*);
    };
}
