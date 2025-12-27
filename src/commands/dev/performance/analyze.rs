//! 性能回归分析命令
//!
//! 对比当前性能与基准性能，检测性能回归。

use crate::{log_error, log_success, log_warning};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 性能数据摘要
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Summary {
    #[serde(default)]
    duration_secs: f64,
}

/// 测试用例结果
#[derive(Debug, Clone, Deserialize, Serialize)]
struct TestCase {
    name: String,
    #[serde(default)]
    duration_secs: f64,
}

/// 性能数据
#[derive(Debug, Clone, Deserialize, Serialize)]
struct PerformanceData {
    summary: Summary,
    #[serde(default)]
    test_cases: Vec<TestCase>,
}

/// 测试变化信息
#[derive(Debug, Clone)]
struct TestChange {
    current: f64,
    baseline: f64,
    diff: f64,
    diff_percent: f64,
}

/// 性能对比结果
#[derive(Debug, Clone)]
struct ComparisonResult {
    regression: bool,
    overall: TestChange,
    test_changes: HashMap<String, TestChange>,
}

/// 性能回归分析命令
pub struct PerformanceAnalyzeCommand {
    current: Option<String>,
    baseline: Option<String>,
    output: Option<String>,
    threshold: f64,
}

impl PerformanceAnalyzeCommand {
    /// 创建新的性能回归分析命令
    pub fn new(
        current: Option<String>,
        baseline: Option<String>,
        output: Option<String>,
        threshold: Option<f64>,
    ) -> Self {
        Self {
            current,
            baseline,
            output,
            threshold: threshold.unwrap_or(0.2),
        }
    }

    /// 分析性能回归
    pub fn analyze(&self) -> Result<()> {
        // 确定当前性能数据文件路径
        let current_path = match &self.current {
            Some(path) => Path::new(path).to_path_buf(),
            None => {
                log_error!("Error: --current is required");
                return Err(color_eyre::eyre::eyre!("--current parameter is required"));
            }
        };

        if !current_path.exists() {
            log_error!(
                "Error: Current performance file not found: {}",
                current_path.display()
            );
            return Err(color_eyre::eyre::eyre!(
                "Current performance file not found: {}",
                current_path.display()
            ));
        }

        // 加载当前性能数据
        let current_data = self.load_performance_data(&current_path)?;

        // 加载基准性能数据
        let baseline_data = if let Some(ref baseline_path) = self.baseline {
            let baseline_path = Path::new(baseline_path);
            if baseline_path.exists() {
                self.load_performance_data(baseline_path)?
            } else {
                log_warning!("Baseline file not found, using current as baseline");
                current_data.clone()
            }
        } else {
            log_warning!("No baseline specified, using current as baseline");
            current_data.clone()
        };

        // 对比性能
        let comparison = self.compare_performance(&current_data, &baseline_data);

        // 确定输出文件路径
        let output_path = match &self.output {
            Some(path) => Path::new(path).to_path_buf(),
            None => Path::new("performance-regression-report.md").to_path_buf(),
        };

        // 生成报告
        self.generate_performance_report(&comparison, &output_path)?;

        // 如果有回归，返回错误
        if comparison.regression {
            log_warning!("Performance regression detected!");
            log_warning!("   Report generated: {}", output_path.display());
            return Err(color_eyre::eyre::eyre!("Performance regression detected"));
        }

        log_success!("Performance report generated: {}", output_path.display());
        Ok(())
    }

    /// 加载性能数据
    fn load_performance_data(&self, metrics_file: &Path) -> Result<PerformanceData> {
        let content = fs::read_to_string(metrics_file)?;
        let data: PerformanceData = serde_json::from_str(&content)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to parse performance data: {}", e))?;
        Ok(data)
    }

