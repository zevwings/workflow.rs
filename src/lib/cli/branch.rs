//! Branch management subcommands

use clap::Subcommand;

use super::args::{DryRunArgs, ForceArgs, JiraIdArg};

/// Branch management subcommands
///
/// Used to manage branches and branch ignore list.
#[derive(Subcommand)]
pub enum BranchSubcommand {
    /// Manage branch ignore list
    ///
    /// Add, remove, or list branches in the ignore list.
    Ignore {
        #[command(subcommand)]
        subcommand: IgnoreSubcommand,
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
    /// Sync branch into current branch
    ///
    /// Sync specified branch into current branch, supporting merge, rebase, or squash.
    /// This is a local Git operation without PR-specific logic.
    /// Will prompt for confirmation before pushing to remote.
    ///
    /// Examples:
    ///   workflow branch sync master                    # Merge master into current branch
    ///   workflow branch sync master --rebase          # Rebase current branch onto master
    ///   workflow branch sync feature-branch --squash  # Squash merge feature-branch
    Sync {
        /// Source branch name to sync (required)
        #[arg(value_name = "SOURCE_BRANCH")]
        source_branch: String,

        /// Use rebase instead of merge (default: merge)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        rebase: bool,

        /// Only allow fast-forward merge (fail if not possible)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        ff_only: bool,

        /// Use squash merge (compress all commits into one)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        squash: bool,
    },
    /// Delete a branch
    ///
    /// Delete local and/or remote branch.
    /// If branch name is not provided, will show an interactive list to select branch.
    ///
    /// Examples:
    ///   workflow branch delete feature/old-feature     # Delete specified branch (local and remote)
    ///   workflow branch delete feature/old-feature --local    # Delete only local branch
    ///   workflow branch delete feature/old-feature --remote   # Delete only remote branch
    ///   workflow branch delete                          # Interactive selection
    Delete {
        /// Branch name (optional, will enter interactive mode if not provided)
        branch_name: Option<String>,

        /// Delete only local branch
        #[arg(long)]
        local: bool,

        /// Delete only remote branch
        #[arg(long)]
        remote: bool,

        /// Preview mode (do not actually delete)
        #[command(flatten)]
        dry_run: DryRunArgs,

        #[command(flatten)]
        force: ForceArgs,
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
