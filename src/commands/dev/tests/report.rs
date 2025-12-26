//! 测试报告生成命令
//!
//! 提供测试报告生成、PR 评论生成等功能。

use crate::base::util::file::FileWriter;
use color_eyre::{eyre::WrapErr, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, BufRead};
use crate::{log_info, log_success, log_warning, log_break};

/// 测试状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestStatus {
    Passed,
    Failed,
    Ignored,
    Timeout,
}

impl TestStatus {
    fn as_str(&self) -> &'static str {
        match self {
            TestStatus::Passed => "Passed",
            TestStatus::Failed => "Failed",
            TestStatus::Ignored => "Ignored",
            TestStatus::Timeout => "Timeout",
        }
    }
}

/// 测试用例结果
#[derive(Debug, Clone, Deserialize, Serialize)]
struct TestCaseResult {
    name: String,
    status: String,
    duration_secs: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,
    module: String,
}

/// 测试报告摘要
#[derive(Debug, Clone, Deserialize, Serialize)]
struct ReportSummary {
    total: usize,
    passed: usize,
    failed: usize,
    ignored: usize,
    timeout: usize,
    success_rate: f64,
    duration_secs: f64,
}

/// 测试报告
#[derive(Debug, Deserialize, Serialize)]
struct TestReport {
    summary: ReportSummary,
    test_cases: Vec<TestCaseResult>,
}

/// Cargo 测试消息
#[derive(Debug, Deserialize)]
struct CargoTestMessage {
    #[serde(rename = "type")]
    msg_type: String,
    name: Option<String>,
    event: Option<String>,
    exec_time: Option<f64>,
    stdout: Option<String>,
}

/// 测试报告生成命令
pub struct TestsReportGenerateCommand {
    format: Option<String>,
    output: Option<String>,
    comment: bool,
    reports: Vec<String>,
}

impl TestsReportGenerateCommand {
    /// 创建新的测试报告生成命令
    pub fn new(
        format: Option<String>,
        output: Option<String>,
        comment: bool,
        reports: Vec<String>,
    ) -> Self {
        Self {
            format,
            output,
            comment,
            reports,
        }
    }

    /// 生成测试报告
    pub fn generate(&self) -> Result<()> {
        // 如果提供了报告文件且需要生成评论，直接生成评论
        if self.comment && !self.reports.is_empty() {
            return self.generate_comment();
        }

        // 从 stdin 读取 cargo test JSON 输出
        log_break!('=');
        log_info!("生成测试报告");
        log_break!('=');
        log_break!();
        log_info!("解析测试输出...");

        let reporter = self.parse_cargo_test_output()?;

        if reporter.test_cases.is_empty() {
            log_warning!("没有找到测试用例");
            return Ok(());
        }

        // 生成报告
        let format = self.format.as_deref().unwrap_or("json");
        let report_content = match format {
            "html" => reporter.generate_html_report(),
            "json" => reporter.generate_json_report(),
            "markdown" | "md" => reporter.generate_markdown_report(),
            "junit" => reporter.generate_junit_report(),
            "csv" => reporter.generate_csv_report(),
            _ => {
                return Err(color_eyre::eyre::eyre!(
                    "不支持的格式: {}。支持: html, json, markdown, junit, csv",
                    format
                ));
            }
        };

        // 输出报告
        if let Some(ref output_path) = self.output {
            log_info!("保存报告到: {}", output_path);
            let writer = FileWriter::new(output_path);
            writer.write_str_with_dir(&report_content)
                .wrap_err_with(|| format!("Failed to write report to: {}", output_path))?;
            log_success!("报告已保存");

            // 如果需要生成评论，也生成评论
            if self.comment {
                self.generate_comment_from_reports(vec![output_path.clone()])?;
            }
        } else {
            // 输出到 stdout
            print!("{}", report_content);
        }

        Ok(())
    }

    /// 生成 PR 评论
    fn generate_comment(&self) -> Result<()> {
        if self.reports.is_empty() {
            return Err(color_eyre::eyre::eyre!("至少需要一个报告文件"));
        }

        self.generate_comment_from_reports(self.reports.clone())
    }

