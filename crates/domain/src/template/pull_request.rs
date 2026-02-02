//! Pull Request 模板变量
//!
//! 描述 PR 的领域属性，用于生成 PR 内容。

use serde::{Deserialize, Serialize};

/// Pull Request 模板变量
///
/// 描述 PR 的领域属性，用于生成 PR 内容。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PullRequestTemplateVars {
    /// JIRA ticket key (optional)
    pub jira_key: Option<String>,
    /// JIRA ticket summary
    pub jira_summary: Option<String>,
    /// JIRA ticket description
    pub jira_description: Option<String>,
    /// JIRA ticket type
    pub jira_type: Option<String>,
    /// JIRA service address (for building links)
    pub jira_service_address: Option<String>,
    /// Change types (array of booleans indicating which types are selected)
    pub change_types: Vec<ChangeTypeItem>,
    /// Short description (optional)
    pub short_description: Option<String>,
    /// Dependency information (optional)
    pub dependency: Option<String>,
}

/// 变更类型项
///
/// 用于 PR 模板中的变更类型选择。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeTypeItem {
    /// Change type name
    pub name: String,
    /// Whether this change type is selected
    pub selected: bool,
}
