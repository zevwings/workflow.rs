//! 别名管理命令
//!
//! 提供别名的列表、添加和删除功能。

pub mod add;
pub mod list;
pub mod remove;

pub use add::AliasAddCommand;
pub use list::AliasListCommand;
pub use remove::AliasRemoveCommand;
