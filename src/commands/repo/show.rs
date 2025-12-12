//! Repository show command
//!
//! Display current repository configuration.

use crate::git::GitRepo;
use crate::repo::config::RepoConfig;
use crate::template::config::TemplateConfig;
use crate::{log_break, log_info, log_message};
use color_eyre::{eyre::WrapErr, Result};

/// Repository show command
pub struct RepoShowCommand;

impl RepoShowCommand {
    /// Show repository configuration
    pub fn show() -> Result<()> {
        log_message!("Repository Configuration\n");

        // 1. 获取仓库名
        let repo_name = GitRepo::extract_repo_name().wrap_err("Not in a Git repository")?;

        log_info!("Repository: {}", repo_name);
        log_break!();

        // 2. 显示分支配置
        log_message!("Branch Configuration");
        log_break!('-', 40);

        // Load from project-level config only
        let project_prefix = RepoConfig::load().ok().and_then(|config| config.branch.prefix);

        if let Some(prefix) = project_prefix {
            log_info!("Prefix: {} (project-level)", prefix);
        } else {
            log_info!("Prefix: (not set)");
            log_info!("Run 'workflow repo setup' to configure branch prefix");
        }

        // 3. 显示模板配置
        log_break!();
        log_message!("Template Configuration");
        log_break!('-', 40);

        let template_config = TemplateConfig::load().unwrap_or_default();

        // Commit template
        log_info!("Commit Template:");
        log_info!("  Use scope: {}", template_config.commit.use_scope);
        log_info!("  Template: {}", template_config.commit.default);

        // Branch template
        log_break!();
        log_info!("Branch Template:");
        log_info!("  Default: {}", template_config.branch.default);
        if let Some(ref feature) = template_config.branch.feature {
            log_info!("  Feature: {}", feature);
        }
        if let Some(ref bugfix) = template_config.branch.bugfix {
            log_info!("  Bugfix: {}", bugfix);
        }
        if let Some(ref hotfix) = template_config.branch.hotfix {
            log_info!("  Hotfix: {}", hotfix);
        }
        if let Some(ref refactoring) = template_config.branch.refactoring {
            log_info!("  Refactoring: {}", refactoring);
        }
        if let Some(ref chore) = template_config.branch.chore {
            log_info!("  Chore: {}", chore);
        }

        // Pull request template
        log_break!();
        log_info!("Pull Request Template:");
        log_info!("  Template: {}", template_config.pull_requests.default);

        Ok(())
    }
}
