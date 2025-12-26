//! 测试趋势相关命令
//!
//! 提供测试趋势分析等功能。

use crate::base::util::directory::DirectoryWalker;
use crate::base::util::file::{FileReader, FileWriter};
use color_eyre::{eyre::WrapErr, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::{log_info, log_success, log_warning, log_break};

/// 指标数据摘要
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct MetricsSummary {
    #[serde(default)]
    total: usize,
    #[serde(default)]
    passed: usize,
    #[serde(default)]
    failed: usize,
    #[serde(default)]
    ignored: usize,
    #[serde(default)]
    timeout: usize,
    #[serde(default)]
    success_rate: f64,
    #[serde(default)]
    duration_secs: f64,
}

/// 指标数据
#[derive(Debug, Deserialize, Serialize)]
struct MetricsData {
    #[serde(default)]
    timestamp: String,
    #[serde(default)]
    summary: MetricsSummary,
    #[serde(default)]
    module_stats: HashMap<String, ModuleStats>,
    #[serde(default)]
    test_cases: Vec<TestCaseMetrics>,
}

/// 模块统计
#[derive(Debug, Clone, Deserialize, Serialize)]
struct ModuleStats {
    #[serde(default)]
    total: usize,
    #[serde(default)]
    passed: usize,
    #[serde(default)]
    failed: usize,
    #[serde(default)]
    ignored: usize,
    #[serde(default)]
    timeout: usize,
    #[serde(default)]
    total_duration: f64,
    #[serde(default)]
    failure_rate: f64,
    #[serde(default)]
    success_rate: f64,
}

/// 测试用例指标
#[derive(Debug, Clone, Deserialize, Serialize)]
struct TestCaseMetrics {
    #[serde(default)]
    name: String,
    #[serde(default)]
    module: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    duration_secs: f64,
}

/// 趋势数据点
#[derive(Debug, Clone, Serialize)]
struct TrendDataPoint {
    timestamp: String,
    failure_rate: f64,
    total: usize,
    failed: usize,
}

/// 不稳定测试
#[derive(Debug, Clone, Serialize)]
struct UnstableTest {
    name: String,
    runs: usize,
    failures: usize,
    failure_rate: f64,
}

/// 趋势分析结果
#[derive(Debug, Serialize)]
struct TrendAnalysis {
    failure_rate: FailureRateTrend,
    stability: StabilityAnalysis,
    duration: DurationTrend,
}

/// 失败率趋势
#[derive(Debug, Serialize)]
struct FailureRateTrend {
    overall: Vec<TrendDataPoint>,
    by_module: HashMap<String, Vec<TrendDataPoint>>,
}

/// 稳定性分析
#[derive(Debug, Serialize)]
struct StabilityAnalysis {
    unstable_tests: Vec<UnstableTest>,
    total_unstable: usize,
}

/// 执行时间趋势
#[derive(Debug, Serialize)]
struct DurationTrend {
    overall: Vec<DurationDataPoint>,
    by_module: HashMap<String, Vec<DurationDataPoint>>,
}

/// 执行时间数据点
#[derive(Debug, Clone, Serialize)]
struct DurationDataPoint {
    timestamp: String,
    duration_secs: f64,
}

/// 测试趋势分析命令
pub struct TestsTrendsAnalyzeCommand {
    metrics_dir: Option<String>,
    output: Option<String>,
}

impl TestsTrendsAnalyzeCommand {
    /// 创建新的测试趋势分析命令
    pub fn new(metrics_dir: Option<String>, output: Option<String>) -> Self {
        Self { metrics_dir, output }
    }

    /// 分析测试趋势
    pub fn analyze(&self) -> Result<()> {
        let metrics_dir = self.metrics_dir.as_ref()
            .ok_or_else(|| color_eyre::eyre::eyre!("--metrics-dir 参数是必需的"))?;
        let output_path = self.output.as_ref()
            .ok_or_else(|| color_eyre::eyre::eyre!("--output 参数是必需的"))?;

        log_break!('=');
        log_info!("分析测试趋势");
        log_break!('=');
        log_break!();

        // 加载历史指标数据
        log_info!("加载历史指标数据: {}", metrics_dir);
        let metrics = self.load_historical_metrics(metrics_dir)?;

        if metrics.is_empty() {
            log_warning!("没有找到历史指标数据");
            return Ok(());
        }

        log_info!("已加载 {} 个指标文件", metrics.len());
        log_break!();

        // 分析趋势
        log_info!("分析趋势...");
        let trends = self.analyze_trends(&metrics)?;

        // 生成报告
        log_info!("生成趋势报告: {}", output_path);
        let report = self.generate_trend_report(&trends)?;

        let writer = FileWriter::new(output_path);
        writer.write_str_with_dir(&report)
            .wrap_err_with(|| format!("Failed to write trend report to: {}", output_path))?;

        log_break!();
        log_success!("趋势分析完成");
        log_info!("   总指标数: {}", metrics.len());
        log_info!("   不稳定测试: {}", trends.stability.total_unstable);
        log_break!();

        Ok(())
    }

    /// 加载历史指标数据
    fn load_historical_metrics(&self, metrics_dir: &str) -> Result<Vec<MetricsData>> {
        let dir = PathBuf::from(metrics_dir);
        if !dir.exists() {
            return Err(color_eyre::eyre::eyre!("指标目录不存在: {}", metrics_dir));
        }

        let walker = DirectoryWalker::new(&dir);
        let files = walker.list_files()?;

        let mut metrics = Vec::new();
        for file in files {
            if file.extension().map(|e| e == "json").unwrap_or(false) {
                match FileReader::new(&file).json::<MetricsData>() {
                    Ok(data) => metrics.push(data),
                    Err(e) => {
                        log_warning!("加载指标文件失败 {}: {}", file.display(), e);
                    }
                }
            }
        }

        // 按时间戳排序
        metrics.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(metrics)
    }

    /// 分析趋势
    fn analyze_trends(&self, metrics: &[MetricsData]) -> Result<TrendAnalysis> {
        let failure_rate = self.analyze_failure_rate_trend(metrics);
        let stability = self.analyze_stability(metrics)?;
        let duration = self.analyze_duration_trend(metrics);

        Ok(TrendAnalysis {
            failure_rate,
            stability,
            duration,
        })
    }

    /// 分析失败率趋势
    fn analyze_failure_rate_trend(&self, metrics: &[MetricsData]) -> FailureRateTrend {
        let mut overall = Vec::new();
        let mut by_module: HashMap<String, Vec<TrendDataPoint>> = HashMap::new();

        for m in metrics {
            let summary = &m.summary;
            let total = summary.total;
            let failed = summary.failed;
            let failure_rate = if total > 0 {
                (failed as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            overall.push(TrendDataPoint {
                timestamp: m.timestamp.clone(),
                failure_rate,
                total,
                failed,
            });

            // 按模块统计
            for (module, stats) in &m.module_stats {
                let module_trend = by_module.entry(module.clone()).or_insert_with(Vec::new);
                module_trend.push(TrendDataPoint {
                    timestamp: m.timestamp.clone(),
                    failure_rate: stats.failure_rate,
                    total: stats.total,
                    failed: stats.failed,
                });
            }
        }

        FailureRateTrend {
            overall,
            by_module,
        }
    }

    /// 分析测试稳定性
    fn analyze_stability(&self, metrics: &[MetricsData]) -> Result<StabilityAnalysis> {
        let mut test_failures: HashMap<String, usize> = HashMap::new();
        let mut test_runs: HashMap<String, usize> = HashMap::new();

        for m in metrics {
            for test_case in &m.test_cases {
                let test_name = test_case.name.clone();
                *test_runs.entry(test_name.clone()).or_insert(0) += 1;
                if test_case.status == "Failed" {
                    *test_failures.entry(test_name).or_insert(0) += 1;
                }
            }
        }

        let mut unstable_tests = Vec::new();
        for (test_name, runs) in test_runs {
            let failures = test_failures.get(&test_name).copied().unwrap_or(0);
            let failure_rate = if runs > 0 {
                (failures as f64 / runs as f64) * 100.0
            } else {
                0.0
            };

            // 失败率在 1%-50% 之间的测试被认为是不稳定的
            if 1.0 <= failure_rate && failure_rate < 50.0 {
                unstable_tests.push(UnstableTest {
                    name: test_name,
                    runs,
                    failures,
                    failure_rate,
                });
            }
        }

        // 按失败率排序
        unstable_tests.sort_by(|a, b| b.failure_rate.partial_cmp(&a.failure_rate).unwrap_or(std::cmp::Ordering::Equal));

        Ok(StabilityAnalysis {
            total_unstable: unstable_tests.len(),
            unstable_tests,
        })
    }

    /// 分析执行时间趋势
    fn analyze_duration_trend(&self, metrics: &[MetricsData]) -> DurationTrend {
        let mut overall = Vec::new();
        let mut by_module: HashMap<String, Vec<DurationDataPoint>> = HashMap::new();

        for m in metrics {
            let summary = &m.summary;
            overall.push(DurationDataPoint {
                timestamp: m.timestamp.clone(),
                duration_secs: summary.duration_secs,
            });

            // 按模块统计
            for (module, stats) in &m.module_stats {
                let module_trend = by_module.entry(module.clone()).or_insert_with(Vec::new);
                module_trend.push(DurationDataPoint {
                    timestamp: m.timestamp.clone(),
                    duration_secs: stats.total_duration,
                });
            }
        }

        DurationTrend {
            overall,
            by_module,
        }
    }

    /// 生成趋势报告
    fn generate_trend_report(&self, trends: &TrendAnalysis) -> Result<String> {
        let mut lines = Vec::new();

        lines.push("# Test Trends Report".to_string());
        lines.push(String::new());
        lines.push(format!("**Generated**: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")));
        lines.push(String::new());

        // 失败率趋势
        lines.push("## Failure Rate Trend".to_string());
        lines.push(String::new());
        if let Some(last) = trends.failure_rate.overall.last() {
            lines.push(format!("**Latest Failure Rate**: {:.2}%", last.failure_rate));
            lines.push(format!("**Total Tests**: {}", last.total));
            lines.push(format!("**Failed Tests**: {}", last.failed));
        }
        lines.push(String::new());

        // 稳定性分析
        lines.push("## Test Stability Analysis".to_string());
        lines.push(String::new());
        lines.push(format!("**Unstable Tests**: {}", trends.stability.total_unstable));
        lines.push(String::new());

        if !trends.stability.unstable_tests.is_empty() {
            lines.push("### Unstable Tests (Failure Rate: 1%-50%)".to_string());
            lines.push(String::new());
            lines.push("| Test Name | Runs | Failures | Failure Rate |".to_string());
            lines.push("|-----------|------|----------|-------------|".to_string());

            for test in trends.stability.unstable_tests.iter().take(50) {
                lines.push(format!(
                    "| `{}` | {} | {} | {:.2}% |",
                    test.name, test.runs, test.failures, test.failure_rate
                ));
            }
            lines.push(String::new());
        }

        // 执行时间趋势
        lines.push("## Duration Trend".to_string());
        lines.push(String::new());
        if let Some(last) = trends.duration.overall.last() {
            lines.push(format!("**Latest Duration**: {:.2}s", last.duration_secs));
        }
        lines.push(String::new());

        Ok(lines.join("\n"))
    }
}
