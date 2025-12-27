//! 测试报告生成器
//!
//! 生成测试执行报告，支持 HTML 和 JSON 格式。

use std::time::{Duration, SystemTime};

/// 单个测试用例的结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestCaseResult {
    pub name: String,
    pub status: TestStatus,
    pub duration: Duration,
    pub error_message: Option<String>,
    pub module: String,
}

/// 测试状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Ignored,
    Timeout,
}

impl TestStatus {
    /// 转换为字符串
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            TestStatus::Passed => "Passed",
            TestStatus::Failed => "Failed",
            TestStatus::Ignored => "Ignored",
            TestStatus::Timeout => "Timeout",
        }
    }
}

/// 测试报告
#[derive(Debug, Clone)]
pub struct TestReporter {
    pub test_cases: Vec<TestCaseResult>,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub timeout: usize,
}

impl TestReporter {
    /// 创建新的测试报告
    pub fn new() -> Self {
        Self {
            test_cases: Vec::new(),
            start_time: SystemTime::now(),
            end_time: None,
            passed: 0,
            failed: 0,
            ignored: 0,
            timeout: 0,
        }
    }

    /// 记录测试用例结果
    pub fn record_test(&mut self, result: TestCaseResult) {
        match result.status {
            TestStatus::Passed => self.passed += 1,
            TestStatus::Failed => self.failed += 1,
            TestStatus::Ignored => self.ignored += 1,
            TestStatus::Timeout => self.timeout += 1,
        }
        self.test_cases.push(result);
    }

    /// 完成报告生成
    pub fn finish(&mut self) {
        self.end_time = Some(SystemTime::now());
    }

    /// 获取总测试数
    pub fn total_tests(&self) -> usize {
        self.test_cases.len()
    }

