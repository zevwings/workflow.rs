//! Workflow CLI 主入口
//!
//! 这是 Workflow CLI 工具的主命令入口，提供配置管理、检查工具、代理管理等核心功能。
//! 其他独立命令（如 `pr`、`qk`）通过 `bin/` 目录下的独立可执行文件实现。

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

use commands::config::{check, completion, github, log, proxy, setup, show};
use commands::lifecycle::{uninstall, update};
use commands::qk::clean::CleanCommand;

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
    /// Clean log directory
    ///
    /// Delete the entire log download base directory and all its contents.
    /// Confirmation is required before deletion.
    Clean {
        /// Preview operation without actually deleting
        #[arg(long, short = 'n')]
        dry_run: bool,
        /// Only list what would be deleted
        #[arg(long, short = 'l')]
        list: bool,
    },
    /// Manage log level (set/check)
    ///
    /// Set or view current log output level (none, error, warn, info, debug).
    Log {
        #[command(subcommand)]
        subcommand: LogSubcommand,
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
enum LogSubcommand {
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
        // 清理日志目录（清理整个基础目录）
        Some(Commands::Clean { dry_run, list }) => {
            CleanCommand::clean("", dry_run, list)?;
        }
        // 日志级别管理命令
        Some(Commands::Log { subcommand }) => match subcommand {
            LogSubcommand::Set => log::LogCommand::set()?,
            LogSubcommand::Check => log::LogCommand::check()?,
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
        // 无命令时显示帮助信息
        None => {
            log_message!("Workflow CLI - Configuration Management");
            log_message!("\nAvailable commands:");
            log_message!("  workflow check      - Run environment checks (Git status and network)");
            log_message!("  workflow clean      - Clean log download directory");
            log_message!("  workflow completion - Manage shell completion (generate/check/remove)");
            log_message!("  workflow config     - View current configuration");
            log_message!("  workflow github     - Manage GitHub accounts (list/add/remove/switch/update/current)");
            log_message!("  workflow log        - Manage log level (set/check)");
            log_message!("  workflow proxy      - Manage proxy settings (on/off/check)");
            log_message!("  workflow setup      - Initialize or update configuration");
            log_message!("  workflow uninstall  - Uninstall Workflow CLI configuration");
            log_message!(
                "  workflow update     - Update Workflow CLI (rebuild and update binaries)"
            );
            log_message!("\nOther CLI tools:");
            log_message!("  pr                  - Pull Request operations (create/merge/close/status/list/update/integrate)");
            log_message!(
                "  qk                  - Quick log operations (download/find/search/clean/info)"
            );
            log_message!("  install             - Install Workflow CLI components (binaries and/or completions)");
            log_message!("\nUse '<command> --help' for more information about each command.");
        }
    }

    Ok(())
}
