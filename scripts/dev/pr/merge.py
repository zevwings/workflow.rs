"""PR 合并实现"""
import sys
import time
from pathlib import Path
from typing import Optional

# 添加父目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_info, log_success, log_warning, log_error, log_break
from utils.github import get_pull_request, merge_pull_request


def _output_ci_result(merged: bool, reason: Optional[str] = None) -> None:
    """输出 CI 模式结果到 GITHUB_OUTPUT"""
    import os

    output_file = os.environ.get('GITHUB_OUTPUT')
    if output_file:
        try:
            with open(output_file, 'a') as f:
                f.write(f'pr_merged={str(merged).lower()}\n')
                if reason:
                    f.write(f'pr_merge_reason={reason}\n')
        except IOError as e:
            log_error(f"Failed to write to GITHUB_OUTPUT: {e}")


def merge(args) -> None:
    """检查并合并 PR"""
    log_break('=')
    log_info("合并 PR")
    log_break('=')
    log_break()

    pr_number = str(args.pr_number)
    max_wait = args.max_wait or 300
    initial_interval = args.initial_interval or 3
    normal_interval = args.normal_interval or 5

    log_info(f"PR 编号: {pr_number}")
    log_break()

    # 等待 PR 状态更新
    log_info("等待 PR 状态更新和 CI 完成...")
    time.sleep(30)
    log_success("等待完成")
    log_break()

    # 检查 PR 状态
    elapsed = 30

    while elapsed < max_wait:
        check_interval = initial_interval if elapsed < 60 else normal_interval

        log_info(f"检查 PR 状态 (已等待 {elapsed}s)...")

        try:
            pr_info = get_pull_request(pr_number)
        except Exception as e:
            log_error(f"Failed to get PR status: {e}")
            sys.exit(1)

        merged = pr_info.get('merged', False)
        mergeable = pr_info.get('mergeable')

        if merged:
            log_success("PR 已合并")
            if args.ci:
                _output_ci_result(True)
            return

        if mergeable is True:
            log_success("PR 可以合并，开始合并...")
            log_break()
            _perform_merge(args, pr_number)
            return

        if mergeable is False:
            log_warning("PR 暂不可合并（可能有冲突或待检查），继续等待...")
        else:
            log_info("PR 合并状态计算中，继续等待...")

        time.sleep(check_interval)
        elapsed += check_interval

    log_error(f"超时: PR 在 {max_wait}s 内未变为可合并状态")
    if args.ci:
        _output_ci_result(False, "timeout")
    sys.exit(1)


def _perform_merge(args, pr_number: str) -> None:
    """执行 PR 合并"""
    commit_title = args.commit_title
    commit_message = args.commit_message
    merge_method = 'squash'  # 默认使用 squash

    log_info("执行 PR 合并...")
    log_info(f"   合并方式: {merge_method}")
    if commit_title:
        log_info(f"   Commit 标题: {commit_title}")
    if commit_message:
        log_info(f"   Commit 消息: {commit_message}")

    try:
        merged = merge_pull_request(
            pr_number,
            commit_title=commit_title,
            commit_message=commit_message,
            merge_method=merge_method
        )

        if merged:
            log_success("PR 合并成功")
            if args.ci:
                _output_ci_result(True)
        else:
            log_error("PR 合并失败")
            if args.ci:
                _output_ci_result(False, "merge_failed")
            sys.exit(1)
    except Exception as e:
        log_error(f"PR 合并失败: {e}")
        if args.ci:
            _output_ci_result(False, str(e))
        sys.exit(1)


def main():
    """CLI 入口（可以直接运行此脚本）"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='merge',
        description='Merge PR'
    )
    parser.add_argument('--pr-number', type=int, required=True, help='PR number')
    parser.add_argument('--max-wait', type=int, help='Max wait time in seconds (default: 300)')
    parser.add_argument('--initial-interval', type=int, help='Initial check interval in seconds (default: 3)')
    parser.add_argument('--normal-interval', type=int, help='Normal check interval in seconds (default: 5)')
    parser.add_argument('--commit-title', help='Commit title')
    parser.add_argument('--commit-message', help='Commit message')
    parser.add_argument('--ci', action='store_true', help='CI mode')

    args = parser.parse_args()
    merge(args)


if __name__ == '__main__':
    main()

