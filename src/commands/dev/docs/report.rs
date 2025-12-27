//! 文档检查报告生成实现

use crate::log_info;
use chrono::Utc;
use color_eyre::{eyre::WrapErr, Result};
use std::env;
use std::fs;
use std::path::Path;

/// 文档检查报告生成命令
pub struct DocsReportGenerateCommand {
    output: Option<String>,
    check_type: String,
}

impl DocsReportGenerateCommand {
    /// 创建新的文档检查报告生成命令
    pub fn new(output: Option<String>, check_type: Option<String>) -> Self {
        Self {
            output,
            check_type: check_type.unwrap_or_else(|| "定期审查".to_string()),
        }
    }

    /// 生成报告
    pub fn generate(&self) -> Result<String> {
        // 确定输出文件路径
        let report_file = if let Some(ref output) = self.output {
            output.clone()
        } else {
            // 默认输出路径：report/doc-check-{timestamp}.md
            let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S");
            format!("report/doc-check-{}.md", timestamp)
        };

        // 确保输出目录存在
        if let Some(parent) = Path::new(&report_file).parent() {
            fs::create_dir_all(parent).wrap_err("Failed to create report directory")?;
        }

        // 生成报告内容
        let report_content = self.generate_report_content()?;

        // 写入文件
        fs::write(&report_file, &report_content)
            .wrap_err_with(|| format!("Failed to write report to: {}", report_file))?;

        log_info!("📄 Report generated: {}", report_file);

        // 如果设置了 GITHUB_OUTPUT，输出报告文件路径
        if let Ok(output_file) = env::var("GITHUB_OUTPUT") {
            use std::fs::OpenOptions;
            use std::io::Write;

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&output_file)
                .wrap_err_with(|| format!("Failed to open GITHUB_OUTPUT: {}", output_file))?;

            writeln!(file, "report_file={}", report_file)
                .wrap_err("Failed to write report_file to GITHUB_OUTPUT")?;
        }

        Ok(report_file)
    }

    /// 生成报告内容
    fn generate_report_content(&self) -> Result<String> {
        let check_date = Utc::now().format("%Y-%m-%d %H:%M:%S");
        let update_date = Utc::now().format("%Y-%m-%d");

        // 尝试从 GITHUB_OUTPUT 读取检查结果
        let (integrity_passed, missing_architecture, invalid_timestamps) =
            self.read_integrity_results();
        let (links_passed, broken_links) = self.read_links_results();

        // 获取仓库信息
        let repository =
            env::var("GITHUB_REPOSITORY").unwrap_or_else(|_| "unknown/repo".to_string());

        let mut content = String::new();
        content.push_str("# 文档检查报告\n\n");
        content.push_str(&format!("**检查日期**：{}\n", check_date));
        content.push_str(&format!("**检查类型**：{}\n\n", self.check_type));
        content.push_str("## 检查结果\n\n");

        // 文档链接检查
        content.push_str("### 文档链接检查\n\n");
        if links_passed {
            content.push_str("✅ 已完成文档链接有效性检查，所有链接有效。\n\n");
        } else {
            content.push_str(&format!(
                "⚠️  已完成文档链接有效性检查，发现 {} 个无效链接。\n\n",
                broken_links
            ));
        }

        // 架构文档存在性检查
        content.push_str("### 架构文档存在性检查\n\n");
        if integrity_passed && missing_architecture == 0 {
            content.push_str("✅ 已完成架构文档存在性检查，所有模块都有对应的架构文档。\n\n");
        } else {
            content.push_str(&format!(
                "⚠️  已完成架构文档存在性检查，发现 {} 个缺失的架构文档。\n\n",
                missing_architecture
            ));
        }

        // 文档时间戳格式检查
        content.push_str("### 文档时间戳格式检查\n\n");
        if integrity_passed && invalid_timestamps == 0 {
            content.push_str("✅ 已完成文档时间戳格式检查，所有文档都有正确的时间戳格式。\n\n");
        } else {
            content.push_str(&format!(
                "⚠️  已完成文档时间戳格式检查，发现 {} 个无效的时间戳格式。\n\n",
                invalid_timestamps
            ));
        }

        // 问题汇总
        content.push_str("## 问题汇总\n\n");
        let total_issues = missing_architecture + invalid_timestamps + broken_links;
        if total_issues == 0 {
            content.push_str("✅ 未发现任何问题，所有检查均通过。\n\n");
        } else {
            content.push_str(&format!("发现 {} 个问题：\n", total_issues));
            if missing_architecture > 0 {
                content.push_str(&format!("- {} 个缺失的架构文档\n", missing_architecture));
            }
            if invalid_timestamps > 0 {
                content.push_str(&format!("- {} 个无效的时间戳格式\n", invalid_timestamps));
            }
            if broken_links > 0 {
                content.push_str(&format!("- {} 个无效的链接\n", broken_links));
            }
            content.push_str("\n请查看上方的检查输出以了解详细问题。\n\n");
        }

        // 改进建议
        content.push_str("## 改进建议\n\n");
        content.push_str("1. 确保所有模块都有对应的架构文档\n");
        content.push_str("2. 确保所有文档都有正确的时间戳格式\n");
        content.push_str("3. 确保所有文档链接都有效\n\n");
        content.push_str("参考文档：\n");
        content.push_str(&format!("- [架构文档审查指南](https://github.com/{}/blob/main/docs/guidelines/development/references/review-architecture-consistency.md)\n", repository));
        content.push_str(&format!("- [文档更新检查清单](https://github.com/{}/blob/main/docs/guidelines/development/code-review.md)\n\n", repository));

        content.push_str("---\n\n");
        content.push_str(&format!("**最后更新**: {}\n", update_date));

        Ok(content)
    }

    /// 从 GITHUB_OUTPUT 读取完整性检查结果
    fn read_integrity_results(&self) -> (bool, usize, usize) {
        // 尝试从环境变量读取（如果检查命令已经运行）
        let passed = env::var("docs_integrity_passed")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(true);

        let missing = env::var("docs_missing_architecture")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        let invalid = env::var("docs_invalid_timestamps")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        (passed, missing, invalid)
    }

    /// 从 GITHUB_OUTPUT 读取链接检查结果
    fn read_links_results(&self) -> (bool, usize) {
        // 尝试从环境变量读取（如果检查命令已经运行）
        let passed = env::var("docs_links_passed")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(true);

        let broken = env::var("docs_broken_links")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        (passed, broken)
    }
}
