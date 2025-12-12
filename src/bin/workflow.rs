//! Workflow CLI 主入口
//!
//! 这是 Workflow CLI 工具的主命令入口，提供配置管理、检查工具、代理管理等核心功能。
//! 所有功能都通过 `workflow` 命令及其子命令提供，包括 `pr`、`log`、`jira` 等子命令。

use anyhow::Result;
use clap::Parser;

use workflow::commands::branch::{
    clean, create as branch_create, ignore, rename, switch, sync as branch_sync,
};
use workflow::commands::check::check;
use workflow::commands::commit::{CommitAmendCommand, CommitRewordCommand, CommitSquashCommand};
use workflow::commands::config::{completion, export, import, log, setup, show, validate};
use workflow::commands::github::github;
use workflow::commands::jira::{
    AttachmentsCommand, ChangelogCommand, CleanCommand, CommentCommand, CommentsCommand,
    InfoCommand, RelatedCommand,
};
use workflow::commands::lifecycle::{uninstall, update as lifecycle_update, version};
use workflow::commands::llm::{LLMSetupCommand, LLMShowCommand};
use workflow::commands::log::{DownloadCommand, FindCommand, SearchCommand};
use workflow::commands::migrate::MigrateCommand;
use workflow::commands::pr::{
    approve, close, comment, create as pr_create, list, merge, pick, rebase, status, summarize,
    sync, update as pr_update,
};
use workflow::commands::proxy::proxy;
use workflow::commands::repo::{setup as repo_setup, show as repo_show};
use workflow::commands::stash::{apply, drop, list as stash_list, pop};

use workflow::cli::{
    BranchSubcommand, Cli, Commands, CommitSubcommand, CompletionSubcommand, ConfigSubcommand,
    GitHubSubcommand, IgnoreSubcommand, JiraSubcommand, LLMSubcommand, LogLevelSubcommand,
    LogSubcommand, PRCommands, ProxySubcommand, RepoSubcommand, StashSubcommand,
};
use workflow::*;

use workflow::base::settings::Settings;

