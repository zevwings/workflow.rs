//! Template system module
//!
//! Provides template rendering functionality for:
//! - Branch naming templates
//! - PR body templates
//! - Commit message templates

pub mod engine;
pub mod loader;
pub mod vars;

pub use engine::{TemplateEngine, TemplateEngineType};
pub use loader::{
    default_commit_template, default_pull_request_template, load_branch_template,
    load_branch_template_by_type, load_commit_template, load_pull_request_template,
};
pub use vars::{BranchTemplateVars, ChangeTypeItem, CommitTemplateVars, PullRequestTemplateVars};
