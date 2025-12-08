//! 清理功能相关实现

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::helpers;
use super::JiraLogs;
use crate::base::util::dialog::ConfirmDialog;
use crate::trace_info;

// ==================== 返回结构体 ====================

/// 目录信息
#[derive(Debug, Clone)]
pub struct DirInfo {
    /// 目录名称（用于显示）
    pub dir_name: String,
    /// 目录路径
    pub dir: PathBuf,
    /// 目录总大小（字节）
    pub size: u64,
    /// 文件数量
    pub file_count: usize,
    /// 是否为基础目录
    pub is_base_dir: bool,
    /// JIRA ID（如果适用）
    pub jira_id: Option<String>,
    /// 目录内容（文件列表）
    pub contents: Vec<DirEntry>,
}

/// 目录条目
#[derive(Debug, Clone)]
pub struct DirEntry {
    /// 条目类型（文件或目录）
    pub entry_type: String,
    /// 条目名称
    pub name: String,
    /// 条目大小（如果是文件）
    pub size: Option<String>,
}

/// 清理结果
#[derive(Debug, Clone)]
pub struct CleanResult {
    /// 是否成功删除
    pub deleted: bool,
    /// 目录是否存在
    pub dir_exists: bool,
    /// 目录信息（如果存在）
    pub dir_info: Option<DirInfo>,
    /// 是否被取消
    pub cancelled: bool,
    /// 是否为 dry run
    pub dry_run: bool,
    /// 是否为 list only
    pub list_only: bool,
}

impl JiraLogs {
    /// 清理指定 JIRA ID 的日志目录
    ///
    /// 自动构建目录路径，然后清理该目录。
    pub fn clean_dir(&self, jira_id: &str, dry_run: bool, list_only: bool) -> Result<CleanResult> {
        let dir = if jira_id.is_empty() {
            // 如果 jira_id 为空，清理整个 jira 目录
            self.base_dir.join("jira")
        } else {
            self.base_dir.join("jira").join(jira_id)
        };
        let dir_name = if jira_id.is_empty() {
            "the entire base directory".to_string()
        } else {
            format!("the directory for {}", jira_id)
        };

        if !dir.exists() {
            trace_info!("Directory does not exist: {:?}", dir);
            return Ok(CleanResult {
                deleted: false,
                dir_exists: false,
                dir_info: None,
                cancelled: false,
                dry_run,
                list_only,
            });
        }

        let (size, file_count) = helpers::calculate_dir_info(&dir)?;
        let is_base_dir = jira_id.is_empty();
        let dir_info = self.display_dir_info(&dir_name, &dir, size, file_count, is_base_dir)?;

        if list_only {
            return Ok(CleanResult {
                deleted: false,
                dir_exists: true,
                dir_info: Some(dir_info),
                cancelled: false,
                dry_run,
                list_only,
            });
        }

        if dry_run {
            trace_info!("[DRY RUN] Would delete {}: {:?}", dir_name, dir);
            trace_info!("[DRY RUN] Total size: {}", helpers::format_size(size));
            trace_info!("[DRY RUN] Total files: {}", file_count);
            return Ok(CleanResult {
                deleted: false,
                dir_exists: true,
                dir_info: Some(dir_info),
                cancelled: false,
                dry_run,
                list_only,
            });
        }

        let confirmed = ConfirmDialog::new(format!(
            "Are you sure you want to delete {}? This will remove {} files ({}).",
            dir_name,
            file_count,
            helpers::format_size(size)
        ))
        .with_default(false)
        .with_cancel_message("Operation cancelled")
        .prompt()?;

        if !confirmed {
            trace_info!("Clean operation cancelled.");
            return Ok(CleanResult {
                deleted: false,
                dir_exists: true,
                dir_info: Some(dir_info),
                cancelled: true,
                dry_run,
                list_only,
            });
        }

        std::fs::remove_dir_all(&dir)
            .with_context(|| format!("Failed to delete {}: {:?}", dir_name, dir))?;

        trace_info!("{} deleted successfully: {:?}", dir_name, dir);
        Ok(CleanResult {
            deleted: true,
            dir_exists: true,
            dir_info: Some(dir_info),
            cancelled: false,
            dry_run,
            list_only,
        })
    }

    /// 显示目录信息
    fn display_dir_info(
        &self,
        dir_name: &str,
        dir: &Path,
        size: u64,
        file_count: usize,
        is_base_dir: bool,
    ) -> Result<DirInfo> {
        let jira_id = if dir_name.starts_with("the directory for") {
            dir_name
                .strip_prefix("the directory for ")
                .map(|s| s.to_string())
        } else {
            None
        };

        let mut contents = Vec::new();

        if is_base_dir {
            // 按 ticket 分区显示
            let ticket_contents = self.get_base_dir_contents(dir)?;
            contents = ticket_contents;
        } else {
            // 单个 ticket 目录，直接列出内容
            let dir_contents = helpers::list_dir_contents(dir)?;
            for path in dir_contents {
                if path.is_file() {
                    let size_str = if let Ok(metadata) = std::fs::metadata(&path) {
                        Some(helpers::format_size(metadata.len()))
                    } else {
                        None
                    };
                    contents.push(DirEntry {
                        entry_type: "📄 File".to_string(),
                        name: path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("-")
                            .to_string(),
                        size: size_str,
                    });
                } else if path.is_dir() {
                    contents.push(DirEntry {
                        entry_type: "📁 Directory".to_string(),
                        name: path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("-")
                            .to_string(),
                        size: None,
                    });
                }
            }
        }

        Ok(DirInfo {
            dir_name: dir_name.to_string(),
            dir: dir.to_path_buf(),
            size,
            file_count,
            is_base_dir,
            jira_id,
            contents,
        })
    }

    /// 获取基础目录内容（按 ticket 分区）
    fn get_base_dir_contents(&self, base_dir: &Path) -> Result<Vec<DirEntry>> {
        // 读取基础目录下的所有条目
        let entries = fs::read_dir(base_dir)
            .with_context(|| format!("Failed to read directory: {:?}", base_dir))?;

        let mut ticket_dirs: Vec<(String, PathBuf)> = Vec::new();

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            if path.is_dir() {
                // 提取 ticket ID（目录名）
                if let Some(ticket_id) = path.file_name().and_then(|n| n.to_str()) {
                    ticket_dirs.push((ticket_id.to_string(), path));
                }
            }
        }

        // 按 ticket ID 排序
        ticket_dirs.sort_by(|a, b| a.0.cmp(&b.0));

        let mut all_contents = Vec::new();

        // 为每个 ticket 收集内容
        for (ticket_id, ticket_dir) in ticket_dirs {
            // 列出该 ticket 目录下的所有文件（不包含 ticket 目录本身）
            let contents = helpers::list_dir_contents(&ticket_dir)?;

            for path in contents {
                // 跳过 ticket 目录本身
                if path == ticket_dir {
                    continue;
                }
                if path.is_file() {
                    let size_str = if let Ok(metadata) = std::fs::metadata(&path) {
                        Some(helpers::format_size(metadata.len()))
                    } else {
                        None
                    };
                    all_contents.push(DirEntry {
                        entry_type: format!("📄 File ({})", ticket_id),
                        name: path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("-")
                            .to_string(),
                        size: size_str,
                    });
                } else if path.is_dir() {
                    all_contents.push(DirEntry {
                        entry_type: format!("📁 Directory ({})", ticket_id),
                        name: path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("-")
                            .to_string(),
                        size: None,
                    });
                }
            }
        }

        Ok(all_contents)
    }
}
