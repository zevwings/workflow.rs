//! Branch type definitions
//!
//! Defines branch types and provides selection functionality.

use crate::base::dialog::SelectDialog;
use crate::log_info;
use crate::repo::config::RepoConfig;
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use std::fmt;

/// Branch type enumeration
///
/// Represents different types of branches in the workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchType {
    /// Feature branch - for new features
    Feature,
    /// Bugfix branch - for bug fixes
    Bugfix,
    /// Refactoring branch - for code refactoring
    Refactoring,
    /// Hotfix branch - for urgent production fixes
    Hotfix,
    /// Chore branch - for maintenance tasks
    Chore,
}

impl BranchType {
    /// Get all available branch types
    pub fn all() -> Vec<BranchType> {
        vec![
            BranchType::Feature,
            BranchType::Bugfix,
            BranchType::Refactoring,
            BranchType::Hotfix,
            BranchType::Chore,
        ]
    }

    /// Get branch type as string (for template selection)
    pub fn as_str(&self) -> &'static str {
        match self {
            BranchType::Feature => "feature",
            BranchType::Bugfix => "bugfix",
            BranchType::Refactoring => "refactoring",
            BranchType::Hotfix => "hotfix",
            BranchType::Chore => "chore",
        }
    }

    /// Get Conventional Commits commit type from branch type
    ///
    /// Maps branch type to Conventional Commits commit type:
    /// - Feature → "feat"
    /// - Bugfix → "fix"
    /// - Refactoring → "refactor"
    /// - Hotfix → "fix" (hotfix is a type of bug fix)
    /// - Chore → "chore"
    pub fn to_commit_type(&self) -> &'static str {
        match self {
            BranchType::Feature => "feat",
            BranchType::Bugfix => "fix",
            BranchType::Refactoring => "refactor",
            BranchType::Hotfix => "fix",
            BranchType::Chore => "chore",
        }
    }

    /// Get display name with description
    pub fn display_name(&self) -> &'static str {
        match self {
            BranchType::Feature => "feature - 新功能开发",
            BranchType::Bugfix => "bugfix - Bug 修复",
            BranchType::Refactoring => "refactoring - 代码重构",
            BranchType::Hotfix => "hotfix - 紧急修复",
            BranchType::Chore => "chore - 杂项任务",
        }
    }

    /// Parse branch type from string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "feature" => Some(BranchType::Feature),
            "bugfix" | "bug" | "fix" => Some(BranchType::Bugfix),
            "refactoring" | "refactor" => Some(BranchType::Refactoring),
            "hotfix" => Some(BranchType::Hotfix),
            "chore" => Some(BranchType::Chore),
            _ => None,
        }
    }

    /// Prompt user to select branch type interactively
    pub fn prompt_selection() -> Result<Self> {
        let options: Vec<BranchType> = Self::all();
        let display_options: Vec<String> =
            options.iter().map(|ty| ty.display_name().to_string()).collect();

        let selected = SelectDialog::new("选择分支类型 (Select branch type)", display_options)
            .with_default(0) // Default to feature
            .prompt()
            .wrap_err("Failed to select branch type")?;

        // Find the corresponding BranchType
        options
            .into_iter()
            .find(|ty| ty.display_name() == selected)
            .ok_or_else(|| eyre!("Invalid branch type selection"))
    }

    /// Resolve branch type with repository prefix fallback
    ///
    /// Priority:
    /// 1. If repository prefix exists and can be converted to BranchType, use it
    /// 2. Otherwise, prompt user to select interactively
    ///
    /// # Returns
    ///
    /// Returns the resolved branch type.
    ///
    /// # Errors
    ///
    /// Returns an error if the user selection fails or if the repository prefix cannot be converted to a branch type.
    pub fn resolve_with_repo_prefix() -> Result<Self> {
        // Check if repository prefix exists and use it as branch type
        if let Some(repo_prefix) = RepoConfig::get_branch_prefix() {
            if let Some(ty) = Self::from_str(&repo_prefix) {
                log_info!("Using repository prefix '{}' as branch type", repo_prefix);
                return Ok(ty);
            }
        }

        // Otherwise, prompt user to select
        Self::prompt_selection()
    }
}

impl fmt::Display for BranchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_type_basic() {
        // Basic validation of branch types
        assert_eq!(BranchType::all().len(), 5);
        assert_eq!(BranchType::Feature.as_str(), "feature");
        assert_eq!(BranchType::from_str("feature"), Some(BranchType::Feature));
        assert_eq!(format!("{}", BranchType::Feature), "feature");
    }

    #[test]
    fn test_branch_type_to_commit_type() {
        assert_eq!(BranchType::Feature.to_commit_type(), "feat");
        assert_eq!(BranchType::Bugfix.to_commit_type(), "fix");
        assert_eq!(BranchType::Refactoring.to_commit_type(), "refactor");
        assert_eq!(BranchType::Hotfix.to_commit_type(), "fix");
        assert_eq!(BranchType::Chore.to_commit_type(), "chore");
    }
}