    /// 从报告文件生成 PR 评论
    fn generate_comment_from_reports(&self, reports: Vec<String>) -> Result<()> {
        log_break!('=');
        log_info!("生成 PR 评论");
        log_break!('=');
        log_break!();

        // 加载并合并报告
        let mut merged_report = TestReport {
            summary: ReportSummary {
                total: 0,
                passed: 0,
                failed: 0,
                ignored: 0,
                timeout: 0,
                success_rate: 0.0,
                duration_secs: 0.0,
            },
            test_cases: Vec::new(),
        };

        for report_path in &reports {
            log_info!("加载报告: {}", report_path);
            let reader = crate::base::util::file::FileReader::new(report_path);
            let report: TestReport = reader.json()
                .wrap_err_with(|| format!("Failed to parse report: {}", report_path))?;

            merged_report = self.merge_reports(vec![merged_report, report])?;
        }

        // 生成 PR 评论内容
        let comment_content = self.generate_pr_comment(&merged_report);

        // 输出评论
        if let Some(ref output_path) = self.output {
            log_info!("保存 PR 评论到: {}", output_path);
            let writer = FileWriter::new(output_path);
            writer.write_str_with_dir(&comment_content)
                .wrap_err_with(|| format!("Failed to write PR comment to: {}", output_path))?;
            log_success!("PR 评论已保存");
        } else {
            // 输出到 stdout
            print!("{}", comment_content);
        }

        Ok(())
    }

    /// 解析 cargo test JSON 输出
    fn parse_cargo_test_output(&self) -> Result<TestReporter> {
        let mut reporter = TestReporter::new();
        let stdin = io::stdin();
        let reader = stdin.lock();

        for line in reader.lines() {
            let line = line.wrap_err("Failed to read from stdin")?;
            let line = line.trim();

            if line.is_empty() || !line.starts_with('{') {
                continue;
            }

            let msg: CargoTestMessage = match serde_json::from_str(line) {
                Ok(m) => m,
                Err(_) => continue,
            };

            if msg.msg_type == "test" {
                if let (Some(name), Some(event)) = (msg.name, msg.event) {
                    let duration = msg.exec_time.unwrap_or(0.0);
                    let module = self.extract_module(&name);
                    let status = match event.as_str() {
                        "ok" => TestStatus::Passed,
                        "failed" => TestStatus::Failed,
                        "ignored" => TestStatus::Ignored,
                        "timeout" => TestStatus::Timeout,
                        _ => continue,
                    };

                    let test_case = TestCaseResult {
                        name,
                        status: status.as_str().to_string(),
                        duration_secs: duration,
                        error_message: if status == TestStatus::Failed {
                            msg.stdout
                        } else {
                            None
                        },
                        module,
                    };

                    reporter.record_test(test_case);
                }
            }
        }

        reporter.finish();
        Ok(reporter)
    }

    /// 提取模块名
    fn extract_module(&self, test_name: &str) -> String {
        if let Some(pos) = test_name.rfind("::") {
            test_name[..pos].replace("::", "::")
        } else {
            "unknown".to_string()
        }
    }

    /// 合并多个测试报告
    fn merge_reports(&self, mut reports: Vec<TestReport>) -> Result<TestReport> {
        if reports.is_empty() {
            return Ok(TestReport {
                summary: ReportSummary {
                    total: 0,
                    passed: 0,
                    failed: 0,
                    ignored: 0,
                    timeout: 0,
                    success_rate: 0.0,
                    duration_secs: 0.0,
                },
                test_cases: Vec::new(),
            });
        }

        if reports.len() == 1 {
            return Ok(reports.remove(0));
        }

        let mut merged_summary = ReportSummary {
            total: 0,
            passed: 0,
            failed: 0,
            ignored: 0,
            timeout: 0,
            success_rate: 0.0,
            duration_secs: 0.0,
        };

        let mut test_case_map: HashMap<String, TestCaseResult> = HashMap::new();

        for report in reports {
            // 累加统计
            merged_summary.total += report.summary.total;
            merged_summary.passed += report.summary.passed;
            merged_summary.failed += report.summary.failed;
            merged_summary.ignored += report.summary.ignored;
            merged_summary.timeout += report.summary.timeout;
            merged_summary.duration_secs += report.summary.duration_secs;

            // 合并测试用例（去重，保留失败或超时的测试）
            for test_case in report.test_cases {
                let test_name = test_case.name.clone();
                if test_case_map.contains_key(&test_name) {
                    // 如果当前测试失败或超时，优先保留
                    if test_case.status == "Failed" || test_case.status == "Timeout" {
                        test_case_map.insert(test_name, test_case);
                    }
                } else {
                    test_case_map.insert(test_name, test_case);
                }
            }
        }

        merged_summary.total = test_case_map.len();
        if merged_summary.total > 0 {
            merged_summary.success_rate = (merged_summary.passed as f64 / merged_summary.total as f64) * 100.0;
        }

        Ok(TestReport {
            summary: merged_summary,
            test_cases: test_case_map.into_values().collect(),
        })
    }

