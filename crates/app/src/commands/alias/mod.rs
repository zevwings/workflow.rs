//! 别名管理命令模块
//!
//! 提供别名的添加、列表和移除功能。

mod add;
mod cli;
mod list;
mod remove;

// 重新导出 CLI 定义
// 重新导出命令实现
pub use add::AliasAddCommand;
pub use cli::AliasCommand;
pub use list::AliasListCommand;
pub use remove::AliasRemoveCommand;
