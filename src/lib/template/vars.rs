//! Template variables
//!
//! Defines template variable structures for different template types.

use serde::Serialize;

/// Template variables for branch naming
#[derive(Debug, Clone, Serialize)]
pub struct BranchTemplateVars {
    /// JIRA ticket key (e.g., "PROJ-123")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jira_key: Option<String>,
    /// JIRA ticket summary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jira_summary: Option<String>,
    /// JIRA ticket summary as slug (URL-friendly format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_slug: Option<String>,
    /// JIRA ticket type (e.g., "Feature", "Bug")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jira_type: Option<String>,
}

/// Template variables for commit messages
#[derive(Debug, Clone, Serialize)]
pub struct CommitTemplateVars {
    /// Commit type (e.g., "feat", "fix", "docs")
    pub commit_type: String,
    /// Commit scope (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Commit subject
    pub subject: String,
    /// Commit body (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// JIRA ticket key (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jira_key: Option<String>,
    /// Whether to use scope (when no ticket id)
    ///
    /// This value comes from configuration and is passed to the template
    pub use_scope: bool,
}

/// Template variables for PR body
#[derive(Debug, Clone, Serialize)]
pub struct PullRequestTemplateVars {
    /// JIRA ticket key (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jira_key: Option<String>,
    /// JIRA ticket summary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jira_summary: Option<String>,
    /// JIRA ticket description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jira_description: Option<String>,
    /// JIRA ticket type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jira_type: Option<String>,
    /// JIRA service address (for building links)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jira_service_address: Option<String>,
    /// Change types (array of booleans indicating which types are selected)
    pub change_types: Vec<ChangeTypeItem>,
    /// Short description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_description: Option<String>,
    /// Dependency information (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency: Option<String>,
}

/// Change type item for PR template
#[derive(Debug, Clone, Serialize)]
pub struct ChangeTypeItem {
    /// Change type name
    pub name: String,
    /// Whether this change type is selected
    pub selected: bool,
}
