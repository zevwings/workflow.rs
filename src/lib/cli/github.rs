//! GitHub account management subcommands

use clap::Subcommand;

/// GitHub account management subcommands
///
/// Used to manage configurations for multiple GitHub accounts.
#[derive(Subcommand)]
pub enum GitHubSubcommand {
    /// List all GitHub accounts
    ///
    /// Display all configured GitHub account information.
    List,
    /// Show current active GitHub account
    ///
    /// Display currently active GitHub account information.
    Current,
    /// Add a new GitHub account
    ///
    /// Interactively add a new GitHub account configuration.
    Add,
    /// Remove a GitHub account
    ///
    /// Remove the specified GitHub account from configuration.
    Remove,
    /// Switch current GitHub account
    ///
    /// Switch between multiple GitHub accounts.
    Switch,
    /// Update GitHub account information
    ///
    /// Update existing GitHub account configuration.
    Update,
    /// Test and show Git authentication status
    ///
    /// Test Git remote authentication (SSH/HTTPS) using git2 and display authentication status.
    Show,
}
