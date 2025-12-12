//! JIRA 添加评论命令
//!
//! 为 JIRA ticket 添加评论，支持交互式输入：
//! - 输入评论文本
//! - 可选择附加文件内容

use crate::base::dialog::{ConfirmDialog, InputDialog};
use crate::base::indicator::Spinner;
use crate::jira::helpers::validate_jira_ticket_format;
use crate::jira::Jira;
use crate::{log_message, log_success};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// 添加评论命令
pub struct CommentCommand;

impl CommentCommand {
    /// 添加评论到 JIRA ticket
    ///
    /// # 参数
    ///
    /// * `jira_id` - JIRA ticket ID（可选，如果不提供会交互式输入）
    ///
    /// # 返回
    ///
    /// 成功时返回 `Ok(())`。
    pub fn add(jira_id: Option<String>) -> Result<()> {
        // 步骤 1: 获取或输入 Jira ticket（参考 pr/create.rs 的方式）
        let ticket = Self::resolve_jira_ticket(jira_id)?;

        // 步骤 2: 输入评论内容
        let message = InputDialog::new("Enter comment message")
            .prompt()
            .context("Failed to get comment message")?;

        if message.trim().is_empty() {
            anyhow::bail!("Comment message cannot be empty");
        }

        // 步骤 3: 询问是否需要附加文件
        let attach_file = ConfirmDialog::new("Do you want to attach a file?")
            .with_default(false)
            .prompt()
            .context("Failed to get file attachment choice")?;

        // 步骤 4: 构建最终评论内容
        let final_comment = if attach_file {
            // 选择文件（支持多个文件）
            let file_paths = Self::select_files()?;

            if file_paths.is_empty() {
                // 如果没有选择文件，只使用文本评论
                message.trim().to_string()
            } else {
                // 读取所有文件内容并附加到评论中
                let mut comment_parts = vec![message.trim().to_string()];

                for file_path in &file_paths {
                    let file_content = Self::read_from_file(file_path)?;
                    comment_parts.push(format!(
                        "---\n\nFile: {}\n\n```\n{}\n```",
                        file_path, file_content
                    ));
                }

                comment_parts.join("\n\n")
            }
        } else {
            // 只使用文本评论
            message.trim().to_string()
        };

        // 步骤 5: 添加评论
        log_message!("Adding comment to ticket {}...", ticket);
        Spinner::with(format!("Adding comment to ticket {}...", ticket), || {
            Jira::add_comment(&ticket, &final_comment)
        })
        .context(format!("Failed to add comment to ticket {}", ticket))?;

        log_success!("Comment added to ticket {} successfully!", ticket);

        Ok(())
    }

    /// 获取或输入 Jira ticket
    ///
    /// 如果提供了 ticket，验证其格式；如果没有提供，提示用户输入并验证。
    /// 参考 `src/commands/pr/create.rs:119-150` 的实现方式。
    fn resolve_jira_ticket(jira_ticket: Option<String>) -> Result<String> {
        let ticket = if let Some(t) = jira_ticket {
            let trimmed = t.trim().to_string();
            if trimmed.is_empty() {
                // 如果为空，提示输入
                InputDialog::new("Enter Jira ticket ID (e.g., PROJ-123)")
                    .prompt()
                    .context("Failed to get Jira ticket ID")?
                    .trim()
                    .to_string()
            } else {
                trimmed
            }
        } else {
            // 如果没有提供，提示输入
            InputDialog::new("Enter Jira ticket ID (e.g., PROJ-123)")
                .prompt()
                .context("Failed to get Jira ticket ID")?
                .trim()
                .to_string()
        };

        // 统一验证逻辑
        if ticket.is_empty() {
            anyhow::bail!("Jira ticket ID cannot be empty");
        }
        validate_jira_ticket_format(&ticket)?;

        Ok(ticket)
    }

    /// 选择文件（支持多个文件）
    ///
    /// 循环提示用户输入文件路径，每次输入后询问是否继续添加更多文件。
    /// 验证文件是否存在且为文件（非目录）。
    ///
    /// # 返回
    ///
    /// 返回文件路径字符串向量
    fn select_files() -> Result<Vec<String>> {
        let mut file_paths = Vec::new();

        loop {
            // 输入文件路径
            let file_path = InputDialog::new("Enter file path")
                .prompt()
                .context("Failed to get file path")?;

            let trimmed = file_path.trim();
            if trimmed.is_empty() {
                anyhow::bail!("File path cannot be empty");
            }

            // 验证文件
            let path = Path::new(trimmed);
            if !path.exists() {
                log_message!("File not found: {}. Please try again.", trimmed);
                continue;
            }

            if !path.is_file() {
                log_message!("Path is not a file: {}. Please try again.", trimmed);
                continue;
            }

            // 添加到列表
            file_paths.push(trimmed.to_string());
            log_success!("File added: {}", trimmed);

            // 询问是否继续添加更多文件
            let add_more = ConfirmDialog::new("Do you want to add another file?")
                .with_default(false)
                .prompt()
                .context("Failed to get add more files choice")?;

            if !add_more {
                break;
            }
        }

        Ok(file_paths)
    }

    /// 从文件读取内容
    ///
    /// # 参数
    ///
    /// * `file_path` - 文件路径
    ///
    /// # 返回
    ///
    /// 返回文件内容作为字符串
    fn read_from_file(file_path: &str) -> Result<String> {
        let path = Path::new(file_path);
        fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", file_path))
            .map(|content| content.trim().to_string())
    }
}
