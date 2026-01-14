//! Tag management subcommands
//!
//! Subcommands for managing Git tags.

use clap::Subcommand;

use super::args::{DryRunArgs, ForceArgs};

/// Tag management subcommands
///
/// Manage Git tags: delete local and remote tags.
#[derive(Subcommand)]
pub enum TagSubcommand {
    /// Delete one or more tags
    ///
    /// Delete local and/or remote tags.
    /// If tag name is not provided, will show an interactive list to select tags.
    ///
    /// Examples:
    ///   workflow tag delete v1.0.0                    # Delete specified tag (local and remote)
    ///   workflow tag delete v1.0.0 --local            # Delete only local tag
    ///   workflow tag delete v1.0.0 --remote          # Delete only remote tag
    ///   workflow tag delete --pattern "v1.*"          # Delete tags matching pattern
    ///   workflow tag delete                          # Interactive selection
    Delete {
        /// Tag name (optional, will enter interactive mode if not provided)
        tag_name: Option<String>,

        /// Delete only local tag
        #[arg(long)]
        local: bool,

        /// Delete only remote tag
        #[arg(long)]
        remote: bool,

        /// Pattern to match tags (e.g., "v1.*")
        #[arg(long)]
        pattern: Option<String>,

        /// Preview mode (do not actually delete)
        #[command(flatten)]
        dry_run: DryRunArgs,

        #[command(flatten)]
        force: ForceArgs,
    },
}
