use crate::base::table::{TableBuilder, TableStyle};
use crate::pr::platform::create_provider_auto;
use crate::{br, info};
use color_eyre::Result;

/// PR 列表命令
#[allow(dead_code)]
pub struct PullRequestListCommand;

#[allow(dead_code)]
impl PullRequestListCommand {
    /// 列出 PR
    pub fn list(state: Option<String>, limit: Option<usize>) -> Result<()> {
        br!('=', 40, "PR List");
        let provider = create_provider_auto()?;

        // 默认只获取 open 状态的 PR
        let state = state.as_deref().unwrap_or("open");

        // 通过 trait 方法获取表格行数据
        let rows = provider.get_pull_requests(Some(state), limit)?;

        if rows.is_empty() {
            info!("No PRs found.");
            return Ok(());
        }

        // 使用表格显示
        info!(
            "{}",
            TableBuilder::new(rows)
                .with_title("Pull Requests")
                .with_style(TableStyle::Modern)
                .render()
        );

        Ok(())
    }
}