    /// 对比当前性能和基准性能
    fn compare_performance(
        &self,
        current: &PerformanceData,
        baseline: &PerformanceData,
    ) -> ComparisonResult {
        let current_duration = current.summary.duration_secs;
        let baseline_duration = baseline.summary.duration_secs;

        // 如果基准时长为 0，返回无变化结果
        if baseline_duration == 0.0 {
            return ComparisonResult {
                regression: false,
                overall: TestChange {
                    current: current_duration,
                    baseline: baseline_duration,
                    diff: 0.0,
                    diff_percent: 0.0,
                },
                test_changes: HashMap::new(),
            };
        }

        let diff = current_duration - baseline_duration;
        let diff_percent = (diff / baseline_duration) * 100.0;

        // 按测试对比
        let mut test_changes = HashMap::new();
        let current_tests: HashMap<String, &TestCase> =
            current.test_cases.iter().map(|t| (t.name.clone(), t)).collect();
        let baseline_tests: HashMap<String, &TestCase> =
            baseline.test_cases.iter().map(|t| (t.name.clone(), t)).collect();

        let all_tests: std::collections::HashSet<String> =
            current_tests.keys().chain(baseline_tests.keys()).cloned().collect();

        for test_name in all_tests {
            let current_test = current_tests.get(&test_name);
            let baseline_test = baseline_tests.get(&test_name);

            let current_dur = current_test.map(|t| t.duration_secs).unwrap_or(0.0);
            let baseline_dur = baseline_test.map(|t| t.duration_secs).unwrap_or(0.0);

            if baseline_dur > 0.0 {
                let test_diff = current_dur - baseline_dur;
                let test_diff_percent = (test_diff / baseline_dur) * 100.0;

                // 如果变化超过阈值，记录
                if test_diff_percent.abs() > self.threshold * 100.0 {
                    test_changes.insert(
                        test_name,
                        TestChange {
                            current: current_dur,
                            baseline: baseline_dur,
                            diff: test_diff,
                            diff_percent: test_diff_percent,
                        },
                    );
                }
            }
        }

        // 判断是否有回归（性能下降超过阈值）
        let regression = diff_percent > self.threshold * 100.0;

        ComparisonResult {
            regression,
            overall: TestChange {
                current: current_duration,
                baseline: baseline_duration,
                diff,
                diff_percent,
            },
            test_changes,
        }
    }

    /// 生成性能回归报告
    fn generate_performance_report(
        &self,
        comparison: &ComparisonResult,
        output: &Path,
    ) -> Result<()> {
        let overall = &comparison.overall;
        let regression = comparison.regression;

        let mut report_lines = vec![
            "# Performance Regression Report".to_string(),
            "".to_string(),
            format!("**Generated**: {}", chrono::Utc::now().to_rfc3339()),
            "".to_string(),
            "## Overall Performance".to_string(),
            "".to_string(),
            format!("**Current Duration**: {:.2}s", overall.current),
            format!("**Baseline Duration**: {:.2}s", overall.baseline),
            format!(
                "**Change**: {:+.2}s ({:+.2}%)",
                overall.diff, overall.diff_percent
            ),
            "".to_string(),
        ];

        if regression {
            report_lines.extend(vec![
                "## ⚠️ Performance Regression Detected".to_string(),
                "".to_string(),
                format!("Performance degraded by {:.2}%.", overall.diff_percent),
                "Please review and optimize slow tests.".to_string(),
                "".to_string(),
            ]);
        } else if overall.diff_percent < 0.0 {
            report_lines.extend(vec![
                "## ✅ Performance Improved".to_string(),
                "".to_string(),
                format!(
                    "Performance improved by {:.2}%.",
                    overall.diff_percent.abs()
                ),
                "".to_string(),
            ]);
        }

        // 测试变化
        if !comparison.test_changes.is_empty() {
            report_lines.extend(vec![
                "## Test Performance Changes".to_string(),
                "".to_string(),
                "| Test Name | Current | Baseline | Change |".to_string(),
                "|-----------|---------|----------|--------|".to_string(),
            ]);

            let mut sorted_changes: Vec<(&String, &TestChange)> =
                comparison.test_changes.iter().collect();
            sorted_changes.sort_by(|a, b| {
                b.1.diff.partial_cmp(&a.1.diff).unwrap_or(std::cmp::Ordering::Equal)
            });

            for (test_name, change) in sorted_changes {
                let diff_str = format!("{:+.2}s ({:+.2}%)", change.diff, change.diff_percent);
                report_lines.push(format!(
                    "| `{}` | {:.2}s | {:.2}s | {} |",
                    test_name, change.current, change.baseline, diff_str
                ));
            }
            report_lines.push("".to_string());
        }

        let report = report_lines.join("\n");
        fs::write(output, report)?;

        Ok(())
    }
}
