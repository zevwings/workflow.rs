//! Main commands enumeration
//!
//! Defines all top-level commands for Workflow CLI.

use clap::Subcommand;

use super::{
    BranchSubcommand, CompletionSubcommand, ConfigSubcommand, GitHubSubcommand, JiraSubcommand,
    LLMSubcommand, LogLevelSubcommand, PRCommands, ProxySubcommand,
};

/// 主命令枚举
///
/// 定义了 Workflow CLI 支持的所有顶级命令。
#[derive(Subcommand)]
pub enum Commands {
    /// Manage proxy settings (on/off/check)
    ///
    /// Manage HTTP/HTTPS proxy configuration via environment variables.
    Proxy {
        #[command(subcommand)]
        subcommand: ProxySubcommand,
        /// Temporary mode: only enable in current shell, don't write to config file
        #[arg(short, long)]
        temporary: bool,
    },
    /// Run environment checks
    ///
    /// Check Git repository status and network connectivity (GitHub).
    Check,
    /// Initialize or update configuration
    ///
    /// Interactively set up various configuration items required by Workflow CLI (e.g., Jira, GitHub, etc.).
    Setup,
    /// Manage configuration
    ///
    /// View, validate, export, and import configuration files.
    Config {
        #[command(subcommand)]
        subcommand: Option<ConfigSubcommand>,
    },
    /// Uninstall Workflow CLI configuration
    ///
    /// Remove all related files: binaries, completion scripts, configuration files, etc.
    Uninstall,
    /// Show Workflow CLI version
    ///
    /// Display the current installed version of Workflow CLI.
    Version,
    /// Update Workflow CLI
    ///
    /// Rebuild release version and update all binaries and shell completion scripts.
    Update {
        /// Specify the version number to update to (e.g., 1.1.2)
        ///
        /// If not specified, will update to the latest version.
        #[arg(long, short = 'v')]
        version: Option<String>,
    },
    /// Manage log level (set/check)
    ///
    /// Set or view current log output level (none, error, warn, info, debug).
    #[command(name = "log-level")]
    LogLevel {
        #[command(subcommand)]
        subcommand: LogLevelSubcommand,
    },
    /// Manage GitHub accounts
    ///
    /// Manage configurations for multiple GitHub accounts (add, remove, switch, update, etc.).
    #[command(name = "github")]
    GitHub {
        #[command(subcommand)]
        subcommand: GitHubSubcommand,
    },
    /// Manage LLM configuration
    ///
    /// Configure LLM provider, API keys, models, and output language settings.
    #[command(name = "llm")]
    Llm {
        #[command(subcommand)]
        subcommand: LLMSubcommand,
    },
    /// Manage shell completion
    ///
    /// Generate and manage shell completion scripts.
    Completion {
        #[command(subcommand)]
        subcommand: CompletionSubcommand,
    },
    /// Manage Git branches
    ///
    /// Clean local branches and manage branch ignore list.
    Branch {
        #[command(subcommand)]
        subcommand: BranchSubcommand,
    },
    /// Migrate configuration to new format
    ///
    /// Execute versioned migrations to update configuration files.
    /// Automatically detects and migrates all pending versions.
    /// Old configuration files will be removed after successful migration.
    Migrate {
        /// Dry run mode (preview changes without actually migrating)
        #[arg(long, short = 'n')]
        dry_run: bool,
        /// Keep old configuration files after migration (do not remove)
        #[arg(long)]
        keep_old: bool,
    },
    /// Pull Request operations
    ///
    /// Create, merge, close, and manage Pull Requests.
    Pr {
        #[command(subcommand)]
        subcommand: PRCommands,
    },
    /// Jira operations (info, attachments, clean, log)
    ///
    /// View and manage Jira ticket information, download attachments, clean local data, and manage log files.
    Jira {
        #[command(subcommand)]
        subcommand: JiraSubcommand,
    },
}
