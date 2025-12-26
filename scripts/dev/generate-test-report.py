#!/usr/bin/env python3
"""测试报告生成工具

解析 `cargo test` 的输出并生成 HTML/JSON 测试报告。

这是项目的统一实现，使用 Python 编写，无需编译，易于维护。

使用方法:
    cargo test --message-format=json 2>&1 | python3 scripts/dev/generate-test-report.py --format html --output report.html

示例:
    # 生成 HTML 报告
    cargo test --message-format=json 2>&1 | python3 scripts/dev/generate-test-report.py -f html -o report.html

    # 生成 JSON 报告
    cargo test --message-format=json 2>&1 | python3 scripts/dev/generate-test-report.py -f json -o report.json
"""

import argparse
import json
import sys
from datetime import datetime
from html import escape
from pathlib import Path
from typing import Dict, List, Optional


class TestStatus:
    """测试状态枚举"""
    PASSED = "Passed"
    FAILED = "Failed"
    IGNORED = "Ignored"
    TIMEOUT = "Timeout"


class TestCaseResult:
    """单个测试用例的结果"""
    def __init__(
        self,
        name: str,
        status: str,
        duration: float,
        error_message: Optional[str],
        module: str,
    ):
        self.name = name
        self.status = status
        self.duration = duration
        self.error_message = error_message
        self.module = module

    def to_dict(self) -> Dict:
        """转换为字典（用于 JSON 序列化）"""
        return {
            "name": self.name,
            "status": self.status,
            "duration_secs": self.duration,
            "error_message": self.error_message,
            "module": self.module,
        }


