#!/usr/bin/env python3
"""分析性能回归

对比当前性能与基准性能，检测性能回归。

使用方法:
    python3 scripts/dev/analyze-performance-regression.py --current test-report.json --baseline baseline.json --output report.md
"""

import argparse
import json
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, Optional


def load_performance_data(metrics_file: Path) -> Optional[Dict]:
    """加载性能数据"""
    try:
        with open(metrics_file, 'r', encoding='utf-8') as f:
            return json.load(f)
    except Exception as e:
        print(f"⚠️  Failed to load performance data: {e}", file=sys.stderr)
        return None


def compare_performance(current: Dict, baseline: Dict, threshold: float = 0.2) -> Dict:
    """对比当前性能和基准性能"""
    current_duration = current.get("summary", {}).get("duration_secs", 0.0)
    baseline_duration = baseline.get("summary", {}).get("duration_secs", 0.0)

    if baseline_duration == 0:
        return {
            "regression": False,
            "overall": {
                "current": current_duration,
                "baseline": baseline_duration,
                "diff": 0.0,
                "diff_percent": 0.0,
            },
            "test_changes": {},
        }

    diff = current_duration - baseline_duration
    diff_percent = (diff / baseline_duration) * 100.0

    # 按测试对比
    test_changes = {}
    current_tests = {t["name"]: t for t in current.get("test_cases", [])}
    baseline_tests = {t["name"]: t for t in baseline.get("test_cases", [])}

    all_tests = set(current_tests.keys()) | set(baseline_tests.keys())

    for test_name in all_tests:
        current_test = current_tests.get(test_name, {})
        baseline_test = baseline_tests.get(test_name, {})

        current_dur = current_test.get("duration_secs", 0.0)
        baseline_dur = baseline_test.get("duration_secs", 0.0)

        if baseline_dur > 0:
            test_diff = current_dur - baseline_dur
            test_diff_percent = (test_diff / baseline_dur) * 100.0

            # 如果变化超过阈值，记录
            if abs(test_diff_percent) > threshold * 100:
                test_changes[test_name] = {
                    "current": current_dur,
                    "baseline": baseline_dur,
                    "diff": test_diff,
                    "diff_percent": test_diff_percent,
                }

    # 判断是否有回归（性能下降超过阈值）
    regression = diff_percent > threshold * 100

    return {
        "regression": regression,
        "overall": {
            "current": current_duration,
            "baseline": baseline_duration,
            "diff": diff,
            "diff_percent": diff_percent,
        },
        "test_changes": test_changes,
    }


def generate_performance_report(comparison: Dict, output: Path):
    """生成性能回归报告"""
    overall = comparison["overall"]
    regression = comparison["regression"]

    report_lines = [
        "# Performance Regression Report",
        "",
        f"**Generated**: {datetime.now().isoformat()}",
        "",
        "## Overall Performance",
        "",
        f"**Current Duration**: {overall['current']:.2f}s",
        f"**Baseline Duration**: {overall['baseline']:.2f}s",
        f"**Change**: {overall['diff']:+.2f}s ({overall['diff_percent']:+.2f}%)",
        "",
    ]

    if regression:
        report_lines.extend([
            "## ⚠️ Performance Regression Detected",
            "",
            f"Performance degraded by {overall['diff_percent']:.2f}%.",
            "Please review and optimize slow tests.",
            "",
        ])
    elif overall["diff_percent"] < 0:
        report_lines.extend([
            "## ✅ Performance Improved",
            "",
            f"Performance improved by {abs(overall['diff_percent']):.2f}%.",
            "",
        ])

    # 测试变化
    test_changes = comparison["test_changes"]
    if test_changes:
        report_lines.extend([
            "## Test Performance Changes",
            "",
            "| Test Name | Current | Baseline | Change |",
            "|-----------|---------|----------|--------|",
        ])

        for test_name, change in sorted(test_changes.items(), key=lambda x: x[1]["diff"], reverse=True):
            diff_str = f"{change['diff']:+.2f}s ({change['diff_percent']:+.2f}%)"
            report_lines.append(
                f"| `{test_name}` | {change['current']:.2f}s | {change['baseline']:.2f}s | {diff_str} |"
            )
        report_lines.append("")

    report = "\n".join(report_lines)
    output.write_text(report, encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(
        description="分析性能回归",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--current", "-c", type=Path, required=True, help="当前性能指标 JSON 文件")
    parser.add_argument("--baseline", "-b", type=Path, help="基准性能指标 JSON 文件")
    parser.add_argument("--output", "-o", type=Path, required=True, help="输出报告文件")
    parser.add_argument("--threshold", "-t", type=float, default=0.2, help="回归阈值（20%）")

    args = parser.parse_args()

    if not args.current.exists():
        print(f"❌ Error: Current performance file not found: {args.current}", file=sys.stderr)
        sys.exit(1)

    # 加载性能数据
    current = load_performance_data(args.current)
    if not current:
        print("❌ Failed to load current performance data", file=sys.stderr)
        sys.exit(1)

    baseline = None
    if args.baseline and args.baseline.exists():
        baseline = load_performance_data(args.baseline)

    if not baseline:
        # 如果没有基准，使用当前作为基准（首次运行）
        baseline = current

    # 对比性能
    comparison = compare_performance(current, baseline, args.threshold)

    # 生成报告
    generate_performance_report(comparison, args.output)

    # 如果有回归，返回非零退出码
    if comparison["regression"]:
        print("⚠️  Performance regression detected!", file=sys.stderr)
        sys.exit(1)

    print(f"✅ Performance report generated: {args.output}", file=sys.stderr)


if __name__ == "__main__":
    main()

