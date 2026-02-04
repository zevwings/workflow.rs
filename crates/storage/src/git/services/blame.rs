//! Blame 业务逻辑服务
//!
//! 提供代码追溯（blame）相关的业务逻辑实现。

use std::path::Path;

use super::GitContext;
use domain::git::{BlameLineInfo, GitError};

/// Blame 服务接口
pub trait BlameService: Send + Sync {
    /// 获取指定版本的文件内容
    fn get_file_content(&self, file_path: &str, revision: Option<&str>)
        -> Result<String, GitError>;

    /// 获取文件的 blame 信息
    fn get_file_blame(
        &self,
        file_path: &str,
        revision: Option<&str>,
    ) -> Result<Vec<BlameLineInfo>, GitError>;

    /// 获取文件指定行范围的 blame 信息
    fn get_file_blame_range(
        &self,
        file_path: &str,
        start_line: usize,
        end_line: usize,
        revision: Option<&str>,
    ) -> Result<Vec<BlameLineInfo>, GitError>;
}

/// Blame 服务实现
pub struct BlameServiceImpl {
    ctx: GitContext,
}

impl BlameServiceImpl {
    /// 创建新的 Blame 服务实例
    pub fn new(ctx: GitContext) -> Self {
        Self { ctx }
    }

    /// 获取 blame 信息的核心实现
    fn get_blame_internal(
        &self,
        file_path: &str,
        revision: Option<&str>,
        start_line: Option<usize>,
        end_line: Option<usize>,
    ) -> Result<Vec<BlameLineInfo>, GitError> {
        let repo = self.ctx.repository();
        let newest_commit = match revision {
            Some(rev) => self.ctx.resolve_commit(rev)?,
            None => self.ctx.head_commit()?,
        };

        // 配置 blame 选项
        let mut opts = git2::BlameOptions::new();
        opts.newest_commit(newest_commit);

        if let (Some(start), Some(end)) = (start_line, end_line) {
            opts.min_line(start);
            opts.max_line(end);
        }

        // 执行 blame
        let blame = repo
            .blame_file(Path::new(file_path), Some(&mut opts))
            .map_err(|e| GitError::OperationFailed(format!("Blame failed: {}", e)))?;

        // 读取文件内容以获取行内容
        let file_content = self.get_file_content(file_path, revision)?;
        let lines: Vec<&str> = file_content.lines().collect();

        let mut result = Vec::new();

        for hunk in blame.iter() {
            let commit_id = hunk.final_commit_id();

            // 获取提交信息
            let commit = repo
                .find_commit(commit_id)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;

            let author = commit.author();
            let commit_message = commit.summary().unwrap_or("(no message)").to_string();

            // 获取原始信息
            let original_commit_sha = if hunk.orig_commit_id() != commit_id {
                Some(hunk.orig_commit_id().to_string())
            } else {
                None
            };

            let original_file_path = hunk.path().and_then(|p| {
                let orig_path = p.to_string_lossy().to_string();
                if orig_path != file_path {
                    Some(orig_path)
                } else {
                    None
                }
            });

            // 为这个 hunk 中的每一行创建 BlameLineInfo
            let start = hunk.final_start_line();
            let lines_in_hunk = hunk.lines_in_hunk();

            for i in 0..lines_in_hunk {
                let line_number = start + i;
                let line_content = if line_number > 0 && line_number <= lines.len() {
                    lines[line_number - 1].to_string()
                } else {
                    String::new()
                };

                // 如果有行范围限制，过滤掉范围外的行
                if let (Some(s), Some(e)) = (start_line, end_line) {
                    if line_number < s || line_number > e {
                        continue;
                    }
                }

                result.push(BlameLineInfo {
                    line_number,
                    line_content,
                    commit_sha: commit_id.to_string(),
                    author: author.name().unwrap_or("Unknown").to_string(),
                    author_email: author.email().unwrap_or("unknown").to_string(),
                    commit_time: author.when().seconds(),
                    commit_message: commit_message.clone(),
                    original_commit_sha: original_commit_sha.clone(),
                    original_file_path: original_file_path.clone(),
                });
            }
        }

        // 按行号排序
        result.sort_by_key(|info| info.line_number);

        Ok(result)
    }
}

impl BlameService for BlameServiceImpl {
    fn get_file_content(
        &self,
        file_path: &str,
        revision: Option<&str>,
    ) -> Result<String, GitError> {
        let repo = self.ctx.repository();
        let commit_id = match revision {
            Some(rev) => self.ctx.resolve_commit(rev)?,
            None => self.ctx.head_commit()?,
        };

        let commit = repo
            .find_commit(commit_id)
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let tree = commit.tree().map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let entry = tree.get_path(Path::new(file_path)).map_err(|_| {
            GitError::ObjectNotFound(format!("File '{}' does not exist", file_path))
        })?;

        let blob = repo
            .find_blob(entry.id())
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let content = std::str::from_utf8(blob.content())
            .map_err(|_| GitError::OperationFailed("File content is not valid UTF-8".into()))?;

        Ok(content.to_string())
    }

    fn get_file_blame(
        &self,
        file_path: &str,
        revision: Option<&str>,
    ) -> Result<Vec<BlameLineInfo>, GitError> {
        self.get_blame_internal(file_path, revision, None, None)
    }

    fn get_file_blame_range(
        &self,
        file_path: &str,
        start_line: usize,
        end_line: usize,
        revision: Option<&str>,
    ) -> Result<Vec<BlameLineInfo>, GitError> {
        if start_line < 1 {
            return Err(GitError::OperationFailed("Start line must be >= 1".into()));
        }
        if end_line < start_line {
            return Err(GitError::OperationFailed(
                "End line must be >= start line".into(),
            ));
        }

        self.get_blame_internal(file_path, revision, Some(start_line), Some(end_line))
    }
}
