#!/usr/bin/env python3
"""生成 PR 评论内容

从测试报告 JSON 生成 PR 评论的 Markdown 内容。

使用方法:
    python3 scripts/dev/generate-pr-comment.py --report test-report.json
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Dict, List, Optional


class TestStatus:
    """测试状态枚举"""
    PASSED = "Passed"
    FAILED = "Failed"
    IGNORED = "Ignored"
    TIMEOUT = "Timeout"


def load_test_report(report_path: Path) -> Dict:
    """加载测试报告 JSON"""
    try:
        with open(report_path, 'r', encoding='utf-8') as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"❌ Error: Report file not found: {report_path}", file=sys.stderr)
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"❌ Error: Invalid JSON in report file: {e}", file=sys.stderr)
        sys.exit(1)


def merge_reports(reports: List[Dict]) -> Dict:
    """合并多个测试报告"""
    if not reports:
        return {
            "summary": {
                "total": 0,
                "passed": 0,
                "failed": 0,
                "ignored": 0,
                "timeout": 0,
                "success_rate": 0.0,
                "duration_secs": 0.0,
            },
            "test_cases": [],
        }

    if len(reports) == 1:
        return reports[0]

    # 合并多个报告
    merged_summary = {
        "total": 0,
        "passed": 0,
        "failed": 0,
        "ignored": 0,
        "timeout": 0,
        "success_rate": 0.0,
        "duration_secs": 0.0,
    }

    merged_test_cases = []
    test_case_map = {}  # 用于去重：test_name -> test_case

    for report in reports:
        summary = report.get("summary", {})
        test_cases = report.get("test_cases", [])

        # 累加统计
        merged_summary["total"] += summary.get("total", 0)
        merged_summary["passed"] += summary.get("passed", 0)
        merged_summary["failed"] += summary.get("failed", 0)
        merged_summary["ignored"] += summary.get("ignored", 0)
        merged_summary["timeout"] += summary.get("timeout", 0)
        merged_summary["duration_secs"] += summary.get("duration_secs", 0.0)

        # 合并测试用例（去重，保留失败或超时的测试）
        for test_case in test_cases:
            test_name = test_case.get("name", "")
            if test_name not in test_case_map:
                test_case_map[test_name] = test_case
            else:
                # 如果当前测试失败或超时，优先保留
                current_status = test_case.get("status", "")
                existing_status = test_case_map[test_name].get("status", "")
                if current_status in [TestStatus.FAILED, TestStatus.TIMEOUT]:
                    test_case_map[test_name] = test_case

    merged_test_cases = list(test_case_map.values())
    merged_summary["total"] = len(merged_test_cases)

    # 计算成功率
    if merged_summary["total"] > 0:
        merged_summary["success_rate"] = (
            merged_summary["passed"] / merged_summary["total"] * 100.0
        )

    return {
        "summary": merged_summary,
        "test_cases": merged_test_cases,
    }


def generate_pr_comment(report: Dict, artifact_url: Optional[str] = None) -> str:
    """生成 PR 评论的 Markdown 内容"""
    summary = report.get("summary", {})
    test_cases = report.get("test_cases", [])

    total = summary.get("total", 0)
    passed = summary.get("passed", 0)
    failed = summary.get("failed", 0)
    ignored = summary.get("ignored", 0)
    timeout = summary.get("timeout", 0)
    success_rate = summary.get("success_rate", 0.0)
    duration_secs = summary.get("duration_secs", 0.0)

    # 确定总体状态
    if failed > 0 or timeout > 0:
        status_emoji = "❌"
        status_text = "Tests Failed"
    elif ignored > 0:
        status_emoji = "⚠️"
        status_text = "Tests Passed (with ignored)"
    else:
        status_emoji = "✅"
        status_text = "Tests Passed"

    # 构建评论内容
    lines = [
        f"## {status_emoji} Test Results Summary",
        "",
        f"**{status_text}**",
        "",
        "### 📊 Test Statistics",
        "",
        "| Metric | Value |",
        "|--------|-------|",
        f"| Total Tests | {total} |",
        f"| ✅ Passed | {passed} |",
        f"| ❌ Failed | {failed} |",
        f"| ⏭️ Ignored | {ignored} |",
        f"| ⏱️ Timeout | {timeout} |",
        f"| Success Rate | {success_rate:.2f}% |",
        f"| Duration | {duration_secs:.2f}s |",
        "",
    ]

    # 失败测试详情
    failed_tests = [tc for tc in test_cases if tc.get("status") == TestStatus.FAILED]
    timeout_tests = [tc for tc in test_cases if tc.get("status") == TestStatus.TIMEOUT]

    if failed_tests or timeout_tests:
        lines.extend([
            "### 🔴 Failed/Timeout Tests",
            "",
        ])

        # 显示前 10 个失败测试
        all_failed = failed_tests + timeout_tests
        display_count = min(10, len(all_failed))

        for i, test in enumerate(all_failed[:display_count], 1):
            test_name = test.get("name", "unknown")
            module = test.get("module", "unknown")
            duration = test.get("duration_secs", 0.0)
            error_msg = test.get("error_message", "")
            status = test.get("status", "")

            status_icon = "⏱️" if status == TestStatus.TIMEOUT else "❌"

            lines.append(f"{i}. `{test_name}` ({module})")
            lines.append(f"   - Status: {status_icon} {status}")
            lines.append(f"   - Duration: {duration:.3f}s")
            if error_msg:
                # 截断错误消息（最多 200 字符）
                error_preview = error_msg[:200] + "..." if len(error_msg) > 200 else error_msg
                error_preview = error_preview.replace("\n", " ").replace("|", "\\|")
                lines.append(f"   - Error: `{error_preview}`")
            lines.append("")

        if len(all_failed) > display_count:
            lines.append(f"*... and {len(all_failed) - display_count} more failed tests*")
            lines.append("")

    # 慢测试警告
    slow_tests = [
        tc for tc in test_cases
        if tc.get("status") == TestStatus.PASSED and tc.get("duration_secs", 0.0) > 1.0
    ]
    if slow_tests:
        slow_tests_sorted = sorted(slow_tests, key=lambda x: x.get("duration_secs", 0.0), reverse=True)
        top_slow = slow_tests_sorted[:5]

        lines.extend([
            "### ⚠️ Slow Tests (>1s)",
            "",
            "| Test Name | Duration |",
            "|-----------|----------|",
        ])

        for test in top_slow:
            test_name = test.get("name", "unknown")
            duration = test.get("duration_secs", 0.0)
            lines.append(f"| `{test_name}` | {duration:.3f}s |")

        if len(slow_tests) > 5:
            lines.append(f"| *... and {len(slow_tests) - 5} more slow tests* | |")

        lines.append("")

    # 按模块分组统计
    module_stats: Dict[str, Dict[str, int]] = {}
    for test in test_cases:
        module = test.get("module", "unknown")
        status = test.get("status", "")

        if module not in module_stats:
            module_stats[module] = {"total": 0, "passed": 0, "failed": 0, "ignored": 0, "timeout": 0}

        module_stats[module]["total"] += 1
        if status == TestStatus.PASSED:
            module_stats[module]["passed"] += 1
        elif status == TestStatus.FAILED:
            module_stats[module]["failed"] += 1
        elif status == TestStatus.IGNORED:
            module_stats[module]["ignored"] += 1
        elif status == TestStatus.TIMEOUT:
            module_stats[module]["timeout"] += 1

    if len(module_stats) > 1:
        lines.extend([
            "### 📋 Test Breakdown by Module",
            "",
            "| Module | Total | Passed | Failed | Ignored |",
            "|--------|-------|--------|--------|---------|",
        ])

        for module in sorted(module_stats.keys()):
            stats = module_stats[module]
            lines.append(
                f"| `{module}` | {stats['total']} | {stats['passed']} | "
                f"{stats['failed']} | {stats['ignored']} |"
            )

        lines.append("")

    # 详细报告链接
    lines.extend([
        "### 📄 Detailed Reports",
        "",
    ])

    if artifact_url:
        lines.append(f"- [View HTML Report]({artifact_url})")
    else:
        lines.append("- View detailed reports in the workflow artifacts")

    lines.extend([
        "",
        "---",
        "",
        f"*Generated by [CI workflow](https://github.com/${{{{ github.repository }}}}/actions/runs/${{{{ github.run_id }}}})*",
    ])

    return "\n".join(lines)


def main():
    """主函数"""
    parser = argparse.ArgumentParser(
        description="生成 PR 评论的 Markdown 内容",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--report",
        "-r",
        type=Path,
        nargs="+",
        help="测试报告 JSON 文件路径（可以指定多个文件进行合并）",
    )
    parser.add_argument(
        "--artifact-url",
        "-a",
        type=str,
        help="Artifact 下载 URL（可选）",
    )
    parser.add_argument(
        "--output",
        "-o",
        type=Path,
        help="输出文件路径（默认：输出到 stdout）",
    )

    args = parser.parse_args()

    if not args.report:
        print("❌ Error: At least one report file is required", file=sys.stderr)
        parser.print_help()
        sys.exit(1)

    # 加载所有报告
    reports = []
    for report_path in args.report:
        try:
            report = load_test_report(report_path)
            reports.append(report)
        except SystemExit:
            # 文件不存在或格式错误，跳过
            continue

    if not reports:
        print("❌ Error: No valid reports found", file=sys.stderr)
        sys.exit(1)

    # 合并报告（如果有多个）
    merged_report = merge_reports(reports)

    # 生成评论内容
    comment = generate_pr_comment(merged_report, args.artifact_url)

    # 输出
    if args.output:
        args.output.write_text(comment, encoding="utf-8")
        print(f"✅ PR comment generated: {args.output}", file=sys.stderr)
    else:
        print(comment)


if __name__ == "__main__":
    main()

