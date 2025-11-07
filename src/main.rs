//! Workflow CLI 主入口
//!
//! 这是 Workflow CLI 工具的主命令入口，提供配置管理、检查工具、代理管理等核心功能。
//! 其他独立命令（如 `pr`、`qk`）通过 `bin/` 目录下的独立可执行文件实现。

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

use commands::{check, config, proxy, setup, uninstall};

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

/// 主函数
///
/// 解析命令行参数并分发到相应的命令处理函数。
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // 代理管理命令
        Some(Commands::Proxy { subcommand }) => match subcommand {
            ProxySubcommand::On => proxy::ProxyCommand::on()?,
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
            config::ConfigCommand::show()?;
        }
        // 卸载
        Some(Commands::Uninstall) => {
            uninstall::UninstallCommand::run()?;
        }
        // 无命令时显示帮助信息
        None => {
            println!("Workflow CLI - Configuration Management");
            println!("\nAvailable commands:");
            println!("  workflow check     - Run environment checks (Git status and network)");
            println!("  workflow proxy     - Manage proxy settings (on/off/check)");
            println!("  workflow setup     - Initialize or update configuration");
            println!("  workflow config    - View current configuration");
            println!("  workflow uninstall - Uninstall Workflow CLI configuration");
            println!("\nInstallation:");
            println!("  Use 'install' command (built separately): install <subcommand>");
            println!("\nUse 'workflow <command> --help' for more information.");
        }
    }

    Ok(())
}
