//! PR 总结命令
//!
//! 读取 PR 修改的内容，然后使用 LLM 总结成文档。

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::base::settings::Settings;
use crate::base::settings::defaults::default_download_base_dir;
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

        // 构建输出路径
        let output_path = Self::build_output_path(&pr_id, &summary.filename)?;

        // 确保目录存在
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        // 写入文件
        fs::write(&output_path, &summary.summary)
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
}

