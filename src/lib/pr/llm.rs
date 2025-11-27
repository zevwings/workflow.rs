//! Pull Request LLM 服务
//!
//! 本模块提供了使用 LLM 生成 Pull Request 内容的功能。
//! 根据 commit 标题自动生成符合规范的分支名和 PR 标题。

use anyhow::{Context, Result};
use serde_json::Value;

use crate::base::llm::{LLMClient, LLMRequestParams};
use crate::base::prompt::{
    generate_summarize_file_change_system_prompt, generate_summarize_pr_system_prompt,
    generate_summarize_test_plan_system_prompt, GENERATE_BRANCH_SYSTEM_PROMPT,
};
use crate::pr::helpers::transform_to_branch_name;

/// PR 内容，包含分支名、PR 标题和描述
///
/// 由 LLM 生成的分支名、PR 标题和描述，用于创建 Pull Request。
#[derive(Debug, Clone)]
pub struct PullRequestContent {
    /// 分支名称（小写，使用连字符分隔）
    pub branch_name: String,
    /// PR 标题（简洁，不超过 8 个单词）
    pub pr_title: String,
    /// PR 描述（基于 Git 修改内容生成）
    pub description: Option<String>,
}

/// Pull Request LLM 服务
///
/// 提供使用 LLM 生成分支名和 PR 标题的功能。
/// 支持多种 LLM 提供商：OpenAI、DeepSeek、代理 API。
pub struct PullRequestLLM;

