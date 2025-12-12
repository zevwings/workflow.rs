//! Template system module
//!
//! Provides template rendering functionality for:
//! - Branch naming templates
//! - PR body templates
//! - Commit message templates

pub mod config;
pub mod engine;
pub mod vars;

pub use config::{CommitTemplates, PullRequestsTemplates, TemplateConfig};
pub use engine::{TemplateEngine, TemplateEngineType};
pub use vars::{BranchTemplateVars, ChangeTypeItem, CommitTemplateVars, PullRequestTemplateVars};
