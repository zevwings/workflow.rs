//! 分支命名模板变量
//!
//! 描述分支的领域属性，用于生成分支名。

use serde::{Deserialize, Serialize};

/// 分支命名模板变量
///
/// 描述分支的领域属性，用于生成分支名。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchTemplateVars {
    /// User prefix (e.g., "zw")
    pub prefix: Option<String>,
    /// JIRA ticket key (e.g., "PROJ-123")
    pub jira_key: Option<String>,
    /// JIRA ticket summary
    pub jira_summary: Option<String>,
    /// JIRA ticket summary as slug (URL-friendly format)
    pub summary_slug: Option<String>,
    /// JIRA ticket type (e.g., "Feature", "Bug")
    pub jira_type: Option<String>,
}