/// 主函数
///
/// 解析命令行参数并分发到相应的命令处理函数。
fn main() -> Result<()> {
    // 初始化日志级别（从配置文件读取，用于 log_*! 宏）
    {
        let config_level = Settings::get()
            .log
            .level
            .as_ref()
            .and_then(|s| s.parse::<workflow::LogLevel>().ok());
        workflow::LogLevel::init(config_level);
    }

    // 初始化 tracing（从配置文件读取，统一管理）
    workflow::Tracer::init();

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
        // 配置管理命令
        Some(Commands::Config { subcommand }) => match subcommand {
            Some(ConfigSubcommand::Show) => show::ConfigCommand::show()?,
            Some(ConfigSubcommand::Validate {
                config_path,
                fix,
                strict,
            }) => {
                validate::ConfigValidateCommand::validate(config_path, fix, strict)?;
            }
            Some(ConfigSubcommand::Export {
                output_path,
                section,
                no_secrets,
                toml,
                json,
                yaml,
            }) => {
                export::ConfigExportCommand::export(
                    output_path,
                    section,
                    no_secrets,
                    toml,
                    json,
                    yaml,
                )?;
            }
            Some(ConfigSubcommand::Import {
                input_path,
                overwrite,
                section,
                dry_run,
            }) => {
                import::ConfigImportCommand::import(
                    input_path,
                    overwrite,
                    section,
                    dry_run.is_dry_run(),
                )?;
            }
            None => {
                // 当没有子命令时，显示帮助信息
                log_message!("Configuration Management");
                log_message!("\nAvailable subcommands:");
                log_message!("  workflow config show     - View current configuration");
                log_message!("  workflow config validate - Validate configuration file");
                log_message!("  workflow config export   - Export configuration to a file");
                log_message!("  workflow config import   - Import configuration from a file");
                log_message!("\nUse 'workflow config <subcommand> --help' for more information.");
            }
        },
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
        Some(Commands::Log { subcommand }) => match subcommand {
            LogLevelSubcommand::Set => log::LogCommand::set()?,
            LogLevelSubcommand::Check => log::LogCommand::check()?,
            LogLevelSubcommand::TraceConsole => log::LogCommand::trace_console()?,
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
                clean::BranchCleanCommand::clean(dry_run.is_dry_run())?;
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
            BranchSubcommand::Create {
                jira_id,
                from_default,
                dry_run,
            } => {
                branch_create::CreateCommand::execute(
                    jira_id.into_option(),
                    from_default,
                    dry_run.is_dry_run(),
                )?;
            }
            BranchSubcommand::Rename => {
                rename::BranchRenameCommand::execute()?;
            }
            BranchSubcommand::Switch { branch_name } => {
                switch::SwitchCommand::execute(branch_name)?;
            }
            BranchSubcommand::Sync {
                source_branch,
                rebase,
                ff_only,
                squash,
            } => {
                branch_sync::BranchSyncCommand::sync(source_branch, rebase, ff_only, squash)?;
            }
        },
        // Commit 操作命令
        Some(Commands::Commit { subcommand }) => match subcommand {
            CommitSubcommand::Amend {
                message,
                no_edit,
                no_verify,
            } => {
                CommitAmendCommand::execute(message, no_edit, no_verify)?;
            }
            CommitSubcommand::Reword { commit_id } => {
                CommitRewordCommand::execute(commit_id)?;
            }
            CommitSubcommand::Squash => {
                CommitSquashCommand::execute()?;
            }
        },
        // PR 操作命令
        Some(Commands::Pr { subcommand }) => match subcommand {
            PRCommands::Create {
                jira_ticket,
                title,
                description,
                dry_run,
            } => {
                pr_create::PullRequestCreateCommand::create(
                    jira_ticket,
                    title,
                    description,
                    dry_run.is_dry_run(),
                )?;
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
            } => {
                sync::PullRequestSyncCommand::sync(source_branch, rebase, ff_only, squash)?;
            }
            PRCommands::Rebase {
                target_branch,
                no_push,
                dry_run,
            } => {
                rebase::PullRequestRebaseCommand::rebase(
                    target_branch,
                    !no_push,
                    dry_run.is_dry_run(),
                )?;
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
                pick::PullRequestPickCommand::pick(from_branch, to_branch, dry_run.is_dry_run())?;
            }
        },
        // Jira 操作命令
        Some(Commands::Jira { subcommand }) => match subcommand {
            JiraSubcommand::Info {
                jira_id,
                output_format,
            } => {
                InfoCommand::show(jira_id.into_option(), output_format)?;
            }
            JiraSubcommand::Related {
                jira_id,
                output_format,
            } => {
                RelatedCommand::show(jira_id.into_option(), output_format)?;
            }
            JiraSubcommand::Changelog {
                jira_id,
                output_format,
            } => {
                ChangelogCommand::show(jira_id.into_option(), output_format)?;
            }
            JiraSubcommand::Comment { jira_id } => {
                CommentCommand::add(jira_id.into_option())?;
            }
            JiraSubcommand::Comments {
                jira_id,
                limit,
                offset,
                author,
                since,
                output_format,
            } => {
                CommentsCommand::show(
                    jira_id.into_option(),
                    limit,
                    offset,
                    author,
                    since,
                    output_format,
                )?;
            }
            JiraSubcommand::Attachments { jira_id } => {
                AttachmentsCommand::download(jira_id.into_option())?;
            }
            JiraSubcommand::Clean {
                jira_id,
                all,
                dry_run,
                list,
            } => {
                CleanCommand::clean(jira_id.into_option(), all, dry_run.is_dry_run(), list)?;
            }
            JiraSubcommand::Log { subcommand } => match subcommand {
                LogSubcommand::Download { jira_id } => {
                    DownloadCommand::download(jira_id.into_option())?;
                }
                LogSubcommand::Find {
                    jira_id,
                    request_id,
                } => {
                    FindCommand::find_request_id(jira_id.into_option(), request_id)?;
                }
                LogSubcommand::Search {
                    jira_id,
                    search_term,
                } => {
                    SearchCommand::search(jira_id.into_option(), search_term)?;
                }
            },
        },
        // 配置迁移命令
        Some(Commands::Migrate { dry_run, keep_old }) => {
            // cleanup = true 表示删除旧文件，keep_old = true 表示保留旧文件
            // 所以 cleanup = !keep_old
            let cleanup = !keep_old;
            MigrateCommand::migrate(dry_run.is_dry_run(), cleanup)?;
        }
        // Stash 管理命令
        Some(Commands::Stash { subcommand }) => match subcommand {
            StashSubcommand::List { stat } => {
                stash_list::StashListCommand::execute(stat)?;
            }
            StashSubcommand::Apply => {
                apply::StashApplyCommand::execute()?;
            }
            StashSubcommand::Drop => {
                drop::StashDropCommand::execute()?;
            }
            StashSubcommand::Pop => {
                pop::StashPopCommand::execute()?;
            }
        },
        // Repository 管理命令
        Some(Commands::Repo { subcommand }) => match subcommand {
            RepoSubcommand::Setup => {
                repo_setup::RepoSetupCommand::run()?;
            }
            RepoSubcommand::Show => {
                repo_show::RepoShowCommand::show()?;
            }
        },
        // 无命令时显示帮助信息
        None => {
            log_message!("Workflow CLI - Configuration Management");
            log_message!("\nAvailable commands:");
            log_message!("  workflow branch     - Manage Git branches (clean/ignore/prefix)");
            log_message!("  workflow check      - Run environment checks (Git status and network)");
            log_message!("  workflow completion - Manage shell completion (generate/check/remove)");
            log_message!("  workflow config     - View current configuration");
            log_message!("  workflow github     - Manage GitHub accounts (list/add/remove/switch/update/current)");
            log_message!("  workflow log        - Manage log level (set/check)");
            log_message!("  workflow migrate    - Migrate configuration to new format");
            log_message!("  workflow proxy      - Manage proxy settings (on/off/check)");
            log_message!("  workflow setup      - Initialize or update configuration");
            log_message!("  workflow uninstall  - Uninstall Workflow CLI configuration");
            log_message!("  workflow version    - Show Workflow CLI version");
            log_message!(
                "  workflow update     - Update Workflow CLI (rebuild and update binaries)"
            );
            log_message!("  workflow pr         - Pull Request operations (create/merge/close/status/list/update/sync)");
            log_message!("  workflow jira       - Jira operations (info/attachments/clean/log)");
            log_message!("  workflow stash      - Git stash management (list/apply/drop/pop)");
            log_message!("\nOther CLI tools:");
            log_message!("  install             - Install Workflow CLI components (binaries and/or completions)");
            log_message!("\nUse '<command> --help' for more information about each command.");
        }
    }

    Ok(())
}
