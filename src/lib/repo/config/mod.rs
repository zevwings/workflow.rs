//! Repository configuration management
//!
//! Provides reusable functions for checking repository configuration.
//! This is a library function that can be called by commands.
//!
//! ## Module Structure
//!
//! - `types` - Common configuration types (BranchConfig, PullRequestsConfig)
//! - `public` - Public repository configuration (project templates, committed to Git)
//! - `private` - Private repository configuration (personal preferences, not committed to Git)
//! - `repo_config` - Unified configuration interface

pub mod private;
pub mod public;
pub mod repo_config;
pub mod types;

// Re-export all public APIs
pub use repo_config::RepoConfig;
pub use types::{BranchConfig, PullRequestsConfig};
