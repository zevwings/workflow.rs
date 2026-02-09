//! Jira 配置管理命令

#[cfg(feature = "develop")]
pub mod assign;
pub mod attachments;
pub mod check;
pub mod clean;
#[cfg(feature = "develop")]
pub mod comment;
pub mod info;
pub mod setup;
#[cfg(feature = "develop")]
pub mod status;
#[cfg(feature = "develop")]
pub mod transition;

// 重新导出常用类型
#[cfg(feature = "develop")]
pub use assign::JiraAssignCommand;
pub use attachments::JiraAttachmentsCommand;
pub use check::JiraCheckCommand;
pub use clean::JiraCleanCommand;
#[cfg(feature = "develop")]
pub use comment::JiraCommentCommand;
pub use info::JiraInfoCommand;
pub use setup::JiraSetupCommand;
#[cfg(feature = "develop")]
pub use status::JiraStatusCommand;
#[cfg(feature = "develop")]
pub use transition::JiraTransitionCommand;
