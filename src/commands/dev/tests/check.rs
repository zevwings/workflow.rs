//! 测试检查命令
//!
//! 提供测试文档检查、覆盖率检查等功能。

use color_eyre::{eyre::WrapErr, Result};
use duct::cmd;
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;
use crate::{log_info, log_success, log_error, log_warning, log_break};
use crate::base::util::file::FileWriter;

/// 测试覆盖率检查命令
pub struct TestsCoverageCheckCommand {
    threshold: f64,
    ci: bool,
    output: Option<String>,
    check_type: Option<String>,
}

impl TestsCoverageCheckCommand {
    /// 创建新的测试覆盖率检查命令
    pub fn new(threshold: Option<f64>, ci: bool, output: Option<String>, check_type: Option<String>) -> Self {
        Self {
            threshold: threshold.unwrap_or(80.0),
            ci,
            output,
            check_type,
        }
    }

    /// 检查测试覆盖率
    pub fn check(&self) -> Result<()> {
        log_break!('=');
        log_info!("测试覆盖率检查");
        log_break!('=');
        log_break!();

        // 检查 cargo-tarpaulin 是否安装
        if !self.check_tarpaulin_installed()? {
            log_error!("cargo-tarpaulin 未安装");
            log_error!("   安装方法: cargo install cargo-tarpaulin");
            if self.ci {
                self.output_ci_result(false, 0.0)?;
                return Ok(());
            }
            std::process::exit(1);
        }

        log_success!("cargo-tarpaulin 已安装");
        let version = self.get_tarpaulin_version()?;
        log_info!("   {}", version);
        log_break!();

        // 生成覆盖率报告
        log_info!("生成覆盖率报告...");
        let coverage_dir = "coverage";
        std::fs::create_dir_all(coverage_dir)?;

        // 生成 JSON 格式的覆盖率报告
        let coverage_output = cmd("cargo", ["tarpaulin", "--out", "Json", "--output-dir", coverage_dir])
            .stderr_capture()
            .unchecked()
            .run()
            .wrap_err("Failed to run cargo tarpaulin")?;

        if !coverage_output.status.success() {
            log_error!("生成覆盖率报告失败");
            log_error!("   错误: {}", String::from_utf8_lossy(&coverage_output.stderr));
            if self.ci {
                self.output_ci_result(false, 0.0)?;
                return Ok(());
            }
            std::process::exit(1);
        }

        // 解析覆盖率数据
        log_break!();
        log_info!("检查覆盖率阈值...");

        let coverage_value = self.parse_coverage_from_json(&coverage_output.stdout)?;

        if coverage_value == 0.0 {
            log_warning!("无法解析覆盖率数据，请检查报告");
            if self.ci {
                self.output_ci_result(false, 0.0)?;
                return Ok(());
            }
            std::process::exit(1);
        }

        log_info!("当前覆盖率: {:.2}%", coverage_value);
        log_info!("目标覆盖率: {:.2}%", self.threshold);

        let passed = coverage_value >= self.threshold;

        if passed {
            log_success!("覆盖率已达到目标阈值 ({:.2}%)", self.threshold);
        } else {
            let gap = self.threshold - coverage_value;
            log_warning!("覆盖率未达到目标阈值 ({:.2}%)", self.threshold);
            log_warning!("   当前覆盖率: {:.2}%", coverage_value);
            log_warning!("   目标覆盖率: {:.2}%", self.threshold);
            log_warning!("   差距: {:.2}%", gap);
        }

        log_break!();
        log_success!("覆盖率检查完成");
        log_info!("   报告位置: {}/tarpaulin-report.html", coverage_dir);
        log_break!();

        // 如果需要生成报告
        if self.output.is_some() {
            self.generate_report(passed, coverage_value, coverage_dir)?;
        }

        // CI 模式：输出到 GITHUB_OUTPUT
        if self.ci {
            self.output_ci_result(passed, coverage_value)?;
            return Ok(());
        }

        // 本地模式：如果未通过则退出
        if !passed {
            std::process::exit(1);
        }

        Ok(())
    }

    /// 检查 cargo-tarpaulin 是否安装
    fn check_tarpaulin_installed(&self) -> Result<bool> {
        let result = cmd("cargo", ["tarpaulin", "--version"])
            .stdout_null()
            .stderr_null()
            .unchecked()
            .run();

        Ok(result.is_ok() && result?.status.success())
    }

    /// 获取 cargo-tarpaulin 版本
    fn get_tarpaulin_version(&self) -> Result<String> {
        let output = cmd("cargo", ["tarpaulin", "--version"])
            .read()
            .wrap_err("Failed to get cargo-tarpaulin version")?;

        Ok(output.trim().to_string())
    }

