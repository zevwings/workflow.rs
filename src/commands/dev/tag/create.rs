//! Git Tag 创建命令
//!
//! 提供创建和推送 Git tag 的功能。

use crate::git::{GitCommand, GitTag};
use crate::{log_break, log_error, log_info, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};

/// Git Tag 创建命令
pub struct TagCreateCommand {
    tag: String,
    commit_sha: Option<String>,
    ci: bool,
}

impl TagCreateCommand {
    /// 创建新的 Tag 创建命令
    pub fn new(tag: String, commit_sha: Option<String>, ci: bool) -> Self {
        Self {
            tag,
            commit_sha,
            ci,
        }
    }

    /// 创建并推送 tag
    pub fn create(&self) -> Result<()> {
        log_break!('=');
        log_info!("创建 Git Tag");
        log_break!('=');
        log_break!();

        let tag_name = &self.tag;
        log_info!("Tag 名称: {}", tag_name);

        // 检查 tag 是否已存在
        let (exists_local, exists_remote) = GitTag::is_tag_exists(tag_name)?;

        if exists_local || exists_remote {
            log_warning!("Tag 已存在");
            log_info!("   本地存在: {}", exists_local);
            log_info!("   远程存在: {}", exists_remote);

            // 获取现有 tag 的 commit SHA
            let existing_tag_info = GitTag::get_tag_info(tag_name)?;
            let current_head = GitCommand::new(["rev-parse", "HEAD"]).read().unwrap_or_default();
            let target_sha = self.commit_sha.as_deref().unwrap_or(&current_head);

            log_info!("   现有 tag commit: {}", existing_tag_info.commit_hash);
            log_info!("   目标 commit: {}", target_sha);

            if existing_tag_info.commit_hash == target_sha {
                log_success!("Tag 已存在且指向正确的 commit");
                if self.ci {
                    self.output_ci_result(true, tag_name)?;
                }
                return Ok(());
            } else {
                log_warning!("Tag 已存在但指向不同的 commit");
                log_warning!("   删除现有 tag 并重新创建...");

                if exists_local {
                    GitTag::delete_local(tag_name)?;
                    log_success!("已删除本地 tag");
                }
                if exists_remote {
                    GitTag::delete_remote(tag_name)?;
                    log_success!("已删除远程 tag");
                }
            }
        }

        // 创建 tag
        log_break!();
        log_info!("创建 tag...");
        GitTag::create(tag_name, self.commit_sha.as_deref())?;
        log_success!("Tag 创建成功");

        // 推送 tag
        log_break!();
        log_info!("推送 tag 到远程...");
        match GitTag::push(tag_name) {
            Ok(()) => {
                log_success!("Tag 推送成功");
            }
            Err(e) => {
                // 检查 tag 是否已存在于远程且指向正确的 commit
                if let Ok(remote_tags) = GitTag::list_remote_tags() {
                    if remote_tags.contains(&tag_name.to_string()) {
                        let remote_tag_info = GitTag::get_tag_info(tag_name)?;
                        let current_head =
                            GitCommand::new(["rev-parse", "HEAD"]).read().unwrap_or_default();
                        let target_sha =
                            self.commit_sha.as_deref().unwrap_or(current_head.as_str());

                        if remote_tag_info.commit_hash == target_sha {
                            log_success!("Tag 已存在于远程且指向正确的 commit");
                            if self.ci {
                                self.output_ci_result(true, tag_name)?;
                            }
                            return Ok(());
                        } else {
                            log_error!("Tag 存在于远程但指向不同的 commit");
                            log_error!("   远程 tag commit: {}", remote_tag_info.commit_hash);
                            log_error!("   目标 commit: {}", target_sha);
                            return Err(e);
                        }
                    }
                }
                return Err(e);
            }
        }

        log_break!();
        log_success!("Tag 创建和推送完成");
        log_info!("   Tag: {}", tag_name);
        if let Some(ref sha) = self.commit_sha {
            log_info!("   Commit: {}", sha);
        } else {
            let current_sha = GitCommand::new(["rev-parse", "HEAD"]).read()?;
            log_info!("   Commit: {}", current_sha);
        }
        log_break!();

        // CI 模式：输出到 GITHUB_OUTPUT
        if self.ci {
            self.output_ci_result(true, tag_name)?;
        }

        Ok(())
    }

    /// 输出 CI 模式结果到 GITHUB_OUTPUT
    fn output_ci_result(&self, success: bool, tag_name: &str) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        if let Ok(output_file) = std::env::var("GITHUB_OUTPUT") {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&output_file)
                .wrap_err_with(|| format!("Failed to open GITHUB_OUTPUT: {}", output_file))?;

            writeln!(file, "tag={}", tag_name).wrap_err("Failed to write tag")?;
            writeln!(file, "tag_created={}", success).wrap_err("Failed to write tag_created")?;
        }

        Ok(())
    }
}
