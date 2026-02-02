//! 日志操作命令

pub mod check;
pub mod setup;

// 重新导出常用类型
pub use check::LogCheckCommand;
pub use setup::LogSetupCommand;
