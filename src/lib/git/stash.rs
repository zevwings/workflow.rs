use chrono::{DateTime, Local};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use git2::StashApplyOptions;

use super::helpers::open_repo;
use crate::trace_warn;

/// Stash 条目信息
#[derive(Debug, Clone)]
pub struct StashEntry {
    /// stash@{n} 中的 n
    pub index: usize,
    /// 创建时的分支
    pub branch: String,
    /// stash 消息
    pub message: String,
    /// commit hash
    pub commit_hash: String,
    /// 创建时间
    pub timestamp: Option<DateTime<Local>>,
}

/// Stash 应用结果
#[derive(Debug, Clone)]
pub struct StashApplyResult {
    /// 是否成功应用
    pub applied: bool,
    /// 是否有冲突
    pub has_conflicts: bool,
    /// 消息
    pub message: Option<String>,
    /// 警告消息列表
    pub warnings: Vec<String>,
    /// 统计信息（可选）
    pub stat: Option<StashStat>,
}

/// Stash 统计信息
#[derive(Debug, Clone)]
pub struct StashStat {
    /// 变更的文件数
    pub files_changed: usize,
    /// 插入的行数
    pub insertions: usize,
    /// 删除的行数
    pub deletions: usize,
}

/// Stash 恢复结果
#[derive(Debug, Clone)]
pub struct StashPopResult {
    /// 是否成功恢复
    pub restored: bool,
    /// 消息
    pub message: Option<String>,
    /// 警告消息列表
    pub warnings: Vec<String>,
}

/// Git Stash 管理
///
/// 提供 stash 相关的操作功能，包括：
/// - 保存未提交的修改到 stash
/// - 恢复 stash 中的修改
/// - 检查是否有未合并的文件（冲突）
pub struct GitStash;

impl GitStash {
    /// 保存未提交的修改到 stash
    ///
    /// 使用 git2 将当前工作区和暂存区的未提交修改保存到 stash。
    /// 如果提供了消息，则添加 stash 消息。
    ///
    /// # 参数
    ///
    /// * `message` - 可选的 stash 消息，用于标识这次 stash 的内容
    ///
    /// # 错误
    ///
    /// 如果 stash 操作失败，返回相应的错误信息。
    pub fn stash_push(message: Option<&str>) -> Result<()> {
        let mut repo = open_repo()?;
        let signature = repo.signature().wrap_err("Failed to get signature")?;
        repo.stash_save(&signature, message.unwrap_or(""), None)
            .map(|_| ())
            .wrap_err("Failed to stash changes")
    }

    /// 检查是否有未合并的文件（冲突文件）
    ///
    /// 使用 git2 检查是否有未合并的路径
    /// 返回 true 如果有冲突文件，false 如果没有
    pub fn has_unmerged() -> Result<bool> {
        let repo = open_repo()?;
        let index = repo.index().wrap_err("Failed to open index")?;
        Ok(index.has_conflicts())
    }

    /// 列出所有 stash 条目
    ///
    /// 使用 git2 获取所有 stash 条目的结构化数据。
    ///
    /// # 返回
    ///
    /// 返回所有 stash 条目的列表，按索引从新到旧排列（stash@{0} 在第一个）。
    pub fn stash_list() -> Result<Vec<StashEntry>> {
        let mut repo = open_repo()?;
        let mut stash_data: Vec<(usize, String, git2::Oid)> = Vec::new();

        repo.stash_foreach(|_index, message: &str, oid: &git2::Oid| {
            stash_data.push((_index, message.to_string(), *oid));
            true // 继续遍历
        })
        .wrap_err("Failed to list stash entries")?;

        let mut entries = Vec::new();
        for (_index, full_message, oid) in stash_data {
            let commit = match repo.find_commit(oid) {
                Ok(c) => c,
                Err(_) => continue, // 跳过这个 stash
            };

            let (branch, stash_message) = Self::extract_branch_and_message(&full_message);

            let time = commit.time();
            let offset_seconds = time.offset_minutes() as i32 * 60;
            let tz = chrono::FixedOffset::east_opt(offset_seconds)
                .unwrap_or_else(|| chrono::FixedOffset::east_opt(0).unwrap());
            let timestamp = chrono::DateTime::from_timestamp(time.seconds(), 0)
                .map(|dt| dt.with_timezone(&tz))
                .map(|dt| dt.with_timezone(&Local));

            entries.push(StashEntry {
                index: _index,
                branch,
                message: stash_message,
                commit_hash: oid.to_string(),
                timestamp,
            });
        }

        // 按索引排序（从新到旧，stash@{0} 在第一个）
        entries.sort_by_key(|e| e.index);

        Ok(entries)
    }

