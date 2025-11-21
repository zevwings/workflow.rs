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
    /// 管理代理设置（开启/关闭/检查）
    ///
    /// 通过环境变量管理 HTTP/HTTPS 代理配置。
    Proxy {
        #[command(subcommand)]
        subcommand: ProxySubcommand,
        /// 临时模式：只在当前 shell 启用，不写入配置文件
        #[arg(short, long)]
        temporary: bool,
    },
    /// 运行环境检查
    ///
    /// 检查 Git 仓库状态和网络连接（GitHub）。
    Check,
    /// 初始化或更新配置
    ///
    /// 交互式设置 Workflow CLI 所需的各种配置项（如 Jira、GitHub 等）。
    Setup,
    /// 查看当前配置
    ///
    /// 显示所有已配置的环境变量和设置项（敏感信息会被掩码）。
    Config,
    /// 卸载 Workflow CLI 配置
    ///
    /// 删除所有相关文件：二进制文件、补全脚本、配置文件等。
    Uninstall,
    /// 更新 Workflow CLI
    ///
    /// 重新构建 release 版本并更新所有二进制文件和 shell completion 脚本。
    Update {
        /// 指定要更新的版本号（例如：1.1.2）
        ///
        /// 如果不指定，将更新到最新版本。
        #[arg(long, short = 'v')]
        version: Option<String>,
    },
    /// 清理日志目录
    ///
    /// 删除整个日志下载基础目录及其所有内容。
    /// 需要确认才能执行删除操作。
    Clean {
        /// 预览操作，不实际删除
        #[arg(long, short = 'n')]
        dry_run: bool,
        /// 只列出将要删除的内容
        #[arg(long, short = 'l')]
        list: bool,
    },
    /// 管理日志级别（设置/检查）
    ///
    /// 设置或查看当前日志输出级别（none, error, warn, info, debug）。
    Log {
        #[command(subcommand)]
        subcommand: LogSubcommand,
    },
    /// 管理 GitHub 账号
    ///
    /// 管理多个 GitHub 账号的配置（添加、删除、切换、更新等）。
    #[command(name = "github")]
    GitHub {
        #[command(subcommand)]
        subcommand: GitHubSubcommand,
    },
    /// 管理 Shell Completion
    ///
    /// 生成和管理 shell completion 脚本。
    Completion {
        #[command(subcommand)]
        subcommand: CompletionSubcommand,
    },
}

/// 代理管理子命令
///
/// 用于管理 HTTP/HTTPS 代理的环境变量配置。
#[derive(Subcommand)]
enum ProxySubcommand {
    /// 开启代理（设置环境变量）
    ///
    /// 设置 `HTTP_PROXY` 和 `HTTPS_PROXY` 环境变量。
    On,
    /// 关闭代理（清除环境变量）
    ///
    /// 取消设置 `HTTP_PROXY` 和 `HTTPS_PROXY` 环境变量。
    Off,
    /// 检查代理状态和配置
    ///
    /// 显示当前代理环境变量的状态和配置信息。
    Check,
}

/// 日志级别管理子命令
///
/// 用于管理日志输出级别。
#[derive(Subcommand)]
enum LogSubcommand {
    /// 设置日志级别（交互式选择）
    ///
    /// 通过交互式菜单选择日志级别：none, error, warn, info, debug。
    Set,
    /// 检查当前日志级别
    ///
    /// 显示当前设置的日志级别和默认级别信息。
    Check,
}

/// GitHub 账号管理子命令
///
/// 用于管理多个 GitHub 账号的配置。
#[derive(Subcommand)]
enum GitHubSubcommand {
    /// 列出所有 GitHub 账号
    ///
    /// 显示所有已配置的 GitHub 账号信息。
    List,
    /// 显示当前激活的 GitHub 账号
    ///
    /// 显示当前正在使用的 GitHub 账号信息。
    Current,
    /// 添加新的 GitHub 账号
    ///
    /// 交互式添加新的 GitHub 账号配置。
    Add,
    /// 删除 GitHub 账号
    ///
    /// 从配置中删除指定的 GitHub 账号。
    Remove,
    /// 切换当前 GitHub 账号
    ///
    /// 在多个 GitHub 账号之间切换。
    Switch,
    /// 更新 GitHub 账号信息
    ///
    /// 更新已存在的 GitHub 账号配置。
    Update,
}

/// Completion 管理子命令
///
/// 用于生成和管理 shell completion 脚本。
#[derive(Subcommand)]
enum CompletionSubcommand {
    /// 生成 completion 脚本
    ///
    /// 自动检测当前 shell 类型，生成对应的 completion 脚本并应用到配置文件。
    Generate,
    /// 检查 completion 状态
    ///
    /// 检查系统中已安装的 shell 类型和已配置 completion 的 shell。
    Check,
    /// 移除 completion 配置
    ///
    /// 交互式选择并移除已配置的 shell completion 配置。
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
            log_message!("  workflow update     - Update Workflow CLI (rebuild and update binaries)");
            log_message!("\nOther CLI tools:");
            log_message!("  pr                  - Pull Request operations (create/merge/close/status/list/update/integrate)");
            log_message!("  qk                  - Quick log operations (download/find/search/clean/info)");
            log_message!("  install             - Install Workflow CLI components (binaries and/or completions)");
            log_message!("\nUse '<command> --help' for more information about each command.");
        }
    }

    Ok(())
}
