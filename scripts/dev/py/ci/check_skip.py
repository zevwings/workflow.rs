"""CI 跳过检查实现"""
import os
import sys
import subprocess
from pathlib import Path

# 添加父目录到路径，以便导入 utils
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_info, log_success, log_error


def _get_current_branch() -> str | None:
    """获取当前 Git 分支"""
    try:
        result = subprocess.run(
            ['git', 'branch', '--show-current'],
            capture_output=True,
            text=True,
            check=True,
            cwd=os.getcwd()
        )
        branch = result.stdout.strip()
        return branch if branch else None
    except (subprocess.CalledProcessError, FileNotFoundError) as e:
        return None


def _output_github_actions(should_skip: bool) -> None:
    """输出到 GitHub Actions GITHUB_OUTPUT"""
    output_file = os.environ.get('GITHUB_OUTPUT')
    if not output_file:
        log_error("GITHUB_OUTPUT not set (required when --ci flag is used)")
        sys.exit(1)

    try:
        with open(output_file, 'a') as f:
            f.write(f'should_skip={should_skip}\n')
        log_success(f"Output should_skip={should_skip} to GITHUB_OUTPUT")
    except IOError as e:
        log_error(f"Failed to write to GITHUB_OUTPUT: {e}")
        sys.exit(1)


def check_skip(args) -> bool:
    """检查是否应该跳过 CI"""
    # 获取分支名称
    branch = (
        args.branch or
        os.environ.get('GITHUB_HEAD_REF') or
        os.environ.get('GITHUB_REF_NAME') or
        _get_current_branch()
    )

    if not branch:
        log_error("Failed to get branch name")
        sys.exit(1)

    event_name = os.environ.get('GITHUB_EVENT_NAME', 'unknown')
    log_info(f"Checking branch: {branch}")
    log_info(f"   Event: {event_name}")

    # 检查是否是版本更新分支（bump-version-*）
    if branch.startswith('bump-version-'):
        log_success(f"Detected bump-version-* branch: {branch}")

        # 对于 PR 事件，验证分支是否由 CI 创建
        if event_name == 'pull_request':
            pr_creator = args.pr_creator or os.environ.get('GITHUB_PR_CREATOR', '')
            expected_user = args.expected_user or os.environ.get('WORKFLOW_USER_NAME', '')

            log_info(f"   PR creator: {pr_creator or 'unknown'}")
            log_info(f"   Expected user: {expected_user or 'unknown'}")

            if pr_creator and expected_user and pr_creator != expected_user:
                log_error("bump-version-* branches can only be created by authorized user")
                log_error(f"   PR creator: {pr_creator}, expected: {expected_user}")
                sys.exit(1)

        should_skip = True
        log_success("Setting should_skip=true (bump-version-* branch detected)")
    else:
        log_info("Not a bump-version-* branch, CI will run normally")
        should_skip = False

    # 输出到 GITHUB_OUTPUT（仅在 CI 模式下）
    if args.ci:
        _output_github_actions(should_skip)
    else:
        # 非 CI 模式，直接输出结果
        print(f"should_skip={should_skip}")

    return should_skip


def main():
    """CLI 入口（可以直接运行此脚本）"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='check-skip',
        description='Check if CI should be skipped'
    )
    parser.add_argument('--branch', help='Branch name')
    parser.add_argument('--pr-creator', help='PR creator username')
    parser.add_argument('--expected-user', help='Expected user name')
    parser.add_argument('--ci', action='store_true', help='CI mode (output to GITHUB_OUTPUT)')

    args = parser.parse_args()
    check_skip(args)


if __name__ == '__main__':
    main()
