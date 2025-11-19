//! 代理检测与管理模块
//!
//! 本模块提供了 macOS 系统代理的检测和管理功能。

mod env;
#[allow(clippy::module_inception)]
mod proxy;

pub use env::EnvFile;
pub use proxy::{Proxy, ProxyDisableResult, ProxyEnableResult, ProxyInfo};
