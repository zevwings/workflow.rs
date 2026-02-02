//! CNB 账号管理命令

pub mod check;
pub mod setup;

// 重新导出常用类型
pub use check::CNBCheckCommand;
pub use setup::CNBSetupCommand;
