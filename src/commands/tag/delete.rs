//! Tag delete command
//!
//! Delete one or more Git tags (local and/or remote).

use crate::base::dialog::{ConfirmDialog, MultiSelectDialog};
use crate::git::GitTag;
use crate::{log_break, log_info, log_message, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};
use regex::Regex;

/// Tag delete command
pub struct TagDeleteCommand;

impl TagDeleteCommand {
    /// Execute the tag delete command
    pub fn execute(
        tag_name: Option<String>,
        local_only: bool,
        remote_only: bool,
        pattern: Option<String>,
        dry_run: bool,
        force: bool,
    ) -> Result<()> {
        log_break!();
        log_message!("Tag Delete");

        // 获取所有 tag
        let all_tags = GitTag::list_all_tags().wrap_err("Failed to list tags")?;

        if all_tags.is_empty() {
            log_info!("No tags found");
            return Ok(());
        }

        // 确定要删除的 tag 列表
        let tags_to_delete = if let Some(pattern_str) = pattern {
            // 模式匹配
            Self::filter_tags_by_pattern(&all_tags, &pattern_str)?
        } else if let Some(tag) = tag_name {
            // 指定 tag 名称
            vec![tag]
        } else {
            // 交互式选择
            Self::select_tags_interactively(&all_tags)?
        };

        if tags_to_delete.is_empty() {
            log_info!("No tags selected for deletion");
            return Ok(());
        }

        // 获取要删除的 tag 信息
        let mut tags_info = Vec::new();
        for tag_name in &tags_to_delete {
            match GitTag::get_tag_info(tag_name) {
                Ok(info) => tags_info.push(info),
                Err(e) => {
                    log_warning!("Tag '{}' not found: {}", tag_name, e);
                }
            }
        }

        if tags_info.is_empty() {
            log_info!("No valid tags to delete");
            return Ok(());
        }

        // 显示预览
        log_break!();
        log_message!("Tags to be deleted:");
        for info in &tags_info {
            let locations = {
                let mut locs = Vec::new();
                if info.exists_local {
                    locs.push("local");
                }
                if info.exists_remote {
                    locs.push("remote");
                }
                locs.join(" + ")
            };
            log_info!(
                "  {} (commit: {}, locations: {})",
                info.name,
                &info.commit_hash[..8.min(info.commit_hash.len())],
                locations
            );
        }

        // Dry-run 模式
        if dry_run {
            log_break!();
            log_info!("Dry-run mode: tags will not be actually deleted");
            return Ok(());
        }

        // 确认删除（除非使用 force）
        if !force {
            let delete_what = if local_only {
                "local"
            } else if remote_only {
                "remote"
            } else {
                "local and remote"
            };

            let confirmed = ConfirmDialog::new(format!(
                "Are you sure you want to delete {} tag(s) ({})?",
                tags_info.len(),
                delete_what
            ))
            .with_default(false)
            .prompt()
            .wrap_err("Failed to get user confirmation")?;

            if !confirmed {
                log_info!("Operation cancelled");
                return Ok(());
            }
        }

        // 执行删除
        let mut deleted_local = 0;
        let mut deleted_remote = 0;
        let mut failed = 0;

        for info in &tags_info {
            let tag_name = &info.name;

            // 确定删除范围
            let should_delete_local = !remote_only && info.exists_local;
            let should_delete_remote = !local_only && info.exists_remote;

            // 删除本地 tag
            if should_delete_local {
                match GitTag::delete_local(tag_name) {
                    Ok(_) => {
                        log_success!("Deleted local tag: {}", tag_name);
                        deleted_local += 1;
                    }
                    Err(e) => {
                        log_warning!("Failed to delete local tag {}: {}", tag_name, e);
                        failed += 1;
                    }
                }
            }

            // 删除远程 tag
            if should_delete_remote {
                match GitTag::delete_remote(tag_name) {
                    Ok(_) => {
                        log_success!("Deleted remote tag: {}", tag_name);
                        deleted_remote += 1;
                    }
                    Err(e) => {
                        log_warning!("Failed to delete remote tag {}: {}", tag_name, e);
                        failed += 1;
                    }
                }
            }
        }

        // 显示结果
        log_break!();
        if deleted_local > 0 || deleted_remote > 0 {
            log_success!("Deletion completed!");
            if deleted_local > 0 {
                log_info!("Deleted {} local tag(s)", deleted_local);
            }
            if deleted_remote > 0 {
                log_info!("Deleted {} remote tag(s)", deleted_remote);
            }
        }
        if failed > 0 {
            log_warning!("Failed to delete {} tag(s)", failed);
        }

        Ok(())
    }

    /// 通过模式匹配过滤 tag
    fn filter_tags_by_pattern(tags: &[crate::git::TagInfo], pattern: &str) -> Result<Vec<String>> {
        // 将 shell 通配符模式转换为正则表达式
        let regex_pattern = pattern.replace(".", "\\.").replace("*", ".*").replace("?", ".");

        let regex = Regex::new(&format!("^{}$", regex_pattern))
            .wrap_err_with(|| format!("Invalid pattern: {}", pattern))?;

        let matched: Vec<String> = tags
            .iter()
            .filter(|tag| regex.is_match(&tag.name))
            .map(|tag| tag.name.clone())
            .collect();

        Ok(matched)
    }

    /// 交互式选择 tag
    fn select_tags_interactively(tags: &[crate::git::TagInfo]) -> Result<Vec<String>> {
        // 构建选项列表，格式：<tag_name> (commit: <hash>, local/remote/both)
        let options: Vec<String> = tags
            .iter()
            .map(|tag| {
                let locations = {
                    let mut locs = Vec::new();
                    if tag.exists_local {
                        locs.push("local");
                    }
                    if tag.exists_remote {
                        locs.push("remote");
                    }
                    let loc_str = if locs.is_empty() {
                        "none".to_string()
                    } else {
                        locs.join("+")
                    };
                    format!(
                        "{} (commit: {}, {})",
                        tag.name,
                        &tag.commit_hash[..8.min(tag.commit_hash.len())],
                        loc_str
                    )
                };
                format!("{} ({})", tag.name, locations)
            })
            .collect();

        // 多选列表
        let selected = MultiSelectDialog::new("Select tags to delete", options)
            .prompt()
            .wrap_err("Failed to select tags")?;

        if selected.is_empty() {
            return Ok(Vec::new());
        }

        // 从选中的字符串中提取 tag 名称
        let tag_names: Vec<String> = selected
            .iter()
            .filter_map(|s| {
                // 格式：<tag_name> (commit: <hash>, <locations>)
                s.split(' ').next().map(|name| name.trim().to_string())
            })
            .collect();

        Ok(tag_names)
    }
}
