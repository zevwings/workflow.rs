//! Alias management subcommands
//!
//! Used to manage command aliases.

use clap::Subcommand;

/// Alias management subcommands
///
/// Used to list, add, and remove command aliases.
#[derive(Subcommand)]
pub enum AliasSubcommand {
    /// List all aliases
    ///
    /// Display all defined aliases in a table format.
    List,
    /// Add a new alias
    ///
    /// Add a new alias mapping a short name to a full command.
    /// If name and command are not provided, enter interactive mode.
    ///
    /// Examples:
    ///   workflow alias add ci "pr create"    # Direct mode
    ///   workflow alias add                    # Interactive mode
    Add {
        /// Alias name (optional, will enter interactive mode if not provided)
        name: Option<String>,
        /// Command to map to (optional, will enter interactive mode if not provided)
        command: Option<String>,
    },
    /// Remove an alias
    ///
    /// Remove one or more aliases.
    /// If name is not provided, enter interactive selection mode.
    ///
    /// Examples:
    ///   workflow alias remove ci              # Direct mode
    ///   workflow alias remove                 # Interactive mode
    Remove {
        /// Alias name to remove (optional, will enter interactive mode if not provided)
        name: Option<String>,
    },
}
