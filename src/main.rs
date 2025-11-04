use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

use commands::{check, config, proxy, setup};

use workflow::*;

#[derive(Parser)]
#[command(name = "workflow")]
#[command(about = "Workflow CLI tool", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage proxy settings
    Proxy {
        #[command(subcommand)]
        subcommand: ProxySubcommand,
    },
    /// Run checks (git status, network, pre-commit)
    Check {
        #[command(subcommand)]
        subcommand: CheckSubcommand,
    },
    /// Initialize or update configuration
    Setup,
    /// View current configuration
    Config,
}

#[derive(Subcommand)]
enum ProxySubcommand {
    /// Turn proxy on (set environment variables)
    On,
    /// Turn proxy off (unset environment variables)
    Off,
    /// Check proxy status and configuration
    Check,
}

#[derive(Subcommand)]
enum CheckSubcommand {
    /// Check git status
    #[command(name = "git_status")]
    GitStatus,
    /// Check network connection (GitHub)
    Network,
    /// Run pre-commit checks
    #[command(name = "pre_commit")]
    PreCommit,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Proxy { subcommand }) => match subcommand {
            ProxySubcommand::On => proxy::ProxyCommand::on()?,
            ProxySubcommand::Off => proxy::ProxyCommand::off()?,
            ProxySubcommand::Check => proxy::ProxyCommand::check()?,
        },
        Some(Commands::Check { subcommand }) => match subcommand {
            CheckSubcommand::GitStatus => check::CheckCommand::check_git_status()?,
            CheckSubcommand::Network => check::CheckCommand::check_network()?,
            CheckSubcommand::PreCommit => check::CheckCommand::check_pre_commit()?,
        },
        Some(Commands::Setup) => {
            setup::SetupCommand::run()?;
        }
        Some(Commands::Config) => {
            config::ConfigCommand::show()?;
        }
        None => {
            // 显示帮助信息
            println!("Workflow CLI - Configuration Management");
            println!("\nAvailable commands:");
            println!("  workflow check  - Run checks (git_status/network/pre_commit)");
            println!("  workflow proxy  - Manage proxy settings (on/off/check)");
            println!("  workflow setup  - Initialize or update configuration");
            println!("  workflow config - View current configuration");
            println!("\nUse 'workflow <command> --help' for more information.");
        }
    }

    Ok(())
}
