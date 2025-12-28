#!/usr/bin/env python3
"""Dev 工具命令入口

提供文档检查、测试检查、报告生成等开发工具功能。

要求 Python 3.13+
"""
import argparse
import sys
from pathlib import Path

# 检查 Python 版本（要求 3.13+）
if sys.version_info < (3, 13):
    print(f"❌ Error: Python 3.13+ required, but found {sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}", file=sys.stderr)
    print(f"   Current version: {sys.version}", file=sys.stderr)
    sys.exit(1)

# 添加当前目录到路径（dev.py 在 py/ 目录下）
sys.path.insert(0, str(Path(__file__).parent))

from utils.logger import log_error


def main():
    """主函数"""
    parser = argparse.ArgumentParser(
        prog='dev',
        description='Dev 工具命令入口'
    )
    subparsers = parser.add_subparsers(dest='command', help='Commands', required=True)

    # ci 命令
    ci_parser = subparsers.add_parser('ci', help='CI commands')
    ci_subparsers = ci_parser.add_subparsers(dest='ci_command', help='CI subcommands', required=True)

    # ci check-skip
    check_skip_parser = ci_subparsers.add_parser('check-skip', help='Check if CI should be skipped')
    check_skip_parser.add_argument('--branch', help='Branch name')
    check_skip_parser.add_argument('--pr-creator', help='PR creator username')
    check_skip_parser.add_argument('--expected-user', help='Expected user name')
    check_skip_parser.add_argument('--ci', action='store_true', help='CI mode (output to GITHUB_OUTPUT)')

    # ci verify
    verify_parser = ci_subparsers.add_parser('verify', help='Verify CI job status')
    verify_parser.add_argument('--jobs', help='Comma-separated list of job names')
    verify_parser.add_argument(
        '--should-skip',
        type=lambda x: x.lower() in ('true', '1', 'yes'),
        help='Whether CI should be skipped'
    )

    # checksum 命令
    checksum_parser = subparsers.add_parser('checksum', help='Checksum commands')
    checksum_subparsers = checksum_parser.add_subparsers(
        dest='checksum_command',
        help='Checksum subcommands',
        required=True
    )
    checksum_calc_parser = checksum_subparsers.add_parser('calculate', help='Calculate file checksum')
    checksum_calc_parser.add_argument('--file', required=True, help='File path')
    checksum_calc_parser.add_argument('--output', help='Output file path')

    # version 命令
    version_parser = subparsers.add_parser('version', help='Version commands')
    version_subparsers = version_parser.add_subparsers(
        dest='version_command',
        help='Version subcommands',
        required=True
    )
    version_gen_parser = version_subparsers.add_parser('generate', help='Generate version number')
    version_gen_parser.add_argument('--master', action='store_true', help='Generate version for master branch')
    version_gen_parser.add_argument('--update', action='store_true', help='Update Cargo.toml and Cargo.lock')
    version_gen_parser.add_argument('--ci', action='store_true', help='CI mode (output to GITHUB_OUTPUT)')

    # tag 命令
    tag_parser = subparsers.add_parser('tag', help='Tag commands')
    tag_subparsers = tag_parser.add_subparsers(
        dest='tag_command',
        help='Tag subcommands',
        required=True
    )
    tag_create_parser = tag_subparsers.add_parser('create', help='Create Git tag')
    tag_create_parser.add_argument('--tag', required=True, help='Tag name')
    tag_create_parser.add_argument('--commit', help='Commit SHA (default: HEAD)')
    tag_create_parser.add_argument('--ci', action='store_true', help='CI mode')

    tag_cleanup_parser = tag_subparsers.add_parser('cleanup', help='Cleanup alpha tags')
    tag_cleanup_parser.add_argument('--merge-commit', required=True, help='Merge commit SHA')
    tag_cleanup_parser.add_argument('--version', required=True, help='Current version')
    tag_cleanup_parser.add_argument('--ci', action='store_true', help='CI mode')

    # pr 命令
    pr_parser = subparsers.add_parser('pr', help='PR commands')
    pr_subparsers = pr_parser.add_subparsers(
        dest='pr_command',
        help='PR subcommands',
        required=True
    )
    pr_create_parser = pr_subparsers.add_parser('create', help='Create PR')
    pr_create_parser.add_argument('--version', required=True, help='Version number')
    pr_create_parser.add_argument('--branch', help='Branch name (default: bump-version-{version})')
    pr_create_parser.add_argument('--base', help='Base branch (default: master)')
    pr_create_parser.add_argument('--ci', action='store_true', help='CI mode')

    pr_merge_parser = pr_subparsers.add_parser('merge', help='Merge PR')
    pr_merge_parser.add_argument('--pr-number', type=int, required=True, help='PR number')
    pr_merge_parser.add_argument('--max-wait', type=int, help='Max wait time in seconds (default: 300)')
    pr_merge_parser.add_argument('--initial-interval', type=int, help='Initial check interval (default: 3)')
    pr_merge_parser.add_argument('--normal-interval', type=int, help='Normal check interval (default: 5)')
    pr_merge_parser.add_argument('--commit-title', help='Commit title')
    pr_merge_parser.add_argument('--commit-message', help='Commit message')
    pr_merge_parser.add_argument('--ci', action='store_true', help='CI mode')

    # homebrew 命令
    homebrew_parser = subparsers.add_parser('homebrew', help='Homebrew commands')
    homebrew_subparsers = homebrew_parser.add_subparsers(
        dest='homebrew_command',
        help='Homebrew subcommands',
        required=True
    )
    homebrew_update_parser = homebrew_subparsers.add_parser('update', help='Update Homebrew Formula')
    homebrew_update_parser.add_argument('--version', required=True, help='Version number')
    homebrew_update_parser.add_argument('--tag', required=True, help='Tag name')
    homebrew_update_parser.add_argument('--formula-path', help='Formula file path')
    homebrew_update_parser.add_argument('--template-path', help='Template file path')
    homebrew_update_parser.add_argument('--repo', help='Repository (owner/repo)')
    homebrew_update_parser.add_argument('--sha256', help='SHA256 checksum')
    homebrew_update_parser.add_argument('--commit', action='store_true', help='Commit changes')
    homebrew_update_parser.add_argument('--push', action='store_true', help='Push to remote')

    # tests 命令
    tests_parser = subparsers.add_parser('tests', help='Tests commands')
    tests_subparsers = tests_parser.add_subparsers(
        dest='tests_command',
        help='Tests subcommands',
        required=True
    )
    tests_check_parser = tests_subparsers.add_parser('check', help='Check tests')
    tests_check_subparsers = tests_check_parser.add_subparsers(
        dest='tests_check_command',
        help='Tests check subcommands',
        required=True
    )
    tests_check_coverage_parser = tests_check_subparsers.add_parser('coverage', help='Check test coverage')
    tests_check_coverage_parser.add_argument('--threshold', type=float, help='Coverage threshold (default: 80.0)')
    tests_check_coverage_parser.add_argument('--ci', action='store_true', help='CI mode')
    tests_check_coverage_parser.add_argument('--output', help='Output report file path')
    tests_check_coverage_parser.add_argument('--check-type', help='Check type')

    tests_report_parser = tests_subparsers.add_parser('report', help='Generate test report')
    tests_report_subparsers = tests_report_parser.add_subparsers(
        dest='tests_report_command',
        help='Tests report subcommands',
        required=True
    )
    tests_report_generate_parser = tests_report_subparsers.add_parser('generate', help='Generate test report')
    tests_report_generate_parser.add_argument('--format', help='Report format (html, json, markdown)')
    tests_report_generate_parser.add_argument('--output', help='Output file path')
    tests_report_generate_parser.add_argument('--comment', action='store_true', help='Generate PR comment')
    tests_report_generate_parser.add_argument('--report', '--reports', dest='reports', action='append', help='Report file paths')

    tests_metrics_parser = tests_subparsers.add_parser('metrics', help='Collect test metrics')
    tests_metrics_subparsers = tests_metrics_parser.add_subparsers(
        dest='tests_metrics_command',
        help='Tests metrics subcommands',
        required=True
    )
    tests_metrics_collect_parser = tests_metrics_subparsers.add_parser('collect', help='Collect test metrics')
    tests_metrics_collect_parser.add_argument('--report', required=True, help='Test report file path')
    tests_metrics_collect_parser.add_argument('--output', required=True, help='Output metrics file path')

    tests_trends_parser = tests_subparsers.add_parser('trends', help='Analyze test trends')
    tests_trends_subparsers = tests_trends_parser.add_subparsers(
        dest='tests_trends_command',
        help='Tests trends subcommands',
        required=True
    )
    tests_trends_analyze_parser = tests_trends_subparsers.add_parser('analyze', help='Analyze test trends')
    tests_trends_analyze_parser.add_argument('--metrics-dir', required=True, help='Metrics directory path')
    tests_trends_analyze_parser.add_argument('--output', required=True, help='Output report file path')

    # performance 命令
    performance_parser = subparsers.add_parser('performance', help='Performance commands')
    performance_subparsers = performance_parser.add_subparsers(
        dest='performance_command',
        help='Performance subcommands',
        required=True
    )
    performance_analyze_parser = performance_subparsers.add_parser('analyze', help='Analyze performance regression')
    performance_analyze_parser.add_argument('--current', required=True, help='Current performance data file')
    performance_analyze_parser.add_argument('--baseline', help='Baseline performance data file')
    performance_analyze_parser.add_argument('--output', help='Output report file path')
    performance_analyze_parser.add_argument('--threshold', type=float, help='Regression threshold (default: 0.2)')

    # docs 命令
    docs_parser = subparsers.add_parser('docs', help='Docs commands')
    docs_subparsers = docs_parser.add_subparsers(
        dest='docs_command',
        help='Docs subcommands',
        required=True
    )
    docs_check_parser = docs_subparsers.add_parser('check', help='Check docs')
    docs_check_subparsers = docs_check_parser.add_subparsers(
        dest='docs_check_command',
        help='Docs check subcommands',
        required=True
    )
    docs_check_integrity_parser = docs_check_subparsers.add_parser('integrity', help='Check document integrity')
    docs_check_integrity_parser.add_argument('--architecture', action='store_true', help='Check architecture docs')
    docs_check_integrity_parser.add_argument('--timestamps', action='store_true', help='Check timestamp format')
    docs_check_integrity_parser.add_argument('--ci', action='store_true', help='CI mode')

    docs_check_links_parser = docs_check_subparsers.add_parser('links', help='Check document links')
    docs_check_links_parser.add_argument('--external', action='store_true', help='Check external links')
    docs_check_links_parser.add_argument('--ci', action='store_true', help='CI mode')

    docs_report_parser = docs_subparsers.add_parser('report', help='Generate docs report')
    docs_report_subparsers = docs_report_parser.add_subparsers(
        dest='docs_report_command',
        help='Docs report subcommands',
        required=True
    )
    docs_report_generate_parser = docs_report_subparsers.add_parser('generate', help='Generate document check report')
    docs_report_generate_parser.add_argument('--output', help='Output report file path')
    docs_report_generate_parser.add_argument('--check-type', help='Check type')

    # 解析参数
    args = parser.parse_args()

    # 路由到对应的命令处理函数
    try:
        if args.command == 'ci':
            if args.ci_command == 'check-skip':
                from ci.check_skip import check_skip
                check_skip(args)
            elif args.ci_command == 'verify':
                from ci.verify import verify
                verify(args)
            else:
                log_error(f"Unknown CI command: {args.ci_command}")
                sys.exit(1)
        elif args.command == 'checksum':
            if args.checksum_command == 'calculate':
                from checksum.calculate import calculate
                calculate(args)
            else:
                log_error(f"Unknown checksum command: {args.checksum_command}")
                sys.exit(1)
        elif args.command == 'version':
            if args.version_command == 'generate':
                from version.generate import generate
                generate(args)
            else:
                log_error(f"Unknown version command: {args.version_command}")
                sys.exit(1)
        elif args.command == 'tag':
            if args.tag_command == 'create':
                from tag.create import create
                create(args)
            elif args.tag_command == 'cleanup':
                from tag.cleanup import cleanup
                cleanup(args)
            else:
                log_error(f"Unknown tag command: {args.tag_command}")
                sys.exit(1)
        elif args.command == 'pr':
            if args.pr_command == 'create':
                from pr.create import create
                create(args)
            elif args.pr_command == 'merge':
                from pr.merge import merge
                merge(args)
            else:
                log_error(f"Unknown PR command: {args.pr_command}")
                sys.exit(1)
        elif args.command == 'homebrew':
            if args.homebrew_command == 'update':
                from homebrew.update import update
                update(args)
            else:
                log_error(f"Unknown homebrew command: {args.homebrew_command}")
                sys.exit(1)
        elif args.command == 'tests':
            if args.tests_command == 'check':
                if args.tests_check_command == 'coverage':
                    from tests.check import check
                    check(args)
                else:
                    log_error(f"Unknown tests check command: {args.tests_check_command}")
                    sys.exit(1)
            elif args.tests_command == 'report':
                if args.tests_report_command == 'generate':
                    from tests.report import generate
                    generate(args)
                else:
                    log_error(f"Unknown tests report command: {args.tests_report_command}")
                    sys.exit(1)
            elif args.tests_command == 'metrics':
                if args.tests_metrics_command == 'collect':
                    from tests.metrics import collect
                    collect(args)
                else:
                    log_error(f"Unknown tests metrics command: {args.tests_metrics_command}")
                    sys.exit(1)
            elif args.tests_command == 'trends':
                if args.tests_trends_command == 'analyze':
                    from tests.trends import analyze
                    analyze(args)
                else:
                    log_error(f"Unknown tests trends command: {args.tests_trends_command}")
                    sys.exit(1)
            else:
                log_error(f"Unknown tests command: {args.tests_command}")
                sys.exit(1)
        elif args.command == 'performance':
            if args.performance_command == 'analyze':
                from performance.analyze import analyze
                analyze(args)
            else:
                log_error(f"Unknown performance command: {args.performance_command}")
                sys.exit(1)
        elif args.command == 'docs':
            if args.docs_command == 'check':
                if args.docs_check_command == 'integrity':
                    from docs.integrity import check
                    check(args)
                elif args.docs_check_command == 'links':
                    from docs.links import check
                    check(args)
                else:
                    log_error(f"Unknown docs check command: {args.docs_check_command}")
                    sys.exit(1)
            elif args.docs_command == 'report':
                if args.docs_report_command == 'generate':
                    from docs.report import generate
                    generate(args)
                else:
                    log_error(f"Unknown docs report command: {args.docs_report_command}")
                    sys.exit(1)
            else:
                log_error(f"Unknown docs command: {args.docs_command}")
                sys.exit(1)
        else:
            log_error(f"Unknown command: {args.command}")
            sys.exit(1)
    except KeyboardInterrupt:
        log_error("Interrupted by user")
        sys.exit(130)
    except Exception as e:
        log_error(f"Error: {e}")
        sys.exit(1)


if __name__ == '__main__':
    sys.exit(main())