    /// 生成 PR 评论内容
    fn generate_pr_comment(&self, report: &TestReport) -> String {
        let summary = &report.summary;
        let mut lines = Vec::new();

        lines.push("## 📊 Test Results".to_string());
        lines.push(String::new());
        lines.push("| Metric | Value |".to_string());
        lines.push("|--------|-------|".to_string());
        lines.push(format!("| Total Tests | {} |", summary.total));
        lines.push(format!("| ✅ Passed | {} |", summary.passed));
        lines.push(format!("| ❌ Failed | {} |", summary.failed));
        lines.push(format!("| ⏭ Ignored | {} |", summary.ignored));
        lines.push(format!("| ⏱ Timeout | {} |", summary.timeout));
        lines.push(format!("| Success Rate | {:.2}% |", summary.success_rate));
        lines.push(format!("| Duration | {:.2}s |", summary.duration_secs));
        lines.push(String::new());

        if summary.failed > 0 {
            lines.push("### ❌ Failed Tests".to_string());
            lines.push(String::new());
            for test_case in &report.test_cases {
                if test_case.status == "Failed" {
                    lines.push(format!("- `{}`", test_case.name));
                    if let Some(ref error) = test_case.error_message {
                        let error_preview = if error.len() > 200 {
                            format!("{}...", &error[..200])
                        } else {
                            error.clone()
                        };
                        lines.push(format!("  ```\n{}\n```", error_preview));
                    }
                }
            }
            lines.push(String::new());
        }

        lines.join("\n")
    }
}

/// 测试报告生成器
struct TestReporter {
    test_cases: Vec<TestCaseResult>,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: Option<chrono::DateTime<chrono::Utc>>,
    passed: usize,
    failed: usize,
    ignored: usize,
    timeout: usize,
}

impl TestReporter {
    fn new() -> Self {
        Self {
            test_cases: Vec::new(),
            start_time: chrono::Utc::now(),
            end_time: None,
            passed: 0,
            failed: 0,
            ignored: 0,
            timeout: 0,
        }
    }

    fn record_test(&mut self, test_case: TestCaseResult) {
        match test_case.status.as_str() {
            "Passed" => self.passed += 1,
            "Failed" => self.failed += 1,
            "Ignored" => self.ignored += 1,
            "Timeout" => self.timeout += 1,
            _ => {}
        }
        self.test_cases.push(test_case);
    }

    fn finish(&mut self) {
        self.end_time = Some(chrono::Utc::now());
    }

    fn total_duration(&self) -> f64 {
        self.end_time
            .map(|end| (end - self.start_time).num_milliseconds() as f64 / 1000.0)
            .unwrap_or(0.0)
    }

    fn success_rate(&self) -> f64 {
        if self.test_cases.is_empty() {
            return 0.0;
        }
        (self.passed as f64 / self.test_cases.len() as f64) * 100.0
    }

    fn generate_json_report(&self) -> String {
        let report = TestReport {
            summary: ReportSummary {
                total: self.test_cases.len(),
                passed: self.passed,
                failed: self.failed,
                ignored: self.ignored,
                timeout: self.timeout,
                success_rate: self.success_rate(),
                duration_secs: self.total_duration(),
            },
            test_cases: self.test_cases.clone(),
        };

        serde_json::to_string_pretty(&report)
            .unwrap_or_else(|_| "{}".to_string())
    }

