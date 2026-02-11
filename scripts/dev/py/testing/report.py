"""测试报告生成实现"""
import json
import sys
from collections import defaultdict
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional

# 添加父目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_info, log_success, log_warning, log_break


class TestReporter:
    """测试报告生成器"""

    def __init__(self):
        self.test_cases: List[Dict] = []
        self.start_time = datetime.utcnow()
        self.end_time: Optional[datetime] = None
        self.passed = 0
        self.failed = 0
        self.ignored = 0
        self.timeout = 0

    def record_test(self, test_case: Dict):
        """记录测试用例"""
        status = test_case.get('status', '')
        if status == 'Passed':
            self.passed += 1
        elif status == 'Failed':
            self.failed += 1
        elif status == 'Ignored':
            self.ignored += 1
        elif status == 'Timeout':
            self.timeout += 1
        self.test_cases.append(test_case)

    def finish(self):
        """完成报告生成"""
        self.end_time = datetime.utcnow()

    def total_duration(self) -> float:
        """总执行时间（秒）"""
        if self.end_time:
            return (self.end_time - self.start_time).total_seconds()
        return 0.0

    def success_rate(self) -> float:
        """成功率（百分比）"""
        if not self.test_cases:
            return 0.0
        return (self.passed / len(self.test_cases)) * 100.0

    def generate_json_report(self) -> str:
        """生成 JSON 格式报告"""
        report = {
            'summary': {
                'total': len(self.test_cases),
                'passed': self.passed,
                'failed': self.failed,
                'ignored': self.ignored,
                'timeout': self.timeout,
                'success_rate': self.success_rate(),
                'duration_secs': self.total_duration()
            },
            'test_cases': self.test_cases
        }
        return json.dumps(report, indent=2, ensure_ascii=False)

    def generate_html_report(self) -> str:
        """生成 HTML 格式报告"""
        duration = self.total_duration()
        success_rate = self.success_rate()
        test_rows = self._generate_test_rows_html()

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
                    <div class="stat-value total">{len(self.test_cases)}</div>
                    <div class="stat-label">Total</div>
                </div>
                <div class="stat">
                    <div class="stat-value rate">{success_rate:.2f}%</div>
                    <div class="stat-label">Success Rate</div>
                </div>
                <div class="stat">
                    <div class="stat-value rate">{duration:.2f}s</div>
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

    def _generate_test_rows_html(self) -> str:
        """生成 HTML 表格行"""
        rows = []
        slow_threshold = 1.0

        for test_case in self.test_cases:
            status = test_case.get('status', '')
            status_class = {
                'Passed': 'status-passed',
                'Failed': 'status-failed',
                'Ignored': 'status-ignored',
                'Timeout': 'status-timeout'
            }.get(status, 'status-passed')

            status_text = {
                'Passed': '✓ Passed',
                'Failed': '✗ Failed',
                'Ignored': '⊘ Ignored',
                'Timeout': '⏱ Timeout'
            }.get(status, status)

            duration = test_case.get('duration_secs', 0.0)
            duration_class = ' class="slow-test"' if duration > slow_threshold else ''
            duration_text = f'{duration:.3f}s ⚠' if duration > slow_threshold else f'{duration:.3f}s'

            error_html = ''
            if test_case.get('error_message'):
                error = self._html_escape(test_case['error_message'])
                error_html = f'<div class="error-message">{error}</div>'

            name = self._html_escape(test_case.get('name', ''))
            module = self._html_escape(test_case.get('module', ''))

            rows.append(f"""
                <tr{duration_class}>
                    <td class="test-name">{name}</td>
                    <td>{module}</td>
                    <td><span class="status-badge {status_class}">{status_text}</span></td>
                    <td>{duration_text}</td>
                    <td>{error_html}</td>
                </tr>
            """)

        return '\n'.join(rows)

    def _html_escape(self, text: str) -> str:
        """HTML 转义"""
        return (text
                .replace('&', '&amp;')
                .replace('<', '&lt;')
                .replace('>', '&gt;')
                .replace('"', '&quot;')
                .replace("'", '&#39;'))

    def generate_markdown_report(self) -> str:
        """生成 Markdown 格式报告"""
        duration = self.total_duration()
        success_rate = self.success_rate()

        lines = [
            '# Test Execution Report',
            '',
            '## Summary',
            '',
            f'- **Total Tests**: {len(self.test_cases)}',
            f'- **✅ Passed**: {self.passed}',
            f'- **❌ Failed**: {self.failed}',
            f'- **⏭ Ignored**: {self.ignored}',
            f'- **⏱ Timeout**: {self.timeout}',
            f'- **Success Rate**: {success_rate:.2f}%',
            f'- **Duration**: {duration:.2f}s',
            '',
            '## Test Cases',
            '',
            '| Test Name | Module | Status | Duration |',
            '|-----------|--------|--------|----------|'
        ]

        for test_case in self.test_cases:
            name = test_case.get('name', '')
            module = test_case.get('module', '')
            status = test_case.get('status', '')
            duration = test_case.get('duration_secs', 0.0)

            status_emoji = {
                'Passed': '✅',
                'Failed': '❌',
                'Ignored': '⏭',
                'Timeout': '⏱'
            }.get(status, '')

            lines.append(f'| `{name}` | {module} | {status_emoji} {status} | {duration:.3f}s |')

        return '\n'.join(lines)

    def generate_pr_comment(self) -> str:
        """生成 PR 评论内容"""
        summary = {
            'total': len(self.test_cases),
            'passed': self.passed,
            'failed': self.failed,
            'ignored': self.ignored,
            'timeout': self.timeout,
            'success_rate': self.success_rate(),
            'duration_secs': self.total_duration()
        }

        lines = [
            '## 📊 Test Results',
            '',
            '| Metric | Value |',
            '|--------|-------|',
            f'| Total Tests | {summary["total"]} |',
            f'| ✅ Passed | {summary["passed"]} |',
            f'| ❌ Failed | {summary["failed"]} |',
            f'| ⏭ Ignored | {summary["ignored"]} |',
            f'| ⏱ Timeout | {summary["timeout"]} |',
            f'| Success Rate | {summary["success_rate"]:.2f}% |',
            f'| Duration | {summary["duration_secs"]:.2f}s |',
            ''
        ]

        if summary['failed'] > 0:
            lines.extend(['### ❌ Failed Tests', ''])
            for test_case in self.test_cases:
                if test_case.get('status') == 'Failed':
                    lines.append(f'- `{test_case.get("name", "")}`')
                    if test_case.get('error_message'):
                        error = test_case['error_message']
                        error_preview = error[:200] + '...' if len(error) > 200 else error
                        lines.append(f'  ```\n{error_preview}\n```')
            lines.append('')

        return '\n'.join(lines)


