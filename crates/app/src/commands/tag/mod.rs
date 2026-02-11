//! Tag 管理命令

mod cli;
pub mod create;
pub mod remove;

// 重新导出 CLI 定义
pub use cli::TagSubcommand;
// 重新导出命令实现
pub use create::TagCreateCommand;
pub use remove::TagRemoveCommand;
