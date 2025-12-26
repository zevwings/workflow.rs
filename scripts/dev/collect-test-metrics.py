#!/usr/bin/env python3
"""收集测试指标数据

从测试报告 JSON 中提取指标数据，用于趋势分析。

使用方法:
    python3 scripts/dev/collect-test-metrics.py --report test-report.json --output metrics.json --test-type unit --platform Linux
"""

import argparse
import json
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict


def collect_metrics_from_report(report_path: Path) -> Dict:
    """从测试报告收集指标"""
    with open(report_path, 'r', encoding='utf-8') as f:
        report = json.load(f)

    summary = report.get("summary", {})
    test_cases = report.get("test_cases", [])

    # 按模块统计
    module_stats = {}
    for test in test_cases:
        module = test.get("module", "unknown")
        status = test.get("status", "")

        if module not in module_stats:
            module_stats[module] = {
                "total": 0,
                "passed": 0,
                "failed": 0,
                "ignored": 0,
                "timeout": 0,
                "total_duration": 0.0,
            }

        module_stats[module]["total"] += 1
        module_stats[module]["total_duration"] += test.get("duration_secs", 0.0)

        if status == "Passed":
            module_stats[module]["passed"] += 1
        elif status == "Failed":
            module_stats[module]["failed"] += 1
        elif status == "Ignored":
            module_stats[module]["ignored"] += 1
        elif status == "Timeout":
            module_stats[module]["timeout"] += 1

    # 计算失败率
    for module, stats in module_stats.items():
        if stats["total"] > 0:
            stats["failure_rate"] = (stats["failed"] / stats["total"]) * 100.0
            stats["success_rate"] = (stats["passed"] / stats["total"]) * 100.0
        else:
            stats["failure_rate"] = 0.0
            stats["success_rate"] = 0.0

    return {
        "timestamp": datetime.now().isoformat(),
        "summary": {
            "total": summary.get("total", 0),
            "passed": summary.get("passed", 0),
            "failed": summary.get("failed", 0),
            "ignored": summary.get("ignored", 0),
            "timeout": summary.get("timeout", 0),
            "success_rate": summary.get("success_rate", 0.0),
            "duration_secs": summary.get("duration_secs", 0.0),
        },
        "module_stats": module_stats,
        "test_cases": [
            {
                "name": tc.get("name"),
                "module": tc.get("module"),
                "status": tc.get("status"),
                "duration_secs": tc.get("duration_secs", 0.0),
            }
            for tc in test_cases
        ],
    }


def main():
    parser = argparse.ArgumentParser(
        description="收集测试指标数据",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--report", "-r", type=Path, required=True, help="测试报告 JSON 文件")
    parser.add_argument("--output", "-o", type=Path, required=True, help="输出指标文件")
    parser.add_argument("--test-type", type=str, help="测试类型 (unit/integration)")
    parser.add_argument("--platform", type=str, help="平台 (Linux/macOS/Windows)")

    args = parser.parse_args()

    if not args.report.exists():
        print(f"❌ Error: Report file not found: {args.report}", file=sys.stderr)
        sys.exit(1)

    metrics = collect_metrics_from_report(args.report)

    # 添加元数据
    metrics["metadata"] = {
        "test_type": args.test_type,
        "platform": args.platform,
    }

    # 保存指标
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with open(args.output, 'w', encoding='utf-8') as f:
        json.dump(metrics, f, indent=2, ensure_ascii=False)

    print(f"✅ Metrics collected: {args.output}", file=sys.stderr)


if __name__ == "__main__":
    main()