def _extract_module(test_name: str) -> str:
    """提取模块名"""
    if '::' in test_name:
        return test_name.rsplit('::', 1)[0]
    return 'unknown'


def _parse_cargo_test_output() -> TestReporter:
    """解析 cargo test JSON 输出"""
    reporter = TestReporter()

    for line in sys.stdin:
        line = line.strip()
        if not line or not line.startswith('{'):
            continue

        try:
            msg = json.loads(line)
            if msg.get('type') == 'test':
                name = msg.get('name')
                event = msg.get('event')
                if name and event:
                    duration = msg.get('exec_time', 0.0)
                    module = _extract_module(name)

                    status_map = {
                        'ok': 'Passed',
                        'failed': 'Failed',
                        'ignored': 'Ignored',
                        'timeout': 'Timeout'
                    }
                    status = status_map.get(event, '')

                    if status:
                        test_case = {
                            'name': name,
                            'status': status,
                            'duration_secs': duration,
                            'error_message': msg.get('stdout') if status == 'Failed' else None,
                            'module': module
                        }
                        reporter.record_test(test_case)
        except json.JSONDecodeError:
            continue

    reporter.finish()
    return reporter


def _merge_reports(reports: List[Dict]) -> Dict:
    """合并多个测试报告"""
    if not reports:
        return {
            'summary': {
                'total': 0, 'passed': 0, 'failed': 0, 'ignored': 0, 'timeout': 0,
                'success_rate': 0.0, 'duration_secs': 0.0
            },
            'test_cases': []
        }

    if len(reports) == 1:
        return reports[0]

    merged_summary = {
        'total': 0, 'passed': 0, 'failed': 0, 'ignored': 0, 'timeout': 0,
        'success_rate': 0.0, 'duration_secs': 0.0
    }

    test_case_map: Dict[str, Dict] = {}

    for report in reports:
        summary = report.get('summary', {})
        merged_summary['total'] += summary.get('total', 0)
        merged_summary['passed'] += summary.get('passed', 0)
        merged_summary['failed'] += summary.get('failed', 0)
        merged_summary['ignored'] += summary.get('ignored', 0)
        merged_summary['timeout'] += summary.get('timeout', 0)
        merged_summary['duration_secs'] += summary.get('duration_secs', 0.0)

        for test_case in report.get('test_cases', []):
            test_name = test_case.get('name', '')
            if test_name in test_case_map:
                # 如果当前测试失败或超时，优先保留
                if test_case.get('status') in ('Failed', 'Timeout'):
                    test_case_map[test_name] = test_case
            else:
                test_case_map[test_name] = test_case

    merged_summary['total'] = len(test_case_map)
    if merged_summary['total'] > 0:
        merged_summary['success_rate'] = (
            merged_summary['passed'] / merged_summary['total']
        ) * 100.0

    return {
        'summary': merged_summary,
        'test_cases': list(test_case_map.values())
    }


