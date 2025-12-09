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
    /// Manage branch prefix for current repository
    ///
    /// Set, get, or remove branch prefix for the current repository.
    Prefix {
        #[command(subcommand)]
        subcommand: PrefixSubcommand,
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

/// Branch prefix management subcommands
#[derive(Subcommand)]
pub enum PrefixSubcommand {
    /// Set branch prefix for current repository
    ///
    /// If prefix is not provided, will prompt interactively.
    Set {
        /// Branch prefix value, if not provided, will prompt interactively
        prefix: Option<String>,
    },
    /// Get branch prefix for current repository
    Get,
    /// Remove branch prefix for current repository
    Remove,
}
