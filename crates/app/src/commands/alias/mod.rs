//! 别名管理命令模块
//!
//! 提供别名的添加、列表和移除功能。

mod add;
mod list;
mod remove;

pub use add::AliasAddCommand;
pub use list::AliasListCommand;
pub use remove::AliasRemoveCommand;
