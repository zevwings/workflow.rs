//! Repository show command
//!
//! Display current repository configuration.

use crate::config::template::config::TemplateConfig;
use crate::domain::repo::config::RepoConfig;
use crate::services::git::GitRepo;
use crate::{br, info, separator};
use color_eyre::{eyre::WrapErr, Result};

/// Repository show command
pub struct RepoShowCommand;

impl RepoShowCommand {
    /// Show repository configuration
    pub fn show() -> Result<()> {
        info!("Repository Configuration\n");

        // 1. 获取仓库名
        let repo_name = GitRepo::extract_repo_name().wrap_err("Not in a Git repository")?;

        info!("Repository: {}", repo_name);
        br!();

        // 2. 显示分支配置
        info!("Branch Configuration");
        separator!('-', 40);

        // Load from personal preference config
        let prefix = RepoConfig::get_branch_prefix();

        if let Some(prefix) = prefix {
            info!("Prefix: {} (personal preference)", prefix);
        } else {
            info!("Prefix: (not set)");
            info!("Run 'workflow repo setup' to configure branch prefix");
        }

        // 3. 显示模板配置
        br!();
        info!("Template Configuration");
        separator!('-', 40);

        let template_config = TemplateConfig::load().unwrap_or_default();

        // Commit template
        info!("Commit Template:");
        info!("  Use scope: {}", template_config.commit.use_scope);
        info!("  Template: {}", template_config.commit.default);

        // Branch template
        br!();
        info!("Branch Template:");
        info!("  Default: {}", template_config.branch.default);
        if let Some(ref feature) = template_config.branch.feature {
            info!("  Feature: {}", feature);
        }
        if let Some(ref bugfix) = template_config.branch.bugfix {
            info!("  Bugfix: {}", bugfix);
        }
        if let Some(ref hotfix) = template_config.branch.hotfix {
            info!("  Hotfix: {}", hotfix);
        }
        if let Some(ref refactoring) = template_config.branch.refactoring {
            info!("  Refactoring: {}", refactoring);
        }
        if let Some(ref chore) = template_config.branch.chore {
            info!("  Chore: {}", chore);
        }

        // Pull request template
        br!();
        info!("Pull Request Template:");
        info!("  Template: {}", template_config.pull_requests.default);

        Ok(())
    }
}
