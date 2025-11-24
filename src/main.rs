//! Workflow CLI 主入口
//!
//! 这是 Workflow CLI 工具的主命令入口，提供配置管理、检查工具、代理管理等核心功能。
//! 所有功能都通过 `workflow` 命令及其子命令提供，包括 `pr`、`log`、`jira` 等子命令。

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

use commands::branch::{clean, ignore};
use commands::check::check;
use commands::config::{completion, log, setup, show};
use commands::github::github;
use commands::jira::{attachments::AttachmentsCommand, clean::CleanCommand, info::InfoCommand};
use commands::lifecycle::{uninstall, update};
use commands::log::{download::DownloadCommand, find::FindCommand, search::SearchCommand};
use commands::pr::{close, create, integrate, list, merge, status, update as pr_update};
use commands::proxy::proxy;

use workflow::*;

/// CLI 主结构体
///
/// 使用 clap 进行命令行参数解析，支持子命令模式。
#[derive(Parser)]
#[command(name = "workflow")]
#[command(about = "Workflow CLI tool", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

/// 主命令枚举
///
/// 定义了 Workflow CLI 支持的所有顶级命令。
#[derive(Subcommand)]
enum Commands {
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
enum ProxySubcommand {
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
enum LogLevelSubcommand {
    /// Set log level (interactive selection)
    ///
    /// Select log level through interactive menu: none, error, warn, info, debug.
    Set,
    /// Check current log level
    ///
    /// Display current configured log level and default level information.
    Check,
}

/// GitHub account management subcommands
///
/// Used to manage configurations for multiple GitHub accounts.
#[derive(Subcommand)]
enum GitHubSubcommand {
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
enum CompletionSubcommand {
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
enum BranchSubcommand {
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
enum IgnoreSubcommand {
    /// Add branch to ignore list
    Add {
        /// Branch name to add
        branch_name: String,
    },
    /// Remove branch from ignore list
    Remove {
        /// Branch name to remove
        branch_name: String,
    },
    /// List ignored branches for current repository
    List,
}

/// PR commands enumeration
///
/// Defines all PR-related subcommands.
#[derive(Subcommand)]
enum PRCommands {
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
    /// Integrate branch into current branch
    ///
    /// Merge specified branch into current branch, and optionally push to remote.
    /// This is a local Git operation, different from the `merge` command (which merges PR via API).
    Integrate {
        /// Source branch name to merge (required)
        #[arg(value_name = "SOURCE_BRANCH")]
        source_branch: String,

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
    /// Close a Pull Request
    ///
    /// Close PR corresponding to current branch, delete remote branch, and switch to default branch.
    Close {
        /// PR ID (optional, auto-detect from current branch if not provided)
        #[arg(value_name = "PR_ID")]
        pull_request_id: Option<String>,
    },
}

/// Log operations subcommands
///
/// Used to manage log file operations.
#[derive(Subcommand)]
enum LogSubcommand {
    /// Download log files from Jira ticket
    ///
    /// Download log files from Jira ticket attachments (supports automatic merging of split files).
    /// Log files will be saved locally with paths automatically resolved based on JIRA ID.
    Download {
        /// Jira ticket ID (e.g., PROJ-123)
        #[arg(value_name = "JIRA_ID")]
        jira_id: String,

        /// Download all attachments (not just log files)
        #[arg(long, short = 'a')]
        all: bool,
    },
    /// Find request ID in log files
    ///
    /// Find specified request ID in log files and extract corresponding response content.
    /// If found, will copy response content to clipboard and automatically open browser.
    Find {
        /// Jira ticket ID (e.g., PROJ-123)
        #[arg(value_name = "JIRA_ID")]
        jira_id: String,

        /// Request ID (optional, will prompt interactively if not provided)
        #[arg(value_name = "REQUEST_ID")]
        request_id: Option<String>,
    },
    /// Search for keywords in log files
    ///
    /// Search for specified keywords in log files and return all matching request information.
    Search {
        /// Jira ticket ID (e.g., PROJ-123)
        #[arg(value_name = "JIRA_ID")]
        jira_id: String,

        /// Search keyword (optional, will prompt interactively if not provided)
        #[arg(value_name = "SEARCH_TERM")]
        search_term: Option<String>,
    },
}

/// Jira operations subcommands
///
/// Used to manage Jira ticket operations.
#[derive(Subcommand)]
enum JiraSubcommand {
    /// Show ticket information
    ///
    /// Display detailed information about a Jira ticket.
    Info {
        /// Jira ticket ID (e.g., PROJ-123)
        #[arg(value_name = "JIRA_ID")]
        jira_id: String,
    },
    /// Download all attachments from Jira ticket
    ///
    /// Download all attachments from Jira ticket (not just log files).
    Attachments {
        /// Jira ticket ID (e.g., PROJ-123)
        #[arg(value_name = "JIRA_ID")]
        jira_id: String,
    },
    /// Clean log directory
    ///
    /// Clean log directory for specified JIRA ID, or clean entire base directory if no JIRA ID provided.
    Clean {
        /// Jira ticket ID (optional, if not provided, clean entire base directory)
        #[arg(value_name = "JIRA_ID")]
        jira_id: Option<String>,

        /// Preview operation without actually deleting
        #[arg(long, short = 'n')]
        dry_run: bool,

        /// Only list what would be deleted
        #[arg(long, short = 'l')]
        list: bool,
    },
}

/// 主函数
///
/// 解析命令行参数并分发到相应的命令处理函数。
fn main() -> Result<()> {
    // 初始化日志级别（从配置文件读取，但不让 logger 模块直接依赖 Settings）
    {
        use crate::base::settings::Settings;
        let config_level = Settings::get()
            .log
            .level
            .as_ref()
            .and_then(|s| s.parse::<crate::LogLevel>().ok());
        crate::LogLevel::init(config_level);
    }

    let cli = Cli::parse();

    match cli.command {
        // 代理管理命令
        Some(Commands::Proxy {
            subcommand,
            temporary,
        }) => match subcommand {
            ProxySubcommand::On => proxy::ProxyCommand::on(temporary)?,
            ProxySubcommand::Off => proxy::ProxyCommand::off()?,
            ProxySubcommand::Check => proxy::ProxyCommand::check()?,
        },
        // 环境检查
        Some(Commands::Check) => {
            check::CheckCommand::run_all()?;
        }
        // 配置初始化
        Some(Commands::Setup) => {
            setup::SetupCommand::run()?;
        }
        // 配置查看
        Some(Commands::Config) => {
            show::ConfigCommand::show()?;
        }
        // 卸载
        Some(Commands::Uninstall) => {
            uninstall::UninstallCommand::run()?;
        }
        // 更新
        Some(Commands::Update { version }) => {
            update::UpdateCommand::update(version)?;
        }
        // 日志级别管理命令
        Some(Commands::LogLevel { subcommand }) => match subcommand {
            LogLevelSubcommand::Set => log::LogCommand::set()?,
            LogLevelSubcommand::Check => log::LogCommand::check()?,
        },
        // GitHub 账号管理命令
        Some(Commands::GitHub { subcommand }) => match subcommand {
            GitHubSubcommand::List => github::GitHubCommand::list()?,
            GitHubSubcommand::Current => github::GitHubCommand::current()?,
            GitHubSubcommand::Add => github::GitHubCommand::add()?,
            GitHubSubcommand::Remove => github::GitHubCommand::remove()?,
            GitHubSubcommand::Switch => github::GitHubCommand::switch()?,
            GitHubSubcommand::Update => github::GitHubCommand::update()?,
        },
        // Completion 管理命令
        Some(Commands::Completion { subcommand }) => match subcommand {
            CompletionSubcommand::Generate => completion::CompletionCommand::generate()?,
            CompletionSubcommand::Check => completion::CompletionCommand::check()?,
            CompletionSubcommand::Remove => completion::CompletionCommand::remove()?,
        },
        // 分支管理命令
        Some(Commands::Branch { subcommand }) => match subcommand {
            BranchSubcommand::Clean { dry_run } => {
                clean::BranchCleanCommand::clean(dry_run)?;
            }
            BranchSubcommand::Ignore { subcommand } => match subcommand {
                IgnoreSubcommand::Add { branch_name } => {
                    ignore::BranchIgnoreCommand::add(branch_name)?;
                }
                IgnoreSubcommand::Remove { branch_name } => {
                    ignore::BranchIgnoreCommand::remove(branch_name)?;
                }
                IgnoreSubcommand::List => {
                    ignore::BranchIgnoreCommand::list()?;
                }
            },
        },
        // PR 操作命令
        Some(Commands::Pr { subcommand }) => match subcommand {
            PRCommands::Create {
                jira_ticket,
                title,
                description,
                dry_run,
            } => {
                create::PullRequestCreateCommand::create(jira_ticket, title, description, dry_run)?;
            }
            PRCommands::Merge {
                pull_request_id,
                force,
            } => {
                merge::PullRequestMergeCommand::merge(pull_request_id, force)?;
            }
            PRCommands::Status {
                pull_request_id_or_branch,
            } => {
                status::PullRequestStatusCommand::show(pull_request_id_or_branch)?;
            }
            PRCommands::List { state, limit } => {
                list::PullRequestListCommand::list(state, limit)?;
            }
            PRCommands::Update => {
                pr_update::PullRequestUpdateCommand::update()?;
            }
            PRCommands::Integrate {
                source_branch,
                ff_only,
                squash,
                no_push,
            } => {
                let should_push = !no_push;
                integrate::PullRequestIntegrateCommand::integrate(
                    source_branch,
                    ff_only,
                    squash,
                    should_push,
                )?;
            }
            PRCommands::Close { pull_request_id } => {
                close::PullRequestCloseCommand::close(pull_request_id)?;
            }
        },
        // 日志操作命令
        Some(Commands::Log { subcommand }) => match subcommand {
            LogSubcommand::Download { jira_id, all: _ } => {
                // Log download only downloads log files, ignoring the 'all' flag
                DownloadCommand::download(&jira_id)?;
            }
            LogSubcommand::Find {
                jira_id,
                request_id,
            } => {
                FindCommand::find_request_id(&jira_id, request_id)?;
            }
            LogSubcommand::Search {
                jira_id,
                search_term,
            } => {
                SearchCommand::search(&jira_id, search_term)?;
            }
        },
        // Jira 操作命令
        Some(Commands::Jira { subcommand }) => match subcommand {
            JiraSubcommand::Info { jira_id } => {
                InfoCommand::show(&jira_id)?;
            }
            JiraSubcommand::Attachments { jira_id } => {
                AttachmentsCommand::download(&jira_id)?;
            }
            JiraSubcommand::Clean {
                jira_id,
                dry_run,
                list,
            } => {
                let jira_id = jira_id.as_deref().unwrap_or("");
                CleanCommand::clean(jira_id, dry_run, list)?;
            }
        },
        // 无命令时显示帮助信息
        None => {
            log_message!("Workflow CLI - Configuration Management");
            log_message!("\nAvailable commands:");
            log_message!("  workflow branch     - Manage Git branches (clean/ignore)");
            log_message!("  workflow check      - Run environment checks (Git status and network)");
            log_message!("  workflow completion - Manage shell completion (generate/check/remove)");
            log_message!("  workflow config     - View current configuration");
            log_message!("  workflow github     - Manage GitHub accounts (list/add/remove/switch/update/current)");
            log_message!("  workflow log-level  - Manage log level (set/check)");
            log_message!("  workflow proxy      - Manage proxy settings (on/off/check)");
            log_message!("  workflow setup      - Initialize or update configuration");
            log_message!("  workflow uninstall  - Uninstall Workflow CLI configuration");
            log_message!(
                "  workflow update     - Update Workflow CLI (rebuild and update binaries)"
            );
            log_message!("  workflow pr         - Pull Request operations (create/merge/close/status/list/update/integrate)");
            log_message!("  workflow log        - Log operations (download/find/search)");
            log_message!("  workflow jira       - Jira operations (info/attachments/clean)");
            log_message!("\nOther CLI tools:");
            log_message!("  install             - Install Workflow CLI components (binaries and/or completions)");
            log_message!("\nUse '<command> --help' for more information about each command.");
        }
    }

    Ok(())
}