class TestReporter:
    """测试报告生成器"""
    def __init__(self):
        self.test_cases: List[TestCaseResult] = []
        self.start_time = datetime.now()
        self.end_time: Optional[datetime] = None
        self.passed = 0
        self.failed = 0
        self.ignored = 0
        self.timeout = 0

    def record_test(self, result: TestCaseResult):
        """记录测试用例结果"""
        if result.status == TestStatus.PASSED:
            self.passed += 1
        elif result.status == TestStatus.FAILED:
            self.failed += 1
        elif result.status == TestStatus.IGNORED:
            self.ignored += 1
        elif result.status == TestStatus.TIMEOUT:
            self.timeout += 1
        self.test_cases.append(result)

    def finish(self):
        """完成报告生成"""
        self.end_time = datetime.now()

    def total_tests(self) -> int:
        """获取总测试数"""
        return len(self.test_cases)

    def total_duration(self) -> float:
        """获取总执行时间（秒）"""
        end = self.end_time or datetime.now()
        delta = end - self.start_time
        return delta.total_seconds()

    def success_rate(self) -> float:
        """获取成功率（百分比）"""
        if self.total_tests() == 0:
            return 0.0
        return (self.passed / self.total_tests()) * 100.0

    def generate_html_report(self, sort_by: str = "name", highlight_slow: bool = True, slow_threshold: float = 1.0, group_by_module: bool = False) -> str:
        """生成 HTML 报告

        Args:
            sort_by: 排序方式 ("name", "duration", "module", "status")
            highlight_slow: 是否高亮慢测试
            slow_threshold: 慢测试阈值（秒）
            group_by_module: 是否按模块分组
        """
        duration = self.total_duration()
        success_rate = self.success_rate()

        if group_by_module:
            test_rows = self._generate_test_rows_html_grouped(sort_by, highlight_slow, slow_threshold)
        else:
            test_rows = self._generate_test_rows_html(sort_by, highlight_slow, slow_threshold)

        return f"""<!DOCTYPE html>
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
        .module-group {{ margin-top: 30px; }}
        .module-header {{ background-color: #e9ecef; padding: 10px; font-weight: bold; border-radius: 5px; margin-bottom: 10px; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Test Execution Report</h1>
        <div class="summary">
            <h2>Summary</h2>
            <div class="stats">
                <div class="stat">
                    <div class="stat-value passed">{self.passed}</div>
                    <div class="stat-label">Passed</div>
                </div>
                <div class="stat">
                    <div class="stat-value failed">{self.failed}</div>
                    <div class="stat-label">Failed</div>
                </div>
                <div class="stat">
                    <div class="stat-value ignored">{self.ignored}</div>
                    <div class="stat-label">Ignored</div>
                </div>
                <div class="stat">
                    <div class="stat-value timeout">{self.timeout}</div>
                    <div class="stat-label">Timeout</div>
                </div>
                <div class="stat">
                    <div class="stat-value total">{self.total_tests()}</div>
                    <div class="stat-label">Total</div>
                </div>
                <div class="stat">
                    <div class="stat-value rate">{success_rate:.2}%</div>
                    <div class="stat-label">Success Rate</div>
                </div>
                <div class="stat">
                    <div class="stat-value rate">{duration:.2}s</div>
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
                {test_rows}
            </tbody>
        </table>
    </div>
</body>
</html>"""

    def _generate_test_rows_html(self, sort_by: str = "name", highlight_slow: bool = True, slow_threshold: float = 1.0) -> str:
        """生成测试行 HTML

        Args:
            sort_by: 排序方式 ("name", "duration", "module", "status")
            highlight_slow: 是否高亮慢测试
            slow_threshold: 慢测试阈值（秒）
        """
        # 排序测试用例
        sorted_tests = self._sort_tests(sort_by)

        rows = []
        for test in sorted_tests:
            status_class_map = {
                TestStatus.PASSED: "status-passed",
                TestStatus.FAILED: "status-failed",
                TestStatus.IGNORED: "status-ignored",
                TestStatus.TIMEOUT: "status-timeout",
            }
            status_text_map = {
                TestStatus.PASSED: "✓ Passed",
                TestStatus.FAILED: "✗ Failed",
                TestStatus.IGNORED: "⊘ Ignored",
                TestStatus.TIMEOUT: "⏱ Timeout",
            }

            status_class = status_class_map.get(test.status, "status-passed")
            status_text = status_text_map.get(test.status, test.status)

            # 慢测试高亮
            duration_class = ""
            if highlight_slow and test.duration > slow_threshold:
                duration_class = ' class="slow-test"'
                duration_text = f"{test.duration:.3}s ⚠"
            else:
                duration_text = f"{test.duration:.3}s"

            error_html = ""
            if test.error_message:
                error_html = f'<div class="error-message">{escape(test.error_message)}</div>'

            rows.append(
                f"""<tr>
                    <td class="test-name">{escape(test.name)}</td>
                    <td>{escape(test.module)}</td>
                    <td><span class="status-badge {status_class}">{status_text}</span></td>
                    <td{duration_class}>{duration_text}</td>
                    <td>{error_html}</td>
                </tr>"""
            )
        return "\n".join(rows)

    def _sort_tests(self, sort_by: str) -> List[TestCaseResult]:
        """排序测试用例"""
        if sort_by == "duration":
            return sorted(self.test_cases, key=lambda x: x.duration, reverse=True)
        elif sort_by == "module":
            return sorted(self.test_cases, key=lambda x: (x.module, x.name))
        elif sort_by == "status":
            status_order = {TestStatus.FAILED: 0, TestStatus.TIMEOUT: 1, TestStatus.IGNORED: 2, TestStatus.PASSED: 3}
            return sorted(self.test_cases, key=lambda x: (status_order.get(x.status, 4), x.name))
        else:  # "name" or default
            return sorted(self.test_cases, key=lambda x: x.name)

    def _group_by_module(self) -> Dict[str, List[TestCaseResult]]:
        """按模块分组测试用例"""
        groups: Dict[str, List[TestCaseResult]] = {}
        for test in self.test_cases:
            if test.module not in groups:
                groups[test.module] = []
            groups[test.module].append(test)
        return groups

    def _generate_test_rows_html_grouped(self, sort_by: str = "name", highlight_slow: bool = True, slow_threshold: float = 1.0) -> str:
        """生成按模块分组的测试行 HTML"""
        groups = self._group_by_module()
        rows = []

        for module in sorted(groups.keys()):
            module_tests = groups[module]
            # 对每个模块内的测试进行排序
            if sort_by == "duration":
                module_tests = sorted(module_tests, key=lambda x: x.duration, reverse=True)
            elif sort_by == "status":
                status_order = {TestStatus.FAILED: 0, TestStatus.TIMEOUT: 1, TestStatus.IGNORED: 2, TestStatus.PASSED: 3}
                module_tests = sorted(module_tests, key=lambda x: (status_order.get(x.status, 4), x.name))
            else:
                module_tests = sorted(module_tests, key=lambda x: x.name)

            # 模块统计
            module_passed = sum(1 for t in module_tests if t.status == TestStatus.PASSED)
            module_failed = sum(1 for t in module_tests if t.status == TestStatus.FAILED)
            module_total = len(module_tests)

            rows.append(f'<tr class="module-group"><td colspan="5" class="module-header">')
            rows.append(f'{escape(module)} ({module_passed}/{module_total} passed, {module_failed} failed)')
            rows.append('</td></tr>')

            for test in module_tests:
                status_class_map = {
                    TestStatus.PASSED: "status-passed",
                    TestStatus.FAILED: "status-failed",
                    TestStatus.IGNORED: "status-ignored",
                    TestStatus.TIMEOUT: "status-timeout",
                }
                status_text_map = {
                    TestStatus.PASSED: "✓ Passed",
                    TestStatus.FAILED: "✗ Failed",
                    TestStatus.IGNORED: "⊘ Ignored",
                    TestStatus.TIMEOUT: "⏱ Timeout",
                }

                status_class = status_class_map.get(test.status, "status-passed")
                status_text = status_text_map.get(test.status, test.status)

                duration_class = ""
                if highlight_slow and test.duration > slow_threshold:
                    duration_class = ' class="slow-test"'
                    duration_text = f"{test.duration:.3}s ⚠"
                else:
                    duration_text = f"{test.duration:.3}s"

                error_html = ""
                if test.error_message:
                    error_html = f'<div class="error-message">{escape(test.error_message)}</div>'

                rows.append(
                    f"""<tr>
                        <td class="test-name">{escape(test.name)}</td>
                        <td>{escape(test.module)}</td>
                        <td><span class="status-badge {status_class}">{status_text}</span></td>
                        <td{duration_class}>{duration_text}</td>
                        <td>{error_html}</td>
                    </tr>"""
                )

        return "\n".join(rows)

    def generate_json_report(self) -> str:
        """生成 JSON 报告"""
        report = {
            "summary": {
                "total": self.total_tests(),
                "passed": self.passed,
                "failed": self.failed,
                "ignored": self.ignored,
                "timeout": self.timeout,
                "success_rate": self.success_rate(),
                "duration_secs": self.total_duration(),
            },
            "test_cases": [test.to_dict() for test in self.test_cases],
        }
        return json.dumps(report, indent=2, ensure_ascii=False)

    def generate_markdown_report(self) -> str:
        """生成 Markdown 报告"""
        duration = self.total_duration()
        success_rate = self.success_rate()

        lines = [
            "# Test Execution Report",
            "",
            f"**Generated**: {self.start_time.strftime('%Y-%m-%d %H:%M:%S')}",
            "",
            "## Summary",
            "",
            "| Metric | Value |",
            "|--------|-------|",
            f"| Total Tests | {self.total_tests()} |",
            f"| Passed | {self.passed} |",
            f"| Failed | {self.failed} |",
            f"| Ignored | {self.ignored} |",
            f"| Timeout | {self.timeout} |",
            f"| Success Rate | {success_rate:.2}% |",
            f"| Duration | {duration:.2}s |",
            "",
            "## Test Cases",
            "",
            "| Test Name | Module | Status | Duration |",
            "|-----------|--------|--------|----------|",
        ]

        # 按模块分组
        groups = self._group_by_module()
        for module in sorted(groups.keys()):
            lines.append(f"### {module}")
            lines.append("")
            lines.append("| Test Name | Status | Duration | Error |")
            lines.append("|-----------|--------|----------|-------|")

            for test in sorted(groups[module], key=lambda x: x.name):
                status_icon = {
                    TestStatus.PASSED: "✅",
                    TestStatus.FAILED: "❌",
                    TestStatus.IGNORED: "⏭️",
                    TestStatus.TIMEOUT: "⏱️",
                }.get(test.status, "❓")

                duration_text = f"{test.duration:.3}s"
                if test.duration > 1.0:
                    duration_text = f"**{duration_text}** ⚠️"

                error_text = test.error_message[:50] + "..." if test.error_message and len(test.error_message) > 50 else (test.error_message or "")
                error_text = error_text.replace("|", "\\|").replace("\n", " ")

                lines.append(f"| `{test.name}` | {status_icon} {test.status} | {duration_text} | {error_text} |")
            lines.append("")

        return "\n".join(lines)

    def generate_junit_xml_report(self) -> str:
        """生成 JUnit XML 报告"""
        from xml.etree.ElementTree import Element, SubElement, tostring
        from xml.dom import minidom

        root = Element("testsuites")
        root.set("name", "Cargo Test Suite")
        root.set("tests", str(self.total_tests()))
        root.set("failures", str(self.failed))
        root.set("errors", "0")
        root.set("time", f"{self.total_duration():.3f}")

        # 按模块分组
        groups = self._group_by_module()
        for module in sorted(groups.keys()):
            suite = SubElement(root, "testsuite")
            suite.set("name", module)
            suite.set("tests", str(len(groups[module])))
            suite.set("failures", str(sum(1 for t in groups[module] if t.status == TestStatus.FAILED)))
            suite.set("errors", "0")
            suite.set("time", f"{sum(t.duration for t in groups[module]):.3f}")

            for test in groups[module]:
                case = SubElement(suite, "testcase")
                case.set("name", test.name)
                case.set("classname", module)
                case.set("time", f"{test.duration:.3f}")

                if test.status == TestStatus.FAILED:
                    failure = SubElement(case, "failure")
                    failure.set("message", test.error_message or "Test failed")
                    if test.error_message:
                        failure.text = test.error_message
                elif test.status == TestStatus.IGNORED:
                    skipped = SubElement(case, "skipped")
                    skipped.set("message", "Test ignored")
                elif test.status == TestStatus.TIMEOUT:
                    failure = SubElement(case, "failure")
                    failure.set("message", "Test timeout")
                    failure.text = "Test exceeded timeout"

        # 格式化 XML
        rough_string = tostring(root, encoding="unicode")
        reparsed = minidom.parseString(rough_string)
        return reparsed.toprettyxml(indent="  ")

    def generate_csv_report(self) -> str:
        """生成 CSV 报告"""
        import csv
        from io import StringIO

        output = StringIO()
        writer = csv.writer(output)

        # 写入表头
        writer.writerow(["Test Name", "Module", "Status", "Duration (s)", "Error Message"])

        # 写入数据
        for test in sorted(self.test_cases, key=lambda x: x.name):
            writer.writerow([
                test.name,
                test.module,
                test.status,
                f"{test.duration:.3f}",
                test.error_message or "",
            ])

        return output.getvalue()

    def save_report(self, path: Path, format: str, **kwargs):
        """保存报告到文件

        Args:
            path: 输出文件路径
            format: 报告格式 ("html", "json", "markdown", "junit", "csv")
            **kwargs: 传递给生成器的额外参数（如 sort_by, highlight_slow 等）
        """
        if format == "html":
            content = self.generate_html_report(**kwargs)
        elif format == "json":
            content = self.generate_json_report()
        elif format == "markdown":
            content = self.generate_markdown_report()
        elif format == "junit":
            content = self.generate_junit_xml_report()
        elif format == "csv":
            content = self.generate_csv_report()
        else:
            raise ValueError(f"Unsupported format: {format}. Supported formats: html, json, markdown, junit, csv")

        path.write_text(content, encoding="utf-8")


