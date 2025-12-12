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

/// 文件信息
struct FileInfo {
    content: String,
    is_binary: bool,
    size: u64,
}

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

        // 步骤 4: 处理文件附件和构建评论内容
        let final_comment = if attach_file {
            // 选择文件（支持多个文件）
            let file_paths = Self::select_files()?;

            if file_paths.is_empty() {
                // 如果没有选择文件，只使用文本评论
                message.trim().to_string()
            } else {
                // 先上传所有文件作为附件
                log_message!("Uploading {} file(s) as attachment(s)...", file_paths.len());
                let mut uploaded_files = Vec::new();
                let mut failed_files = Vec::new();

                for file_path in &file_paths {
                    let file_name = Path::new(file_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(file_path);

                    match Jira::upload_attachment(&ticket, file_path) {
                        Ok(attachments) => {
                            if let Some(attachment) = attachments.first() {
                                uploaded_files.push((file_name.to_string(), attachment.clone()));
                                log_success!("Uploaded: {}", file_name);
                            }
                        }
                        Err(e) => {
                            log_message!("Failed to upload {}: {}", file_name, e);
                            failed_files.push((file_name.to_string(), file_path.clone()));
                        }
                    }
                }

                // 构建评论内容
                let mut comment_parts = vec![message.trim().to_string()];

                // 添加已上传的附件引用
                if !uploaded_files.is_empty() {
                    comment_parts.push("---\n\n**Attachments:**".to_string());
                    for (file_name, attachment) in &uploaded_files {
                        comment_parts
                            .push(format!("- [{}]({})", file_name, attachment.content_url));
                    }
                }

                // 对于文本文件，也包含内容预览（如果文件较小）
                for file_path in &file_paths {
                    let file_info = Self::read_from_file(file_path)?;
                    let file_name = Path::new(file_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(file_path);

                    // 只对文本文件且未上传失败的文件显示内容
                    if !file_info.is_binary
                        && !failed_files.iter().any(|(_, path)| path == file_path)
                    {
                        const MAX_TEXT_SIZE: usize = 20 * 1024; // 20KB 限制
                        if file_info.content.len() <= MAX_TEXT_SIZE {
                            comment_parts.push(format!(
                                "---\n\n**File: {}** ({} bytes)\n\n```\n{}\n```",
                                file_name,
                                Self::format_file_size(file_info.size),
                                file_info.content
                            ));
                        }
                    }
                }

                // 如果有上传失败的文件，在评论中提及
                if !failed_files.is_empty() {
                    comment_parts.push(
                        "\n---\n\n*Note: Some files failed to upload as attachments.*".to_string(),
                    );
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

            // 去除首尾空白和引号
            let trimmed = file_path.trim();
            let cleaned = Self::strip_quotes(trimmed);
            if cleaned.is_empty() {
                anyhow::bail!("File path cannot be empty");
            }

            // 验证文件
            let path = Path::new(cleaned);
            if !path.exists() {
                log_message!("File not found: {}. Please try again.", cleaned);
                continue;
            }

            if !path.is_file() {
                log_message!("Path is not a file: {}. Please try again.", cleaned);
                continue;
            }

            // 添加到列表
            file_paths.push(cleaned.to_string());
            log_success!("File added: {}", cleaned);

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
    /// 自动检测文件类型：如果是文本文件，直接读取为 UTF-8 字符串；
    /// 如果是二进制文件，不读取内容（避免大小问题）。
    ///
    /// # 参数
    ///
    /// * `file_path` - 文件路径
    ///
    /// # 返回
    ///
    /// 返回 `FileInfo` 结构体，包含文件内容、类型和大小信息
    fn read_from_file(file_path: &str) -> Result<FileInfo> {
        let path = Path::new(file_path);
        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to get file metadata: {}", file_path))?;
        let file_size = metadata.len();

        let bytes =
            fs::read(path).with_context(|| format!("Failed to read file: {}", file_path))?;

        // 尝试将字节解码为 UTF-8 字符串
        match String::from_utf8(bytes.clone()) {
            Ok(text) => {
                // 成功解码为 UTF-8，是文本文件
                Ok(FileInfo {
                    content: text.trim().to_string(),
                    is_binary: false,
                    size: file_size,
                })
            }
            Err(_) => {
                // 无法解码为 UTF-8，是二进制文件
                // 不包含内容，避免 base64 编码导致的大小问题
                Ok(FileInfo {
                    content: String::new(),
                    is_binary: true,
                    size: file_size,
                })
            }
        }
    }

    /// 格式化文件大小
    ///
    /// 将字节数转换为人类可读的格式（B, KB, MB, GB）。
    fn format_file_size(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes < KB {
            format!("{} B", bytes)
        } else if bytes < MB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else if bytes < GB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        }
    }

    /// 去除字符串首尾的引号
    ///
    /// 支持单引号（'）和双引号（"），只去除首尾匹配的引号。
    ///
    /// # 参数
    ///
    /// * `s` - 要处理的字符串
    ///
    /// # 返回
    ///
    /// 返回去除引号后的字符串
    fn strip_quotes(s: &str) -> &str {
        let s = s.trim();
        if s.len() >= 2 {
            let chars: Vec<char> = s.chars().collect();
            if let (Some(&first), Some(&last)) = (chars.first(), chars.last()) {
                if (first == '"' && last == '"') || (first == '\'' && last == '\'') {
                    // 计算去除首尾字符后的范围
                    let start = first.len_utf8();
                    let end = s.len() - last.len_utf8();
                    if start < end {
                        return &s[start..end];
                    }
                }
            }
        }
        s
    }
}
