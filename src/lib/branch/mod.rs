//! Branch naming and management module
//!
//! This module provides branch naming functionality, including:
//! - Branch name generation from JIRA tickets, titles, and templates
//! - Branch name sanitization and validation
//!
//! **Note**: Branch configuration management has been migrated to `lib/repo/config.rs`.
//! Use `RepoConfig` and `ProjectBranchConfig` for configuration management.
//!
//! Branch naming now uses the template system, which handles prefixes automatically.
//!
//! # Usage
//!
//! ```rust
//! use workflow::branch::BranchNaming;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Generate branch name from JIRA ticket (uses template system)
//! let branch_name = BranchNaming::from_jira_ticket(
//!     "PROJ-123",
//!     "Add user authentication",
//!     Some("Feature"),
//!     true,
//! )?;
//!
//! // Or generate from branch type and slug (uses template system)
//! let branch_name = BranchNaming::from_type_and_slug(
//!     "feature",
//!     "add-user-auth",
//!     Some("PROJ-123"),
//! )?;
//! # Ok(())
//! # }
//! ```

pub mod llm;
pub mod naming;
pub mod sync;
pub mod types;

// Re-export structs and functions
pub use llm::BranchLLM;
pub use naming::BranchNaming;
pub use sync::{
    BranchSync, BranchSyncCallbacks, BranchSyncOptions, BranchSyncResult, SourceBranchInfo,
    SyncStrategy,
};
pub use types::BranchType;
