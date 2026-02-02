//! 创建 Tag 命令

use color_eyre::Result;
use domain::{TagCreateScope, TagDeleteScope};
use prompt::{error, info, success, warning};

use crate::registry;

/// Tag Create 命令
pub struct TagCreateCommand {
    tag_name: String,
    target: Option<String>,
    message: Option<String>,
    local: bool,
    force: bool,
}

impl TagCreateCommand {
    /// 创建新的 TagCreateCommand
    pub fn new(
        tag_name: String,
        target: Option<String>,
        message: Option<String>,
        local: bool,
        force: bool,
    ) -> Self {
        Self {
            tag_name,
            target,
            message,
            local,
            force,
        }
    }

    /// 运行 `workflow tag create` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let tag_repo = registry::get_git_repository();

        // 检查 tag 是否已存在
        let (exists_local, exists_remote) = tag_repo
            .has_tag(&self.tag_name)
            .map_err(|e| format!("Failed to check tag existence: {}", e))?;

        if (exists_local || exists_remote) && !self.force {
            error!("Tag '{}' already exists", self.tag_name);
            if exists_local {
                info!("  Local tag exists");
            }
            if exists_remote {
                info!("  Remote tag exists");
            }
            error!("Use --force to overwrite existing tag");
            return Err(format!("Tag '{}' already exists", self.tag_name).into());
        }

        // 如果 tag 已存在且使用 force，先删除
        if (exists_local || exists_remote) && self.force {
            warning!("Tag '{}' already exists, will overwrite", self.tag_name);
            let scope = if exists_local && exists_remote {
                TagDeleteScope::Both
            } else if exists_local {
                TagDeleteScope::Local
            } else {
                TagDeleteScope::Remote
            };

            tag_repo
                .delete_tag(&self.tag_name, scope, true)
                .map_err(|e| format!("Failed to delete existing tag: {}", e))?;
        }

        // 确定创建范围
        let create_scope = if self.local {
            TagCreateScope::Local
        } else {
            TagCreateScope::Both
        };

        // 创建 tag
        let tag_type = if self.message.is_some() {
            "annotated tag"
        } else {
            "lightweight tag"
        };

        info!("Creating {} '{}'...", tag_type, self.tag_name);

        if let Some(target) = &self.target {
            info!("  Target: {}", target);
        }
        if let Some(msg) = &self.message {
            info!("  Message: {}", msg);
        }

        let create_info = tag_repo
            .create_tag(
                &self.tag_name,
                self.target.as_deref(),
                self.message.as_deref(),
                create_scope,
                self.force,
            )
            .map_err(|e| format!("Failed to create tag: {}", e))?;

        // 显示创建结果
        if create_info.created_local {
            success!("Created local tag '{}'", self.tag_name);
        }
        if create_info.created_remote {
            success!("Pushed tag '{}' to remote", self.tag_name);
        }

        Ok(())
    }
}
