//! Branch management subcommands

use clap::Subcommand;

use super::common::{DryRunArgs, JiraIdArg};

/// Branch management subcommands
///
/// Used to clean branches and manage branch ignore list.
#[derive(Subcommand)]
pub enum BranchSubcommand {
    /// Clean local branches
    ///
    /// Delete all local branches except main/master, develop, current branch, and branches in ignore list.
    Clean {
        #[command(flatten)]
        dry_run: DryRunArgs,
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
    /// Create a new branch
    ///
    /// Create a new branch, optionally from a JIRA ticket.
    Create {
        #[command(flatten)]
        jira_id: JiraIdArg,
        /// Create from default branch (main/master)
        #[arg(long)]
        from_default: bool,
        #[command(flatten)]
        dry_run: DryRunArgs,
    },
    /// Rename a branch
    ///
    /// Fully interactive branch rename command.
    /// All operations are done through interactive prompts.
    ///
    /// Example:
    ///   workflow branch rename    # Interactive mode
    Rename,
    /// Switch to a branch
    ///
    /// Switch to a branch, with interactive selection if branch name is not provided.
    /// Fuzzy filter is automatically enabled when branch count > 25.
    /// If branch does not exist, will prompt user to confirm creation.
    ///
    /// Examples:
    ///   workflow branch switch feature/new-feature    # Switch to specified branch (prompt if not exists)
    ///   workflow branch switch                        # Interactive selection (auto fuzzy if > 25 branches)
    Switch {
        /// Branch name (optional, will enter interactive mode if not provided)
        branch_name: Option<String>,
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
