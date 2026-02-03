//! 删除 Tag 命令

use domain::TagDeleteScope;
use prompt::{confirm, error, info, select, success, warning};

use crate::registry;

/// Tag Remove 命令
pub struct TagRemoveCommand {
    tag_name: Option<String>,
    local: bool,
    remote: bool,
    pattern: Option<String>,
    dry_run: bool,
    force: bool,
}

impl TagRemoveCommand {
    /// 创建新的 TagRemoveCommand
    pub fn new(
        tag_name: Option<String>,
        local: bool,
        remote: bool,
        pattern: Option<String>,
        dry_run: bool,
        force: bool,
    ) -> Self {
        Self {
            tag_name,
            local,
            remote,
            pattern,
            dry_run,
            force,
        }
    }

    /// 运行 `workflow tag remove` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let tag_repo = registry::get_git_repository();

        // 确定删除范围
        let scope = if self.local {
            TagDeleteScope::Local
        } else if self.remote {
            TagDeleteScope::Remote
        } else {
            TagDeleteScope::Both
        };

        // 如果提供了 pattern，使用批量删除
        if let Some(pattern) = &self.pattern {
            return self.remove_by_pattern(tag_repo.as_ref(), pattern, scope);
        }

        // 确定要删除的 tag
        let target_tag = if let Some(name) = &self.tag_name {
            name.clone()
        } else {
            // 交互式选择 tag
            let tags =
                tag_repo.list_tags(true).map_err(|e| format!("Failed to list tags: {}", e))?;

            if tags.is_empty() {
                error!("No tags found");
                return Err("No tags available".into());
            }

            select!("Select tag to remove:", tags)
                .prompt()
                .map_err(|e| format!("Failed to select tag: {}", e))?
        };

        // 检查 tag 是否存在
        let (exists_local, exists_remote) = tag_repo
            .has_tag(&target_tag)
            .map_err(|e| format!("Failed to check tag existence: {}", e))?;

        if !exists_local && !exists_remote {
            error!("Tag '{}' not found", target_tag);
            return Err(format!("Tag '{}' not found", target_tag).into());
        }

        // 根据 scope 确定实际要删除的范围
        let remove_local = match scope {
            TagDeleteScope::Local => exists_local,
            TagDeleteScope::Remote => false,
            TagDeleteScope::Both => exists_local,
        };
        let remove_remote = match scope {
            TagDeleteScope::Local => false,
            TagDeleteScope::Remote => exists_remote,
            TagDeleteScope::Both => exists_remote,
        };

        if !remove_local && !remove_remote {
            error!("No tags to remove");
            return Err("No tags to remove".into());
        }

        // 预览模式
        if self.dry_run {
            let preview_info = tag_repo
                .preview_delete(Some(&target_tag), None, scope)
                .map_err(|e| format!("Failed to preview remove: {}", e))?;

            if preview_info.is_empty() {
                info!("[DRY RUN] No tags would be removed");
                return Ok(());
            }

            for info in preview_info {
                if info.exists_local && remove_local {
                    info!("[DRY RUN] Would remove local tag '{}'", info.name);
                }
                if info.exists_remote && remove_remote {
                    info!("[DRY RUN] Would remove remote tag '{}'", info.name);
                }
            }
            return Ok(());
        }

        // 确认删除（除非使用 --force）
        if !self.force {
            let mut confirm_msg = format!("Remove tag '{}'?", target_tag);
            if remove_local && remove_remote {
                confirm_msg.push_str(" (local and remote)");
            } else if remove_local {
                confirm_msg.push_str(" (local only)");
            } else if remove_remote {
                confirm_msg.push_str(" (remote only)");
            }

            let confirmed = confirm!(confirm_msg)
                .default(false)
                .prompt()
                .map_err(|e| format!("Failed to get confirmation: {}", e))?;

            if !confirmed {
                info!("Tag removal cancelled");
                return Ok(());
            }
        }

        // 执行删除
        let remove_info = tag_repo
            .delete_tag(&target_tag, scope, self.force)
            .map_err(|e| format!("Failed to remove tag: {}", e))?;

        // 显示删除结果
        if remove_info.exists_local && remove_local {
            success!("Removed local tag '{}'", target_tag);
        }
        if remove_info.exists_remote && remove_remote {
            success!("Removed remote tag '{}'", target_tag);
        }

        Ok(())
    }

    /// 按模式删除 tag
    fn remove_by_pattern(
        &self,
        tag_repo: &dyn domain::GitRepository,
        pattern: &str,
        scope: TagDeleteScope,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 预览模式
        if self.dry_run {
            let preview_info = tag_repo
                .preview_delete(None, Some(pattern), scope)
                .map_err(|e| format!("Failed to preview remove: {}", e))?;

            if preview_info.is_empty() {
                info!("[DRY RUN] No tags matching pattern '{}' found", pattern);
                return Ok(());
            }

            info!(
                "[DRY RUN] Would remove {} tag(s) matching pattern '{}':",
                preview_info.len(),
                pattern
            );
            for info in preview_info {
                if info.exists_local {
                    info!("  - {} (local)", info.name);
                }
                if info.exists_remote {
                    info!("  - {} (remote)", info.name);
                }
            }
            return Ok(());
        }

        // 确认删除（除非使用 --force）
        if !self.force {
            let confirmed = confirm!("Remove all tags matching pattern '{}'?", pattern)
                .default(false)
                .prompt()
                .map_err(|e| format!("Failed to get confirmation: {}", e))?;

            if !confirmed {
                info!("Tag removal cancelled");
                return Ok(());
            }
        }

        // 执行批量删除
        let remove_results = tag_repo
            .delete_tags_by_pattern(pattern, scope, self.force)
            .map_err(|e| format!("Failed to remove tags: {}", e))?;

        if remove_results.is_empty() {
            warning!("No tags matching pattern '{}' found", pattern);
            return Ok(());
        }

        success!(
            "Removed {} tag(s) matching pattern '{}'",
            remove_results.len(),
            pattern
        );

        Ok(())
    }
}