def extract_module(test_name: str) -> str:
    """从测试名中提取模块名"""
    # 测试名格式通常是: module::path::test_name
    # 我们提取模块部分
    if "::" in test_name:
        return test_name.rsplit("::", 1)[0]
    return "unknown"


def parse_cargo_test_output() -> TestReporter:
    """解析 cargo test JSON 输出"""
    reporter = TestReporter()

    for line in sys.stdin:
        line = line.strip()
        if not line or not line.startswith("{"):
            continue

        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue

        msg_type = msg.get("type")
        if msg_type == "test":
            name = msg.get("name", "")
            event = msg.get("event", "")
            exec_time = msg.get("exec_time")
            stdout = msg.get("stdout")

            module = extract_module(name)
            duration = exec_time if exec_time is not None else 0.0

            if event == "ok":
                reporter.record_test(
                    TestCaseResult(
                        name=name,
                        status=TestStatus.PASSED,
                        duration=duration,
                        error_message=None,
                        module=module,
                    )
                )
            elif event == "failed":
                reporter.record_test(
                    TestCaseResult(
                        name=name,
                        status=TestStatus.FAILED,
                        duration=duration,
                        error_message=stdout,
                        module=module,
                    )
                )
            elif event == "ignored":
                reporter.record_test(
                    TestCaseResult(
                        name=name,
                        status=TestStatus.IGNORED,
                        duration=duration,
                        error_message=None,
                        module=module,
                    )
                )
            elif event == "timeout":
                reporter.record_test(
                    TestCaseResult(
                        name=name,
                        status=TestStatus.TIMEOUT,
                        duration=duration,
                        error_message="Test timeout",
                        module=module,
                    )
                )
        elif msg_type == "suite":
            # Suite 消息包含测试套件的汇总信息
            # 可以用于验证报告完整性（未来可能使用）
            pass

    reporter.finish()
    return reporter


