//! Template variables
//!
//! Defines template variable structures for different template types.

use serde::Serialize;
use serde_with::skip_serializing_none;

/// Template variables for branch naming
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Default)]
pub struct BranchTemplateVars {
    /// JIRA ticket key (e.g., "PROJ-123")
    pub jira_key: Option<String>,
    /// JIRA ticket summary
    pub jira_summary: Option<String>,
    /// JIRA ticket summary as slug (URL-friendly format)
    pub summary_slug: Option<String>,
    /// JIRA ticket type (e.g., "Feature", "Bug")
    pub jira_type: Option<String>,
    /// Branch prefix (e.g., "feature", "hotfix")
    pub prefix: Option<String>,
    /// Timestamp for branch creation
    pub timestamp: Option<String>,
}

/// Template variables for commit messages
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
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

/// Template variables for PR body
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
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
    /// PR title
    pub title: Option<String>,
    /// PR description
    pub description: Option<String>,
    /// Branch name
    pub branch_name: Option<String>,
    /// Author name
    pub author: Option<String>,
    /// List of changed files
    pub files_changed: Option<Vec<String>>,
    /// Number of commits
    pub commit_count: Option<usize>,
    /// Timestamp
    pub timestamp: Option<String>,
}

/// Change type item for PR template
#[derive(Debug, Clone, Serialize)]
pub struct ChangeTypeItem {
    /// Change type name
    pub name: String,
    /// Whether this change type is selected
    pub selected: bool,
}

impl Default for PullRequestTemplateVars {
    fn default() -> Self {
        Self {
            jira_key: None,
            jira_summary: None,
            jira_description: None,
            jira_type: None,
            jira_service_address: None,
            change_types: Vec::new(),
            short_description: None,
            dependency: None,
            title: None,
            description: None,
            branch_name: None,
            author: None,
            files_changed: None,
            commit_count: None,
            timestamp: None,
        }
    }
}
