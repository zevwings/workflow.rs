//! Jira Repository 内部服务层
//!
//! 提供统一的业务协调服务，包括：
//! - Issue 数据获取服务
//! - 附件下载服务
//! - 状态管理服务
//! - 配置上下文

mod issue;
pub mod status;
mod user;

pub use issue::{IssueService, IssueServiceImpl};
pub use status::{StatusService, StatusServiceImpl};
pub use user::{UserService, UserServiceImpl};
