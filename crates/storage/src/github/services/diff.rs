//! Pull Request Diff 服务
//!
//! 提供 Pull Request diff 相关的业务逻辑实现

use std::sync::Arc;

use domain::GitHubError;
use serde_json::Value;
use toolkit::log_debug;

use crate::github::{services::ServiceContext, types::PullRequestFile};
use client::GitHubClient;

const MAX_FILES: usize = 50;
const MAX_LINES: usize = 15000;

/// Pull Request Diff 服务接口
pub trait PullRequestDiffService: Send + Sync {
    /// 获取 PR 的 diff 内容
    fn get_pull_request_diff(&self, pull_request_id: &str) -> Result<String, GitHubError>;
}

/// Pull Request Diff 服务实现
pub struct PullRequestDiffServiceImpl {
    client: Arc<dyn GitHubClient>,
    context: Arc<dyn ServiceContext>,
}

impl PullRequestDiffServiceImpl {
    pub fn new(client: Arc<dyn GitHubClient>, context: Arc<dyn ServiceContext>) -> Self {
        Self { client, context }
    }

    /// 获取 PR 文件列表
    fn get_pull_request_files(
        &self,
        owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<PullRequestFile>, GitHubError> {
        let url = format!("/repos/{}/{}/pulls/{}/files", owner, repo_name, pr_number);

        let response = self.client.get(&url)?;
        let json_value: Value = response
            .json()
            .map_err(|e| GitHubError::ApiError(format!("Failed to parse PR files JSON: {}", e)))?;
        let files: Vec<PullRequestFile> = serde_json::from_value(json_value)
            .map_err(|e| GitHubError::ApiError(format!("Failed to deserialize PR files: {}", e)))?;

        Ok(files)
    }

    /// 获取 PR diff 的替代方案（当 diff 超过 20000 行时）
    fn get_pull_request_diff_fallback(
        &self,
        owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<String, GitHubError> {
        let files = self.get_pull_request_files(owner, repo_name, pr_number)?;

        if files.is_empty() {
            return Err(GitHubError::ApiError("No files found in PR".to_string()));
        }

        let files_to_process = if files.len() > MAX_FILES {
            log_debug!(
                "PR has {} files, limiting to first {} files",
                files.len(),
                MAX_FILES
            );
            &files[..MAX_FILES]
        } else {
            &files
        };

        let mut diff_parts = Vec::new();
        let mut total_lines = 0;

        for file in files_to_process {
            if let Some(ref patch) = file.patch {
                let patch_lines: Vec<&str> = patch.lines().collect();
                if total_lines + patch_lines.len() > MAX_LINES {
                    let remaining_lines = MAX_LINES.saturating_sub(total_lines);
                    if remaining_lines > 0 {
                        let partial_patch =
                            patch_lines[..remaining_lines.min(patch_lines.len())].join("\n");
                        diff_parts.push(format!(
                            "diff --git a/{} b/{}\n{}",
                            file.filename, file.filename, partial_patch
                        ));
                    }
                    diff_parts.push(format!(
                        "\n... (diff truncated: {} files processed, {} total files in PR)",
                        files_to_process.len(),
                        files.len()
                    ));
                    break;
                }

                diff_parts.push(format!(
                    "diff --git a/{} b/{}\n{}",
                    file.filename, file.filename, patch
                ));
                total_lines += patch_lines.len();
            } else {
                let diff_header = match file.status.as_str() {
                    "added" => format!(
                        "diff --git a/{} b/{}\nnew file mode 100644\n--- /dev/null\n+++ b/{}\n@@ -0,0 +1,{} @@\n... (file too large, {} additions)",
                        file.filename, file.filename, file.filename, file.additions, file.additions
                    ),
                    "removed" => format!(
                        "diff --git a/{} b/{}\ndeleted file mode 100644\n--- a/{}\n+++ /dev/null\n@@ -1,{} +0,0 @@\n... (file too large, {} deletions)",
                        file.filename, file.filename, file.filename, file.deletions, file.deletions
                    ),
                    _ => format!(
                        "diff --git a/{} b/{}\nindex 0000000..0000000\n--- a/{}\n+++ b/{}\n@@ -1,{} +1,{} @@\n... (file too large, {} additions, {} deletions)",
                        file.filename,
                        file.filename,
                        file.filename,
                        file.filename,
                        file.deletions,
                        file.additions,
                        file.additions,
                        file.deletions
                    ),
                };
                diff_parts.push(diff_header);
            }
        }

        if files.len() > files_to_process.len() {
            diff_parts.push(format!(
                "\n... ({} more files not included due to size limit)",
                files.len() - files_to_process.len()
            ));
        }

        Ok(diff_parts.join("\n"))
    }
}

impl PullRequestDiffService for PullRequestDiffServiceImpl {
    fn get_pull_request_diff(&self, pull_request_id: &str) -> Result<String, GitHubError> {
        let (owner, repo_name) = self.context.get_owner_and_repo()?;
        let pr_number: u64 = self.context.parse_pr_number(pull_request_id)?;

        let url = format!("/repos/{}/{}/pulls/{}.diff", owner, repo_name, pr_number);

        let response = self.client.get(&url)?;

        let status = response.status_code();

        if status == 406 {
            let is_too_large = if let Ok(json) = response.json::<Value>() {
                json.get("errors")
                    .and_then(|v| v.as_array())
                    .map(|errors| {
                        errors.iter().any(|err| {
                            err.get("code")
                                .and_then(|c| c.as_str())
                                .map(|c| c == "too_large")
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false)
            } else {
                false
            };

            if is_too_large {
                log_debug!("PR diff exceeds GitHub API limit (20000 lines), using fallback method");
                return self.get_pull_request_diff_fallback(&owner, &repo_name, pr_number);
            }
        }

        if status >= 400 {
            return Err(GitHubError::ApiError(format!(
                "Failed to get PR diff: HTTP {}",
                status
            )));
        }

        let text = response
            .text()
            .map_err(|e| GitHubError::ApiError(format!("Failed to parse response text: {}", e)))?;
        Ok(text.to_string())
    }
}
