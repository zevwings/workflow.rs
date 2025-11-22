//! 代理检测与管理模块
//!
//! 本模块提供了 macOS 系统代理的检测和管理功能。

mod config_generator;
mod manager;
#[allow(clippy::module_inception)]
mod proxy;
mod system_reader;

pub use config_generator::ProxyConfigGenerator;
pub use manager::ProxyManager;
pub use proxy::{ProxyConfig, ProxyDisableResult, ProxyEnableResult, ProxyInfo, ProxyType};
pub use system_reader::SystemProxyReader;
