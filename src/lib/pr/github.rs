use crate::Git;
use anyhow::{Context, Result};
use duct::cmd;

use super::helpers::{extract_github_repo_from_url, extract_pr_id_from_url};
use super::provider::Platform;

/// GitHub API 模块
pub struct GitHub;

impl Platform for GitHub {
    /// 创建 Pull Request
    fn create_pr(
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: Option<&str>,
    ) -> Result<String> {
        // 获取仓库的 owner/repo
        let remote_url = Git::get_remote_url().context("Failed to get remote URL")?;
        let repo = extract_github_repo_from_url(&remote_url)
            .context("Failed to extract GitHub repo from remote URL")?;

        let mut args = vec![
            "pr",
            "create",
            "--repo",
            &repo,
            "--title",
            title,
            "--body",
            body,
            "--head",
            source_branch,
        ];

        if let Some(base_branch) = target_branch {
            args.push("--base");
            args.push(base_branch);
        }

        let output = cmd("gh", &args)
            .read()
            .context("Failed to create PR via GitHub CLI")?;

        // 提取 PR URL（gh CLI 输出的是完整的 URL）
        Ok(output.trim().to_string())
    }

    /// 合并 Pull Request
    fn merge_pr(pr_id: &str, delete_branch: bool) -> Result<()> {
        let mut args = vec!["pr", "merge", pr_id, "--merge"];

        if delete_branch {
            args.push("--delete-branch");
        }

        cmd("gh", &args)
            .run()
            .context(format!("Failed to merge PR: {}", pr_id))?;

        Ok(())
    }

    /// 获取 PR 信息
    fn get_pr_info(pr_id: &str) -> Result<String> {
        let output = cmd("gh", &["pr", "view", pr_id])
            .read()
            .context(format!("Failed to get PR info: {}", pr_id))?;

        Ok(output)
    }

    /// 获取 PR URL
    #[allow(dead_code)]
    fn get_pr_url(pr_id: &str) -> Result<String> {
        let output = cmd(
            "gh",
            &["pr", "view", pr_id, "--json", "url", "--jq", ".url"],
        )
        .read()
        .context(format!("Failed to get PR URL: {}", pr_id))?;

        Ok(output.trim().to_string())
    }

    /// 获取 PR 标题
    fn get_pr_title(pr_id: &str) -> Result<String> {
        let output = cmd(
            "gh",
            &["pr", "view", pr_id, "--json", "title", "--jq", ".title"],
        )
        .read()
        .context(format!("Failed to get PR title: {}", pr_id))?;

        Ok(output.trim().to_string())
    }

    /// 列出 PR
    fn list_prs(state: Option<&str>, limit: Option<u32>) -> Result<String> {
        let mut args: Vec<String> = vec!["pr".to_string(), "list".to_string()];

        if let Some(s) = state {
            args.push("--state".to_string());
            args.push(s.to_string());
        }

        if let Some(l) = limit {
            args.push("--limit".to_string());
            args.push(l.to_string());
        }

        // 转换为 &str 切片
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let output = cmd("gh", &args_refs).read().context("Failed to list PRs")?;

        Ok(output)
    }

    /// 获取当前分支的 PR
    fn get_current_branch_pr() -> Result<Option<String>> {
        let output = cmd(
            "gh",
            &[
                "pr",
                "status",
                "--json",
                "url",
                "--jq",
                ".currentBranch.url",
            ],
        )
        .read();

        match output {
            Ok(url) => {
                let url = url.trim();
                if url.is_empty() || url == "null" {
                    Ok(None)
                } else {
                    // 从 URL 提取 PR ID
                    let pr_id = extract_pr_id_from_url(url)?;
                    Ok(Some(pr_id))
                }
            }
            Err(_) => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::helpers::extract_pr_id_from_url;

    #[test]
    fn test_extract_pr_id_from_url() {
        assert_eq!(
            extract_pr_id_from_url("https://github.com/owner/repo/pull/123").unwrap(),
            "123"
        );
        assert_eq!(
            extract_pr_id_from_url("https://github.com/owner/repo/pull/456/").unwrap(),
            "456"
        );
    }
}
