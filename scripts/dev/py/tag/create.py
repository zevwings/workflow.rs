"""Git Tag 创建实现"""
import os
import sys
from pathlib import Path
from typing import Optional

# 添加父目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_info, log_success, log_warning, log_error, log_break
from utils.git import (
    tag_exists, create_tag, delete_local_tag, delete_remote_tag,
    push_tag, get_last_commit_sha, run_git_command
)


def _get_tag_info(tag_name: str) -> Optional[str]:
    """获取 tag 指向的 commit SHA"""
    try:
        result = run_git_command(['rev-list', '-n', '1', tag_name], check=False)
        if result.returncode == 0:
            return result.stdout.strip()
    except Exception:
        pass
    return None


def _output_ci_result(created: bool, tag_name: str) -> None:
    """输出 CI 结果到 GITHUB_OUTPUT"""
    output_file = os.environ.get('GITHUB_OUTPUT')
    if output_file:
        try:
            with open(output_file, 'a') as f:
                f.write(f'tag_created={str(created).lower()}\n')
                f.write(f'tag_name={tag_name}\n')
        except IOError as e:
            log_error(f"Failed to write to GITHUB_OUTPUT: {e}")


def create(args) -> None:
    """创建并推送 tag"""
    log_break('=')
    log_info("创建 Git Tag")
    log_break('=')
    log_break()

    tag_name = args.tag
    log_info(f"Tag 名称: {tag_name}")

    # 检查 tag 是否已存在
    exists_local, exists_remote = tag_exists(tag_name)

    if exists_local or exists_remote:
        log_warning("Tag 已存在")
        log_info(f"   本地存在: {exists_local}")
        log_info(f"   远程存在: {exists_remote}")

        # 获取现有 tag 的 commit SHA
        existing_tag_sha = _get_tag_info(tag_name)
        current_head = get_last_commit_sha() or ""
        target_sha = args.commit or current_head

        if existing_tag_sha:
            log_info(f"   现有 tag commit: {existing_tag_sha}")
        log_info(f"   目标 commit: {target_sha}")

        if existing_tag_sha == target_sha:
            log_success("Tag 已存在且指向正确的 commit")
            if args.ci:
                _output_ci_result(False, tag_name)
            return
        else:
            log_warning("Tag 已存在但指向不同的 commit")
            log_warning("   删除现有 tag 并重新创建...")

            if exists_local:
                delete_local_tag(tag_name)
                log_success("已删除本地 tag")
            if exists_remote:
                delete_remote_tag(tag_name)
                log_success("已删除远程 tag")

    # 创建 tag
    log_break()
    log_info("创建 tag...")
    create_tag(tag_name, args.commit)
    log_success("Tag 创建成功")

    # 推送 tag
    log_break()
    log_info("推送 tag 到远程...")
    try:
        push_tag(tag_name)
        log_success("Tag 推送成功")
    except Exception as e:
        log_warning(f"Tag 推送失败（可能已存在）: {e}")
        # 验证远程 tag 是否指向正确的 commit
        try:
            result = run_git_command(['ls-remote', '--tags', 'origin', tag_name], check=False)
            if tag_name in result.stdout:
                remote_tag_sha = result.stdout.split()[0]
                if remote_tag_sha == target_sha:
                    log_success("Tag 已存在于远程且指向正确的 commit")
                else:
                    log_error("Tag 存在于远程但指向不同的 commit")
                    log_error(f"   远程 tag commit: {remote_tag_sha}")
                    log_error(f"   目标 commit: {target_sha}")
                    sys.exit(1)
        except Exception:
            pass

    log_success("Tag 创建和推送完成")
    log_info(f"   Tag: {tag_name}")
    if args.commit:
        log_info(f"   Commit: {args.commit}")
    else:
        current_sha = get_last_commit_sha()
        if current_sha:
            log_info(f"   Commit: {current_sha}")

    if args.ci:
        _output_ci_result(True, tag_name)


def main():
    """CLI 入口（可以直接运行此脚本）"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='create',
        description='Create and push Git tag'
    )
    parser.add_argument('--tag', required=True, help='Tag name')
    parser.add_argument('--commit', help='Commit SHA (default: HEAD)')
    parser.add_argument('--ci', action='store_true', help='CI mode')

    args = parser.parse_args()
    create(args)


if __name__ == '__main__':
    main()

