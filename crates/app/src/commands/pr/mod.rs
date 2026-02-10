//! Pull Request 操作命令

mod approve;
mod cli;
mod close;
mod comment;
mod create;
mod list;
mod merge;
mod reword;
mod update;
pub(crate) mod utils;

// 重新导出 CLI 定义
pub use cli::PrSubcommand;

// 重新导出命令实现
pub use approve::PullRequestApproveCommand;
pub use close::PullRequestCloseCommand;
pub use comment::PullRequestCommentCommand;
pub use create::PullRequestCreateCommand;
pub use list::PullRequestListCommand;
pub use merge::PullRequestMergeCommand;
pub use reword::PullRequestRewordCommand;
pub use update::PullRequestUpdateCommand;

// 重新导出工具函数（供跨模块使用）
pub use utils::{generate_pull_request_body, generate_pull_request_title};
