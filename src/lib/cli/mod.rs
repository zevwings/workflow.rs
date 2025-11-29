//! CLI 命令结构定义
//!
//! 这个模块定义了 Workflow CLI 的命令结构，供 `main.rs` 和补全生成器使用。
//! 这样可以确保补全脚本与实际命令结构保持同步。

use clap::{CommandFactory, Parser, Subcommand};

/// CLI 主结构体
///
/// 使用 clap 进行命令行参数解析，支持子命令模式。
#[derive(Parser)]
#[command(name = "workflow")]
#[command(about = "Workflow CLI tool", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

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
    /// View current configuration
    ///
    /// Display all configured environment variables and settings (sensitive information will be masked).
    Config,
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
    /// Pull Request operations
    ///
    /// Create, merge, close, and manage Pull Requests.
    Pr {
        #[command(subcommand)]
        subcommand: PRCommands,
    },
    /// Log operations (download, find, search)
    ///
    /// Download log files from Jira tickets, search and find content in logs.
    Log {
        #[command(subcommand)]
        subcommand: LogSubcommand,
    },
    /// Jira operations (info, attachments, clean)
    ///
    /// View and manage Jira ticket information, download attachments, and clean local data.
    Jira {
        #[command(subcommand)]
        subcommand: JiraSubcommand,
    },
}

/// Proxy management subcommands
///
/// Used to manage HTTP/HTTPS proxy environment variable configuration.
#[derive(Subcommand)]
pub enum ProxySubcommand {
    /// Enable proxy (set environment variables)
    ///
    /// Set HTTP_PROXY and HTTPS_PROXY environment variables.
    On,
    /// Disable proxy (clear environment variables)
    ///
    /// Unset HTTP_PROXY and HTTPS_PROXY environment variables.
    Off,
    /// Check proxy status and configuration
    ///
    /// Display current proxy environment variable status and configuration information.
    Check,
}

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
}

/// LLM configuration management subcommands
///
/// Used to manage LLM provider, API keys, models, and language settings.
#[derive(Subcommand)]
pub enum LLMSubcommand {
    /// Show current LLM configuration
    ///
    /// Display current LLM provider, API key (masked), model, and language settings.
    Show,
    /// Setup LLM configuration
    ///
    /// Interactively configure LLM provider, proxy URL, API key, model, and language settings.
    Setup,
}

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