    fn generate_html_report(&self) -> String {
        let duration = self.total_duration();
        let success_rate = self.success_rate();
        let test_rows = self.generate_test_rows_html();

        format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Report</title>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; background-color: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 20px; border-radius: 5px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .summary {{ background: #f5f5f5; padding: 20px; border-radius: 5px; margin-bottom: 20px; }}
        .summary h2 {{ margin-top: 0; color: #333; }}
        .stats {{ display: flex; flex-wrap: wrap; gap: 20px; }}
        .stat {{ display: flex; flex-direction: column; align-items: center; padding: 15px; background: white; border-radius: 5px; min-width: 100px; }}
        .stat-value {{ font-size: 28px; font-weight: bold; margin-bottom: 5px; }}
        .stat-label {{ font-size: 14px; color: #666; }}
        .passed {{ color: #28a745; }}
        .failed {{ color: #dc3545; }}
        .ignored {{ color: #ffc107; }}
        .timeout {{ color: #fd7e14; }}
        .total {{ color: #007bff; }}
        .rate {{ color: #6c757d; }}
        table {{ width: 100%; border-collapse: collapse; margin-top: 20px; }}
        th, td {{ padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }}
        th {{ background-color: #4CAF50; color: white; position: sticky; top: 0; }}
        .test-name {{ font-family: monospace; font-size: 13px; }}
        .error-message {{ color: #dc3545; font-size: 12px; font-family: monospace; background: #f8f9fa; padding: 8px; border-radius: 3px; margin-top: 5px; white-space: pre-wrap; }}
        .status-badge {{ display: inline-block; padding: 4px 8px; border-radius: 3px; font-size: 12px; font-weight: bold; }}
        .status-passed {{ background: #d4edda; color: #155724; }}
        .status-failed {{ background: #f8d7da; color: #721c24; }}
        .status-ignored {{ background: #fff3cd; color: #856404; }}
        .status-timeout {{ background: #ffeaa7; color: #d63031; }}
        .slow-test {{ background-color: #fff3cd; font-weight: bold; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Test Execution Report</h1>
        <div class="summary">
            <h2>Summary</h2>
            <div class="stats">
                <div class="stat">
                    <div class="stat-value passed">{}</div>
                    <div class="stat-label">Passed</div>
                </div>
                <div class="stat">
                    <div class="stat-value failed">{}</div>
                    <div class="stat-label">Failed</div>
                </div>
                <div class="stat">
                    <div class="stat-value ignored">{}</div>
                    <div class="stat-label">Ignored</div>
                </div>
                <div class="stat">
                    <div class="stat-value timeout">{}</div>
                    <div class="stat-label">Timeout</div>
                </div>
                <div class="stat">
                    <div class="stat-value total">{}</div>
                    <div class="stat-label">Total</div>
                </div>
                <div class="stat">
                    <div class="stat-value rate">{:.2}%</div>
                    <div class="stat-label">Success Rate</div>
                </div>
                <div class="stat">
                    <div class="stat-value rate">{:.2}s</div>
                    <div class="stat-label">Duration</div>
                </div>
            </div>
        </div>

        <h2>Test Cases</h2>
        <table>
            <thead>
                <tr>
                    <th>Test Name</th>
                    <th>Module</th>
                    <th>Status</th>
                    <th>Duration</th>
                    <th>Error</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>
    </div>
</body>
</html>"#,
            self.passed,
            self.failed,
            self.ignored,
            self.timeout,
            self.test_cases.len(),
            success_rate,
            duration,
            test_rows
        )
    }

    fn generate_test_rows_html(&self) -> String {
        let mut rows = Vec::new();
        let slow_threshold = 1.0;

        for test_case in &self.test_cases {
            let status_class = match test_case.status.as_str() {
                "Passed" => "status-passed",
                "Failed" => "status-failed",
                "Ignored" => "status-ignored",
                "Timeout" => "status-timeout",
                _ => "status-passed",
            };

            let status_text = match test_case.status.as_str() {
                "Passed" => "✓ Passed",
                "Failed" => "✗ Failed",
                "Ignored" => "⊘ Ignored",
                "Timeout" => "⏱ Timeout",
                _ => &test_case.status,
            };

            // 慢测试高亮
            let (duration_class, duration_text) = if test_case.duration_secs > slow_threshold {
                (" class=\"slow-test\"", format!("{:.3}s ⚠", test_case.duration_secs))
            } else {
                ("", format!("{:.3}s", test_case.duration_secs))
            };

            let error_html = if let Some(ref error) = test_case.error_message {
                let escaped_error = Self::html_escape(error);
                format!("<div class=\"error-message\">{}</div>", escaped_error)
            } else {
                String::new()
            };

            let escaped_name = Self::html_escape(&test_case.name);
            let escaped_module = Self::html_escape(&test_case.module);

            rows.push(format!(
                r#"<tr>
                    <td class="test-name">{}</td>
                    <td>{}</td>
                    <td><span class="status-badge {}">{}</span></td>
                    <td{}>{}</td>
                    <td>{}</td>
                </tr>"#,
                escaped_name, escaped_module, status_class, status_text, duration_class, duration_text, error_html
            ));
        }

        rows.join("\n")
    }

    fn html_escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }

    fn xml_escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    fn generate_markdown_report(&self) -> String {
        let mut lines = Vec::new();
        lines.push("# Test Execution Report".to_string());
        lines.push(String::new());
        lines.push(format!("**Generated**: {}", self.start_time.format("%Y-%m-%d %H:%M:%S")));
        lines.push(String::new());
        lines.push("## Summary".to_string());
        lines.push(String::new());
        lines.push("| Metric | Value |".to_string());
        lines.push("|--------|-------|".to_string());
        lines.push(format!("| Total Tests | {} |", self.test_cases.len()));
        lines.push(format!("| Passed | {} |", self.passed));
        lines.push(format!("| Failed | {} |", self.failed));
        lines.push(format!("| Ignored | {} |", self.ignored));
        lines.push(format!("| Timeout | {} |", self.timeout));
        lines.push(format!("| Success Rate | {:.2}% |", self.success_rate()));
        lines.push(format!("| Duration | {:.2}s |", self.total_duration()));
        lines.push(String::new());
        lines.push("## Test Cases".to_string());
        lines.push(String::new());
        lines.push("| Test Name | Module | Status | Duration |".to_string());
        lines.push("|-----------|--------|--------|----------|".to_string());

        for test_case in &self.test_cases {
            lines.push(format!(
                "| {} | {} | {} | {:.3}s |",
                test_case.name, test_case.module, test_case.status, test_case.duration_secs
            ));
        }

        lines.join("\n")
    }

    fn generate_junit_report(&self) -> String {
        let mut xml = Vec::new();
        xml.push("<?xml version=\"1.0\" encoding=\"UTF-8\"?>".to_string());

        let total_tests = self.test_cases.len();
        let failures = self.failed;
        let errors = 0;
        let skipped = self.ignored;
        let time = self.total_duration();

        xml.push(format!(
            r#"<testsuites name="cargo test" tests="{}" failures="{}" errors="{}" skipped="{}" time="{:.3}">"#,
            total_tests, failures, errors, skipped, time
        ));

        // 按模块分组
        let mut module_tests: std::collections::HashMap<String, Vec<&TestCaseResult>> = std::collections::HashMap::new();
        for test_case in &self.test_cases {
            module_tests.entry(test_case.module.clone()).or_insert_with(Vec::new).push(test_case);
        }

        for (module, tests) in module_tests {
            let module_failures = tests.iter().filter(|t| t.status == "Failed").count();
            let module_skipped = tests.iter().filter(|t| t.status == "Ignored").count();
            let module_time: f64 = tests.iter().map(|t| t.duration_secs).sum();

            xml.push(format!(
                r#"  <testsuite name="{}" tests="{}" failures="{}" skipped="{}" time="{:.3}">"#,
                Self::xml_escape(&module), tests.len(), module_failures, module_skipped, module_time
            ));

            for test_case in tests {
                let classname = format!("{}.{}", test_case.module, test_case.name.split("::").next().unwrap_or(&test_case.name));
                let name = test_case.name.split("::").last().unwrap_or(&test_case.name);
                let time = test_case.duration_secs;

                if test_case.status == "Failed" {
                    xml.push(format!(
                        r#"    <testcase classname="{}" name="{}" time="{:.3}">"#,
                        Self::xml_escape(&classname), Self::xml_escape(name), time
                    ));
                    if let Some(ref error) = test_case.error_message {
                        xml.push(format!(
                            r#"      <failure message="Test failed">{}</failure>"#,
                            Self::xml_escape(error)
                        ));
                    } else {
                        xml.push(r#"      <failure message="Test failed"></failure>"#.to_string());
                    }
                    xml.push("    </testcase>".to_string());
                } else if test_case.status == "Ignored" {
                    xml.push(format!(
                        r#"    <testcase classname="{}" name="{}" time="{:.3}">"#,
                        Self::xml_escape(&classname), Self::xml_escape(name), time
                    ));
                    xml.push("      <skipped/>".to_string());
                    xml.push("    </testcase>".to_string());
                } else {
                    xml.push(format!(
                        r#"    <testcase classname="{}" name="{}" time="{:.3}"/>"#,
                        Self::xml_escape(&classname), Self::xml_escape(name), time
                    ));
                }
            }

            xml.push("  </testsuite>".to_string());
        }

        xml.push("</testsuites>".to_string());
        xml.join("\n")
    }

    fn generate_csv_report(&self) -> String {
        let mut lines = Vec::new();
        lines.push("Test Name,Module,Status,Duration (s)".to_string());
        for test_case in &self.test_cases {
            lines.push(format!(
                "{},{},{},{:.3}",
                test_case.name, test_case.module, test_case.status, test_case.duration_secs
            ));
        }
        lines.join("\n")
    }
}
