//! Commit Amend 业务逻辑
//!
//! 提供 amend 操作相关的业务逻辑，包括：
//! - 预览信息生成
//! - 格式化显示
//! - 完成提示生成

use crate::git::{CommitInfo, GitBranch, GitCommit, WorktreeStatus};
use color_eyre::Result;

/// Amend 预览信息
#[derive(Debug, Clone)]
pub struct AmendPreview {
    /// 原始 commit SHA
    pub original_sha: String,
    /// 新提交消息
    pub new_message: Option<String>,
    /// 原始提交消息
    pub original_message: String,
    /// 要添加的文件列表
    pub files_to_add: Vec<String>,
    /// 操作类型
    pub operation_type: String,
    /// 是否已推送到远程
    pub is_pushed: bool,
}

/// Commit Amend 业务逻辑
pub struct CommitAmend;

impl CommitAmend {
    /// 创建 amend 预览信息
    ///
    /// # 参数
    ///
    /// * `commit_info` - 原始 commit 信息
    /// * `new_message` - 新提交消息（可选）
    /// * `files_to_add` - 要添加的文件列表
    /// * `operation_type` - 操作类型
    /// * `current_branch` - 当前分支名
    ///
    /// # 返回
    ///
    /// 返回 amend 预览信息。
    pub fn create_preview(
        commit_info: &CommitInfo,
        new_message: &Option<String>,
        files_to_add: &[String],
        operation_type: &str,
        current_branch: &str,
    ) -> Result<AmendPreview> {
        let is_pushed =
            GitBranch::is_commit_in_remote(current_branch, &commit_info.sha).unwrap_or(false);

        Ok(AmendPreview {
            original_sha: commit_info.sha.clone(),
            new_message: new_message.clone(),
            original_message: commit_info.message.clone(),
            files_to_add: files_to_add.to_vec(),
            operation_type: operation_type.to_string(),
            is_pushed,
        })
    }

    /// 格式化 amend 预览信息为字符串
    ///
    /// # 参数
    ///
    /// * `preview` - Amend 预览信息
    ///
    /// # 返回
    ///
    /// 返回格式化的字符串。
    pub fn format_preview(preview: &AmendPreview) -> String {
        let mut result = format!(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n                         Commit Amend Preview\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n  Original Commit SHA:  {}\n  New Commit SHA:       (will be regenerated)\n\n",
            &preview.original_sha[..8]
        );

        if let Some(msg) = &preview.new_message {
            result.push_str(&format!(
                "  Original message:     {}\n  New message:          {}\n",
                preview.original_message, msg
            ));
        } else {
            result.push_str(&format!(
                "  Message:              {} (unchanged)\n",
                preview.original_message
            ));
        }

        result.push('\n');

        if !preview.files_to_add.is_empty() {
            result.push_str("  Files to add:\n");
            for file in &preview.files_to_add {
                result.push_str(&format!("    ✓ {}\n", file));
            }
        } else {
            result.push_str("  Files to add:         None\n");
        }

        result.push_str(&format!(
            "\n  Operation type:       {}\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            preview.operation_type
        ));

        if preview.is_pushed {
            result.push_str(
                "\n\n⚠️  Warning: This commit may have been pushed to remote\n\nAfter amend, you'll need to force push to update the remote branch:\n  git push --force\n\nThis may affect other collaborators. Please ensure team members are notified.\n",
            );
        }

        result
    }

    /// 格式化 commit 信息为详细字符串（包含工作区状态）
    ///
    /// # 参数
    ///
    /// * `commit_info` - Commit 信息
    /// * `branch` - 分支名
    /// * `status` - 可选的工作区状态
    ///
    /// # 返回
    ///
    /// 返回格式化的字符串。
    pub fn format_commit_info_detailed(
        commit_info: &CommitInfo,
        branch: &str,
        status: Option<&WorktreeStatus>,
    ) -> String {
        let mut result = format!(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n                           当前 Commit 信息\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n  Commit SHA:    {}\n  提交消息:      {}\n  作者:          {}\n  日期:          {}\n  分支:          {}\n",
            &commit_info.sha[..8],
            commit_info.message,
            commit_info.author,
            commit_info.date,
            branch
        );

        if let Some(s) = status {
            result.push_str(&format!("\n{}\n", GitCommit::format_worktree_status(s)));
        }

        result.push_str(
            "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        );

        result
    }

    /// 检查是否需要显示 force push 警告
    ///
    /// # 参数
    ///
    /// * `current_branch` - 当前分支名
    /// * `old_sha` - 原始 commit SHA
    ///
    /// # 返回
    ///
    /// 如果已推送，返回 `true`。
    pub fn should_show_force_push_warning(current_branch: &str, old_sha: &str) -> Result<bool> {
        GitBranch::is_commit_in_remote(current_branch, old_sha)
    }

    /// 生成完成提示信息
    ///
    /// # 参数
    ///
    /// * `current_branch` - 当前分支名
    /// * `old_sha` - 原始 commit SHA
    ///
    /// # 返回
    ///
    /// 如果已推送，返回提示信息字符串；否则返回 `None`。
    pub fn format_completion_message(
        current_branch: &str,
        old_sha: &str,
    ) -> Result<Option<String>> {
        let is_pushed = Self::should_show_force_push_warning(current_branch, old_sha)?;

        if is_pushed {
            Ok(Some("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n                        Commit Amend Complete\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n  ✓ Commit has been modified\n\n  Note:\n    - If this commit has been pushed to remote, you need to force push:\n      git push --force\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string()))
        } else {
            Ok(None)
        }
    }
}