    /// 从 stash 完整消息中提取分支名和消息
    ///
    /// stash 消息格式：
    /// - `WIP on <branch>: <message>`
    /// - `On <branch>: <message>`
    fn extract_branch_and_message(full_message: &str) -> (String, String) {
        // 尝试匹配 "WIP on <branch>: " 或 "On <branch>: "
        if let Some(pos) = full_message.find("WIP on ") {
            let after_wip = &full_message[pos + 7..]; // "WIP on " 的长度是 7
            if let Some(colon_pos) = after_wip.find(": ") {
                let branch = after_wip[..colon_pos].to_string();
                let message = after_wip[colon_pos + 2..].to_string();
                return (branch, message);
            }
        } else if let Some(pos) = full_message.find("On ") {
            let after_on = &full_message[pos + 3..]; // "On " 的长度是 3
            if let Some(colon_pos) = after_on.find(": ") {
                let branch = after_on[..colon_pos].to_string();
                let message = after_on[colon_pos + 2..].to_string();
                return (branch, message);
            }
        }

        // 如果无法提取，返回整个消息作为消息，分支为 unknown
        ("unknown".to_string(), full_message.to_string())
    }

    /// 应用指定的 stash（不删除）
    ///
    /// 使用 git2 应用指定的 stash，保留 stash 条目。
    ///
    /// # 参数
    ///
    /// * `stash_ref` - Stash 索引（如 0 表示最新的 stash@{0}），如果为 None 则应用最新的
    ///
    /// # 返回
    ///
    /// 返回 `StashApplyResult`，包含应用状态、冲突信息和警告。
    pub fn stash_apply(stash_ref: Option<&str>) -> Result<StashApplyResult> {
        let mut repo = open_repo()?;
        let stash_index = stash_ref
            .and_then(|s| {
                s.strip_prefix("stash@{")
                    .and_then(|s| s.strip_suffix("}"))
                    .and_then(|s| s.parse::<usize>().ok())
            })
            .unwrap_or(0);

        let mut opts = StashApplyOptions::default();

        let result = repo.stash_apply(stash_index, Some(&mut opts));

        match result {
            Ok(_) => {
                // 检查是否有冲突
                let has_conflicts = Self::has_unmerged().unwrap_or(false);
                Ok(StashApplyResult {
                    applied: true,
                    has_conflicts,
                    message: Some(format!("Stash {{{}}} applied successfully", stash_index)),
                    warnings: if has_conflicts {
                        vec!["Merge conflicts detected. Please resolve them manually.".to_string()]
                    } else {
                        vec![]
                    },
                    stat: Self::stash_show_stat(&format!("stash@{{{}", stash_index)).ok(),
                })
            }
            Err(e) => {
                // 检查是否有冲突
                let has_conflicts = Self::has_unmerged().unwrap_or(false);
                Ok(StashApplyResult {
                    applied: false,
                    has_conflicts,
                    message: None,
                    warnings: vec![
                        format!("Failed to apply stash {{{}}}: {}", stash_index, e),
                        if has_conflicts {
                            "Merge conflicts detected. Please resolve them manually.".to_string()
                        } else {
                            "The stash entry is kept. You can try again later.".to_string()
                        },
                    ],
                    stat: None,
                })
            }
        }
    }

    /// 删除指定的 stash
    ///
    /// 使用 git2 删除指定的 stash 条目。
    ///
    /// # 参数
    ///
    /// * `stash_ref` - Stash 引用（如 "stash@{0}" 或 "stash@{1}"），如果为 None 则删除最新的
    ///
    /// # 错误
    ///
    /// 如果删除失败，返回相应的错误信息。
    pub fn stash_drop(stash_ref: Option<&str>) -> Result<()> {
        let mut repo = open_repo()?;
        let stash_index = stash_ref
            .and_then(|s| {
                s.strip_prefix("stash@{")
                    .and_then(|s| s.strip_suffix("}"))
                    .and_then(|s| s.parse::<usize>().ok())
            })
            .unwrap_or(0);
        repo.stash_drop(stash_index)
            .wrap_err_with(|| format!("Failed to drop stash {{{}}}", stash_index))
    }

