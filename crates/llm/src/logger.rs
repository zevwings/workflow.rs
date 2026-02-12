//! 内部日志宏
//!
//! 自动带 `target: "llm"`，可通过 `RUST_LOG=llm=debug` 启用。

#[doc(hidden)]
#[allow(unused_imports)] // warn 供 llm_warn! 宏展开时使用
pub(crate) mod __tracing {
    pub use tracing::{debug, warn};
}

/// 内部调试日志，自动带 `target: "llm"`，可通过 `RUST_LOG=llm=debug` 启用。
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::logger::__tracing::debug!(target: "llm", $($arg)*);
    };
}

/// 内部警告日志，自动带 `target: "llm"`。
#[macro_export]
macro_rules! llm_warn {
    ($($arg:tt)*) => {
        $crate::logger::__tracing::warn!(target: "llm", $($arg)*);
    };
}
