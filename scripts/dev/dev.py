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

# 添加当前目录到路径
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

