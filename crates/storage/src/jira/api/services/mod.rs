//! Jira API 服务层
//!
//! 提供与 Jira REST API 交互的业务服务，包括：
//! - Issue 数据获取服务
//! - 状态管理服务
//! - 用户信息服务
//! - 附件下载服务

mod attachment;
mod issue;
pub mod status;
mod user;

pub use attachment::{AttachmentService, AttachmentServiceImpl};
pub use issue::{IssueService, IssueServiceImpl};
pub use status::{StatusService, StatusServiceImpl};
pub use user::{UserService, UserServiceImpl};
