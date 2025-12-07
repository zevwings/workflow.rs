//! Branch management subcommands

use clap::Subcommand;

/// Branch management subcommands
///
/// Used to clean branches and manage branch ignore list.
#[derive(Subcommand)]
pub enum BranchSubcommand {
    /// Clean local branches
    ///
    /// Delete all local branches except main/master, develop, current branch, and branches in ignore list.
    Clean {
        /// Dry run mode (show what would be deleted without actually deleting)
        #[arg(long, short = 'n')]
        dry_run: bool,
    },
    /// Manage branch ignore list
    ///
    /// Add, remove, or list branches in the ignore list.
    Ignore {
        #[command(subcommand)]
        subcommand: IgnoreSubcommand,
    },
}

/// Branch ignore list management subcommands
#[derive(Subcommand)]
pub enum IgnoreSubcommand {
    /// Add branch to ignore list
    Add {
        /// Branch name to add
        branch_name: Option<String>,
    },
    /// Remove branch from ignore list
    Remove {
        /// Branch name to remove
        branch_name: Option<String>,
    },
    /// List ignored branches for current repository
    List,
}