/// Branch management subcommands
///
/// Used to clean branches and manage branch ignore list.
#[derive(Subcommand)]
pub enum BranchSubcommand {
    /// Clean local branches
    ///
    /// Delete all local branches except main/master, develop, current branch, and branches in ignore list.
    Clean {
        /// Dry run mode (show what would be deleted without actually deleting)
        #[arg(long, short = 'n')]
        dry_run: bool,
    },
    /// Manage branch ignore list
    ///
    /// Add, remove, or list branches in the ignore list.
    Ignore {
        #[command(subcommand)]
        subcommand: IgnoreSubcommand,
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

/// PR commands enumeration
///
/// Defines all PR-related subcommands.
#[derive(Subcommand)]
pub enum PRCommands {
    /// Create a new Pull Request
    ///
    /// Supports auto-detection of repository type (GitHub/Codeup), and optionally uses AI to generate PR title.
    /// If a Jira ticket is provided, will automatically update Jira status.
    Create {
        /// Jira ticket ID (optional, e.g., PROJ-123)
        #[arg(value_name = "JIRA_TICKET")]
        jira_ticket: Option<String>,

        /// PR title (optional, will use AI generation if not provided)
        #[arg(short, long)]
        title: Option<String>,

        /// Short description (optional)
        #[arg(short, long)]
        description: Option<String>,

        /// Dry run mode (don't actually create PR, only show what would be done)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        dry_run: bool,
    },
    /// Merge a Pull Request
    ///
    /// Auto-detect PR corresponding to current branch, or manually specify PR ID.
    /// Will automatically update corresponding Jira ticket status after merging.
    Merge {
        /// PR ID (optional, auto-detect from current branch if not provided)
        #[arg(value_name = "PR_ID")]
        pull_request_id: Option<String>,

        /// Force merge (skip checks)
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        force: bool,
    },
    /// Show PR status information
    ///
    /// Display detailed information about a specific PR, including status, author, comments, etc.
    Status {
        /// PR ID or branch name (optional, auto-detect from current branch if not provided)
        #[arg(value_name = "PR_ID_OR_BRANCH")]
        pull_request_id_or_branch: Option<String>,
    },
    /// List Pull Requests
    ///
    /// List all PRs in the repository, supports filtering by status and limiting the number of results.
    List {
        /// Filter by state (open, closed, merged)
        #[arg(short, long)]
        state: Option<String>,

        /// Limit the number of results
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// Update code (use PR title as commit message)
    ///
    /// Commit current changes to PR branch using PR title as commit message.
    Update,
    /// Sync branch into current branch
    ///
    /// Sync specified branch into current branch, supporting merge, rebase, or squash.
    /// This is a local Git operation, different from the `merge` command (which merges PR via API).
    /// Merged functionality from `integrate` and `sync` commands.
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

        /// Don't push to remote (pushes by default)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        no_push: bool,
    },
    /// Rebase current branch onto target branch and update PR base
    ///
    /// Rebase the current branch onto the specified target branch,
    /// and update the PR's base branch if a PR exists (with user confirmation).
    /// PR ID is automatically detected from the current branch.
    Rebase {
        /// Target branch to rebase onto (required)
        #[arg(value_name = "TARGET_BRANCH")]
        target_branch: String,

        /// Don't push to remote (only rebase locally)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        no_push: bool,

        /// Dry run mode (show what would be done without actually doing it)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        dry_run: bool,
    },
    /// Close a Pull Request
    ///
    /// Close PR corresponding to current branch, delete remote branch, and switch to default branch.
    Close {
        /// PR ID (optional, auto-detect from current branch if not provided)
        #[arg(value_name = "PR_ID")]
        pull_request_id: Option<String>,
    },
    /// Summarize a Pull Request
    ///
    /// Read PR changes and generate a summary document using LLM.
    /// The document will be saved to ~/Documents/Workflow/{PR_ID}/{filename}.md
    /// where filename is automatically generated by AI based on PR content.
    /// Language is determined by the config file (defaults to "en" if not configured).
    Summarize {
        /// PR ID (optional, auto-detect from current branch if not provided)
        #[arg(value_name = "PR_ID")]
        pull_request_id: Option<String>,
    },
    /// Approve a Pull Request
    ///
    /// Approve a PR by adding a 👍 comment.
    Approve {
        /// PR ID (optional, auto-detect from current branch if not provided)
        #[arg(value_name = "PR_ID")]
        pull_request_id: Option<String>,
    },
    /// Add a comment to a Pull Request
    ///
    /// Add a comment to a PR.
    Comment {
        /// PR ID (optional, auto-detect from current branch if not provided)
        #[arg(value_name = "PR_ID")]
        pull_request_id: Option<String>,

        /// Comment message (required, can be multiple words)
        #[arg(value_name = "MESSAGE", trailing_var_arg = true)]
        message: Vec<String>,
    },
    /// Pick commits from one branch to another and create a new PR
    ///
    /// Cherry-pick all commits from the source branch to the target branch,
    /// then interactively create a new branch and PR (similar to `pr create`).
    /// This is similar to backport/forwardport but supports any direction.
    ///
    /// The command will:
    /// 1. Switch to the target branch
    /// 2. Cherry-pick commits (without committing)
    /// 3. Interactively create PR (with LLM-generated branch name, Jira integration, etc.)
    Pick {
        /// Source branch name (branch to cherry-pick from)
        #[arg(value_name = "FROM_BRANCH")]
        from_branch: String,

        /// Target branch name (base branch for the new PR)
        #[arg(value_name = "TO_BRANCH")]
        to_branch: String,

        /// Dry run mode (show what would be done without actually doing it)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        dry_run: bool,
    },
}

/// Log operations subcommands
///
/// Used to manage log file operations.
#[derive(Subcommand)]
pub enum LogSubcommand {
    /// Download log files from Jira ticket
    ///
    /// Download log files from Jira ticket attachments (supports automatic merging of split files).
    /// Log files will be saved locally with paths automatically resolved based on JIRA ID.
    Download {
        /// Jira ticket ID (optional, will prompt interactively if not provided)
        #[arg(value_name = "JIRA_ID")]
        jira_id: Option<String>,
    },
    /// Find request ID in log files
    ///
    /// Find specified request ID in log files and extract corresponding response content.
    /// If found, will copy response content to clipboard and automatically open browser.
    Find {
        /// Jira ticket ID (optional, will prompt interactively if not provided)
        #[arg(value_name = "JIRA_ID")]
        jira_id: Option<String>,

        /// Request ID (optional, will prompt interactively if not provided)
        #[arg(value_name = "REQUEST_ID")]
        request_id: Option<String>,
    },
    /// Search for keywords in log files
    ///
    /// Search for specified keywords in log files and return all matching request information.
    Search {
        /// Jira ticket ID (optional, will prompt interactively if not provided)
        #[arg(value_name = "JIRA_ID")]
        jira_id: Option<String>,

        /// Search keyword (optional, will prompt interactively if not provided)
        #[arg(value_name = "SEARCH_TERM")]
        search_term: Option<String>,
    },
}

/// Jira operations subcommands
///
/// Used to manage Jira ticket operations.
#[derive(Subcommand)]
pub enum JiraSubcommand {
    /// Show ticket information
    ///
    /// Display detailed information about a Jira ticket.
    Info {
        /// Jira ticket ID (optional, will prompt interactively if not provided)
        #[arg(value_name = "JIRA_ID")]
        jira_id: Option<String>,
    },
    /// Download all attachments from Jira ticket
    ///
    /// Download all attachments from Jira ticket (not just log files).
    Attachments {
        /// Jira ticket ID (optional, will prompt interactively if not provided)
        #[arg(value_name = "JIRA_ID")]
        jira_id: Option<String>,
    },
    /// Clean log directory
    ///
    /// Clean log directory for specified JIRA ID, or clean entire base directory if --all is specified.
    Clean {
        /// Jira ticket ID (optional, will prompt interactively if not provided)
        #[arg(value_name = "JIRA_ID")]
        jira_id: Option<String>,

        /// Clean entire base directory (all tickets)
        #[arg(long, short = 'a')]
        all: bool,

        /// Preview operation without actually deleting
        #[arg(long, short = 'n')]
        dry_run: bool,

        /// Only list what would be deleted
        #[arg(long, short = 'l')]
        list: bool,
    },
}

/// 获取 CLI 命令结构（用于生成补全脚本）
///
/// 返回 `Cli` 结构体的 `Command` 实例，用于自动生成 shell completion 脚本。
/// 这样可以确保补全脚本与实际命令结构保持同步。
pub fn get_cli_command() -> clap::Command {
    Cli::command()
}
