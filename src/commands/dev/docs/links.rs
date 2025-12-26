//! 文档链接检查命令
//!
//! 检查文档中的链接有效性。

use crate::base::util::directory::DirectoryWalker;
use crate::base::util::file::FileReader;
use color_eyre::{eyre::WrapErr, Result};
use duct::cmd;
use regex::Regex;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use crate::{log_info, log_success, log_warning, log_error, log_break};

/// 断链信息
#[derive(Debug, Clone)]
struct BrokenLink {
    file: PathBuf,
    link: String,
    target_file: PathBuf,
}

/// 文档链接检查命令
pub struct DocsLinksCheckCommand {
    external: bool,
    ci: bool,
}

impl DocsLinksCheckCommand {
    /// 创建新的文档链接检查命令
    pub fn new(external: bool, ci: bool) -> Self {
        Self { external, ci }
    }

    /// 检查文档链接
    pub fn check(&self) -> Result<()> {
        log_break!('=');
        log_info!("文档链接有效性检查");
        log_break!('=');
        log_break!();

        let mut broken_links = Vec::new();
        let mut internal_link_count = 0;

        // 检查内部链接
        log_info!("📋 检查内部链接...");
        self.check_internal_links(&mut broken_links, &mut internal_link_count)?;

        // 显示断链信息
        if !broken_links.is_empty() {
            log_break!();
            log_info!("发现的断链:");
            for broken_link in &broken_links {
                log_error!(
                    "  断链: {} -> {} (目标文件: {})",
                    broken_link.file.display(),
                    broken_link.link,
                    broken_link.target_file.display()
                );
            }
            log_break!();
        }

        log_info!("检查了 {} 个内部链接", internal_link_count);
        if broken_links.is_empty() {
            log_success!("所有内部链接有效");
        } else {
            log_error!("发现 {} 个断链", broken_links.len());
        }

        // 检查外部链接（如果指定）
        if self.external {
            log_break!();
            log_info!("📋 检查外部链接...");
            self.check_external_links()?;
        } else {
            log_break!();
            log_info!("跳过外部链接检查（使用 --external 启用）");
            log_info!("   安装方法: cargo install lychee");
        }

        log_break!();
        log_success!("链接检查完成");

        // CI 模式：输出到 GITHUB_OUTPUT
        if self.ci {
            self.output_ci_result(&broken_links)?;
            return Ok(());
        }

        // 本地模式：如果有断链则退出
        if !broken_links.is_empty() {
            std::process::exit(1);
        }

        Ok(())
    }

    /// 检查内部链接
    fn check_internal_links(
        &self,
        broken_links: &mut Vec<BrokenLink>,
        link_count: &mut usize,
    ) -> Result<()> {
        let link_pattern = Regex::new(r"\]\(([^)]+)\)")?;

        let docs_walker = DirectoryWalker::new("docs");
        let all_files = docs_walker.list_files()?;

        // 过滤文档文件
        let doc_files: Vec<PathBuf> = all_files
            .into_iter()
            .filter(|f| {
                f.extension().map(|e| e == "md").unwrap_or(false)
                    && !self.should_skip_file(f)
            })
            .collect();

        for file_path in doc_files {
            let reader = FileReader::new(&file_path);
            let content = reader.to_string()?;

            // 提取所有链接
            for cap in link_pattern.captures_iter(&content) {
                if let Some(link_match) = cap.get(1) {
                    let link = link_match.as_str();
                    *link_count += 1;

                    // 跳过空链接
                    if link.is_empty() {
                        continue;
                    }

                    // 跳过外部链接
                    if link.starts_with("http://") || link.starts_with("https://") {
                        continue;
                    }

                    // 跳过锚点链接（只检查文件存在性）
                    if link.starts_with('#') {
                        continue;
                    }

                    // 解析链接路径
                    let target_file = self.resolve_link_path(&file_path, link)?;

                    // 检查文件是否存在
                    if !target_file.exists() {
                        broken_links.push(BrokenLink {
                            file: file_path.clone(),
                            link: link.to_string(),
                            target_file,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// 解析链接路径
    fn resolve_link_path(&self, file_path: &Path, link: &str) -> Result<PathBuf> {
        // 移除锚点部分（#anchor）
        let link_without_anchor = link.split('#').next().unwrap_or(link);

        let target_file = if link_without_anchor.starts_with('/') {
            // 绝对路径（从项目根目录开始）
            PathBuf::from(&link_without_anchor[1..])
        } else {
            // 相对路径
            let file_dir = file_path.parent()
                .ok_or_else(|| color_eyre::eyre::eyre!("Invalid file path"))?;
            file_dir.join(link_without_anchor)
        };

        // 规范化路径（移除多余的 ./ 和 ..）
        let mut normalized = PathBuf::new();
        for component in target_file.components() {
            match component {
                std::path::Component::ParentDir => {
                    normalized.pop();
                }
                std::path::Component::CurDir => {
                    // 忽略当前目录
                }
                _ => {
                    normalized.push(component);
                }
            }
        }

        Ok(normalized)
    }

    /// 判断是否应该跳过文件
    fn should_skip_file(&self, path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy();
        path_str.contains("/templates/")
    }

    /// 检查外部链接
    fn check_external_links(&self) -> Result<()> {
        // 检查 lychee 是否安装
        let lychee_check = cmd("lychee", ["--version"])
            .stdout_null()
            .stderr_null()
            .unchecked()
            .run();

        if lychee_check.is_err() || !lychee_check?.status.success() {
            log_warning!("lychee 未安装，跳过外部链接检查");
            log_warning!("   安装方法: cargo install lychee");
            log_warning!("   注意: 外部链接检查在 CI/CD 中会自动运行");
            return Ok(());
        }

        log_success!("lychee 已安装");

        // 运行 lychee 检查外部链接
        let result = cmd("lychee", ["docs/**/*.md", "--exclude-all-private", "--exclude-loopback"])
            .stderr_capture()
            .unchecked()
            .run()
            .wrap_err("Failed to run lychee")?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            log_warning!("发现外部链接问题:");
            log_warning!("{}", stderr);
        } else {
            log_success!("所有外部链接有效");
        }

        Ok(())
    }

    /// 输出 CI 模式结果到 GITHUB_OUTPUT
    fn output_ci_result(&self, broken_links: &[BrokenLink]) -> Result<()> {
        if let Ok(output_file) = std::env::var("GITHUB_OUTPUT") {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&output_file)
                .wrap_err_with(|| format!("Failed to open GITHUB_OUTPUT: {}", output_file))?;

            writeln!(file, "docs_links_passed={}", broken_links.is_empty())
                .wrap_err("Failed to write links_passed")?;
            writeln!(file, "docs_broken_links={}", broken_links.len())
                .wrap_err("Failed to write broken_links")?;
        }

        // CI 模式：非阻塞退出
        Ok(())
    }
}
