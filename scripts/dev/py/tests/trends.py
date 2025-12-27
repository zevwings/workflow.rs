"""测试趋势分析实现"""
import json
import sys
from collections import defaultdict
from datetime import datetime
from pathlib import Path
from typing import Dict, List

# 添加父目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_info, log_success, log_warning, log_break


def analyze(args) -> None:
    """分析测试趋势"""
    if not args.metrics_dir:
        print("Error: --metrics-dir 参数是必需的", file=sys.stderr)
        sys.exit(1)
    if not args.output:
        print("Error: --output 参数是必需的", file=sys.stderr)
        sys.exit(1)

    log_break('=')
    log_info("分析测试趋势")
    log_break('=')
    log_break()

    # 加载历史指标数据
    log_info(f"加载历史指标数据: {args.metrics_dir}")
    metrics = _load_historical_metrics(args.metrics_dir)

    if not metrics:
        log_warning("没有找到历史指标数据")
        return

    log_info(f"已加载 {len(metrics)} 个指标文件")
    log_break()

    # 分析趋势
    log_info("分析趋势...")
    trends = _analyze_trends(metrics)

    # 生成报告
    log_info(f"生成趋势报告: {args.output}")
    report = _generate_trend_report(trends)

    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(report)

    log_break()
    log_success("趋势分析完成")
    log_info(f"   总指标数: {len(metrics)}")
    log_info(f"   不稳定测试: {trends['stability']['total_unstable']}")
    log_break()


def _load_historical_metrics(metrics_dir: str) -> List[Dict]:
    """加载历史指标数据"""
    dir_path = Path(metrics_dir)
    if not dir_path.exists():
        print(f"Error: 指标目录不存在: {metrics_dir}", file=sys.stderr)
        sys.exit(1)

    metrics = []
    for json_file in dir_path.rglob("*.json"):
        try:
            with open(json_file, 'r') as f:
                data = json.load(f)
                metrics.append(data)
        except Exception as e:
            log_warning(f"加载指标文件失败 {json_file}: {e}")

    # 按时间戳排序
    metrics.sort(key=lambda x: x.get('timestamp', ''))
    return metrics


def _analyze_trends(metrics: List[Dict]) -> Dict:
    """分析趋势"""
    failure_rate = _analyze_failure_rate_trend(metrics)
    stability = _analyze_stability(metrics)
    duration = _analyze_duration_trend(metrics)

    return {
        'failure_rate': failure_rate,
        'stability': stability,
        'duration': duration
    }


def _analyze_failure_rate_trend(metrics: List[Dict]) -> Dict:
    """分析失败率趋势"""
    overall = []
    by_module: Dict[str, List] = defaultdict(list)

    for m in metrics:
        summary = m.get('summary', {})
        total = summary.get('total', 0)
        failed = summary.get('failed', 0)
        failure_rate = (failed / total * 100.0) if total > 0 else 0.0

        overall.append({
            'timestamp': m.get('timestamp', ''),
            'failure_rate': failure_rate,
            'total': total,
            'failed': failed
        })

        # 按模块统计
        module_stats = m.get('module_stats', {})
        for module, stats in module_stats.items():
            by_module[module].append({
                'timestamp': m.get('timestamp', ''),
                'failure_rate': stats.get('failure_rate', 0.0),
                'total': stats.get('total', 0),
                'failed': stats.get('failed', 0)
            })

    return {
        'overall': overall,
        'by_module': dict(by_module)
    }


def _analyze_stability(metrics: List[Dict]) -> Dict:
    """分析测试稳定性"""
    test_failures: Dict[str, int] = defaultdict(int)
    test_runs: Dict[str, int] = defaultdict(int)

    for m in metrics:
        for test_case in m.get('test_cases', []):
            test_name = test_case.get('name', '')
            test_runs[test_name] += 1
            if test_case.get('status') == 'Failed':
                test_failures[test_name] += 1

    unstable_tests = []
    for test_name, runs in test_runs.items():
        failures = test_failures.get(test_name, 0)
        failure_rate = (failures / runs * 100.0) if runs > 0 else 0.0

        # 失败率在 1%-50% 之间的测试被认为是不稳定的
        if 1.0 <= failure_rate < 50.0:
            unstable_tests.append({
                'name': test_name,
                'runs': runs,
                'failures': failures,
                'failure_rate': failure_rate
            })

    # 按失败率排序
    unstable_tests.sort(key=lambda x: x['failure_rate'], reverse=True)

    return {
        'total_unstable': len(unstable_tests),
        'unstable_tests': unstable_tests
    }


def _analyze_duration_trend(metrics: List[Dict]) -> Dict:
    """分析执行时间趋势"""
    overall = []
    by_module: Dict[str, List] = defaultdict(list)

    for m in metrics:
        summary = m.get('summary', {})
        overall.append({
            'timestamp': m.get('timestamp', ''),
            'duration_secs': summary.get('duration_secs', 0.0)
        })

        # 按模块统计
        module_stats = m.get('module_stats', {})
        for module, stats in module_stats.items():
            by_module[module].append({
                'timestamp': m.get('timestamp', ''),
                'duration_secs': stats.get('total_duration', 0.0)
            })

    return {
        'overall': overall,
        'by_module': dict(by_module)
    }


def _generate_trend_report(trends: Dict) -> str:
    """生成趋势报告"""
    lines = [
        '# Test Trends Report',
        '',
        f'**Generated**: {datetime.utcnow().strftime("%Y-%m-%d %H:%M:%S")}',
        ''
    ]

    # 失败率趋势
    lines.extend(['## Failure Rate Trend', ''])
    failure_rate = trends.get('failure_rate', {})
    overall = failure_rate.get('overall', [])
    if overall:
        last = overall[-1]
        lines.extend([
            f'**Latest Failure Rate**: {last.get("failure_rate", 0.0):.2f}%',
            f'**Total Tests**: {last.get("total", 0)}',
            f'**Failed Tests**: {last.get("failed", 0)}',
            ''
        ])

    # 稳定性分析
    lines.extend(['## Stability Analysis', ''])
    stability = trends.get('stability', {})
    lines.extend([
        f'**Total Unstable Tests**: {stability.get("total_unstable", 0)}',
        ''
    ])

    unstable_tests = stability.get('unstable_tests', [])
    if unstable_tests:
        lines.extend(['### Unstable Tests', '', '| Test Name | Runs | Failures | Failure Rate |', '|-----------|------|----------|--------------|'])
        for test in unstable_tests[:20]:  # 只显示前20个
            lines.append(
                f'| `{test.get("name", "")}` | {test.get("runs", 0)} | '
                f'{test.get("failures", 0)} | {test.get("failure_rate", 0.0):.2f}% |'
            )
        lines.append('')

    # 执行时间趋势
    lines.extend(['## Duration Trend', ''])
    duration = trends.get('duration', {})
    overall_duration = duration.get('overall', [])
    if overall_duration:
        last = overall_duration[-1]
        lines.extend([
            f'**Latest Duration**: {last.get("duration_secs", 0.0):.2f}s',
            ''
        ])

    return '\n'.join(lines)


def main():
    """CLI 入口（可以直接运行此脚本）"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='trends',
        description='Analyze test trends'
    )
    parser.add_argument('--metrics-dir', required=True, help='Metrics directory path')
    parser.add_argument('--output', required=True, help='Output report file path')

    args = parser.parse_args()
    analyze(args)


if __name__ == '__main__':
    main()

