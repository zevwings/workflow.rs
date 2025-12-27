"""性能回归分析实现"""
import json
import sys
from pathlib import Path
from typing import Dict, Optional

# 添加父目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_info, log_success, log_warning


def analyze(args) -> None:
    """分析性能回归"""
    if not args.current:
        print("Error: --current 参数是必需的", file=sys.stderr)
        sys.exit(1)

    current_path = Path(args.current)
    if not current_path.exists():
        print(f"Error: Current performance file not found: {current_path}", file=sys.stderr)
        sys.exit(1)

    # 加载当前性能数据
    current_data = _load_performance_data(current_path)

    # 加载基准性能数据
    if args.baseline:
        baseline_path = Path(args.baseline)
        if baseline_path.exists():
            baseline_data = _load_performance_data(baseline_path)
        else:
            log_warning("Baseline file not found, using current as baseline")
            baseline_data = current_data.copy()
    else:
        log_warning("No baseline specified, using current as baseline")
        baseline_data = current_data.copy()

    # 对比性能
    comparison = _compare_performance(current_data, baseline_data, args.threshold or 0.2)

    # 确定输出文件路径
    output_path = Path(args.output) if args.output else Path("performance-regression-report.md")

    # 生成报告
    report = _generate_performance_report(comparison)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(report)

    # 如果有回归，返回错误
    if comparison['regression']:
        log_warning("Performance regression detected!")
        log_warning(f"   Report generated: {output_path}")
        sys.exit(1)

    log_success(f"Performance report generated: {output_path}")


def _load_performance_data(metrics_file: Path) -> Dict:
    """加载性能数据"""
    with open(metrics_file, 'r') as f:
        return json.load(f)


def _compare_performance(current: Dict, baseline: Dict, threshold: float) -> Dict:
    """对比当前性能和基准性能"""
    current_duration = current.get('summary', {}).get('duration_secs', 0.0)
    baseline_duration = baseline.get('summary', {}).get('duration_secs', 0.0)

    # 如果基准时长为 0，返回无变化结果
    if baseline_duration == 0.0:
        return {
            'regression': False,
            'overall': {
                'current': current_duration,
                'baseline': baseline_duration,
                'diff': 0.0,
                'diff_percent': 0.0
            },
            'test_changes': {}
        }

    diff = current_duration - baseline_duration
    diff_percent = (diff / baseline_duration) * 100.0

    # 按测试对比
    test_changes = {}
    current_tests = {t['name']: t for t in current.get('test_cases', [])}
    baseline_tests = {t['name']: t for t in baseline.get('test_cases', [])}

    all_tests = set(current_tests.keys()) | set(baseline_tests.keys())

    for test_name in all_tests:
        current_test = current_tests.get(test_name)
        baseline_test = baseline_tests.get(test_name)

        current_dur = current_test.get('duration_secs', 0.0) if current_test else 0.0
        baseline_dur = baseline_test.get('duration_secs', 0.0) if baseline_test else 0.0

        if baseline_dur > 0.0:
            test_diff = current_dur - baseline_dur
            test_diff_percent = (test_diff / baseline_dur) * 100.0

            # 如果变化超过阈值，记录
            if abs(test_diff_percent) > threshold * 100.0:
                test_changes[test_name] = {
                    'current': current_dur,
                    'baseline': baseline_dur,
                    'diff': test_diff,
                    'diff_percent': test_diff_percent
                }

    # 判断是否有回归（性能下降超过阈值）
    regression = diff_percent > threshold * 100.0

    return {
        'regression': regression,
        'overall': {
            'current': current_duration,
            'baseline': baseline_duration,
            'diff': diff,
            'diff_percent': diff_percent
        },
        'test_changes': test_changes
    }


def _generate_performance_report(comparison: Dict) -> str:
    """生成性能回归报告"""
    overall = comparison.get('overall', {})
    regression = comparison.get('regression', False)

    lines = [
        '# Performance Regression Report',
        '',
        '## Summary',
        ''
    ]

    if regression:
        lines.append('⚠️ **Performance regression detected!**')
    else:
        lines.append('✅ **No performance regression detected.**')

    lines.extend([
        '',
        '| Metric | Value |',
        '|--------|-------|',
        f'| Current Duration | {overall.get("current", 0.0):.3f}s |',
        f'| Baseline Duration | {overall.get("baseline", 0.0):.3f}s |',
        f'| Difference | {overall.get("diff", 0.0):.3f}s |',
        f'| Difference % | {overall.get("diff_percent", 0.0):.2f}% |',
        ''
    ])

    test_changes = comparison.get('test_changes', {})
    if test_changes:
        lines.extend(['## Test Changes', '', '| Test Name | Current | Baseline | Difference | Difference % |', '|-----------|---------|---------|------------|---------------|'])
        for test_name, change in sorted(test_changes.items(), key=lambda x: abs(x[1]['diff_percent']), reverse=True):
            lines.append(
                f'| `{test_name}` | {change["current"]:.3f}s | '
                f'{change["baseline"]:.3f}s | {change["diff"]:.3f}s | '
                f'{change["diff_percent"]:.2f}% |'
            )
        lines.append('')

    return '\n'.join(lines)


def main():
    """CLI 入口（可以直接运行此脚本）"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='analyze',
        description='Analyze performance regression'
    )
    parser.add_argument('--current', required=True, help='Current performance data file')
    parser.add_argument('--baseline', help='Baseline performance data file')
    parser.add_argument('--output', help='Output report file path')
    parser.add_argument('--threshold', type=float, help='Regression threshold (default: 0.2)')

    args = parser.parse_args()
    analyze(args)


if __name__ == '__main__':
    main()

