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
}
