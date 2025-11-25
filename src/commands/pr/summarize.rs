//! PR 总结命令
//!
//! 读取 PR 修改的内容，然后使用 LLM 总结成文档。

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::base::settings::defaults::default_download_base_dir;
use crate::base::settings::Settings;
use crate::log_info;
use crate::log_success;
use crate::pr::helpers::get_current_branch_pr_id;
use crate::pr::llm::PullRequestLLM;
use crate::pr::platform::create_provider;

/// PR 总结命令
pub struct SummarizeCommand;

impl SummarizeCommand {
    /// 执行 PR 总结命令
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - PR ID（可选，如果不提供则自动检测当前分支的 PR）
    ///
    /// # 返回
    ///
    /// 返回保存的文件路径
    pub fn summarize(pull_request_id: Option<String>) -> Result<String> {
        // 创建平台提供者
        let provider = create_provider()?;

        // 获取 PR ID
        let pr_id = if let Some(id) = pull_request_id {
            id
        } else {
            // 自动检测当前分支的 PR
            get_current_branch_pr_id()?
                .context("No PR found for current branch. Please specify PR ID manually.")?
        };

        log_info!("Fetching PR #{} information...", pr_id);

        // 获取 PR 标题
        let pr_title = provider
            .get_pull_request_title(&pr_id)
            .context("Failed to get PR title")?;

        log_info!("PR Title: {}", pr_title);

        // 获取 PR diff
        log_info!("Fetching PR diff...");
        let pr_diff = provider
            .get_pull_request_diff(&pr_id)
            .context("Failed to get PR diff")?;

        log_info!("Generating summary with LLM...");

        // 使用 LLM 生成总结
        let summary = PullRequestLLM::summarize_pr(&pr_title, &pr_diff)
            .context("Failed to generate PR summary")?;

        // 解析 diff，提取所有文件的修改
        log_info!("Parsing PR diff to extract file changes...");
        log_info!("Diff length: {} characters", pr_diff.len());

        if pr_diff.trim().is_empty() {
            log_info!("PR diff is empty, skipping code changes extraction");
        } else {
            // 输出 diff 的前几行用于调试
            let preview_lines: Vec<&str> = pr_diff.lines().take(10).collect();
            log_info!("Diff preview (first 10 lines):");
            for (idx, line) in preview_lines.iter().enumerate() {
                log_info!("  [{}] {}", idx + 1, line);
            }
        }

        let file_changes =
            Self::parse_diff_to_file_changes(&pr_diff).context("Failed to parse PR diff")?;
        log_info!("Found {} file(s) with changes", file_changes.len());

        if !file_changes.is_empty() {
            for (file_path, content) in &file_changes {
                log_info!(
                    "  - {}: {} characters, {} lines",
                    file_path,
                    content.len(),
                    content.lines().count()
                );
            }
        } else {
            log_info!(
                "Warning: No file changes extracted from diff. This may indicate a parsing issue."
            );
        }

        // 将文件修改格式化为 markdown
        let code_changes_section = Self::format_file_changes_as_markdown(&file_changes);
        log_info!(
            "Code changes section length: {} characters",
            code_changes_section.len()
        );

        if code_changes_section.is_empty() {
            log_info!("Warning: Code changes section is empty. Final document will not include code changes.");
        }

        // 合并总结和代码修改部分
        let final_summary = if code_changes_section.is_empty() {
            log_info!(
                "Warning: No code changes found in diff. The generated document may be incomplete."
            );
            summary.summary
        } else {
            format!(
                "{}\n\n## Code Changes\n\n{}",
                summary.summary, code_changes_section
            )
        };

        // 构建输出路径
        let output_path = Self::build_output_path(&pr_id, &summary.filename)?;

        // 确保目录存在
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        // 写入文件
        fs::write(&output_path, &final_summary)
            .with_context(|| format!("Failed to write summary to: {:?}", output_path))?;

        log_success!("PR summary saved to: {}", output_path.display());

        Ok(output_path.to_string_lossy().to_string())
    }

    /// 构建输出路径
    ///
    /// 格式: `~/Downloads/Workflow/{PR_ID}/{filename}.md`
    fn build_output_path(pr_id: &str, filename: &str) -> Result<PathBuf> {
        let settings = Settings::get();
        let base_dir = settings
            .log
            .download_base_dir
            .clone()
            .unwrap_or_else(default_download_base_dir);

        let output_dir = PathBuf::from(&base_dir).join(pr_id);
        let output_path = output_dir.join(format!("{}.md", filename));

        Ok(output_path)
    }