def generate(args) -> None:
    """生成测试报告"""
    # 如果提供了报告文件且需要生成评论，直接生成评论
    if args.comment and args.reports:
        _generate_comment_from_reports(args.reports, args.output)
        return

    log_break('=')
    log_info("生成测试报告")
    log_break('=')
    log_break()
    log_info("解析测试输出...")

    reporter = _parse_cargo_test_output()

    if not reporter.test_cases:
        log_warning("没有找到测试用例")
        return

    # 生成报告
    format_type = args.format or 'json'
    if format_type == 'html':
        report_content = reporter.generate_html_report()
    elif format_type == 'json':
        report_content = reporter.generate_json_report()
    elif format_type in ('markdown', 'md'):
        report_content = reporter.generate_markdown_report()
    else:
        log_warning(f"不支持的格式: {format_type}。支持: html, json, markdown")
        sys.exit(1)

    # 输出报告
    if args.output:
        output_path = Path(args.output)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(report_content)
        log_success(f"报告已保存到: {args.output}")

        # 如果需要生成评论，也生成评论
        if args.comment:
            _generate_comment_from_reports([args.output], None)
    else:
        print(report_content)


def _generate_comment_from_reports(report_paths: List[str], output_path: Optional[str]) -> None:
    """从报告文件生成 PR 评论"""
    log_break('=')
    log_info("生成 PR 评论")
    log_break('=')
    log_break()

    # 加载并合并报告
    reports = []
    for report_path in report_paths:
        log_info(f"加载报告: {report_path}")
        try:
            with open(report_path, 'r') as f:
                report = json.load(f)
                reports.append(report)
        except Exception as e:
            log_warning(f"Failed to load report {report_path}: {e}")

    if not reports:
        log_warning("没有找到有效的报告文件")
        return

    merged_report = _merge_reports(reports)

    # 生成 PR 评论内容
    summary = merged_report.get('summary', {})
    test_cases = merged_report.get('test_cases', [])

    lines = [
        '## 📊 Test Results',
        '',
        '| Metric | Value |',
        '|--------|-------|',
        f'| Total Tests | {summary.get("total", 0)} |',
        f'| ✅ Passed | {summary.get("passed", 0)} |',
        f'| ❌ Failed | {summary.get("failed", 0)} |',
        f'| ⏭ Ignored | {summary.get("ignored", 0)} |',
        f'| ⏱ Timeout | {summary.get("timeout", 0)} |',
        f'| Success Rate | {summary.get("success_rate", 0.0):.2f}% |',
        f'| Duration | {summary.get("duration_secs", 0.0):.2f}s |',
        ''
    ]

    if summary.get('failed', 0) > 0:
        lines.extend(['### ❌ Failed Tests', ''])
        for test_case in test_cases:
            if test_case.get('status') == 'Failed':
                lines.append(f'- `{test_case.get("name", "")}`')
                if test_case.get('error_message'):
                    error = test_case['error_message']
                    error_preview = error[:200] + '...' if len(error) > 200 else error
                    lines.append(f'  ```\n{error_preview}\n```')
        lines.append('')

    comment_content = '\n'.join(lines)

    if output_path:
        output_file = Path(output_path)
        output_file.parent.mkdir(parents=True, exist_ok=True)
        output_file.write_text(comment_content)
        log_success(f"PR 评论已保存到: {output_path}")
    else:
        print(comment_content)


def main():
    """CLI 入口（可以直接运行此脚本）"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='report',
        description='Generate test report'
    )
    parser.add_argument('--format', help='Report format (html, json, markdown)')
    parser.add_argument('--output', help='Output file path')
    parser.add_argument('--comment', action='store_true', help='Generate PR comment')
    parser.add_argument('--report', '--reports', dest='reports', action='append', help='Report file paths')

    args = parser.parse_args()
    generate(args)


if __name__ == '__main__':
    main()

