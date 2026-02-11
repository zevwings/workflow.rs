#!/usr/bin/env python3
"""测试覆盖率趋势分析脚本

用法:
    python3 scripts/dev/py/testing/coverage/trends.py coverage/history/

功能:
    - 分析历史覆盖率数据
    - 生成趋势图表
    - 识别覆盖率下降
"""
import json
import sys
from datetime import datetime
from pathlib import Path
from typing import List, Tuple


def load_history_files(history_dir: str) -> List[Tuple[datetime, float]]:
    """加载历史覆盖率数据"""
    history_path = Path(history_dir)

    if not history_path.exists():
        print(f"ℹ️  历史目录不存在: {history_dir}")
        return []

    data_points = []

    for json_file in sorted(history_path.glob("*.json")):
        filename = json_file.stem
        try:
            timestamp = datetime.strptime(filename, "%Y%m%d_%H%M%S")
        except ValueError:
            continue

        try:
            with open(json_file, 'r', encoding='utf-8') as f:
                report = json.load(f)
            coverage = float(report.get('coverage', report.get('coverage_percent', 0.0)))
            data_points.append((timestamp, coverage))
        except (json.JSONDecodeError, OSError):
            continue

    return data_points


def analyze_trend(data_points: List[Tuple[datetime, float]]) -> None:
    """分析覆盖率趋势"""
    if len(data_points) < 2:
        print("ℹ️  历史数据不足，无法分析趋势")
        return

    print("\n" + "=" * 60)
    print("📈 测试覆盖率趋势分析")
    print("=" * 60)

    oldest_time, oldest_coverage = data_points[0]
    latest_time, latest_coverage = data_points[-1]

    coverage_change = latest_coverage - oldest_coverage
    change_icon = "📈" if coverage_change > 0 else "📉" if coverage_change < 0 else "➡️"

    print(f"\n时间范围: {oldest_time.strftime('%Y-%m-%d')} 至 {latest_time.strftime('%Y-%m-%d')}")
    print(f"数据点数: {len(data_points)}")
    print(f"\n最早覆盖率: {oldest_coverage:.2f}%")
    print(f"最新覆盖率: {latest_coverage:.2f}%")
    print(f"{change_icon} 变化: {coverage_change:+.2f}%")

    if len(data_points) >= 5:
        recent_points = data_points[-5:]
        recent_start = recent_points[0][1]
        recent_end = recent_points[-1][1]
        recent_change = recent_end - recent_start
        print(f"\n最近趋势（最近 5 次）: {recent_change:+.2f}%")

    print(f"\n📊 历史记录:")
    print("-" * 60)
    for timestamp, coverage in data_points[-10:]:
        print(f"  {timestamp.strftime('%Y-%m-%d %H:%M:%S')}  {coverage:6.2f}%")

    print("=" * 60 + "\n")

    if coverage_change < -5:
        print("⚠️  警告: 覆盖率下降超过 5%，请检查最近的代码变更！")


def main() -> None:
    if len(sys.argv) != 2:
        print("用法: python3 scripts/dev/py/testing/coverage/trends.py <history_dir>")
        print("示例: python3 scripts/dev/py/testing/coverage/trends.py coverage/history/")
        sys.exit(1)

    history_dir = sys.argv[1]
    data_points = load_history_files(history_dir)
    analyze_trend(data_points)


if __name__ == '__main__':
    main()
