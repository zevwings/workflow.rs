use chrono::{DateTime, FixedOffset, Local, TimeZone};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};

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
    /// 使用 git2 库将当前工作区和暂存区的未提交修改保存到 stash。
    /// 如果提供了消息，则使用该消息作为 stash 消息。
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

        // 获取签名
        let signature = repo.signature().wrap_err("Failed to get repository signature")?;

        // 构建 stash 消息
        let stash_message = if let Some(msg) = message {
            msg.to_string()
        } else {
            // 如果没有提供消息，使用默认格式
            let branch =
                super::GitBranch::current_branch().unwrap_or_else(|_| "unknown".to_string());
            format!(
                "WIP on {}: {}",
                branch,
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            )
        };

        // 保存 stash
        repo.stash_save(&signature, &stash_message, None)
            .wrap_err("Failed to stash changes")?;

        Ok(())
    }

    /// 检查是否有未合并的文件（冲突文件）
    ///
    /// 使用 git2 库检查是否有未合并的路径（冲突文件）。
    /// 返回 true 如果有冲突文件，false 如果没有
    pub fn has_unmerged() -> Result<bool> {
        let repo = open_repo()?;

        // 使用 index 检查未合并的文件
        let _index = repo.index().wrap_err("Failed to open repository index")?;

        // 遍历索引条目，查找未合并的文件
        // git2 的 IndexEntry 不直接暴露 stage，我们使用 statuses 检查冲突

        // 更可靠的方法：使用 statuses 检查是否有冲突状态
        let mut status_options = git2::StatusOptions::new();
        status_options.include_untracked(false);
        status_options.include_ignored(false);

        let statuses = repo
            .statuses(Some(&mut status_options))
            .wrap_err("Failed to get repository statuses")?;

        // 检查是否有冲突状态的文件
        for entry in statuses.iter() {
            let status = entry.status();
            // 检查是否有冲突标记（CONFLICTED 状态）
            if status.contains(git2::Status::CONFLICTED) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// 列出所有 stash 条目
    ///
    /// 使用 git2 库获取所有 stash 条目的结构化数据。
    ///
    /// # 返回
    ///
    /// 返回所有 stash 条目的列表，按索引从新到旧排列（stash@{0} 在第一个）。
    pub fn stash_list() -> Result<Vec<StashEntry>> {
        let mut repo = open_repo()?;
        let mut entries = Vec::new();

        // 使用 stash_foreach 遍历所有 stash
        // 注意：需要先收集所有 stash OID，然后在回调外处理，避免借用冲突
        let mut stash_oids = Vec::new();
        let mut stash_messages = Vec::new();

        repo.stash_foreach(|_stash_index, message, stash_oid| {
            stash_oids.push(*stash_oid);
            stash_messages.push(message.to_string());
            true // 继续遍历
        })
        .wrap_err("Failed to list stash entries")?;

        // 现在处理每个 stash
        for (idx, stash_oid) in stash_oids.iter().enumerate() {
            // 获取 commit 对象以获取更多信息
            if let Ok(commit) = repo.find_commit(*stash_oid) {
                // 获取时间戳（使用与 commit.rs 相同的方式）
                let time = commit.time();
                let offset = FixedOffset::east_opt(time.offset_minutes() * 60)
                    .unwrap_or_else(|| FixedOffset::east_opt(0).unwrap());
                let timestamp = offset
                    .timestamp_opt(time.seconds(), 0)
                    .single()
                    .map(|dt| dt.with_timezone(&Local));

                // 获取完整消息
                let full_message = stash_messages.get(idx).cloned().unwrap_or_default();

                // 从完整消息中提取分支名和消息
                let (branch, message) = Self::extract_branch_and_message(&full_message);

                entries.push(StashEntry {
                    index: idx,
                    branch,
                    message,
                    commit_hash: stash_oid.to_string(),
                    timestamp,
                });
            }
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
    /// 使用 git2 库应用指定的 stash，保留 stash 条目。
    ///
    /// # 参数
    ///
    /// * `stash_ref` - Stash 引用（如 "stash@{0}" 或 "stash@{1}"），如果为 None 则应用最新的
    ///
    /// # 返回
    ///
    /// 返回 `StashApplyResult`，包含应用状态、冲突信息和警告。
    pub fn stash_apply(stash_ref: Option<&str>) -> Result<StashApplyResult> {
        let mut repo = open_repo()?;

        // 解析 stash 索引
        let stash_index = if let Some(ref_str) = stash_ref {
            // 从 "stash@{n}" 中提取索引
            ref_str
                .strip_prefix("stash@{")
                .and_then(|s| s.strip_suffix("}"))
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or_else(|| eyre!("Invalid stash reference: {}", ref_str))?
        } else {
            0 // 默认应用最新的（索引 0）
        };

        // 应用 stash
        let mut apply_options = git2::StashApplyOptions::new();
        // 注意：git2 0.18 版本中 StashApplyOptions 没有 reinstate_index 方法
        // 默认行为是不恢复索引状态

        let result = repo.stash_apply(stash_index, Some(&mut apply_options));

        match result {
            Ok(_) => {
                // 检查是否有冲突
                let has_conflicts = Self::has_unmerged().unwrap_or(false);
                Ok(StashApplyResult {
                    applied: true,
                    has_conflicts,
                    message: Some(format!(
                        "Stash {} applied successfully",
                        stash_ref.unwrap_or("stash@{0}")
                    )),
                    warnings: if has_conflicts {
                        vec!["Merge conflicts detected. Please resolve them manually.".to_string()]
                    } else {
                        vec![]
                    },
                    stat: Self::stash_show_stat(stash_ref.unwrap_or("stash@{0}")).ok(),
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
                        format!(
                            "Failed to apply stash {}: {}",
                            stash_ref.unwrap_or("stash@{0}"),
                            e
                        ),
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
    /// 使用 git2 库删除指定的 stash 条目。
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

        // 解析 stash 索引
        let stash_index = if let Some(ref_str) = stash_ref {
            // 从 "stash@{n}" 中提取索引
            ref_str
                .strip_prefix("stash@{")
                .and_then(|s| s.strip_suffix("}"))
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or_else(|| eyre!("Invalid stash reference: {}", ref_str))?
        } else {
            0 // 默认删除最新的（索引 0）
        };

        // 删除 stash
        repo.stash_drop(stash_index)
            .wrap_err_with(|| format!("Failed to drop stash {}", stash_ref.unwrap_or("stash@{0}")))
    }

    /// 应用并删除指定的 stash
    ///
    /// 使用 git2 库应用并删除指定的 stash 条目。
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

        // 解析 stash 索引
        let stash_index = if let Some(ref_str) = stash_ref {
            // 从 "stash@{n}" 中提取索引
            ref_str
                .strip_prefix("stash@{")
                .and_then(|s| s.strip_suffix("}"))
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or_else(|| eyre!("Invalid stash reference: {}", ref_str))?
        } else {
            0 // 默认应用并删除最新的（索引 0）
        };

        // 先应用 stash
        let mut apply_options = git2::StashApplyOptions::new();
        // 注意：git2 0.18 版本中 StashApplyOptions 没有 reinstate_index 方法
        // 默认行为是不恢复索引状态

        let apply_result = repo.stash_apply(stash_index, Some(&mut apply_options));

        match apply_result {
            Ok(_) => {
                // 应用成功，删除 stash
                repo.stash_drop(stash_index)
                    .wrap_err("Failed to drop stash after successful apply")?;

                Ok(StashPopResult {
                    restored: true,
                    message: Some(format!(
                        "Stash {} applied and removed",
                        stash_ref.unwrap_or("stash@{0}")
                    )),
                    warnings: vec![],
                })
            }
            Err(e) => {
                // 应用失败，检查是否有冲突
                let has_conflicts = Self::has_unmerged().unwrap_or(false);

                if has_conflicts {
                    let warnings = vec![
                        format!(
                            "Merge conflicts detected when applying stash {}.",
                            stash_ref.unwrap_or("stash@{0}")
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
                        format!(
                            "Failed to apply stash {}: {}",
                            stash_ref.unwrap_or("stash@{0}"),
                            e
                        ),
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
    /// 使用 `git stash show --stat` 获取 stash 的文件变更统计。
    ///
    /// # 参数
    ///
    /// * `stash_ref` - Stash 引用（如 "stash@{0}"）
    ///
    /// # 返回
    ///
    /// 返回 `StashStat`，包含文件变更统计信息。
    pub fn stash_show_stat(stash_ref: &str) -> Result<StashStat> {
        // 注意：stash_show_stat 仍使用 GitCommand，因为 git2 没有直接提供统计信息
        // 这是一个辅助函数，不影响主要迁移目标
        use super::GitCommand;
        let output = GitCommand::new(["stash", "show", "--stat", stash_ref])
            .read()
            .wrap_err("Failed to get stash statistics")?;

        // 解析输出，例如：
        //  file1.txt | 2 +-
        //  file2.txt | 5 +++--
        //  2 files changed, 5 insertions(+), 3 deletions(-)
        let mut files_changed = 0;
        let mut insertions = 0;
        let mut deletions = 0;

        // 查找最后一行统计信息
        for line in output.lines().rev() {
            if let Some(stat_line) = line.strip_suffix(")") {
                // 解析格式：2 files changed, 5 insertions(+), 3 deletions(-)
                if let Some(files_part) = stat_line.split(',').next() {
                    if let Some(num) =
                        files_part.split_whitespace().next().and_then(|s| s.parse::<usize>().ok())
                    {
                        files_changed = num;
                    }
                }

                // 解析 insertions
                if let Some(ins_part) = stat_line.split(',').nth(1) {
                    if let Some(num) =
                        ins_part.split_whitespace().next().and_then(|s| s.parse::<usize>().ok())
                    {
                        insertions = num;
                    }
                }

                // 解析 deletions
                if let Some(del_part) = stat_line.split(',').nth(2) {
                    if let Some(num) =
                        del_part.split_whitespace().next().and_then(|s| s.parse::<usize>().ok())
                    {
                        deletions = num;
                    }
                }
                break;
            }
        }

        Ok(StashStat {
            files_changed,
            insertions,
            deletions,
        })
    }
}
