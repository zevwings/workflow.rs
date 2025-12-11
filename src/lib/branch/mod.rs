//! Branch naming and management module
//!
//! This module provides branch naming functionality, including:
//! - Branch name generation from JIRA tickets, titles, and templates
//! - Branch name sanitization and validation
//! - Branch prefix management
//! - Branch configuration management
//!
//! # Usage
//!
//! ```rust
//! use workflow::branch::{BranchNaming, BranchPrefix};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Generate branch name from JIRA ticket
//! let branch_name = BranchNaming::from_jira_ticket(
//!     "PROJ-123",
//!     "Add user authentication",
//!     Some("Feature"),
//!     true,
//! )?;
//!
//! // Apply prefixes
//! let final_name = BranchPrefix::apply(branch_name, Some("PROJ-123"), false)?;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod llm;
pub mod naming;
pub mod prefix;
pub mod types;

// Re-export structs and functions
pub use config::{BranchConfig, RepositoryConfig};
pub use llm::BranchLLM;
pub use naming::BranchNaming;
pub use prefix::BranchPrefix;
pub use types::BranchType;