impl PullRequestLLM {
    /// 同时生成分支名和 PR 标题（通过一个 LLM 请求）
    ///
    /// 根据 commit 标题生成符合规范的分支名和 PR 标题。
    /// 分支名和 PR 标题都会自动翻译为英文（如果输入包含非英文内容）。
    ///
    /// # 参数
    ///
    /// * `commit_title` - commit 标题或描述
    /// * `exists_branches` - 已存在的分支列表（可选）
    /// * `git_diff` - Git 工作区和暂存区的修改内容（可选）
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestContent` 结构体，包含：
    /// - `branch_name` - 分支名称（小写，使用连字符分隔）
    /// - `pr_title` - PR 标题（简洁，不超过 8 个单词）
    /// - `description` - PR 描述（基于 Git 修改内容生成，可选）
    ///
    /// # 错误
    ///
    /// 如果 LLM API 调用失败或响应格式不正确，返回相应的错误信息。
    pub fn generate(
        commit_title: &str,
        exists_branches: Option<Vec<String>>,
        git_diff: Option<String>,
    ) -> Result<PullRequestContent> {
        // 使用统一的 v2 客户端
        let client = LLMClient::global();

        // 构建请求参数
        let user_prompt = Self::user_prompt(commit_title, exists_branches, git_diff);
        // 使用编译时嵌入的 system prompt
        let system_prompt = GENERATE_BRANCH_SYSTEM_PROMPT.to_string();

        let params = LLMRequestParams {
            system_prompt,
            user_prompt,
            max_tokens: Some(500), // 增加到 500，确保有足够空间返回完整的 JSON（包括 description）
            temperature: 0.5,
            model: String::new(), // model 会从 Settings 自动获取，这里可以留空
        };

        // 调用 LLM API
        let response = client.call(&params).with_context(|| {
            format!(
                "Failed to call LLM API for generating branch name from commit title: '{}'",
                commit_title
            )
        })?;

        // 解析响应
        Self::parse_llm_response(response).with_context(|| {
            format!(
                "Failed to parse LLM response for commit title: '{}'",
                commit_title
            )
        })
    }

    /// 生成同时生成分支名和 PR 标题的 user prompt
    fn user_prompt(
        commit_title: &str,
        exists_branches: Option<Vec<String>>,
        git_diff: Option<String>,
    ) -> String {
        // 提取分支列表，如果没有或为空则使用空数组
        // 注意：exists_branches 已经通过 get_all_branches(true) 获取，已经去掉了前缀
        let base_branch_names: Vec<String> = exists_branches
            .filter(|b| !b.is_empty())
            .unwrap_or_default();

        // 组装 prompt 内容
        let mut parts = vec![format!("Commit title: {}", commit_title)];

        if !base_branch_names.is_empty() {
            parts.push(format!(
                "Existing base branch names: {}",
                base_branch_names.join(", ")
            ));
        }

        if let Some(diff) = git_diff {
            if !diff.trim().is_empty() {
                // 限制 diff 长度，避免请求过大
                // 对于生成分支名和 PR 标题，不需要完整的 diff
                // 限制在 3000 字符左右（约 750-1000 tokens），足够提供上下文信息
                const MAX_DIFF_LENGTH: usize = 3000;
                let diff_trimmed = {
                    let char_count = diff.chars().count();
                    if char_count > MAX_DIFF_LENGTH {
                        // 使用字符边界安全截取，避免在多字节字符中间截断
                        let mut char_boundary = diff.len(); // 默认到字符串末尾
                        for (idx, _) in diff.char_indices().take(MAX_DIFF_LENGTH + 1) {
                            char_boundary = idx;
                        }
                        let truncated = &diff[..char_boundary];
                        // 尝试在最后一个换行符处截断，避免截断中间的行
                        let last_newline = truncated.rfind('\n').unwrap_or(0);
                        let truncated_diff = if last_newline > 0 {
                            &diff[..last_newline]
                        } else {
                            truncated
                        };
                        format!(
                            "{}\n... (diff truncated, {} characters total)",
                            truncated_diff, char_count
                        )
                    } else {
                        diff
                    }
                };
                parts.push(format!("Git changes:\n{}", diff_trimmed));
            }
        }

        parts.join("\n")
    }

    /// 解析 LLM 返回的 JSON 响应，提取分支名和 PR 标题
    ///
    /// 从 LLM 的 JSON 响应中提取 `branch_name` 和 `pr_title` 字段。
    /// 支持处理包含 markdown 代码块的响应格式。
    ///
    /// # 参数
    ///
    /// * `response` - LLM 返回的响应字符串（可能是 JSON 或包含 JSON 的 markdown 代码块）
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestContent` 结构体，包含清理后的分支名和 PR 标题。
    ///
    /// # 错误
    ///
    /// 如果响应格式不正确或缺少必要字段，返回相应的错误信息。
    fn parse_llm_response(response: String) -> Result<PullRequestContent> {
        let trimmed = response.trim();

        // 尝试提取 JSON（可能包含 markdown 代码块）
        let json_str = if trimmed.starts_with("```json") {
            // 移除 ```json 开头和 ``` 结尾
            let start = trimmed.find('\n').unwrap_or(0);
            let end = trimmed.rfind("```").unwrap_or(trimmed.len());
            trimmed[start..end].trim()
        } else if trimmed.starts_with("```") {
            // 移除 ``` 开头和 ``` 结尾
            let start = trimmed.find('\n').unwrap_or(0);
            let end = trimmed.rfind("```").unwrap_or(trimmed.len());
            trimmed[start..end].trim()
        } else {
            trimmed
        };

        // 解析 JSON
        let json: Value = serde_json::from_str(json_str).with_context(|| {
            format!(
                "Failed to parse LLM response as JSON. Raw response: {}",
                json_str
            )
        })?;

        let branch_name = json
            .get("branch_name")
            .and_then(|v| v.as_str())
            .context("Missing 'branch_name' field in LLM response")?
            .to_string();

        let pr_title = json
            .get("pr_title")
            .and_then(|v| v.as_str())
            .context("Missing 'pr_title' field in LLM response")?
            .to_string();

        // description 是可选的
        let description = json
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        // 清理分支名，确保只保留 ASCII 字符
        let cleaned_branch_name = transform_to_branch_name(branch_name.trim());

        Ok(PullRequestContent {
            branch_name: cleaned_branch_name,
            pr_title: pr_title.trim().to_string(),
            description,
        })
    }
}

/// PR 总结结果，包含总结文档和文件名
///
/// 由 LLM 生成的 PR 总结文档和对应的文件名。
#[derive(Debug, Clone)]
pub struct PullRequestSummary {
    /// PR 总结文档（Markdown 格式）
    pub summary: String,
    /// 文件名（不含路径和扩展名）
    pub filename: String,
}

