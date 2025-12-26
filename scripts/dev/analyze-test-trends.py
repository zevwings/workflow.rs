#!/usr/bin/env python3
"""分析测试趋势

分析测试指标的历史数据，生成趋势报告。

使用方法:
    python3 scripts/dev/analyze-test-trends.py --metrics-dir metrics/ --output trends-report.md
"""

import argparse
import json
import sys
from collections import defaultdict
from datetime import datetime
from pathlib import Path
from typing import Dict, List


def load_historical_metrics(metrics_dir: Path) -> List[Dict]:
    """加载历史指标数据"""
    metrics = []
    for metrics_file in sorted(metrics_dir.glob("*.json")):
        try:
            with open(metrics_file, 'r', encoding='utf-8') as f:
                data = json.load(f)
                metrics.append(data)
        except Exception as e:
            print(f"⚠️  Failed to load {metrics_file}: {e}", file=sys.stderr)
    return metrics


def analyze_failure_rate_trend(metrics: List[Dict]) -> Dict:
    """分析失败率趋势"""
    trend = {
        "overall": [],
        "by_module": defaultdict(list),
    }

    for m in metrics:
        summary = m.get("summary", {})
        timestamp = m.get("timestamp", "")

        # 整体失败率
        total = summary.get("total", 0)
        failed = summary.get("failed", 0)
        failure_rate = (failed / total * 100.0) if total > 0 else 0.0

        trend["overall"].append({
            "timestamp": timestamp,
            "failure_rate": failure_rate,
            "total": total,
            "failed": failed,
        })

        # 按模块统计
        module_stats = m.get("module_stats", {})
        for module, stats in module_stats.items():
            trend["by_module"][module].append({
                "timestamp": timestamp,
                "failure_rate": stats.get("failure_rate", 0.0),
                "total": stats.get("total", 0),
                "failed": stats.get("failed", 0),
            })

    return trend


def analyze_stability(metrics: List[Dict]) -> Dict:
    """分析测试稳定性（检测不稳定测试）"""
    # 统计每个测试的失败次数
    test_failures = defaultdict(int)
    test_runs = defaultdict(int)

    for m in metrics:
        test_cases = m.get("test_cases", [])
        for test in test_cases:
            test_name = test.get("name", "")
            test_runs[test_name] += 1
            if test.get("status") == "Failed":
                test_failures[test_name] += 1

    # 计算失败率
    unstable_tests = []
    for test_name in test_runs:
        runs = test_runs[test_name]
        failures = test_failures[test_name]
        failure_rate = (failures / runs * 100.0) if runs > 0 else 0.0

        # 失败率在 1%-50% 之间的测试被认为是不稳定的
        if 1.0 <= failure_rate < 50.0:
            unstable_tests.append({
                "name": test_name,
                "runs": runs,
                "failures": failures,
                "failure_rate": failure_rate,
            })

    # 按失败率排序
    unstable_tests.sort(key=lambda x: x["failure_rate"], reverse=True)

    return {
        "unstable_tests": unstable_tests,
        "total_tests": len(test_runs),
        "unstable_count": len(unstable_tests),
    }


def analyze_duration_trend(metrics: List[Dict]) -> Dict:
    """分析执行时间趋势"""
    trend = {
        "overall": [],
        "by_module": defaultdict(list),
        "by_test": defaultdict(list),
    }

    for m in metrics:
        summary = m.get("summary", {})
        timestamp = m.get("timestamp", "")
        duration = summary.get("duration_secs", 0.0)

        trend["overall"].append({
            "timestamp": timestamp,
            "duration_secs": duration,
        })

        # 按模块统计
        module_stats = m.get("module_stats", {})
        for module, stats in module_stats.items():
            trend["by_module"][module].append({
                "timestamp": timestamp,
                "duration_secs": stats.get("total_duration", 0.0),
            })

        # 按测试统计
        test_cases = m.get("test_cases", [])
        for test in test_cases:
            test_name = test.get("name", "")
            trend["by_test"][test_name].append({
                "timestamp": timestamp,
                "duration_secs": test.get("duration_secs", 0.0),
            })

    return trend


def generate_trend_report(trends: Dict, output: Path):
    """生成趋势报告"""
    report_lines = [
        "# Test Trends Report",
        "",
        f"**Generated**: {datetime.now().isoformat()}",
        "",
        "## Failure Rate Trend",
        "",
    ]

    # 整体失败率趋势
    overall = trends["failure_rate"]["overall"]
    if overall:
        latest = overall[-1]
        report_lines.extend([
            f"**Latest Failure Rate**: {latest['failure_rate']:.2f}%",
            f"**Total Tests**: {latest['total']}",
            f"**Failed Tests**: {latest['failed']}",
            "",
        ])

    # 不稳定测试
    stability = trends["stability"]
    report_lines.extend([
        "## Test Stability Analysis",
        "",
        f"**Total Tests**: {stability['total_tests']}",
        f"**Unstable Tests**: {stability['unstable_count']}",
        "",
    ])

    if stability["unstable_tests"]:
        report_lines.extend([
            "### Top Unstable Tests",
            "",
            "| Test Name | Runs | Failures | Failure Rate |",
            "|-----------|------|----------|--------------|",
        ])

        for test in stability["unstable_tests"][:10]:
            report_lines.append(
                f"| `{test['name']}` | {test['runs']} | {test['failures']} | {test['failure_rate']:.2f}% |"
            )
        report_lines.append("")

    # 执行时间趋势
    duration = trends["duration"]
    if duration["overall"]:
        latest = duration["overall"][-1]
        report_lines.extend([
            "## Duration Trend",
            "",
            f"**Latest Duration**: {latest['duration_secs']:.2f}s",
            "",
        ])

    report = "\n".join(report_lines)
    output.write_text(report, encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(
        description="分析测试趋势",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--metrics-dir", "-d", type=Path, required=True, help="指标数据目录")
    parser.add_argument("--output", "-o", type=Path, required=True, help="输出报告文件")

    args = parser.parse_args()

    if not args.metrics_dir.exists():
        print(f"❌ Error: Metrics directory not found: {args.metrics_dir}", file=sys.stderr)
        sys.exit(1)

    # 加载历史数据
    metrics = load_historical_metrics(args.metrics_dir)

    if not metrics:
        print("⚠️  No historical metrics found", file=sys.stderr)
        sys.exit(0)

    # 分析趋势
    trends = {
        "failure_rate": analyze_failure_rate_trend(metrics),
        "stability": analyze_stability(metrics),
        "duration": analyze_duration_trend(metrics),
    }

    # 生成报告
    generate_trend_report(trends, args.output)

    print(f"✅ Trend report generated: {args.output}", file=sys.stderr)


if __name__ == "__main__":
    main()