    /// 从 JSON 输出中解析覆盖率百分比
    fn parse_coverage_from_json(&self, json_output: &[u8]) -> Result<f64> {
        // cargo tarpaulin 的 JSON 输出可能包含多行，最后一行是 JSON
        let output_str = String::from_utf8_lossy(json_output);
        let lines: Vec<&str> = output_str.lines().collect();

        // 尝试解析每一行，找到有效的 JSON
        for line in lines.iter().rev() {
            if let Ok(json) = serde_json::from_str::<Value>(line.trim()) {
                if let Some(coverage) = json.get("coverage_percent") {
                    if let Some(coverage_f64) = coverage.as_f64() {
                        return Ok(coverage_f64);
                    }
                }
            }
        }

        // 如果单行解析失败，尝试解析整个输出
        if let Ok(json) = serde_json::from_slice::<Value>(json_output) {
            if let Some(coverage) = json.get("coverage_percent") {
                if let Some(coverage_f64) = coverage.as_f64() {
                    return Ok(coverage_f64);
                }
            }
        }

        Ok(0.0)
    }

    /// 输出 CI 模式结果到 GITHUB_OUTPUT
    fn output_ci_result(&self, passed: bool, coverage: f64) -> Result<()> {
        if let Ok(output_file) = std::env::var("GITHUB_OUTPUT") {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&output_file)
                .wrap_err_with(|| format!("Failed to open GITHUB_OUTPUT: {}", output_file))?;

            writeln!(file, "coverage_status={}", if passed { "pass" } else { "fail" })
                .wrap_err("Failed to write coverage_status")?;
            writeln!(file, "coverage_passed={}", passed)
                .wrap_err("Failed to write coverage_passed")?;
            writeln!(file, "coverage_value={:.2}", coverage)
                .wrap_err("Failed to write coverage_value")?;
            writeln!(file, "target_coverage={:.2}", self.threshold)
                .wrap_err("Failed to write target_coverage")?;
        }

        // CI 模式：非阻塞退出
        Ok(())
    }

    /// 生成覆盖率检查报告
    fn generate_report(&self, passed: bool, coverage_value: f64, coverage_dir: &str) -> Result<()> {
        log_info!("生成覆盖率检查报告...");

        let check_date = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let update_date = Local::now().format("%Y-%m-%d").to_string();
        let check_type = self.check_type.as_deref().unwrap_or("定期审查");

        let (status_emoji, status_text, details) = if passed {
            (
                "✅",
                "通过",
                format!("✅ 测试覆盖率已达到目标阈值（{:.2}%）。", self.threshold),
            )
        } else {
            let gap = self.threshold - coverage_value;
            (
                "⚠️",
                "未达标",
                format!(
                    "⚠️  测试覆盖率未达到目标阈值。\n\n**当前覆盖率**：{:.2}%\n**目标覆盖率**：{:.2}%\n**差距**：{:.2}%\n\n建议：\n1. 检查未覆盖的代码模块\n2. 为关键业务逻辑添加更多测试用例\n3. 参考 `docs/requirements/QUICK_COVERAGE_MODULES.md` 了解快速提升覆盖率的模块",
                    coverage_value, self.threshold, gap
                ),
            )
        };

        let report_content = format!(
            r#"# 测试覆盖率检查报告

**检查日期**：{}
**检查类型**：{}

## 检查结果

### 覆盖率统计

- **当前覆盖率**：{:.2}%
- **目标覆盖率**：{:.2}%
- **检查状态**：{} {}

### 详细说明

{}

## 报告文件

- HTML 报告已上传为 Artifact：`coverage-report`
- 查看详细覆盖率数据：下载 Artifact 中的 `{}/tarpaulin-report.html`

## 改进建议

1. 确保关键业务逻辑的测试覆盖率 > 90%
2. 定期审查未覆盖的代码模块
3. 为新功能添加相应的测试用例

参考文档：
- [测试覆盖率改进分析](docs/requirements/coverage-improvement.md)
- [快速覆盖率提升模块](docs/requirements/QUICK_COVERAGE_MODULES.md)
- [测试规范](docs/guidelines/testing.md)

---

**最后更新**: {}
"#,
            check_date,
            check_type,
            coverage_value,
            self.threshold,
            status_emoji,
            status_text,
            details,
            coverage_dir,
            update_date
        );

        if let Some(ref output_path) = self.output {
            let writer = FileWriter::new(output_path);
            writer.write_str_with_dir(&report_content)
                .wrap_err_with(|| format!("Failed to write report to: {}", output_path))?;
            log_success!("报告已保存到: {}", output_path);
        } else {
            print!("{}", report_content);
        }

        Ok(())
    }
}