impl PullRequestLLM {
    /// 生成 PR 总结文档和文件名
    ///
    /// 根据 PR 的 diff 内容生成总结文档和合适的文件名。
    ///
    /// # 参数
    ///
    /// * `pr_title` - PR 标题
    /// * `pr_diff` - PR 的 diff 内容
    /// * `language` - 可选的语言代码（如 "en", "zh", "zh-CN", "zh-TW"），如果为 None，则从配置文件读取
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestSummary` 结构体，包含：
    /// - `summary` - PR 总结文档（Markdown 格式）
    /// - `filename` - 文件名（不含路径和扩展名）
    ///
    /// # 错误
    ///
    /// 如果 LLM API 调用失败或响应格式不正确，返回相应的错误信息。
    pub fn summarize_pr(pr_title: &str, pr_diff: &str) -> Result<PullRequestSummary> {
        // 使用统一的 v2 客户端
        let client = LLMClient::global();

        // 构建请求参数
        let user_prompt = Self::summarize_user_prompt(pr_title, pr_diff);
        // 根据语言生成 system prompt（语言选择逻辑在 prompt 生成函数内部处理）
        let system_prompt = generate_summarize_pr_system_prompt();

        let params = LLMRequestParams {
            system_prompt,
            user_prompt,
            max_tokens: None, // 增加 token 数量，确保有足够空间返回完整的总结文档
            temperature: 0.3, // 降低温度，使输出更稳定
            model: String::new(), // model 会从 Settings 自动获取，这里可以留空
        };

        // 调用 LLM API
        let response = client.call(&params).with_context(|| {
            format!("Failed to call LLM API for summarizing PR: '{}'", pr_title)
        })?;

        // 解析响应
        Self::parse_summary_response(response).with_context(|| {
            format!(
                "Failed to parse LLM response for PR summary: '{}'",
                pr_title
            )
        })
    }

    /// 生成 PR 总结的 user prompt
    fn summarize_user_prompt(pr_title: &str, pr_diff: &str) -> String {
        let mut parts = vec![format!("PR Title: {}", pr_title)];

        if !pr_diff.trim().is_empty() {
            // 限制 diff 长度，避免请求过大
            // 对于总结，我们需要更多的 diff 内容，但也要避免超过 token 限制
            const MAX_DIFF_LENGTH: usize = 15000; // 增加长度，因为总结需要更多上下文
            let diff_trimmed = {
                let char_count = pr_diff.chars().count();
                if char_count > MAX_DIFF_LENGTH {
                    // 使用字符边界安全截取
                    let mut char_boundary = pr_diff.len();
                    for (idx, _) in pr_diff.char_indices().take(MAX_DIFF_LENGTH + 1) {
                        char_boundary = idx;
                    }
                    let truncated = &pr_diff[..char_boundary];
                    // 尝试在最后一个换行符处截断
                    let last_newline = truncated.rfind('\n').unwrap_or(0);
                    let truncated_diff = if last_newline > 0 {
                        &pr_diff[..last_newline]
                    } else {
                        truncated
                    };
                    format!(
                        "{}\n... (diff truncated, {} characters total)",
                        truncated_diff, char_count
                    )
                } else {
                    pr_diff.to_string()
                }
            };
            parts.push(format!("PR Diff:\n{}", diff_trimmed));
        }

        parts.join("\n\n")
    }

    /// 解析 LLM 返回的 JSON 响应，提取总结文档和文件名
    ///
    /// 从 LLM 的 JSON 响应中提取 `summary` 和 `filename` 字段。
    /// 支持处理包含 markdown 代码块的响应格式。
    ///
    /// # 参数
    ///
    /// * `response` - LLM 返回的响应字符串（可能是 JSON 或包含 JSON 的 markdown 代码块）
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestSummary` 结构体，包含清理后的总结文档和文件名。
    ///
    /// # 错误
    ///
    /// 如果响应格式不正确或缺少必要字段，返回相应的错误信息。
    fn parse_summary_response(response: String) -> Result<PullRequestSummary> {
        let trimmed = response.trim();

        // 尝试提取 JSON（可能包含 markdown 代码块）
        let json_str = if trimmed.starts_with("```json") {
            // 移除 ```json 开头和 ``` 结尾
            let start = trimmed.find('\n').unwrap_or(0);
            let end = trimmed.rfind("```").unwrap_or(trimmed.len());
            trimmed[start..end].trim()
        } else if trimmed.starts_with("```") {
            // 移除 ``` 开头和 ``` 结尾
            let start = trimmed.find('\n').unwrap_or(0);
            let end = trimmed.rfind("```").unwrap_or(trimmed.len());
            trimmed[start..end].trim()
        } else {
            trimmed
        };

        // 解析 JSON
        let json: Value = serde_json::from_str(json_str).with_context(|| {
            format!(
                "Failed to parse LLM response as JSON. Raw response: {}",
                json_str
            )
        })?;

        let summary = json
            .get("summary")
            .and_then(|v| v.as_str())
            .context("Missing 'summary' field in LLM response")?
            .to_string();

        let filename = json
            .get("filename")
            .and_then(|v| v.as_str())
            .context("Missing 'filename' field in LLM response")?
            .to_string();

        // 清理文件名，确保只包含有效的文件名字符
        let cleaned_filename = filename
            .trim()
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect::<String>();

        // 移除 .md 扩展名（如果存在），因为我们会自动添加
        let cleaned_filename = cleaned_filename.trim_end_matches(".md").to_string();

        if cleaned_filename.is_empty() {
            anyhow::bail!("Generated filename is empty after cleaning");
        }

        Ok(PullRequestSummary {
            summary: summary.trim().to_string(),
            filename: cleaned_filename,
        })
    }

