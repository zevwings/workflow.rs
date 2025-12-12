//! Stash management subcommands

use clap::Subcommand;

/// Stash management subcommands
///
/// Manage Git stash entries: list, apply, drop, and pop.
#[derive(Subcommand)]
pub enum StashSubcommand {
    /// List all stash entries
    ///
    /// Display all stash entries in a table format, showing index, message, branch, and timestamp.
    /// Use --stat to show file change statistics.
    List {
        /// Show file change statistics for each stash
        #[arg(long)]
        stat: bool,
    },
    /// Apply a stash (keep the stash entry)
    ///
    /// Apply a stash without removing it.
    /// Will prompt to apply the latest stash or select from a list.
    ///
    /// Examples:
    ///   workflow stash apply                    # Prompt to apply latest or select
    Apply,
    /// Drop (delete) stash entries
    ///
    /// Delete one or more stash entries.
    /// Will show a multi-select list to choose which stashes to delete.
    ///
    /// Examples:
    ///   workflow stash drop                     # Multi-select and delete stashes
    Drop,
    /// Pop a stash (apply and delete)
    ///
    /// Apply a stash and remove it.
    /// Will prompt to pop the latest stash or select from a list.
    /// If application fails due to conflicts, the stash entry is kept.
    ///
    /// Examples:
    ///   workflow stash pop                      # Prompt to pop latest or select
    Pop,
    /// Push (save) current changes to stash
    ///
    /// Save current working directory and staged changes to stash.
    /// Will prompt for an optional message to identify the stash entry.
    ///
    /// Examples:
    ///   workflow stash push                     # Prompt for message and stash changes
    Push,
}
