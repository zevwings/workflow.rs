//! Settings 模块
//! 用于管理应用程序的各种设置和配置

pub mod paths;
#[allow(clippy::module_inception)]
pub mod settings;
pub mod table;

// 导出公共类型和函数
pub use paths::Paths;
pub use settings::{LLMSettings, Settings};
pub use table::{GitHubAccountListRow, GitHubAccountRow, JiraConfigRow, LLMConfigRow};