    /// 生成单个文件的修改总结
    ///
    /// 根据文件的 diff 内容生成该文件的修改总结。
    ///
    /// # 参数
    ///
    /// * `file_path` - 文件路径
    /// * `file_diff` - 文件的 diff 内容
    /// * `language` - 可选的语言代码（如 "en", "zh", "zh-CN", "zh-TW"），如果为 None，则从配置文件读取
    ///
    /// # 返回
    ///
    /// 返回文件的修改总结（纯文本）
    ///
    /// # 错误
    ///
    /// 如果 LLM API 调用失败，返回相应的错误信息。
    pub fn summarize_file_change(file_path: &str, file_diff: &str) -> Result<String> {
        // 使用统一的 v2 客户端
        let client = LLMClient::global();

        // 限制 diff 长度，避免请求过大
        // 对于单个文件的总结，限制在 5000 字符左右
        const MAX_DIFF_LENGTH: usize = 5000;
        let file_diff_trimmed = if file_diff.len() > MAX_DIFF_LENGTH {
            let truncated = &file_diff[..MAX_DIFF_LENGTH];
            let last_newline = truncated.rfind('\n').unwrap_or(0);
            if last_newline > 0 {
                format!(
                    "{}\n... (diff truncated, {} characters total)",
                    &file_diff[..last_newline],
                    file_diff.len()
                )
            } else {
                format!(
                    "{}\n... (diff truncated, {} characters total)",
                    truncated,
                    file_diff.len()
                )
            }
        } else {
            file_diff.to_string()
        };

        // 构建请求参数
        let user_prompt = Self::summarize_file_change_user_prompt(file_path, &file_diff_trimmed);
        // 根据语言生成 system prompt（语言选择逻辑在 prompt 生成函数内部处理）
        let system_prompt = generate_summarize_file_change_system_prompt();

        let params = LLMRequestParams {
            system_prompt,
            user_prompt,
            max_tokens: Some(300), // 单个文件的总结应该比较简短
            temperature: 0.3,
            model: String::new(), // model 会从 Settings 自动获取，这里可以留空
        };

        // 调用 LLM API
        let response = client.call(&params).with_context(|| {
            format!(
                "Failed to call LLM API for summarizing file change: '{}'",
                file_path
            )
        })?;

        // 清理响应（移除可能的 markdown 代码块包装）
        let summary = Self::clean_file_change_summary_response(response);

        Ok(summary)
    }

