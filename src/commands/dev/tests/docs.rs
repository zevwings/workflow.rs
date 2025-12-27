//! 测试文档相关命令
//!
//! 提供测试文档检查等功能。

use crate::base::util::directory::DirectoryWalker;
use crate::base::util::file::FileReader;
use crate::{log_break, log_info, log_warning};
use color_eyre::{eyre::WrapErr, Result};
use regex::Regex;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

/// 文件统计信息
#[derive(Debug, Default, Clone)]
struct FileStats {
    path: PathBuf,
    test_count: usize,
    documented_count: usize,
    percent: f64,
}

/// 总体统计信息
#[derive(Debug, Default)]
struct OverallStats {
    total_tests: usize,
    total_documented: usize,
    total_files: usize,
    files_100: usize,
    files_partial: usize,
    files_0: usize,
    high_priority_0: usize,
    high_priority_partial: usize,
    high_priority_missing: usize,
    medium_priority_0: usize,
    medium_priority_partial: usize,
    medium_priority_missing: usize,
    low_priority_0: usize,
    low_priority_partial: usize,
    low_priority_missing: usize,
}

/// 测试文档检查命令
pub struct TestsDocsCheckCommand {
    ci: bool,
}

impl TestsDocsCheckCommand {
    /// 创建新的测试文档检查命令
    pub fn new(ci: bool) -> Self {
        Self { ci }
    }

    /// 检查测试文档
    pub fn check(&self) -> Result<()> {
        log_break!('=');
        log_info!("测试文档注释统计");
        log_break!('=');
        log_break!();

        let tests_dir = PathBuf::from("tests");
        if !tests_dir.exists() {
            log_warning!("tests 目录不存在");
            if self.ci {
                self.output_ci_result(&OverallStats::default())?;
            }
            return Ok(());
        }

        let walker = DirectoryWalker::new(&tests_dir);
        let all_files = walker.list_files()?;

        // 过滤测试文件
        let test_files: Vec<PathBuf> = all_files
            .into_iter()
            .filter(|f| {
                f.extension().map(|e| e == "rs").unwrap_or(false) && !self.should_skip_file(f)
            })
            .collect();

        let mut stats = OverallStats::default();
        let mut file_stats_list = Vec::new();
        let mut high_partial_files = Vec::new();
        let mut medium_partial_files = Vec::new();
        let mut low_0_files = Vec::new();
        let mut low_partial_files = Vec::new();

        // 分析每个文件
        for file_path in test_files {
            let file_stat = self.analyze_test_file(&file_path)?;

            if file_stat.test_count == 0 {
                continue;
            }

            stats.total_tests += file_stat.test_count;
            stats.total_documented += file_stat.documented_count;
            stats.total_files += 1;
            file_stats_list.push(file_stat.clone());

            // 分类统计
            if file_stat.percent == 100.0 {
                stats.files_100 += 1;
            } else if file_stat.percent == 0.0 {
                stats.files_0 += 1;
                if file_stat.test_count >= 20 {
                    stats.high_priority_0 += 1;
                    low_0_files.push(format!(
                        "{} - 0/{} (0%)",
                        file_stat.path.display(),
                        file_stat.test_count
                    ));
                } else if file_stat.test_count >= 10 {
                    stats.medium_priority_0 += 1;
                    low_0_files.push(format!(
                        "{} - 0/{} (0%)",
                        file_stat.path.display(),
                        file_stat.test_count
                    ));
                } else {
                    stats.low_priority_0 += 1;
                    low_0_files.push(format!(
                        "{} - 0/{} (0%)",
                        file_stat.path.display(),
                        file_stat.test_count
                    ));
                }
            } else {
                stats.files_partial += 1;
                let missing = file_stat.test_count - file_stat.documented_count;
                if file_stat.test_count >= 20 {
                    stats.high_priority_partial += 1;
                    stats.high_priority_missing += missing;
                    high_partial_files.push(format!(
                        "{} - {}/{} ({:.0}%)，缺少{}个",
                        file_stat.path.display(),
                        file_stat.documented_count,
                        file_stat.test_count,
                        file_stat.percent,
                        missing
                    ));
                } else if file_stat.test_count >= 10 {
                    stats.medium_priority_partial += 1;
                    stats.medium_priority_missing += missing;
                    medium_partial_files.push(format!(
                        "{} - {}/{} ({:.0}%)，缺少{}个",
                        file_stat.path.display(),
                        file_stat.documented_count,
                        file_stat.test_count,
                        file_stat.percent,
                        missing
                    ));
                } else {
                    stats.low_priority_partial += 1;
                    stats.low_priority_missing += missing;
                    low_partial_files.push(format!(
                        "{} - {}/{} ({:.0}%)，缺少{}个",
                        file_stat.path.display(),
                        file_stat.documented_count,
                        file_stat.test_count,
                        file_stat.percent,
                        missing
                    ));
                }
            }
        }

        // 输出统计结果
        self.print_summary(
            &stats,
            &high_partial_files,
            &medium_partial_files,
            &low_0_files,
            &low_partial_files,
        );

        // CI 模式：输出到 GITHUB_OUTPUT
        if self.ci {
            self.output_ci_result(&stats)?;
        }

        Ok(())
    }

