use crate::git::GitBranch;
use crate::jira::history::JiraWorkHistory;
use crate::{log_break, log_message};
use anyhow::Result;
use serde_json;
use std::collections::HashMap;

use super::helpers::{get_jira_id, OutputFormat};

/// 显示关联信息命令
pub struct RelatedCommand;

impl RelatedCommand {
    /// 显示 ticket 的关联信息（PR 和分支）
    pub fn show(
        jira_id: Option<String>,
        table: bool,
        json: bool,
        yaml: bool,
        markdown: bool,
    ) -> Result<()> {
        // 获取 JIRA ID（从参数或交互式输入）
        let jira_id = get_jira_id(jira_id, None)?;

        // 确定输出格式
        let format = OutputFormat::from_args(table, json, yaml, markdown);

        // 根据输出格式选择不同的显示方式
        match format {
            OutputFormat::Json => Self::output_json(&jira_id)?,
            OutputFormat::Yaml => Self::output_yaml(&jira_id)?,
            OutputFormat::Markdown => Self::output_markdown(&jira_id)?,
            OutputFormat::Table => Self::output_table(&jira_id)?,
        }

        Ok(())
    }

    /// 表格格式输出
    fn output_table(jira_id: &str) -> Result<()> {
        log_break!();
        log_break!('=', 40, "Related Information");
        log_message!("Jira Ticket: {}", jira_id);
        log_break!();

        // 1. 查找关联的 PR
        let pr_entries = JiraWorkHistory::find_prs_by_jira_ticket(jira_id)?;
        if !pr_entries.is_empty() {
            log_message!("Related Pull Requests:");
            for entry in &pr_entries {
                if let Some(pr_url) = &entry.pull_request_url {
                    log_message!("  - {}", pr_url);
                    if let Some(branch) = &entry.branch {
                        log_message!("    Branch: {}", branch);
                    }
                    if let Some(created) = &entry.created_at {
                        log_message!("    Created: {}", created);
                    }
                    if let Some(merged) = &entry.merged_at {
                        log_message!("    Merged: {}", merged);
                    }
                }
            }
        } else {
            log_message!("Related Pull Requests: None");
        }

        // 2. 查找关联的分支（从工作历史记录）
        let branches_from_history = JiraWorkHistory::find_branches_by_jira_ticket(jira_id)?;

        // 3. 从 Git 仓库中查找包含 ticket ID 的分支（补充）
        let branches_from_git = Self::find_branches_by_ticket_id(jira_id)?;

        // 合并并去重
        let mut all_branches: Vec<String> = branches_from_history;
        for branch in branches_from_git {
            if !all_branches.contains(&branch) {
                all_branches.push(branch);
            }
        }

        if !all_branches.is_empty() {
            log_break!();
            log_message!("Related Branches:");
            for branch in &all_branches {
                // 检查分支是否存在
                let (local_exists, remote_exists) = match GitBranch::is_branch_exists(branch) {
                    Ok(result) => result,
                    Err(_) => {
                        // 如果不在 Git 仓库中，跳过状态检查
                        log_message!("  - {}", branch);
                        continue;
                    }
                };
                let status = match (local_exists, remote_exists) {
                    (true, true) => "(local & remote)",
                    (true, false) => "(local only)",
                    (false, true) => "(remote only)",
                    (false, false) => "(not found)",
                };
                log_message!("  - {} {}", branch, status);
            }
        } else {
            log_break!();
            log_message!("Related Branches: None");
        }

        Ok(())
    }

