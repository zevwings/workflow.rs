//! Alpha Tag 清理命令
//!
//! 清理已合并到 master 分支的 alpha tag。

use crate::git::{GitCommand, GitTag};
use crate::{log_break, log_info, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};

/// Alpha Tag 清理命令
pub struct TagCleanupCommand {
    merge_commit_sha: String,
    current_version: String,
    ci: bool,
}

impl TagCleanupCommand {
    /// 创建新的 Alpha Tag 清理命令
    pub fn new(merge_commit_sha: String, current_version: String, ci: bool) -> Self {
        Self {
            merge_commit_sha,
            current_version,
            ci,
        }
    }

    /// 清理 alpha tags
    pub fn cleanup(&self) -> Result<()> {
        log_break!('=');
        log_info!("清理 Alpha Tags");
        log_break!('=');
        log_break!();

        log_info!("合并提交 SHA: {}", self.merge_commit_sha);
        log_info!("当前版本: {}", self.current_version);

        // 提取基础版本号（移除 'v' 前缀和 alpha 后缀）
        let base_version = self
            .current_version
            .trim_start_matches('v')
            .split('.')
            .take(3)
            .collect::<Vec<_>>()
            .join(".");

        log_info!("基础版本号: {}", base_version);
        log_break!();

        // 获取 master 分支的 first parent（合并前的最后一个提交）
        let first_parent = GitCommand::new(["rev-parse", &format!("{}^1", self.merge_commit_sha)])
            .read()
            .wrap_err("Failed to get first parent commit")?;
        log_info!("First parent (master before merge): {}", first_parent);

        // 获取 master 分支的当前 HEAD（合并后的状态）
        let master_head = GitCommand::new(["rev-parse", "HEAD"]).read()?;
        log_info!("Master HEAD (after merge): {}", master_head);
        log_break!();

        // 查找所有 alpha tag
        log_info!("查找 alpha tags...");
        let alpha_tags = GitTag::list_alpha_tags()?;

        if alpha_tags.is_empty() {
            log_success!("未找到 alpha tags，无需清理");
            if self.ci {
                self.output_ci_result(0)?;
            }
            return Ok(());
        }

        log_info!("找到 {} 个 alpha tags:", alpha_tags.len());
        for tag in &alpha_tags {
            log_info!("   - {}", tag);
        }
        log_break!();

        // 检查每个 alpha tag 是否指向已合并的提交
        log_info!("检查哪些 alpha tags 指向已合并的提交...");
        let mut tags_to_delete = Vec::new();

        for tag in &alpha_tags {
            let tag_commit = match GitCommand::new(["rev-parse", tag]).read() {
                Ok(sha) => sha,
                Err(_) => {
                    log_warning!("Tag {}: 无法解析 commit", tag);
                    continue;
                }
            };

            // 提取 tag 的版本号
            let tag_version = GitTag::extract_version(tag);

            // 检查 tag 是否在 master 分支的 first-parent 路径上
            if GitTag::is_ancestor(&tag_commit, &first_parent) {
                // Tag 在 master 的 first-parent 路径上，保留它
                log_info!(
                    "   ⏭️  Tag {} ({}) 在 master 分支 first-parent 路径上，保留",
                    tag,
                    tag_commit
                );
            } else if GitTag::is_ancestor(&tag_commit, &master_head) {
                // Tag 在合并提交的祖先中，但不在 first-parent 路径上
                // 说明它来自已合并的分支，应该删除
                if let Some(ref tv) = tag_version {
                    if tv == &base_version {
                        log_info!(
                            "   ✅ Tag {} ({}) 版本 {} 匹配当前版本 {} 且来自已合并分支，将删除",
                            tag,
                            tag_commit,
                            tv,
                            base_version
                        );
                    } else {
                        log_info!("   ✅ Tag {} ({}) 来自已合并分支，将删除", tag, tag_commit);
                    }
                } else {
                    log_info!("   ✅ Tag {} ({}) 来自已合并分支，将删除", tag, tag_commit);
                }
                tags_to_delete.push(tag.clone());
            } else {
                // Tag 不在合并提交的祖先中，检查版本号是否匹配
                if let Some(ref tv) = tag_version {
                    if tv == &base_version {
                        log_warning!(
                            "   ⚠️  Tag {} ({}) 版本 {} 匹配当前版本 {} 但 commit 不在合并祖先中",
                            tag,
                            tag_commit,
                            tv,
                            base_version
                        );
                        log_info!("   💡 由于版本号匹配，考虑删除...");
                        tags_to_delete.push(tag.clone());
                    } else {
                        log_info!("   ⏭️  Tag {} ({}) 与此合并无关，保留", tag, tag_commit);
                    }
                } else {
                    log_info!("   ⏭️  Tag {} ({}) 与此合并无关，保留", tag, tag_commit);
                }
            }
        }

        if tags_to_delete.is_empty() {
            log_break!();
            log_success!("没有需要删除的 alpha tags");
            if self.ci {
                self.output_ci_result(0)?;
            }
            return Ok(());
        }

        log_break!();
        log_info!("删除 {} 个 alpha tags...", tags_to_delete.len());

        // 删除本地 tag
        for tag in &tags_to_delete {
            log_info!("删除本地 tag: {}", tag);
            if let Err(e) = GitTag::delete_local(tag) {
                log_warning!("   删除本地 tag 失败: {} (可能不存在)", e);
            }
        }

        // 删除远程 tag
        log_break!();
        log_info!("删除远程 tags...");
        let mut deleted_count = 0;
        for tag in &tags_to_delete {
            log_info!("删除远程 tag: {}", tag);
            if let Err(e) = GitTag::delete_remote(tag) {
                log_warning!("   删除远程 tag 失败: {} (可能不存在或已删除)", e);
            } else {
                deleted_count += 1;
            }
        }

        log_break!();
        log_success!("清理完成: 删除了 {} 个 alpha tag(s)", deleted_count);

        // CI 模式：输出到 GITHUB_OUTPUT
        if self.ci {
            self.output_ci_result(deleted_count)?;
        }

        Ok(())
    }

    /// 输出 CI 模式结果到 GITHUB_OUTPUT
    fn output_ci_result(&self, deleted_count: usize) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        if let Ok(output_file) = std::env::var("GITHUB_OUTPUT") {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&output_file)
                .wrap_err_with(|| format!("Failed to open GITHUB_OUTPUT: {}", output_file))?;

            writeln!(file, "deleted_count={}", deleted_count)
                .wrap_err("Failed to write deleted_count")?;
        }

        Ok(())
    }
}
