//! 测试指标相关命令
//!
//! 提供测试指标收集等功能。

use crate::base::util::file::{FileReader, FileWriter};
use color_eyre::{eyre::WrapErr, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{log_info, log_success, log_break};

/// 测试报告摘要
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct ReportSummary {
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

/// 测试用例
#[derive(Debug, Clone, Deserialize, Serialize)]
struct TestCase {
    name: String,
    #[serde(default)]
    module: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    duration_secs: f64,
}

/// 测试报告
#[derive(Debug, Clone, Deserialize, Serialize)]
struct TestReport {
    #[serde(default)]
    summary: ReportSummary,
    #[serde(default)]
    test_cases: Vec<TestCase>,
}

/// 模块统计
#[derive(Debug, Clone, Serialize)]
struct ModuleStats {
    total: usize,
    passed: usize,
    failed: usize,
    ignored: usize,
    timeout: usize,
    total_duration: f64,
    failure_rate: f64,
    success_rate: f64,
}

/// 指标数据
#[derive(Debug, Serialize)]
struct MetricsData {
    timestamp: String,
    summary: ReportSummary,
    module_stats: HashMap<String, ModuleStats>,
    test_cases: Vec<TestCase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<Metadata>,
}

/// 元数据
#[derive(Debug, Clone, Serialize)]
struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    test_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform: Option<String>,
}

/// 测试指标收集命令
pub struct TestsMetricsCollectCommand {
    report: Option<String>,
    output: Option<String>,
}

impl TestsMetricsCollectCommand {
    /// 创建新的测试指标收集命令
    pub fn new(report: Option<String>, output: Option<String>) -> Self {
        Self { report, output }
    }

    /// 收集测试指标
    pub fn collect(&self) -> Result<()> {
        let report_path = self.report.as_ref()
            .ok_or_else(|| color_eyre::eyre::eyre!("--report 参数是必需的"))?;
        let output_path = self.output.as_ref()
            .ok_or_else(|| color_eyre::eyre::eyre!("--output 参数是必需的"))?;

        log_break!('=');
        log_info!("收集测试指标");
        log_break!('=');
        log_break!();

        // 读取测试报告
        log_info!("读取测试报告: {}", report_path);
        let reader = FileReader::new(report_path);
        let report: TestReport = reader.json()
            .wrap_err_with(|| format!("Failed to parse test report: {}", report_path))?;

        // 收集指标
        log_info!("收集指标数据...");
        let metrics = self.collect_metrics_from_report(&report)?;

        // 保存指标文件
        log_info!("保存指标文件: {}", output_path);
        let writer = FileWriter::new(output_path);
        let json_content = serde_json::to_string_pretty(&metrics)
            .wrap_err("Failed to serialize metrics")?;
        writer.write_str_with_dir(&json_content)
            .wrap_err_with(|| format!("Failed to write metrics file: {}", output_path))?;

        log_break!();
        log_success!("指标收集完成");
        log_info!("   总测试数: {}", metrics.summary.total);
        log_info!("   通过: {}", metrics.summary.passed);
        log_info!("   失败: {}", metrics.summary.failed);
        log_info!("   成功率: {:.2}%", metrics.summary.success_rate);
        log_info!("   模块数: {}", metrics.module_stats.len());
        log_break!();

        Ok(())
    }

    /// 从测试报告收集指标
    fn collect_metrics_from_report(&self, report: &TestReport) -> Result<MetricsData> {
        let summary = &report.summary;
        let test_cases = &report.test_cases;

        // 按模块统计
        let mut module_stats: HashMap<String, ModuleStats> = HashMap::new();

        for test in test_cases {
            let module = test.module.clone();
            let stats = module_stats.entry(module).or_insert_with(|| ModuleStats {
                total: 0,
                passed: 0,
                failed: 0,
                ignored: 0,
                timeout: 0,
                total_duration: 0.0,
                failure_rate: 0.0,
                success_rate: 0.0,
            });

            stats.total += 1;
            stats.total_duration += test.duration_secs;

            match test.status.as_str() {
                "Passed" => stats.passed += 1,
                "Failed" => stats.failed += 1,
                "Ignored" => stats.ignored += 1,
                "Timeout" => stats.timeout += 1,
                _ => {}
            }
        }

        // 计算失败率和成功率
        for stats in module_stats.values_mut() {
            if stats.total > 0 {
                stats.failure_rate = (stats.failed as f64 / stats.total as f64) * 100.0;
                stats.success_rate = (stats.passed as f64 / stats.total as f64) * 100.0;
            }
        }

        // 生成时间戳
        let timestamp = chrono::Utc::now().to_rfc3339();

        // 构建指标数据
        let metrics = MetricsData {
            timestamp,
            summary: summary.clone(),
            module_stats,
            test_cases: test_cases.clone(),
            metadata: None,
        };

        Ok(metrics)
    }
}
