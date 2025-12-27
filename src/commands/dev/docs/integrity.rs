//! 文档完整性检查命令
//!
//! 检查项目文档的完整性和格式。

use crate::base::util::directory::DirectoryWalker;
use crate::base::util::file::FileReader;
use crate::{log_break, log_info, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};
use regex::Regex;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

/// 检查结果
#[derive(Debug, Default)]
struct CheckResult {
    missing_architecture_docs: Vec<(String, PathBuf)>,
    invalid_timestamps: Vec<PathBuf>,
}

/// 文档完整性检查命令
pub struct DocsIntegrityCheckCommand {
    architecture: bool,
    timestamps: bool,
    ci: bool,
}

impl DocsIntegrityCheckCommand {
    /// 创建新的文档完整性检查命令
    pub fn new(architecture: bool, timestamps: bool, ci: bool) -> Self {
        Self {
            architecture,
            timestamps,
            ci,
        }
    }

    /// 检查文档完整性
    pub fn check(&self) -> Result<()> {
        log_break!('=');
        log_info!("文档完整性检查");
        log_break!('=');
        log_break!();

        let mut result = CheckResult::default();
        let mut has_issues = false;

        // 如果未指定具体检查项，则检查所有项
        let check_architecture = self.architecture || !self.timestamps;
        let check_timestamps = self.timestamps || !self.architecture;

        // 检查架构文档存在性
        if check_architecture {
            log_info!("📝 检查架构文档存在性...");
            self.check_architecture_docs(&mut result)?;

            if !result.missing_architecture_docs.is_empty() {
                has_issues = true;
                log_break!();
                log_info!(
                    "📋 发现 {} 个缺失的架构文档:",
                    result.missing_architecture_docs.len()
                );
                for (module, doc_path) in &result.missing_architecture_docs {
                    log_warning!("  模块 '{}' 缺少架构文档: {}", module, doc_path.display());
                }
            } else {
                log_success!("所有模块都有架构文档");
            }
            log_break!();
        }

        // 检查文档时间戳格式
        if check_timestamps {
            log_info!("📅 检查文档时间戳格式...");
            self.check_timestamp_format(&mut result)?;

            if !result.invalid_timestamps.is_empty() {
                has_issues = true;
                log_break!();
                log_info!(
                    "📋 发现 {} 个文档的时间戳格式无效:",
                    result.invalid_timestamps.len()
                );
                for file in &result.invalid_timestamps {
                    log_warning!("  无效的时间戳格式: {}", file.display());
                }
            } else {
                log_success!("所有文档都有有效的时间戳格式");
            }
            log_break!();
        }

        // CI 模式：输出到 GITHUB_OUTPUT
        if self.ci {
            self.output_ci_result(&result, has_issues)?;
            return Ok(());
        }

        // 本地模式：如果有问题则退出
        if has_issues {
            std::process::exit(1);
        }

        log_success!("文档完整性检查完成");
        Ok(())
    }

    /// 检查架构文档存在性
    fn check_architecture_docs(&self, result: &mut CheckResult) -> Result<()> {
        // 检查 lib 层模块
        log_info!("检查所有 lib 层模块...");
        let lib_walker = DirectoryWalker::new("src/lib");
        let lib_dirs = lib_walker.list_direct_dirs()?;

        for dir in lib_dirs {
            let module = dir
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| color_eyre::eyre::eyre!("Invalid module name"))?;

            let doc_path = PathBuf::from("docs/architecture").join(format!("{}.md", module));
            if !doc_path.exists() {
                let doc_path_clone = doc_path.clone();
                result.missing_architecture_docs.push((module.to_string(), doc_path));
                log_warning!(
                    "  Missing: {} (module: {})",
                    doc_path_clone.display(),
                    module
                );
            } else {
                log_success!("  {} -> {}", module, doc_path.display());
            }
        }

        // 检查 commands 层模块
        log_break!();
        log_info!("检查所有 commands 层模块...");
        let cmd_walker = DirectoryWalker::new("src/commands");
        let cmd_dirs = cmd_walker.list_direct_dirs()?;

        for dir in cmd_dirs {
            let module = dir
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| color_eyre::eyre::eyre!("Invalid module name"))?;

            let doc_path = PathBuf::from("docs/architecture").join(format!("{}.md", module));
            if !doc_path.exists() {
                let doc_path_clone = doc_path.clone();
                result.missing_architecture_docs.push((module.to_string(), doc_path));
                log_warning!(
                    "  Missing: {} (module: {})",
                    doc_path_clone.display(),
                    module
                );
            } else {
                log_success!("  {} -> {}", module, doc_path.display());
            }
        }

        Ok(())
    }

    /// 检查文档时间戳格式
    fn check_timestamp_format(&self, result: &mut CheckResult) -> Result<()> {
        let timestamp_pattern = Regex::new(r"\*\*最后更新\*\*: \d{4}-\d{2}-\d{2}")?;

        let docs_walker = DirectoryWalker::new("docs");
        let all_files = docs_walker.list_files()?;

        // 过滤文档文件
        let doc_files: Vec<PathBuf> = all_files
            .into_iter()
            .filter(|f| {
                f.extension().map(|e| e == "md").unwrap_or(false) && !self.should_skip_file(f)
            })
            .collect();

        let mut checked_count = 0;
        for file_path in doc_files {
            checked_count += 1;

            let reader = FileReader::new(&file_path);
            let lines = reader.lines()?;

            // 检查最后5行
            let last_lines: Vec<&str> = lines.iter().rev().take(5).map(|s| s.as_str()).collect();

            let has_valid_timestamp =
                last_lines.iter().any(|line| timestamp_pattern.is_match(line));

            if !has_valid_timestamp {
                result.invalid_timestamps.push(file_path);
            }
        }

        log_info!("检查了 {} 个文档", checked_count);
        Ok(())
    }

    /// 判断是否应该跳过文件
    fn should_skip_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        path_str.contains("/templates/")
            || path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n == "README.md")
                .unwrap_or(false)
    }

    /// 输出 CI 模式结果到 GITHUB_OUTPUT
    fn output_ci_result(&self, result: &CheckResult, has_issues: bool) -> Result<()> {
        if let Ok(output_file) = std::env::var("GITHUB_OUTPUT") {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&output_file)
                .wrap_err_with(|| format!("Failed to open GITHUB_OUTPUT: {}", output_file))?;

            writeln!(file, "docs_integrity_passed={}", !has_issues)
                .wrap_err("Failed to write integrity_passed")?;
            writeln!(
                file,
                "docs_missing_architecture={}",
                result.missing_architecture_docs.len()
            )
            .wrap_err("Failed to write missing_architecture")?;
            writeln!(
                file,
                "docs_invalid_timestamps={}",
                result.invalid_timestamps.len()
            )
            .wrap_err("Failed to write invalid_timestamps")?;
        }

        // CI 模式：非阻塞退出
        Ok(())
    }
}