    /// 获取总执行时间
    pub fn total_duration(&self) -> Duration {
        self.end_time
            .unwrap_or_else(SystemTime::now)
            .duration_since(self.start_time)
            .unwrap_or_default()
    }

    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_tests() == 0 {
            return 0.0;
        }
        (self.passed as f64) / (self.total_tests() as f64) * 100.0
    }

    /// 生成 HTML 报告
    ///
    /// # 返回
    ///
    /// 返回 HTML 格式的测试报告字符串。
    pub fn generate_html_report(&self) -> String {
        let duration = self.total_duration();
        let success_rate = self.success_rate();

        format!(
            r#"
<!DOCTYPE html>
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
</html>
        "#,
            self.passed,
            self.failed,
            self.ignored,
            self.timeout,
            self.total_tests(),
            success_rate,
            duration.as_secs_f64(),
            self.generate_test_rows_html()
        )
    }

    /// 生成测试行 HTML
    fn generate_test_rows_html(&self) -> String {
        self.test_cases
            .iter()
            .map(|test| {
                let status_class = match test.status {
                    TestStatus::Passed => "status-passed",
                    TestStatus::Failed => "status-failed",
                    TestStatus::Ignored => "status-ignored",
                    TestStatus::Timeout => "status-timeout",
                };
                let status_text = match test.status {
                    TestStatus::Passed => "✓ Passed",
                    TestStatus::Failed => "✗ Failed",
                    TestStatus::Ignored => "⊘ Ignored",
                    TestStatus::Timeout => "⏱ Timeout",
                };
                let error_html = test.error_message.as_ref().map_or_else(String::new, |msg| {
                    format!(r#"<div class="error-message">{}</div>"#, html_escape(msg))
                });

                format!(
                    r#"<tr>
                        <td class="test-name">{}</td>
                        <td>{}</td>
                        <td><span class="status-badge {}">{}</span></td>
                        <td>{:.3}s</td>
                        <td>{}</td>
                    </tr>"#,
                    html_escape(&test.name),
                    html_escape(&test.module),
                    status_class,
                    status_text,
                    test.duration.as_secs_f64(),
                    error_html
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 生成 JSON 报告
    ///
    /// # 返回
    ///
    /// 返回 JSON 格式的测试报告字符串。
    pub fn generate_json_report(&self) -> color_eyre::Result<String> {
        #[derive(serde::Serialize)]
        struct Report {
            summary: Summary,
            test_cases: Vec<TestCaseResult>,
        }

        #[derive(serde::Serialize)]
        struct Summary {
            total: usize,
            passed: usize,
            failed: usize,
            ignored: usize,
            timeout: usize,
            success_rate: f64,
            duration_secs: f64,
        }

        let report = Report {
            summary: Summary {
                total: self.total_tests(),
                passed: self.passed,
                failed: self.failed,
                ignored: self.ignored,
                timeout: self.timeout,
                success_rate: self.success_rate(),
                duration_secs: self.total_duration().as_secs_f64(),
            },
            test_cases: self.test_cases.clone(),
        };

        Ok(serde_json::to_string_pretty(&report)?)
    }

    /// 保存报告到文件
    ///
    /// # 参数
    ///
    /// * `path` - 文件路径
    /// * `format` - 报告格式（"html" 或 "json"）
    #[allow(dead_code)]
    pub fn save_report(&self, path: &std::path::Path, format: &str) -> color_eyre::Result<()> {
        use color_eyre::eyre::Context;
        use std::fs;

        let content = match format {
            "html" => self.generate_html_report(),
            "json" => self.generate_json_report()?,
            _ => {
                return Err(color_eyre::eyre::eyre!("Unsupported format: {}", format));
            }
        };

        fs::write(path, content)
            .wrap_err_with(|| format!("Failed to write report to {}", path.display()))?;

        Ok(())
    }
}

impl Default for TestReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// HTML 转义辅助函数
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_reporter_new() {
        let reporter = TestReporter::new();
        assert_eq!(reporter.total_tests(), 0);
        assert_eq!(reporter.passed, 0);
        assert_eq!(reporter.failed, 0);
    }

    #[test]
    fn test_reporter_record_test() {
        let mut reporter = TestReporter::new();
        reporter.record_test(TestCaseResult {
            name: "test_example".to_string(),
            status: TestStatus::Passed,
            duration: Duration::from_millis(100),
            error_message: None,
            module: "tests::example".to_string(),
        });

        assert_eq!(reporter.total_tests(), 1);
        assert_eq!(reporter.passed, 1);
    }

    #[test]
    fn test_reporter_success_rate() {
        let mut reporter = TestReporter::new();
        reporter.record_test(TestCaseResult {
            name: "test_pass".to_string(),
            status: TestStatus::Passed,
            duration: Duration::from_millis(100),
            error_message: None,
            module: "tests".to_string(),
        });
        reporter.record_test(TestCaseResult {
            name: "test_fail".to_string(),
            status: TestStatus::Failed,
            duration: Duration::from_millis(50),
            error_message: Some("Error".to_string()),
            module: "tests".to_string(),
        });

        assert_eq!(reporter.success_rate(), 50.0);
    }

    #[test]
    fn test_reporter_generate_html() {
        let mut reporter = TestReporter::new();
        reporter.record_test(TestCaseResult {
            name: "test_example".to_string(),
            status: TestStatus::Passed,
            duration: Duration::from_millis(100),
            error_message: None,
            module: "tests::example".to_string(),
        });
        reporter.finish();

        let html = reporter.generate_html_report();
        assert!(html.contains("Test Execution Report"));
        assert!(html.contains("test_example"));
    }

    #[test]
    fn test_reporter_generate_json() -> color_eyre::Result<()> {
        let mut reporter = TestReporter::new();
        reporter.record_test(TestCaseResult {
            name: "test_example".to_string(),
            status: TestStatus::Passed,
            duration: Duration::from_millis(100),
            error_message: None,
            module: "tests::example".to_string(),
        });
        reporter.finish();

        let json = reporter.generate_json_report()?;
        assert!(json.contains("test_example"));
        assert!(json.contains("summary"));
        Ok(())
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }
}
