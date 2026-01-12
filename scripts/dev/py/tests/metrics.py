"""测试指标收集实现"""
import json
import sys
from collections import defaultdict
from datetime import datetime
from pathlib import Path
from typing import Dict, List

# 添加父目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_info, log_success, log_break


def collect(args) -> None:
    """收集测试指标"""
    if not args.report:
        print("Error: --report 参数是必需的", file=sys.stderr)
        sys.exit(1)
    if not args.output:
        print("Error: --output 参数是必需的", file=sys.stderr)
        sys.exit(1)

    log_break('=')
    log_info("收集测试指标")
    log_break('=')
    log_break()

    # 读取测试报告
    log_info(f"读取测试报告: {args.report}")
    try:
        with open(args.report, 'r') as f:
            report = json.load(f)
    except Exception as e:
        print(f"Error: Failed to parse test report: {e}", file=sys.stderr)
        sys.exit(1)

    # 收集指标
    log_info("收集指标数据...")
    metrics = _collect_metrics_from_report(report)

    # 保存指标文件
    log_info(f"保存指标文件: {args.output}")
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    with open(output_path, 'w') as f:
        json.dump(metrics, f, indent=2, ensure_ascii=False)

    log_break()
    log_success("指标收集完成")
    summary = metrics.get('summary', {})
    log_info(f"   总测试数: {summary.get('total', 0)}")
    log_info(f"   通过: {summary.get('passed', 0)}")
    log_info(f"   失败: {summary.get('failed', 0)}")
    log_info(f"   成功率: {summary.get('success_rate', 0.0):.2f}%")
    log_info(f"   模块数: {len(metrics.get('module_stats', {}))}")
    log_break()


def _collect_metrics_from_report(report: Dict) -> Dict:
    """从测试报告收集指标"""
    summary = report.get('summary', {})
    test_cases = report.get('test_cases', [])

    # 按模块统计
    module_stats: Dict[str, Dict] = defaultdict(lambda: {
        'total': 0,
        'passed': 0,
        'failed': 0,
        'ignored': 0,
        'timeout': 0,
        'total_duration': 0.0,
        'failure_rate': 0.0,
        'success_rate': 0.0
    })

    for test in test_cases:
        module = test.get('module', 'unknown')
        stats = module_stats[module]

        stats['total'] += 1
        stats['total_duration'] += test.get('duration_secs', 0.0)

        status = test.get('status', '')
        if status == 'Passed':
            stats['passed'] += 1
        elif status == 'Failed':
            stats['failed'] += 1
        elif status == 'Ignored':
            stats['ignored'] += 1
        elif status == 'Timeout':
            stats['timeout'] += 1

    # 计算失败率和成功率
    for stats in module_stats.values():
        if stats['total'] > 0:
            stats['failure_rate'] = (stats['failed'] / stats['total']) * 100.0
            stats['success_rate'] = (stats['passed'] / stats['total']) * 100.0

    # 生成时间戳
    timestamp = datetime.utcnow().isoformat() + 'Z'

    # 构建指标数据
    metrics = {
        'timestamp': timestamp,
        'summary': summary.copy(),
        'module_stats': dict(module_stats),
        'test_cases': test_cases.copy(),
        'metadata': None
    }

    return metrics


def main():
    """CLI 入口（可以直接运行此脚本）"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='metrics',
        description='Collect test metrics'
    )
    parser.add_argument('--report', required=True, help='Test report file path')
    parser.add_argument('--output', required=True, help='Output metrics file path')

    args = parser.parse_args()
    collect(args)


if __name__ == '__main__':
    main()

