//! Jira 配置管理命令

#[cfg(feature = "develop")]
pub mod assign;
pub mod attachments;
pub mod check;
pub mod clean;
mod cli;
#[cfg(feature = "develop")]
pub mod comment;
pub mod info;
pub mod setup;
#[cfg(feature = "develop")]
pub mod status;
#[cfg(feature = "develop")]
pub mod transition;
pub(crate) mod utils;

// 重新导出 CLI 定义
// 重新导出命令实现
#[cfg(feature = "develop")]
pub use assign::JiraAssignCommand;
pub use attachments::JiraAttachmentsCommand;
pub use check::JiraCheckCommand;
pub use clean::JiraCleanCommand;
pub use cli::{AttachmentsArgs, CleanArgs, InfoArgs, JiraCommand, OutputFormat};
#[cfg(feature = "develop")]
pub use comment::JiraCommentCommand;
pub use info::JiraInfoCommand;
pub use setup::JiraSetupCommand;
#[cfg(feature = "develop")]
pub use status::JiraStatusCommand;
#[cfg(feature = "develop")]
pub use transition::JiraTransitionCommand;
// 重新导出工具函数（供跨模块使用）
pub use utils::{
    ensure_jira_status_config, get_jira_id_interactive, get_jira_id_interactive_optional,
};