    /// 生成测试计划
    ///
    /// 根据 PR 标题、diff 和文件变更生成详细的测试计划。
    ///
    /// # 参数
    ///
    /// * `pr_title` - PR 标题
    /// * `pr_diff` - PR diff 内容
    /// * `file_changes` - 文件变更列表（文件路径和 diff 内容）
    ///
    /// # 返回
    ///
    /// 返回测试计划内容（Markdown 格式），以 "### Test Plan" 开头
    ///
    /// # 错误
    ///
    /// 如果 LLM API 调用失败，返回相应的错误信息。
    pub fn generate_test_plan(
        pr_title: &str,
        pr_diff: &str,
        file_changes: &[(String, String)],
    ) -> Result<String> {
        // 使用统一的 v2 客户端
        let client = LLMClient::global();

        // 尝试找到修改的方法对应的接口
        let related_endpoints = Self::find_related_endpoints(file_changes).unwrap_or_default();

        // 构建请求参数
        let user_prompt =
            Self::test_plan_user_prompt(pr_title, pr_diff, file_changes, &related_endpoints);
        // 根据语言生成 system prompt（语言选择逻辑在 prompt 生成函数内部处理）
        let system_prompt = generate_summarize_test_plan_system_prompt();

        let params = LLMRequestParams {
            system_prompt,
            user_prompt,
            max_tokens: Some(2000), // 测试计划可能需要更多 token
            temperature: 0.3,
            model: String::new(), // model 会从 Settings 自动获取，这里可以留空
        };

        // 调用 LLM API
        let response = client.call(&params).with_context(|| {
            format!(
                "Failed to call LLM API for generating test plan: '{}'",
                pr_title
            )
        })?;

        // 清理响应（移除可能的 markdown 代码块包装）
        let test_plan = Self::clean_test_plan_response(response);

        Ok(test_plan)
    }

    /// 找到修改的方法对应的接口
    fn find_related_endpoints(
        file_changes: &[(String, String)],
    ) -> Result<Vec<crate::pr::search::EndpointInfo>> {
        use crate::{log_debug, log_info};

        let searcher = crate::pr::search::CodebaseSearcher::new()?;
        let endpoints = searcher.find_endpoints_for_modified_methods(file_changes)?;

        if !endpoints.is_empty() {
            log_info!("Found {} endpoint(s) from code analysis:", endpoints.len());
            for endpoint in &endpoints {
                log_debug!(
                    "  - {} {} (in {})",
                    endpoint.method,
                    endpoint.path,
                    endpoint.file_path
                );
            }
        } else {
            log_debug!("No endpoints found from code analysis");
        }

        Ok(endpoints)
    }

