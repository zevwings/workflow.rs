//! Configuration management subcommands

use clap::Subcommand;

use super::args::DryRunArgs;

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

/// Configuration management subcommands
///
/// Used to manage configuration files (validate, export, import).
#[derive(Subcommand)]
pub enum ConfigSubcommand {
    /// View current configuration
    ///
    /// Display all configured environment variables and settings (sensitive information will be masked).
    Show,
    /// Validate configuration file
    ///
    /// Verify the integrity and validity of the configuration file.
    /// Supports TOML, JSON, and YAML formats.
    Validate {
        /// Configuration file path (optional, defaults to workflow.toml)
        #[arg(value_name = "CONFIG_PATH")]
        config_path: Option<String>,
        /// Automatically fix configuration errors
        #[arg(long)]
        fix: bool,
        /// Strict mode (treat all warnings as errors)
        #[arg(long)]
        strict: bool,
    },
    /// Export configuration file
    ///
    /// Export configuration to a file for backup or migration.
    Export {
        /// Output file path
        #[arg(value_name = "OUTPUT_PATH")]
        output_path: String,
        /// Only export specific section (e.g., jira, pr)
        #[arg(long)]
        section: Option<String>,
        /// Exclude sensitive information
        #[arg(long)]
        no_secrets: bool,
        /// Export as TOML format (default)
        #[arg(long)]
        toml: bool,
        /// Export as JSON format
        #[arg(long)]
        json: bool,
        /// Export as YAML format
        #[arg(long)]
        yaml: bool,
    },
    /// Import configuration file
    ///
    /// Import configuration from a file (merge or overwrite mode).
    Import {
        /// Input file path
        #[arg(value_name = "INPUT_PATH")]
        input_path: String,
        /// Overwrite mode (completely replace existing configuration)
        #[arg(long)]
        overwrite: bool,
        /// Only import specific section (e.g., jira, pr)
        #[arg(long)]
        section: Option<String>,
        #[command(flatten)]
        dry_run: DryRunArgs,
    },
}
