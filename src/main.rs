//! Workflow CLI 主入口
//!
//! 这是 Workflow CLI 工具的主命令入口，提供配置管理、检查工具、代理管理等核心功能。
//! 所有功能都通过 `workflow` 命令及其子命令提供，包括 `pr`、`log`、`jira` 等子命令。

use anyhow::Result;
use clap::Parser;

mod commands;

use commands::branch::{clean, ignore};
use commands::check::check;
use commands::config::{completion, log, setup, show};
use commands::github::github;
use commands::jira::{AttachmentsCommand, CleanCommand, InfoCommand};
use commands::lifecycle::{uninstall, update as lifecycle_update, version};
use commands::llm::{LLMSetupCommand, LLMShowCommand};
use commands::log::{DownloadCommand, FindCommand, SearchCommand};
use commands::pr::{
    approve, close, comment, create, list, merge, pick, rebase, status, summarize, sync,
    update as pr_update,
};
use commands::proxy::proxy;

use workflow::cli::{
    BranchSubcommand, Cli, Commands, CompletionSubcommand, GitHubSubcommand, IgnoreSubcommand,
    JiraSubcommand, LLMSubcommand, LogLevelSubcommand, LogSubcommand, PRCommands, ProxySubcommand,
};
use workflow::*;

use crate::base::settings::Settings;

/// 主函数
///
/// 解析命令行参数并分发到相应的命令处理函数。
fn main() -> Result<()> {
    // 初始化日志级别（从配置文件读取，但不让 logger 模块直接依赖 Settings）
    {
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
        // 版本信息
        Some(Commands::Version) => {
            version::VersionCommand::show()?;
        }
        // 更新
        Some(Commands::Update { version }) => {
            lifecycle_update::UpdateCommand::update(version)?;
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
        // LLM 配置管理命令
        Some(Commands::Llm { subcommand }) => match subcommand {
            LLMSubcommand::Show => LLMShowCommand::show()?,
            LLMSubcommand::Setup => LLMSetupCommand::setup()?,
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
            PRCommands::Sync {
                source_branch,
                rebase,
                ff_only,
                squash,
                no_push,
            } => {
                let should_push = !no_push;
                sync::PullRequestSyncCommand::sync(
                    source_branch,
                    rebase,
                    ff_only,
                    squash,
                    should_push,
                )?;
            }
            PRCommands::Rebase {
                target_branch,
                no_push,
                dry_run,
            } => {
                rebase::PullRequestRebaseCommand::rebase(target_branch, !no_push, dry_run)?;
            }
            PRCommands::Close { pull_request_id } => {
                close::PullRequestCloseCommand::close(pull_request_id)?;
            }
            PRCommands::Summarize { pull_request_id } => {
                summarize::SummarizeCommand::summarize(pull_request_id)?;
            }
            PRCommands::Approve { pull_request_id } => {
                approve::PullRequestApproveCommand::approve(pull_request_id)?;
            }
            PRCommands::Comment {
                pull_request_id,
                message,
            } => {
                comment::PullRequestCommentCommand::comment(pull_request_id, message)?;
            }
            PRCommands::Pick {
                from_branch,
                to_branch,
                dry_run,
            } => {
                pick::PullRequestPickCommand::pick(from_branch, to_branch, dry_run)?;
            }
        },
        // 日志操作命令
        Some(Commands::Log { subcommand }) => match subcommand {
            LogSubcommand::Download { jira_id } => {
                DownloadCommand::download(jira_id)?;
            }
            LogSubcommand::Find {
                jira_id,
                request_id,
            } => {
                FindCommand::find_request_id(jira_id, request_id)?;
            }
            LogSubcommand::Search {
                jira_id,
                search_term,
            } => {
                SearchCommand::search(jira_id, search_term)?;
            }
        },
        // Jira 操作命令
        Some(Commands::Jira { subcommand }) => match subcommand {
            JiraSubcommand::Info { jira_id } => {
                InfoCommand::show(jira_id)?;
            }
            JiraSubcommand::Attachments { jira_id } => {
                AttachmentsCommand::download(jira_id)?;
            }
            JiraSubcommand::Clean {
                jira_id,
                all,
                dry_run,
                list,
            } => {
                CleanCommand::clean(jira_id, all, dry_run, list)?;
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
            log_message!("  workflow version    - Show Workflow CLI version");
            log_message!(
                "  workflow update     - Update Workflow CLI (rebuild and update binaries)"
            );
            log_message!("  workflow pr         - Pull Request operations (create/merge/close/status/list/update/sync)");
            log_message!("  workflow log        - Log operations (download/find/search)");
            log_message!("  workflow jira       - Jira operations (info/attachments/clean)");
            log_message!("\nOther CLI tools:");
            log_message!("  install             - Install Workflow CLI components (binaries and/or completions)");
            log_message!("\nUse '<command> --help' for more information about each command.");
        }
    }

    Ok(())
}