    /// 生成测试计划的 user prompt
    fn test_plan_user_prompt(
        pr_title: &str,
        pr_diff: &str,
        file_changes: &[(String, String)],
        related_endpoints: &[crate::pr::search::EndpointInfo],
    ) -> String {
        let mut parts = vec![
            format!("PR Title: {}", pr_title),
            "".to_string(),
            "## Instructions".to_string(),
            "".to_string(),
            "Please analyze the code changes below and generate a comprehensive test plan. Focus on:".to_string(),
            "1. **Code Logic Analysis**: Identify all functions, conditions, branches, and error paths in the changes".to_string(),
            "2. **Parameter Analysis**: Extract all parameters, their types, constraints, and validation rules".to_string(),
            "3. **Test Coverage**: Generate test cases for ALL code paths, branches, and error conditions".to_string(),
            "4. **Actionable Tests**: Provide executable CURL commands with complete test data".to_string(),
            "".to_string(),
        ];

        // 添加文件变更信息（简要）
        if !file_changes.is_empty() {
            let file_list: Vec<String> = file_changes
                .iter()
                .map(|(file_path, _)| format!("- {}", file_path))
                .collect();
            parts.push(format!("Modified Files:\n{}", file_list.join("\n")));

            // 分析文件路径，提供 API 相关的提示
            let mut api_related_files = Vec::new();
            let mut service_files = Vec::new();
            let mut model_files = Vec::new();
            let mut test_files = Vec::new();

            for (file_path, _) in file_changes {
                let path_lower = file_path.to_lowercase();
                if path_lower.contains("/api/")
                    || path_lower.contains("/routes/")
                    || path_lower.contains("controller")
                    || path_lower.contains("handler")
                {
                    api_related_files.push(file_path.clone());
                } else if path_lower.contains("service") {
                    service_files.push(file_path.clone());
                } else if path_lower.contains("model")
                    || path_lower.contains("schema")
                    || path_lower.contains("dto")
                    || path_lower.contains("entity")
                {
                    model_files.push(file_path.clone());
                } else if path_lower.contains("test") || path_lower.contains("spec") {
                    test_files.push(file_path.clone());
                }
            }

            if !api_related_files.is_empty() {
                parts.push("### API Endpoint Files".to_string());
                parts.push("These files likely contain HTTP endpoint definitions:".to_string());
                for file in &api_related_files {
                    parts.push(format!("- `{}`", file));
                }
                parts.push("".to_string());
                parts.push("**Action Required**: Analyze these files to extract:".to_string());
                parts.push("- HTTP method and path for each endpoint".to_string());
                parts.push("- Request parameters (path, query, body) and their types".to_string());
                parts.push("- Response structure and status codes".to_string());
                parts.push("- Validation rules and constraints".to_string());
                parts.push("- Error handling and error responses".to_string());
                parts.push("".to_string());
            }

            if !service_files.is_empty() {
                parts.push(format!(
                    "## Service Layer Files Detected\n\nThese service files may be called by controllers that expose HTTP endpoints:\n{}\n\n**Important**: Even though these are service layer files, they may affect API behavior. Please analyze:\n1. What APIs might call these services?\n2. How do the changes affect API responses?\n3. What should be tested in the API layer?",
                    service_files.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n")
                ));
            }
        }

        // 添加找到的相关接口信息
        if !related_endpoints.is_empty() {
            parts.push("## Identified Endpoints".to_string());
            parts.push("".to_string());
            parts.push(
                "The following endpoints were automatically detected from code analysis:"
                    .to_string(),
            );
            parts.push("".to_string());

            for endpoint in related_endpoints {
                let location = if let Some(line) = endpoint.line_number {
                    format!("{}:{}", endpoint.file_path, line)
                } else {
                    endpoint.file_path.clone()
                };
                parts.push(format!(
                    "- **{} {}** (defined in `{}`)",
                    endpoint.method, endpoint.path, location
                ));
            }

            parts.push("".to_string());
            parts.push("**Critical**: You MUST generate comprehensive test plans for ALL of these endpoints.".to_string());
            parts.push("".to_string());
        }

        // 添加 PR Diff（代码变更内容）
        if !pr_diff.trim().is_empty() {
            parts.push("## Code Changes (PR Diff)".to_string());
            parts.push("".to_string());
            parts.push("Analyze the following code changes carefully:".to_string());
            parts.push("".to_string());
            parts.push("**Code Analysis Checklist**:".to_string());
            parts.push(
                "- [ ] Identify all functions/methods that were added, modified, or removed"
                    .to_string(),
            );
            parts.push("- [ ] Extract function signatures (parameters, return types)".to_string());
            parts.push(
                "- [ ] Identify all conditional branches (if/else, switch/case, try/catch)"
                    .to_string(),
            );
            parts.push("- [ ] Identify all validation checks and constraints".to_string());
            parts.push("- [ ] Identify all error handling paths and exception cases".to_string());
            parts.push("- [ ] Identify all loops and iterations".to_string());
            parts.push(
                "- [ ] Identify all external dependencies (API calls, database operations)"
                    .to_string(),
            );
            parts.push("- [ ] Identify all state changes and side effects".to_string());
            parts.push("".to_string());
            parts.push(
                "**For each identified code path, generate a corresponding test case.**"
                    .to_string(),
            );
            parts.push("".to_string());

            // 限制 diff 长度，避免请求过大
            // 对于测试计划，我们需要足够的 diff 内容来识别接口
            const MAX_DIFF_LENGTH: usize = 20000; // 增加长度以提供更多上下文
            let diff_trimmed = {
                let char_count = pr_diff.chars().count();
                if char_count > MAX_DIFF_LENGTH {
                    let mut char_boundary = pr_diff.len();
                    for (idx, _) in pr_diff.char_indices().take(MAX_DIFF_LENGTH + 1) {
                        char_boundary = idx;
                    }
                    let truncated = &pr_diff[..char_boundary];
                    let last_newline = truncated.rfind('\n').unwrap_or(0);
                    let truncated_diff = if last_newline > 0 {
                        &pr_diff[..last_newline]
                    } else {
                        truncated
                    };
                    format!(
                        "{}\n\n... (diff truncated, {} characters total. Focus on the visible changes for test plan generation.)",
                        truncated_diff, char_count
                    )
                } else {
                    pr_diff.to_string()
                }
            };
            parts.push("```diff".to_string());
            parts.push(diff_trimmed);
            parts.push("```".to_string());
            parts.push("".to_string());
        }

        // 添加测试生成指导
        parts.push("## Test Generation Requirements".to_string());
        parts.push("".to_string());
        parts.push("For each endpoint or component identified, generate:".to_string());
        parts.push("".to_string());
        parts.push("1. **Complete Endpoint Information**:".to_string());
        parts.push("   - HTTP method and full path".to_string());
        parts.push("   - Purpose and description".to_string());
        parts.push("   - Test priority (High/Medium/Low)".to_string());
        parts.push("".to_string());
        parts.push("2. **Detailed Parameter Analysis**:".to_string());
        parts.push("   - Path parameters with types and constraints".to_string());
        parts.push("   - Query parameters with types, required/optional, defaults".to_string());
        parts.push(
            "   - Request body structure with all fields, types, and validation rules".to_string(),
        );
        parts.push("   - Required headers".to_string());
        parts.push("".to_string());
        parts.push("3. **Comprehensive Test Scenarios**:".to_string());
        parts.push("   - Happy path tests (normal, minimal, complete cases)".to_string());
        parts.push("   - Validation tests (missing fields, invalid types, invalid formats, invalid values)".to_string());
        parts.push(
            "   - Boundary tests (min/max values, empty/null values, very long strings)"
                .to_string(),
        );
        parts.push("   - Error handling tests (authentication, authorization, not found, conflicts, server errors)".to_string());
        parts.push(
            "   - Integration tests (database operations, external services, concurrent requests)"
                .to_string(),
        );
        parts.push("".to_string());
        parts.push("4. **Executable Test Commands**:".to_string());
        parts.push("   - Complete CURL commands for each test scenario".to_string());
        parts.push("   - Realistic test data that matches the code's expectations".to_string());
        parts.push("   - Expected response status codes and structures".to_string());
        parts.push("   - Clear verification steps".to_string());
        parts.push("".to_string());
        parts.push("**Remember**: Generate test cases for EVERY code path, branch, and error condition you identify in the code changes.".to_string());

        parts.join("\n\n")
    }