    /// JSON 格式输出
    fn output_json(jira_id: &str) -> Result<()> {
        let mut output: HashMap<String, serde_json::Value> = HashMap::new();
        output.insert("jira_ticket".to_string(), serde_json::json!(jira_id));

        let pr_entries = JiraWorkHistory::find_prs_by_jira_ticket(jira_id)?;
        let prs: Vec<serde_json::Value> = pr_entries
            .iter()
            .map(|e| {
                let mut pr = serde_json::Map::new();
                if let Some(url) = &e.pull_request_url {
                    pr.insert("url".to_string(), serde_json::json!(url));
                }
                if let Some(branch) = &e.branch {
                    pr.insert("branch".to_string(), serde_json::json!(branch));
                }
                if let Some(created) = &e.created_at {
                    pr.insert("created_at".to_string(), serde_json::json!(created));
                }
                if let Some(merged) = &e.merged_at {
                    pr.insert("merged_at".to_string(), serde_json::json!(merged));
                }
                serde_json::Value::Object(pr)
            })
            .collect();
        output.insert("pull_requests".to_string(), serde_json::json!(prs));

        let branches_from_history = JiraWorkHistory::find_branches_by_jira_ticket(jira_id)?;
        let branches_from_git = Self::find_branches_by_ticket_id(jira_id)?;
        let mut all_branches: Vec<String> = branches_from_history;
        for branch in branches_from_git {
            if !all_branches.contains(&branch) {
                all_branches.push(branch);
            }
        }
        output.insert("branches".to_string(), serde_json::json!(all_branches));

        log_message!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    /// YAML 格式输出
    fn output_yaml(jira_id: &str) -> Result<()> {
        // 暂时使用 JSON 格式
        Self::output_json(jira_id)
    }

    /// Markdown 格式输出
    fn output_markdown(jira_id: &str) -> Result<()> {
        log_message!("# Related Information for {}\n", jira_id);

        let pr_entries = JiraWorkHistory::find_prs_by_jira_ticket(jira_id)?;
        if !pr_entries.is_empty() {
            log_message!("## Related Pull Requests\n");
            for entry in &pr_entries {
                if let Some(pr_url) = &entry.pull_request_url {
                    log_message!("- [{}]({})", pr_url, pr_url);
                    if let Some(branch) = &entry.branch {
                        log_message!("  - Branch: `{}`", branch);
                    }
                    if let Some(created) = &entry.created_at {
                        log_message!("  - Created: {}", created);
                    }
                    if let Some(merged) = &entry.merged_at {
                        log_message!("  - Merged: {}", merged);
                    }
                }
            }
        } else {
            log_message!("## Related Pull Requests\n\nNone\n");
        }

        let branches_from_history = JiraWorkHistory::find_branches_by_jira_ticket(jira_id)?;
        let branches_from_git = Self::find_branches_by_ticket_id(jira_id)?;
        let mut all_branches: Vec<String> = branches_from_history;
        for branch in branches_from_git {
            if !all_branches.contains(&branch) {
                all_branches.push(branch);
            }
        }

        if !all_branches.is_empty() {
            log_message!("\n## Related Branches\n");
            for branch in &all_branches {
                let (local_exists, remote_exists) = match GitBranch::is_branch_exists(branch) {
                    Ok(result) => result,
                    Err(_) => {
                        log_message!("- `{}`", branch);
                        continue;
                    }
                };
                let status = match (local_exists, remote_exists) {
                    (true, true) => "local & remote",
                    (true, false) => "local only",
                    (false, true) => "remote only",
                    (false, false) => "not found",
                };
                log_message!("- `{}` ({})", branch, status);
            }
        } else {
            log_message!("\n## Related Branches\n\nNone\n");
        }

        Ok(())
    }

    /// 从 Git 仓库中查找包含 ticket ID 的分支
    fn find_branches_by_ticket_id(jira_ticket: &str) -> Result<Vec<String>> {
        // 使用 GitBranch::get_all_branches 获取所有分支（包括本地和远程）
        let all_branches = match GitBranch::get_all_branches(false) {
            Ok(branches) => branches,
            Err(_) => {
                // 如果不在 Git 仓库中，返回空列表
                return Ok(Vec::new());
            }
        };

        // 过滤包含 ticket ID 的分支
        let matching_branches: Vec<String> =
            all_branches.into_iter().filter(|branch| branch.contains(jira_ticket)).collect();

        Ok(matching_branches)
    }
}
