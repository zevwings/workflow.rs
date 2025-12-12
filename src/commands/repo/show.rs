//! Repository show command
//!
//! Display current repository configuration.

use crate::branch::config::BranchConfig;
use crate::git::GitRepo;
use crate::template::config::TemplateConfig;
use crate::{log_break, log_info, log_message};
use anyhow::{Context, Result};

/// Repository show command
pub struct RepoShowCommand;

impl RepoShowCommand {
    /// Show repository configuration
    pub fn show() -> Result<()> {
        log_message!("Repository Configuration\n");

        // 1. 获取仓库名
        let repo_name = GitRepo::extract_repo_name()
            .context("Not in a Git repository")?;

        log_info!("Repository: {}", repo_name);
        log_break!();

        // 2. 显示分支配置
        log_message!("Branch Configuration");
        log_break!('-', 40);

        if let Some(prefix) = BranchConfig::get_prefix_for_current_repo() {
            log_info!("Prefix: {}", prefix);
        } else {
            log_info!("Prefix: (not set)");
        }

        // 3. 显示模板配置
        log_break!();
        log_message!("Template Configuration");
        log_break!('-', 40);

        let template_config = TemplateConfig::load().unwrap_or_default();
        // TODO: 显示 use_scope 和 auto_extract_scope（需要先实现这些字段）
        log_info!("Template: {}", template_config.commit.default);

        Ok(())
    }
}