def main():
    """主函数"""
    parser = argparse.ArgumentParser(
        description="生成测试执行报告（HTML 或 JSON 格式）",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
使用方法:
  cargo test --message-format=json 2>&1 | python3 scripts/dev/generate-test-report.py --format html --output test-report.html

示例:
  # 生成 HTML 报告
  cargo test --message-format=json 2>&1 | python3 scripts/dev/generate-test-report.py -f html -o report.html

  # 生成 JSON 报告
  cargo test --message-format=json 2>&1 | python3 scripts/dev/generate-test-report.py -f json -o report.json

  # 生成 Markdown 报告
  cargo test --message-format=json 2>&1 | python3 scripts/dev/generate-test-report.py -f markdown -o report.md

  # 生成 JUnit XML 报告（用于 CI 集成）
  cargo test --message-format=json 2>&1 | python3 scripts/dev/generate-test-report.py -f junit -o report.xml

  # 生成 CSV 报告
  cargo test --message-format=json 2>&1 | python3 scripts/dev/generate-test-report.py -f csv -o report.csv

  # 按模块分组，按执行时间排序，高亮慢测试
  cargo test --message-format=json 2>&1 | python3 scripts/dev/generate-test-report.py -f html --group-by-module --sort-by duration --slow-threshold 0.5
        """,
    )
    parser.add_argument(
        "--format",
        "-f",
        choices=["html", "json", "markdown", "junit", "csv"],
        default="html",
        help="报告格式: html, json, markdown, junit, csv (默认: html)",
    )
    parser.add_argument(
        "--output",
        "-o",
        type=Path,
        default=Path("test-report.html"),
        help="输出文件路径 (默认: test-report.html)",
    )
    parser.add_argument(
        "--sort-by",
        choices=["name", "duration", "module", "status"],
        default="name",
        help="排序方式: name, duration, module, status (默认: name)",
    )
    parser.add_argument(
        "--highlight-slow",
        action="store_true",
        default=True,
        help="高亮慢测试（默认: True）",
    )
    parser.add_argument(
        "--slow-threshold",
        type=float,
        default=1.0,
        help="慢测试阈值（秒）(默认: 1.0)",
    )
    parser.add_argument(
        "--group-by-module",
        action="store_true",
        help="按模块分组显示",
    )

    args = parser.parse_args()

    # 解析测试输出
    reporter = parse_cargo_test_output()

    # 生成并保存报告
    report_kwargs = {
        "sort_by": args.sort_by,
        "highlight_slow": args.highlight_slow,
        "slow_threshold": args.slow_threshold,
    }
    if args.format == "html":
        report_kwargs["group_by_module"] = args.group_by_module

    reporter.save_report(args.output, args.format, **report_kwargs)

    # 输出摘要
    print(f"✅ Test report generated: {args.output}", file=sys.stderr)
    print(f"   Format: {args.format}", file=sys.stderr)
    print(f"   Total tests: {reporter.total_tests()}", file=sys.stderr)
    print(f"   Passed: {reporter.passed}", file=sys.stderr)
    print(f"   Failed: {reporter.failed}", file=sys.stderr)
    print(f"   Success rate: {reporter.success_rate():.2}%", file=sys.stderr)


if __name__ == "__main__":
    main()

