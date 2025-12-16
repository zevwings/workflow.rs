//! Repository management subcommands
//!
//! Subcommands for managing repository-level configuration.

use clap::Subcommand;

use super::common::DryRunArgs;

/// Repository management subcommands
///
/// Used to initialize and manage repository-level configuration.
#[derive(Subcommand)]
pub enum RepoSubcommand {
    /// Initialize repository configuration
    ///
    /// Interactively set up repository configuration including:
    /// - Branch prefix
    /// - Commit template settings (use_scope)
    Setup,
    /// Show current repository configuration
    ///
    /// Display all repository-level configuration settings.
    Show,
    /// Clean local branches
    ///
    /// Delete all local branches except main/master, develop, current branch, and branches in ignore list.
    Clean {
        #[command(flatten)]
        dry_run: DryRunArgs,
    },
}
