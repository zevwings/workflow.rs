//! 提交消息模板变量
//!
//! 描述提交的领域属性，用于生成提交消息。

use serde::{Deserialize, Serialize};

/// 提交消息模板变量
///
/// 描述提交的领域属性，用于生成提交消息。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitTemplateVars {
    /// Commit type (e.g., "feat", "fix", "docs")
    pub commit_type: String,
    /// Commit scope (optional)
    pub scope: Option<String>,
    /// Commit subject
    pub subject: String,
    /// Commit body (optional)
    pub body: Option<String>,
    /// JIRA ticket key (optional)
    pub jira_key: Option<String>,
    /// Whether to use scope (when no ticket id)
    ///
    /// This value comes from configuration and is passed to the template
    pub use_scope: bool,
}
