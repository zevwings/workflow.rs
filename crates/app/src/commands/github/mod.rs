//! GitHub 账号管理命令

pub mod check;
mod cli;
pub mod setup;

// 重新导出 CLI 定义
// 重新导出命令实现
pub use check::GithubCheckCommand;
pub use cli::GithubCommand;
pub use setup::GithubSetupCommand;
