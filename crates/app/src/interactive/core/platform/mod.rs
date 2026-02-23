//! 通用平台逻辑模块
//!
//! 提供用于管理平台账号（GitHub 等）的 trait 和通用函数。

mod account;
mod config;
mod traits;
mod types;

pub use account::add_account_generic;
pub use config::configure_platform;
pub use traits::{GlobalConfigAccessor, PlatformAccount, PlatformConfigurator, PlatformSettings};
pub use types::AccountSetMode;
