use anyhow::{Context, Result};
use duct::cmd;

/// Jira CLI 命令模块
pub struct Jira;

impl Jira {
    /// 获取当前 Jira 用户邮箱
    pub fn get_current_user() -> Result<String> {
        let output = cmd("jira", &["me"])
            .read()
            .context("Failed to get Jira user (run 'jira init' to configure)")?;

        Ok(output.trim().to_string())
    }

    /// 获取 ticket 信息
    pub fn get_ticket_info(ticket: &str) -> Result<String> {
        let output = cmd("jira", &["issue", "view", ticket])
            .read()
            .context(format!("Failed to get ticket info: {}", ticket))?;

        Ok(output)
    }

    /// 更新 ticket 状态
    pub fn move_ticket(ticket: &str, status: &str) -> Result<()> {
        cmd("jira", &["issue", "move", ticket, status])
            .run()
            .context(format!(
                "Failed to move ticket {} to status {}",
                ticket, status
            ))?;

        Ok(())
    }

    /// 分配 ticket 给用户
    pub fn assign_ticket(ticket: &str, assignee: Option<&str>) -> Result<()> {
        let assignee = match assignee {
            Some(user) => user.to_string(),
            None => {
                // 如果没有指定，分配给当前用户
                Self::get_current_user()?
            }
        };

        cmd("jira", &["issue", "assign", ticket, &assignee])
            .run()
            .context(format!(
                "Failed to assign ticket {} to {}",
                ticket, assignee
            ))?;

        Ok(())
    }

    /// 添加评论到 ticket
    pub fn add_comment(ticket: &str, comment: &str) -> Result<()> {
        let child = cmd("jira", &["issue", "comment", "add", ticket])
            .stdin_bytes(comment)
            .run()
            .context(format!("Failed to add comment to ticket {}", ticket))?;

        if !child.status.success() {
            anyhow::bail!("Failed to add comment to ticket {}", ticket);
        }

        Ok(())
    }
}

