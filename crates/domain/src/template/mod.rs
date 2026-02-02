//! 模板变量实体（值对象）
//!
//! 描述业务领域的模板变量，用于分支命名、提交消息、PR 内容生成。

pub mod branch;
pub mod commit;
pub mod pull_request;

// Re-export public types
pub use branch::BranchTemplateVars;
pub use commit::CommitTemplateVars;
pub use pull_request::{ChangeTypeItem, PullRequestTemplateVars};
