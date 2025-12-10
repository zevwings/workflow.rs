use crate::base::util::table::{TableBuilder, TableStyle};
use crate::pr::platform::create_provider;
use crate::{log_break, log_message};
use anyhow::Result;

/// PR 列表命令
#[allow(dead_code)]
pub struct PullRequestListCommand;

#[allow(dead_code)]
impl PullRequestListCommand {
    /// 列出 PR
    pub fn list(state: Option<String>, limit: Option<u32>) -> Result<()> {
        log_break!('=', 40, "PR List");
        let provider = create_provider()?;

        // 默认只获取 open 状态的 PR
        let state = state.as_deref().unwrap_or("open");

        // 通过 trait 方法获取表格行数据
        let rows = provider.get_pull_requests(Some(state), limit)?;

        if rows.is_empty() {
            log_message!("No PRs found.");
            return Ok(());
        }

        // 使用表格显示
        log_message!(
            "{}",
            TableBuilder::new(rows)
                .with_title("Pull Requests")
                .with_style(TableStyle::Modern)
                .render()
        );

        Ok(())
    }
}