    /// 判断是否应该跳过文件
    fn should_skip_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        path_str.contains("/common/")
            || path_str.contains("/utils/")
            || path_str.contains("/fixtures/")
    }

    /// 分析测试文件
    fn analyze_test_file(&self, file_path: &PathBuf) -> Result<FileStats> {
        let reader = FileReader::new(file_path);
        let content = reader
            .to_string()
            .wrap_err_with(|| format!("Failed to read file: {:?}", file_path))?;

        let lines: Vec<&str> = content.lines().collect();

        // 匹配测试函数：`#[test]` 或 `fn test_`
        let test_pattern = Regex::new(r"^\s*#\[test\]|^\s*fn\s+test_").unwrap();
        let doc_pattern =
            Regex::new(r"///.*(测试目的|测试场景|预期结果|## 测试目的|## 测试场景|## 预期结果)")
                .unwrap();

        let mut test_count = 0;
        let mut documented_count = 0;

        for (i, line) in lines.iter().enumerate() {
            if test_pattern.is_match(line) {
                test_count += 1;

                // 向前查找最多50行，看是否有文档注释
                let start = i.saturating_sub(50);
                let has_doc =
                    lines.iter().take(i).skip(start).any(|line| doc_pattern.is_match(line));

                if has_doc {
                    documented_count += 1;
                }
            }
        }

        let percent = if test_count > 0 {
            (documented_count as f64 / test_count as f64) * 100.0
        } else {
            0.0
        };

        Ok(FileStats {
            path: file_path.clone(),
            test_count,
            documented_count,
            percent,
        })
    }

    /// 打印统计摘要
    fn print_summary(
        &self,
        stats: &OverallStats,
        high_partial: &[String],
        medium_partial: &[String],
        low_0: &[String],
        _low_partial: &[String],
    ) {
        let total_percent = if stats.total_tests > 0 {
            (stats.total_documented as f64 / stats.total_tests as f64) * 100.0
        } else {
            0.0
        };

        log_info!("**总体统计**:");
        log_info!("- ✅ 100%完成：{} 个文件", stats.files_100);
        log_info!("- 🚧 部分完成：{} 个文件", stats.files_partial);
        log_info!("- ❌ 0%完成：{} 个文件", stats.files_0);
        log_info!(
            "- **总计**: {} 个文件，{} 个测试函数",
            stats.total_files,
            stats.total_tests
        );
        log_info!(
            "- **已添加文档注释**: {} 个测试函数",
            stats.total_documented
        );
        log_info!(
            "- **缺少文档注释**: {} 个测试函数",
            stats.total_tests - stats.total_documented
        );
        log_info!("- **覆盖率**: {:.1}%", total_percent);
        log_break!();

        log_info!("**高优先级文件（≥20个测试）**:");
        log_info!("- 0%完成：{} 个文件", stats.high_priority_0);
        log_info!(
            "- 部分完成：{} 个文件，缺少 {} 个文档注释",
            stats.high_priority_partial,
            stats.high_priority_missing
        );
        if !high_partial.is_empty() {
            log_break!();
            log_info!("部分完成的文件：");
            for file in high_partial.iter().take(20) {
                log_info!("  {}", file);
            }
        }
        log_break!();

        log_info!("**中优先级文件（10-19个测试）**:");
        log_info!("- 0%完成：{} 个文件", stats.medium_priority_0);
        log_info!(
            "- 部分完成：{} 个文件，缺少 {} 个文档注释",
            stats.medium_priority_partial,
            stats.medium_priority_missing
        );
        if !medium_partial.is_empty() {
            log_break!();
            log_info!("部分完成的文件：");
            for file in medium_partial {
                log_info!("  {}", file);
            }
        }
        log_break!();

        log_info!("**低优先级文件（<10个测试）**:");
        log_info!("- 0%完成：{} 个文件", stats.low_priority_0);
        log_info!(
            "- 部分完成：{} 个文件，缺少 {} 个文档注释",
            stats.low_priority_partial,
            stats.low_priority_missing
        );
        if stats.low_priority_0 <= 20 && !low_0.is_empty() {
            log_break!();
            log_info!("0%完成的文件：");
            for file in low_0.iter().take(20) {
                log_info!("  {}", file);
            }
        }
        log_break!();
    }

    /// 输出 CI 模式结果到 GITHUB_OUTPUT
    fn output_ci_result(&self, stats: &OverallStats) -> Result<()> {
        if let Ok(output_file) = std::env::var("GITHUB_OUTPUT") {
            let total_percent = if stats.total_tests > 0 {
                (stats.total_documented as f64 / stats.total_tests as f64) * 100.0
            } else {
                0.0
            };

            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&output_file)
                .wrap_err_with(|| format!("Failed to open GITHUB_OUTPUT: {}", output_file))?;

            writeln!(file, "test_docs_total_files={}", stats.total_files)
                .wrap_err("Failed to write total_files")?;
            writeln!(file, "test_docs_total_tests={}", stats.total_tests)
                .wrap_err("Failed to write total_tests")?;
            writeln!(file, "test_docs_documented={}", stats.total_documented)
                .wrap_err("Failed to write documented")?;
            writeln!(file, "test_docs_coverage={:.1}", total_percent)
                .wrap_err("Failed to write coverage")?;
        }

        // CI 模式：非阻塞退出
        Ok(())
    }
}