    /// 应用并删除指定的 stash
    ///
    /// 使用 git2 应用并删除指定的 stash 条目。
    /// 如果应用失败（冲突），保留 stash 条目。
    ///
    /// # 参数
    ///
    /// * `stash_ref` - Stash 引用（如 "stash@{0}" 或 "stash@{1}"），如果为 None 则应用并删除最新的
    ///
    /// # 返回
    ///
    /// 返回 `StashPopResult`，包含恢复状态、消息和警告信息。
    pub fn stash_pop(stash_ref: Option<&str>) -> Result<StashPopResult> {
        let mut repo = open_repo()?;
        let stash_index = stash_ref
            .and_then(|s| {
                s.strip_prefix("stash@{")
                    .and_then(|s| s.strip_suffix("}"))
                    .and_then(|s| s.parse::<usize>().ok())
            })
            .unwrap_or(0);

        let mut opts = StashApplyOptions::default();

        let result = repo.stash_pop(stash_index, Some(&mut opts));

        match result {
            Ok(_) => Ok(StashPopResult {
                restored: true,
                message: Some(format!("Stash {{{}}} applied and removed", stash_index)),
                warnings: vec![],
            }),
            Err(e) => {
                // 检查是否有未合并的路径（冲突文件）
                if Self::has_unmerged().unwrap_or(false) {
                    let warnings = vec![
                        format!(
                            "Merge conflicts detected when applying stash {{{}}}.",
                            stash_index
                        ),
                        "The stash entry is kept in case you need it again.".to_string(),
                        "Please resolve the conflicts manually and then:".to_string(),
                        "  1. Resolve conflicts in the affected files".to_string(),
                        "  2. Stage the resolved files with: git add <file>".to_string(),
                        "  3. Continue with your workflow".to_string(),
                    ];
                    // 记录到 tracing（用于调试）
                    for warning in &warnings {
                        trace_warn!("{}", warning);
                    }
                    // 返回包含警告的结果，而不是抛出错误
                    Ok(StashPopResult {
                        restored: false,
                        message: None,
                        warnings,
                    })
                } else {
                    // 没有冲突但失败了，返回包含警告的结果
                    let warnings = vec![
                        format!("Failed to apply stash {{{}}}: {}", stash_index, e),
                        "The stash entry is kept. You can try again later.".to_string(),
                    ];
                    // 记录到 tracing（用于调试）
                    for warning in &warnings {
                        trace_warn!("{}", warning);
                    }
                    // 返回包含警告的结果，而不是抛出错误
                    Ok(StashPopResult {
                        restored: false,
                        message: None,
                        warnings,
                    })
                }
            }
        }
    }

    /// 获取 stash 的统计信息
    ///
    /// 使用 git2 获取 stash 的文件变更统计。
    ///
    /// # 参数
    ///
    /// * `stash_ref` - Stash 引用（如 "stash@{0}"）
    ///
    /// # 返回
    ///
    /// 返回 `StashStat`，包含文件变更统计信息。
    pub fn stash_show_stat(stash_ref: &str) -> Result<StashStat> {
        let mut repo = open_repo()?;
        let stash_index = stash_ref
            .strip_prefix("stash@{")
            .and_then(|s| s.strip_suffix("}"))
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        let mut stash_oid = None;
        repo.stash_foreach(|index, _message: &str, oid: &git2::Oid| {
            if index == stash_index {
                stash_oid = Some(*oid);
                false // 停止遍历
            } else {
                true // 继续遍历
            }
        })
        .wrap_err("Failed to find stash")?;

        let stash_oid = stash_oid.ok_or_else(|| eyre!("Stash not found"))?;
        let stash_commit = repo.find_commit(stash_oid).wrap_err("Failed to find stash commit")?;
        let stash_tree = stash_commit.tree().wrap_err("Failed to get stash tree")?;

        // 获取 stash 的父提交（通常是 HEAD）
        let parent_tree = stash_commit
            .parent(0)
            .ok()
            .and_then(|p| p.tree().ok())
            .or_else(|| {
                repo.head()
                    .ok()
                    .and_then(|h| h.peel_to_tree().ok())
                    .and_then(|t| repo.find_tree(t.id()).ok())
            })
            .ok_or_else(|| eyre!("Failed to get parent tree"))?;

        let diff = repo
            .diff_tree_to_tree(Some(&parent_tree), Some(&stash_tree), None)
            .wrap_err("Failed to create diff")?;

        let stats = diff.stats().wrap_err("Failed to get diff stats")?;

        Ok(StashStat {
            files_changed: stats.files_changed(),
            insertions: stats.insertions(),
            deletions: stats.deletions(),
        })
    }
}
