//! Commit management subcommands

use clap::Subcommand;

/// Commit management subcommands
///
/// Used to manage Git commits, including amending the last commit.
#[derive(Subcommand)]
pub enum CommitSubcommand {
    /// Amend the last commit
    ///
    /// Modify the last commit, including message and files.
    ///
    /// Examples:
    ///   workflow commit amend                              # Interactive amend
    ///   workflow commit amend --message "New message"      # Modify message only
    ///   workflow commit amend --no-edit                    # Don't edit message
    Amend {
        /// New commit message
        #[arg(short, long)]
        message: Option<String>,
        /// Don't edit the commit message
        #[arg(long)]
        no_edit: bool,
        /// Skip pre-commit hooks
        #[arg(long)]
        no_verify: bool,
    },
    /// Reword a commit message
    ///
    /// Modify the message of a specific commit without changing its content.
    ///
    /// Examples:
    ///   workflow commit reword                              # Reword HEAD (default)
    ///   workflow commit reword HEAD                         # Reword HEAD explicitly
    ///   workflow commit reword HEAD~2                       # Reword the second-to-last commit
    ///   workflow commit reword abc1234                      # Reword a specific commit by SHA
    Reword {
        /// Commit reference (HEAD, HEAD~n, SHA, etc.)
        /// If not provided, defaults to HEAD
        #[arg(value_name = "COMMIT_ID")]
        commit_id: Option<String>,
    },
    /// Squash multiple commits
    ///
    /// Combine multiple commits into one, simplifying commit history.
    /// Only commits created after the current branch was created can be squashed.
    ///
    /// Examples:
    ///   workflow commit squash                              # Interactive squash (select commits)
    Squash,
}
