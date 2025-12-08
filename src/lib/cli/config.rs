//! Configuration management subcommands

use clap::Subcommand;

/// Log level management subcommands
///
/// Used to manage log output level.
#[derive(Subcommand)]
pub enum LogLevelSubcommand {
    /// Set log level (interactive selection)
    ///
    /// Select log level through interactive menu: none, error, warn, info, debug.
    Set,
    /// Check current log level
    ///
    /// Display current configured log level and default level information.
    Check,
    /// Enable/disable tracing console output
    ///
    /// Control whether tracing logs are also output to console (stderr) in addition to file.
    /// If enabled, tracing logs will be output to both file and console.
    /// If not set, defaults to true in debug mode, false in release mode.
    TraceConsole,
}

/// Completion management subcommands
///
/// Used to generate and manage shell completion scripts.
#[derive(Subcommand)]
pub enum CompletionSubcommand {
    /// Generate completion scripts
    ///
    /// Auto-detect current shell type, generate corresponding completion scripts and apply to configuration files.
    Generate,
    /// Check completion status
    ///
    /// Check installed shell types and shells with configured completion.
    Check,
    /// Remove completion configuration
    ///
    /// Interactively select and remove configured shell completion configuration.
    Remove,
}
