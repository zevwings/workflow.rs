//! Stash pop 命令
//!
//! 应用并删除 stash 条目。

use color_eyre::Result;
use prompt::{confirm, info, select, success, warning};

use crate::registry;

/// Stash pop 命令
pub struct StashPopCommand;

impl StashPopCommand {
    /// 执行 stash pop 命令
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let repo = registry::get_git_repository();

        // 获取所有 stash 条目
        let entries = repo
            .stash_list()
            .map_err(|e| format!("Failed to list stash entries: {}", e))?;

        if entries.is_empty() {
            warning!("No stash entries available");
            return Ok(());
        }

        // 获取最新的 stash 信息
        let latest_stash = entries.first().ok_or("No stash entries available")?;
        let latest_stash_ref = format!("stash@{{{}}}", latest_stash.index);

        // 提示是否应用最新的 stash
        let prompt = format!(
            "Pop latest stash {}?\n  Message: {}\n  Branch: {}",
            latest_stash_ref, latest_stash.message, latest_stash.branch
        );

        let use_latest = confirm!(prompt)
            .default(true)
            .prompt()
            .map_err(|e| format!("Failed to get user confirmation: {}", e))?;

        // 确定要应用的 stash
        let target_index = if use_latest {
            latest_stash.index
        } else {
            // 交互式选择
            let options: Vec<String> = entries
                .iter()
                .map(|entry| {
                    format!(
                        "stash@{{{}}}: {} (On {})",
                        entry.index, entry.message, entry.branch
                    )
                })
                .collect();

            let selected = select!("Select a stash entry", options)
                .default(0)
                .prompt()
                .map_err(|e| format!("Failed to select stash entry: {}", e))?;

            // 从选中的字符串中提取索引
            selected
                .split(':')
                .next()
                .and_then(|s| s.trim().strip_prefix("stash@{"))
                .and_then(|s| s.strip_suffix("}"))
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or("Failed to parse selected stash index")?
        };

        info!("Popping stash: stash@{{{}}}", target_index);

        // 应用并删除 stash
        let result = repo
            .stash_pop(target_index)
            .map_err(|e| format!("Failed to pop stash: {}", e))?;

        if result.restored {
            success!("Stash stash@{{{}}} applied and removed", target_index);
            if let Some(msg) = result.message {
                info!("{}", msg);
            }
        } else {
            warning!("Failed to apply stash: stash@{{{}}}", target_index);
            warning!("The stash entry is kept due to conflicts or errors.");
            for warn in &result.warnings {
                warning!("{}", warn);
            }
        }

        Ok(())
    }
}