    /// 解析 diff，提取每个文件的修改
    ///
    /// Git diff 格式：
    /// ```
    /// diff --git a/path/to/file b/path/to/file
    /// index ...
    /// --- a/path/to/file
    /// +++ b/path/to/file
    /// @@ ... @@
    /// 代码修改内容
    /// ```
    ///
    /// # 返回
    ///
    /// 返回文件路径和修改内容的映射
    fn parse_diff_to_file_changes(diff: &str) -> Result<Vec<(String, String)>> {
        let mut file_changes = Vec::new();
        let lines: Vec<&str> = diff.lines().collect();
        log_info!("Total diff lines: {}", lines.len());

        if lines.is_empty() {
            log_info!("Diff is empty, returning empty file changes");
            return Ok(file_changes);
        }

        let mut i = 0;
        let mut diff_start_count = 0;

        while i < lines.len() {
            // 查找 "diff --git" 行
            // GitHub API 返回标准的 unified diff 格式
            if lines[i].starts_with("diff --git") {
                diff_start_count += 1;
                log_info!("Found diff block #{} at line {}", diff_start_count, i + 1);
                // 提取文件路径
                // 格式: diff --git a/path/to/file b/path/to/file
                let file_path = match Self::extract_file_path_from_diff_line(lines[i]) {
                    Ok(path) => path,
                    Err(e) => {
                        // 如果提取失败，记录错误但继续处理下一个文件
                        log_info!(
                            "Failed to extract file path from line: {}, error: {}",
                            lines[i],
                            e
                        );
                        i += 1;
                        continue;
                    }
                };

                // 检查是否是二进制文件
                let mut is_binary = false;
                let mut found_hunk = false;
                let mut start_idx = i + 1;

                // 跳过 index 行和检查是否是二进制文件
                while start_idx < lines.len() {
                    let line = lines[start_idx];
                    if line.starts_with("Binary files") || line.starts_with("GIT binary patch") {
                        is_binary = true;
                        break;
                    }
                    if line.starts_with("@@") {
                        found_hunk = true;
                        break;
                    }
                    if line.starts_with("diff --git") {
                        // 没有找到 @@ 行，可能是新增/删除的空文件
                        break;
                    }
                    start_idx += 1;
                }

                // 如果是二进制文件，跳过
                if is_binary {
                    log_info!("Skipping binary file: {}", file_path);
                    i += 1;
                    while i < lines.len() && !lines[i].starts_with("diff --git") {
                        i += 1;
                    }
                    continue;
                }

                // 如果找到了 @@ 行，从下一行开始收集修改内容
                if found_hunk && start_idx < lines.len() {
                    let mut change_content = Vec::new();
                    let mut end_idx = start_idx + 1;

                    // 收集直到下一个 "diff --git" 或文件结束
                    while end_idx < lines.len() && !lines[end_idx].starts_with("diff --git") {
                        change_content.push(lines[end_idx]);
                        end_idx += 1;
                    }

                    // 如果收集到了内容，添加到结果中
                    if !change_content.is_empty() {
                        let content = change_content.join("\n");
                        let content_len = content.len();
                        let line_count = change_content.len();
                        file_changes.push((file_path.clone(), content));
                        log_info!(
                            "Extracted changes for file: {} ({} lines, {} chars)",
                            file_path,
                            line_count,
                            content_len
                        );
                    } else {
                        log_info!(
                            "No content found for file: {} (hunk found but no content)",
                            file_path
                        );
                    }

                    i = end_idx;
                } else {
                    // 没有找到 @@ 行，可能是新增/删除的空文件，跳过到下一个 diff
                    log_info!("No hunk found for file: {}, skipping", file_path);
                    i += 1;
                    while i < lines.len() && !lines[i].starts_with("diff --git") {
                        i += 1;
                    }
                }
            } else {
                i += 1;
            }
        }

        log_info!(
            "Parsed {} diff block(s), extracted {} file(s) with changes",
            diff_start_count,
            file_changes.len()
        );
        Ok(file_changes)
    }

    /// 从 diff 行中提取文件路径
    ///
    /// 格式: `diff --git a/path/to/file b/path/to/file`
    /// 返回: `path/to/file`
    fn extract_file_path_from_diff_line(line: &str) -> Result<String> {
        // 查找 "b/" 后面的路径（这是新文件的路径）
        if let Some(b_pos) = line.find(" b/") {
            let path_start = b_pos + 3; // " b/" 的长度是 3
            let path = &line[path_start..];
            Ok(path.trim().to_string())
        } else {
            anyhow::bail!("Invalid diff line format: {}", line)
        }
    }

    /// 将文件修改格式化为 markdown
    ///
    /// 格式：
    /// ```markdown
    /// ### src/path/to/file.rs:
    ///
    /// ```rust
    /// 代码修改内容
    /// ```
    /// ```
    fn format_file_changes_as_markdown(file_changes: &[(String, String)]) -> String {
        if file_changes.is_empty() {
            return String::new();
        }

        let mut sections = Vec::new();

        for (file_path, content) in file_changes {
            // 根据文件扩展名确定代码块语言
            let language = Self::detect_language_from_path(file_path);

            sections.push(format!(
                "### {}:\n\n```{}\n{}\n```",
                file_path, language, content
            ));
        }

        sections.join("\n\n")
    }

    /// 根据文件路径检测代码块语言
    fn detect_language_from_path(path: &str) -> &'static str {
        if let Some(ext) = path.split('.').next_back() {
            match ext.to_lowercase().as_str() {
                "rs" => "rust",
                "js" | "jsx" => "javascript",
                "ts" | "tsx" => "typescript",
                "py" => "python",
                "go" => "go",
                "java" => "java",
                "cpp" | "cc" | "cxx" => "cpp",
                "c" => "c",
                "md" => "markdown",
                "json" => "json",
                "yaml" | "yml" => "yaml",
                "toml" => "toml",
                "sh" | "bash" => "bash",
                "sql" => "sql",
                "html" => "html",
                "css" => "css",
                "xml" => "xml",
                _ => "",
            }
        } else {
            ""
        }
    }
}