    /// 清理测试计划响应
    ///
    /// 移除可能的 markdown 代码块包装，确保返回以 "### Test Plan" 开头的纯 Markdown 内容。
    fn clean_test_plan_response(response: String) -> String {
        let trimmed = response.trim();

        // 如果响应被包装在 markdown 代码块中，移除包装
        if trimmed.starts_with("```") {
            let start = trimmed.find('\n').unwrap_or(0);
            let end = trimmed.rfind("```").unwrap_or(trimmed.len());
            trimmed[start..end].trim().to_string()
        } else {
            trimmed.to_string()
        }
    }

    /// 生成单个文件修改总结的 user prompt
    fn summarize_file_change_user_prompt(file_path: &str, file_diff: &str) -> String {
        format!("File path: {}\n\nFile diff:\n{}", file_path, file_diff)
    }

    /// 清理文件修改总结响应
    ///
    /// 移除可能的 markdown 代码块包装，返回纯文本。
    fn clean_file_change_summary_response(response: String) -> String {
        let trimmed = response.trim();

        // 如果响应被包装在 markdown 代码块中，移除包装
        if trimmed.starts_with("```") {
            // 移除 ``` 开头和 ``` 结尾
            let start = trimmed.find('\n').unwrap_or(0);
            let end = trimmed.rfind("```").unwrap_or(trimmed.len());
            trimmed[start..end].trim().to_string()
        } else {
            trimmed.to_string()
        }
    }
}
