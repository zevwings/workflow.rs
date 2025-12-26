#!/usr/bin/env python3
"""分析覆盖率趋势

分析覆盖率的历史数据，检测覆盖率变化和回归。

使用方法:
    python3 scripts/dev/analyze-coverage-trends.py --current coverage.json --baseline baseline.json --output report.md
"""

import argparse
import json
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, Optional


def load_coverage_data(coverage_file: Path) -> Optional[Dict]:
    """加载覆盖率数据"""
    try:
        with open(coverage_file, 'r', encoding='utf-8') as f:
            return json.load(f)
    except Exception as e:
        print(f"⚠️  Failed to load coverage data: {e}", file=sys.stderr)
        return None


def compare_coverage(current: Dict, baseline: Dict) -> Dict:
    """对比当前覆盖率和基准覆盖率"""
    current_coverage = current.get("coverage_percent", 0.0)
    baseline_coverage = baseline.get("coverage_percent", 0.0)

    diff = current_coverage - baseline_coverage

    # 按模块对比（如果数据中有模块信息）
    module_changes = {}
    current_modules = current.get("modules", {})
    baseline_modules = baseline.get("modules", {})

    if current_modules and baseline_modules:
        all_modules = set(current_modules.keys()) | set(baseline_modules.keys())

        for module in all_modules:
            current_cov = current_modules.get(module, {}).get("coverage_percent", 0.0)
            baseline_cov = baseline_modules.get(module, {}).get("coverage_percent", 0.0)
            module_diff = current_cov - baseline_cov

            if abs(module_diff) > 0.1:  # 变化超过 0.1%
                module_changes[module] = {
                    "current": current_cov,
                    "baseline": baseline_cov,
                    "diff": module_diff,
                }

    return {
        "overall": {
            "current": current_coverage,
            "baseline": baseline_coverage,
            "diff": diff,
        },
        "module_changes": module_changes,
        "regression": diff < -1.0,  # 下降超过 1% 认为是回归
    }


def generate_coverage_report(comparison: Dict, output: Path):
    """生成覆盖率变化报告"""
    overall = comparison["overall"]
    regression = comparison["regression"]

    report_lines = [
        "# Coverage Change Report",
        "",
        f"**Generated**: {datetime.now().isoformat()}",
        "",
        "## Overall Coverage",
        "",
        f"**Current Coverage**: {overall['current']:.2f}%",
        f"**Baseline Coverage**: {overall['baseline']:.2f}%",
        f"**Change**: {overall['diff']:+.2f}%",
        "",
    ]

    if regression:
        report_lines.extend([
            "## ⚠️ Coverage Regression Detected",
            "",
            f"Coverage decreased by {abs(overall['diff']):.2f}%.",
            "Please add tests for new code.",
            "",
        ])
    elif overall["diff"] > 0:
        report_lines.extend([
            "## ✅ Coverage Improved",
            "",
            f"Coverage increased by {overall['diff']:.2f}%.",
            "",
        ])

    # 模块变化
    module_changes = comparison["module_changes"]
    if module_changes:
        report_lines.extend([
            "## Module Changes",
            "",
            "| Module | Current | Baseline | Change |",
            "|--------|---------|----------|--------|",
        ])

        for module, change in sorted(module_changes.items(), key=lambda x: x[1]["diff"]):
            diff_str = f"{change['diff']:+.2f}%"
            report_lines.append(
                f"| `{module}` | {change['current']:.2f}% | {change['baseline']:.2f}% | {diff_str} |"
            )
        report_lines.append("")

    report = "\n".join(report_lines)
    output.write_text(report, encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(
        description="分析覆盖率趋势",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--current", "-c", type=Path, required=True, help="当前覆盖率 JSON 文件")
    parser.add_argument("--baseline", "-b", type=Path, help="基准覆盖率 JSON 文件")
    parser.add_argument("--output", "-o", type=Path, required=True, help="输出报告文件")
    parser.add_argument("--threshold", "-t", type=float, default=1.0, help="回归阈值（%）")

    args = parser.parse_args()

    if not args.current.exists():
        print(f"❌ Error: Current coverage file not found: {args.current}", file=sys.stderr)
        sys.exit(1)

    # 加载覆盖率数据
    current = load_coverage_data(args.current)
    if not current:
        print("❌ Failed to load current coverage data", file=sys.stderr)
        sys.exit(1)

    baseline = None
    if args.baseline and args.baseline.exists():
        baseline = load_coverage_data(args.baseline)

    if not baseline:
        # 如果没有基准，使用当前作为基准（首次运行）
        baseline = current

    # 对比覆盖率
    comparison = compare_coverage(current, baseline)
    comparison["regression"] = comparison["overall"]["diff"] < -args.threshold

    # 生成报告
    generate_coverage_report(comparison, args.output)

    # 如果有回归，返回非零退出码
    if comparison["regression"]:
        print("⚠️  Coverage regression detected!", file=sys.stderr)
        sys.exit(1)

    print(f"✅ Coverage report generated: {args.output}", file=sys.stderr)


if __name__ == "__main__":
    main()

