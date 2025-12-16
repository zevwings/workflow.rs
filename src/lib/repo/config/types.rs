//! Repository configuration types
//!
//! Common types used for repository configuration.

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Personal preference branch configuration
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BranchConfig {
    /// Branch prefix (personal preference)
    pub prefix: Option<String>,
    /// List of branches to ignore (personal preference)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore: Vec<String>,
}

/// Personal preference PR configuration
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PullRequestsConfig {
    /// Auto-accept change type selection (personal preference)
    pub auto_accept_change_type: Option<bool>,
}
