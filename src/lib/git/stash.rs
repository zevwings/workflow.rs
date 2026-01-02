use chrono::{DateTime, FixedOffset, Local, TimeZone};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};

use super::commands::{GitCommitCommand, GitStashCommand};
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
    /// 使用 Git 命令行将当前工作区和暂存区的未提交修改保存到 stash。
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
        GitStashCommand::stash_push(message, None).wrap_err("Failed to stash changes")
    }

    /// 检查是否有未合并的文件（冲突文件）
    ///
    /// 使用 Git 命令行检查是否有未合并的路径（冲突文件）。
    /// 返回 true 如果有冲突文件，false 如果没有
    pub fn has_unmerged() -> Result<bool> {
        // 使用 git status --porcelain 检查冲突状态
        // 冲突文件的状态码包含 'U' (unmerged)
        let output = GitCommitCommand::status(None).wrap_err("Failed to get repository status")?;

        // 检查输出中是否有冲突标记（状态码包含 'U'）
        // Git status --porcelain 格式：
        // - 第一个字符：索引状态
        // - 第二个字符：工作树状态
        // - 'U' 表示未合并（冲突）
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line.len() >= 2 {
                let index_status = line.chars().nth(0).unwrap_or(' ');
                let worktree_status = line.chars().nth(1).unwrap_or(' ');
                // 'U' 表示未合并（冲突）
                if index_status == 'U' || worktree_status == 'U' {
                    return Ok(true);
                }
            }
        }

        // 也检查 git diff --check 是否有冲突标记（检查工作区文件中的冲突标记）
        if let Ok(has_conflicts) = GitCommitCommand::check_conflicts(None) {
            if has_conflicts {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// 列出所有 stash 条目
    ///
    /// 使用 Git 命令行获取所有 stash 条目的结构化数据。
    ///
    /// # 返回
    ///
    /// 返回所有 stash 条目的列表，按索引从新到旧排列（stash@{0} 在第一个）。
    pub fn stash_list() -> Result<Vec<StashEntry>> {
        // 使用 GitStashCommand 获取基础信息
        let stash_entries =
            GitStashCommand::list_stash(None).wrap_err("Failed to list stash entries")?;

        let mut entries = Vec::new();

        // 为每个 stash 获取时间戳和 commit hash
        for stash_entry in stash_entries {
            // 获取 commit hash（如果还没有）
            let commit_hash = if stash_entry.commit_hash.is_empty() {
                GitCommitCommand::rev_parse(&format!("stash@{{{}}}", stash_entry.index), None)
                    .ok()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default()
            } else {
                stash_entry.commit_hash.clone()
            };

            // 获取时间戳
            let timestamp = Self::get_stash_timestamp(stash_entry.index).ok();

            entries.push(StashEntry {
                index: stash_entry.index,
                branch: stash_entry.branch,
                message: stash_entry.message,
                commit_hash,
                timestamp,
            });
        }

        // 按索引排序（从新到旧，stash@{0} 在第一个）
        entries.sort_by_key(|e| e.index);

        Ok(entries)
    }

    /// 获取 stash 的时间戳
    ///
    /// 使用 `git log` 命令获取 stash commit 的时间戳。
    fn get_stash_timestamp(stash_index: usize) -> Result<DateTime<Local>> {
        let stash_ref = format!("stash@{{{}}}", stash_index);
        // 使用 git log 获取时间戳
        // 格式：%ct 是 Unix 时间戳，%z 是时区偏移
        let stash_ref_str = stash_ref.as_str();
        let output = GitCommitCommand::log(Some(1), "%ct %z", Some(stash_ref_str), false, None)
            .wrap_err_with(|| format!("Failed to get timestamp for {}", stash_ref))?;

        let parts: Vec<&str> = output.split_whitespace().collect();
        if parts.is_empty() {
            return Err(eyre!("Invalid timestamp format"));
        }

        let timestamp_secs: i64 = parts[0]
            .parse()
            .wrap_err_with(|| format!("Failed to parse timestamp: {}", parts[0]))?;

        // 解析时区偏移（格式：+0800 或 -0500）
        let offset_secs = if parts.len() > 1 {
            let offset_str = parts[1];
            if offset_str.len() == 5 {
                let sign = if offset_str.starts_with('+') { 1 } else { -1 };
                let hours: i32 = offset_str[1..3].parse().unwrap_or(0);
                let minutes: i32 = offset_str[3..5].parse().unwrap_or(0);
                sign * (hours * 3600 + minutes * 60)
            } else {
                0
            }
        } else {
            0
        };

        let offset =
            FixedOffset::east_opt(offset_secs).unwrap_or_else(|| FixedOffset::east_opt(0).unwrap());
        let dt = offset
            .timestamp_opt(timestamp_secs, 0)
            .single()
            .ok_or_else(|| eyre!("Invalid timestamp"))?;

        Ok(dt.with_timezone(&Local))
    }

    /// 应用指定的 stash（不删除）
    ///
    /// 使用 Git 命令行应用指定的 stash，保留 stash 条目。
    ///
    /// # 参数
    ///
    /// * `stash_ref` - Stash 引用（如 "stash@{0}" 或 "stash@{1}"），如果为 None 则应用最新的
    ///
    /// # 返回
    ///
    /// 返回 `StashApplyResult`，包含应用状态、冲突信息和警告。
    pub fn stash_apply(stash_ref: Option<&str>) -> Result<StashApplyResult> {
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

        let stash_ref_str = stash_ref.unwrap_or("stash@{0}");

        // 应用 stash
        let result = GitStashCommand::stash_apply(Some(stash_index), None);

        match result {
            Ok(_) => {
                // 检查是否有冲突
                let has_conflicts = Self::has_unmerged().unwrap_or(false);
                Ok(StashApplyResult {
                    applied: true,
                    has_conflicts,
                    message: Some(format!("Stash {} applied successfully", stash_ref_str)),
                    warnings: if has_conflicts {
                        vec!["Merge conflicts detected. Please resolve them manually.".to_string()]
                    } else {
                        vec![]
                    },
                    stat: Self::stash_show_stat(stash_ref_str).ok(),
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
                        format!("Failed to apply stash {}: {}", stash_ref_str, e),
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
    /// 使用 Git 命令行删除指定的 stash 条目。
    ///
    /// # 参数
    ///
    /// * `stash_ref` - Stash 引用（如 "stash@{0}" 或 "stash@{1}"），如果为 None 则删除最新的
    ///
    /// # 错误
    ///
    /// 如果删除失败，返回相应的错误信息。
    pub fn stash_drop(stash_ref: Option<&str>) -> Result<()> {
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

        GitStashCommand::drop_stash(Some(stash_index), None)
            .wrap_err_with(|| format!("Failed to drop stash {}", stash_ref.unwrap_or("stash@{0}")))
    }

    /// 应用并删除指定的 stash
    ///
    /// 使用 Git 命令行应用并删除指定的 stash 条目。
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

        let stash_ref_str = stash_ref.unwrap_or("stash@{0}");

        // 使用 stash pop 命令（会自动删除）
        let result = GitStashCommand::stash_pop(Some(stash_index), None);

        match result {
            Ok(_) => Ok(StashPopResult {
                restored: true,
                message: Some(format!("Stash {} applied and removed", stash_ref_str)),
                warnings: vec![],
            }),
            Err(e) => {
                // 应用失败，检查是否有冲突
                let has_conflicts = Self::has_unmerged().unwrap_or(false);

                if has_conflicts {
                    let warnings = vec![
                        format!(
                            "Merge conflicts detected when applying stash {}.",
                            stash_ref_str
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
                        format!("Failed to apply stash {}: {}", stash_ref_str, e),
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
    /// 使用 Git 命令行获取 stash 的文件变更统计。
    ///
    /// # 参数
    ///
    /// * `stash_ref` - Stash 引用（如 "stash@{0}"）
    ///
    /// # 返回
    ///
    /// 返回 `StashStat`，包含文件变更统计信息。
    pub fn stash_show_stat(stash_ref: &str) -> Result<StashStat> {
        // 使用 git show --stat 获取统计信息
        let output = GitCommitCommand::show(stash_ref, true, Some(""), None)
            .wrap_err_with(|| format!("Failed to get stash stat for {}", stash_ref))?;

        // 解析输出
        // git show --stat 输出格式示例：
        //  file1.txt | 5 +++++
        //  file2.txt | 10 +++++-----
        //  2 files changed, 10 insertions(+), 5 deletions(-)
        let mut files_changed = 0;
        let mut insertions = 0;
        let mut deletions = 0;

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // 检查是否是统计行（最后一行）
            if line.contains("files changed") {
                // 解析格式：2 files changed, 10 insertions(+), 5 deletions(-)
                // 或：2 files changed, 10 insertions(+)
                // 或：2 files changed, 5 deletions(-)
                if let Some(files_pos) = line.find("files changed") {
                    // 提取文件数
                    if let Some(files_str) = line[..files_pos].split_whitespace().next() {
                        files_changed = files_str.parse().unwrap_or(0);
                    }

                    // 提取插入数
                    if let Some(ins_pos) = line.find("insertion") {
                        let before_ins = &line[..ins_pos];
                        if let Some(ins_str) = before_ins.split_whitespace().last() {
                            insertions = ins_str.parse().unwrap_or(0);
                        }
                    }

                    // 提取删除数
                    if let Some(del_pos) = line.find("deletion") {
                        let before_del = &line[..del_pos];
                        if let Some(del_str) = before_del.split_whitespace().last() {
                            deletions = del_str.parse().unwrap_or(0);
                        }
                    }
                }
            } else if line.contains('|') {
                // 这是文件变更行，统计文件数
                // 格式：file.txt | 5 +++++
                if let Some(pipe_pos) = line.find('|') {
                    let file_part = line[..pipe_pos].trim();
                    if !file_part.is_empty() {
                        files_changed += 1;
                    }
                }
            }
        }

        // 如果从统计行没有解析到文件数，使用文件行数
        if files_changed == 0 {
            // 重新计算文件数（从文件行）
            files_changed = output
                .lines()
                .filter(|line| {
                    let line = line.trim();
                    line.contains('|') && !line.contains("files changed")
                })
                .count();
        }

        Ok(StashStat {
            files_changed,
            insertions,
            deletions,
        })
    }
}
