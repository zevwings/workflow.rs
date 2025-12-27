"""Alpha Tag 清理实现"""
import os
import re
import sys
from pathlib import Path
from typing import List, Optional

# 添加父目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_info, log_success, log_warning, log_error, log_break
from utils.git import list_tags, delete_local_tag, delete_remote_tag, run_git_command


def _extract_version(tag_name: str) -> Optional[str]:
    """从 tag 名称提取版本号（如 v1.6.0.alpha-xxx -> 1.6.0）"""
    # 匹配 vx.x.x 或 vx.x.x.alpha-xxx 格式
    match = re.match(r'^v(\d+\.\d+\.\d+)', tag_name)
    if match:
        return match.group(1)
    return None


def _list_alpha_tags() -> List[str]:
    """列出所有 alpha tag（格式：vx.x.x.alpha-xxx）"""
    all_tags = list_tags()
    alpha_pattern = re.compile(r'^v\d+\.\d+\.\d+\.alpha-')
    return [tag for tag in all_tags if alpha_pattern.match(tag)]


def _is_ancestor(commit1: str, commit2: str) -> bool:
    """检查 commit1 是否是 commit2 的祖先"""
    try:
        result = run_git_command(['merge-base', '--is-ancestor', commit1, commit2], check=False)
        return result.returncode == 0
    except Exception:
        return False


def _get_tag_commit(tag_name: str) -> Optional[str]:
    """获取 tag 指向的 commit SHA"""
    try:
        result = run_git_command(['rev-parse', tag_name], check=False)
        if result.returncode == 0:
            return result.stdout.strip()
    except Exception:
        pass
    return None


def cleanup(args) -> None:
    """清理 alpha tags"""
    log_break('=')
    log_info("清理 Alpha Tags")
    log_break('=')
    log_break()

    merge_commit_sha = args.merge_commit
    current_version = args.version

    log_info(f"合并提交 SHA: {merge_commit_sha}")
    log_info(f"当前版本: {current_version}")

    # 提取基础版本号（移除 'v' 前缀和 alpha 后缀）
    base_version = current_version.lstrip('v').split('.')[:3]
    base_version_str = '.'.join(base_version)

    log_info(f"基础版本号: {base_version_str}")
    log_break()

    # 获取 master 分支的 first parent（合并前的最后一个提交）
    try:
        result = run_git_command(['rev-parse', f'{merge_commit_sha}^1'])
        first_parent = result.stdout.strip()
        log_info(f"First parent (master before merge): {first_parent}")
    except Exception as e:
        log_error(f"Failed to get first parent commit: {e}")
        sys.exit(1)

    # 获取 master 分支的当前 HEAD（合并后的状态）
    try:
        result = run_git_command(['rev-parse', 'HEAD'])
        master_head = result.stdout.strip()
        log_info(f"Master HEAD (after merge): {master_head}")
    except Exception as e:
        log_error(f"Failed to get master HEAD: {e}")
        sys.exit(1)

    log_break()

    # 查找所有 alpha tag
    log_info("查找 alpha tags...")
    alpha_tags = _list_alpha_tags()

    if not alpha_tags:
        log_success("未找到 alpha tags，无需清理")
        if args.ci:
            _output_ci_result(0)
        return

    log_info(f"找到 {len(alpha_tags)} 个 alpha tags:")
    for tag in alpha_tags:
        log_info(f"   - {tag}")
    log_break()

    # 检查每个 alpha tag 是否指向已合并的提交
    log_info("检查哪些 alpha tags 指向已合并的提交...")
    tags_to_delete = []

    for tag in alpha_tags:
        tag_commit = _get_tag_commit(tag)
        if not tag_commit:
            log_warning(f"Tag {tag}: 无法解析 commit")
            continue

        tag_version = _extract_version(tag)

        # 检查 tag 是否在 master 分支的 first-parent 路径上
        if _is_ancestor(tag_commit, first_parent):
            # Tag 在 master 的 first-parent 路径上，保留它
            log_info(f"   ⏭️  Tag {tag} ({tag_commit}) 在 master 分支 first-parent 路径上，保留")
        elif _is_ancestor(tag_commit, master_head):
            # Tag 在合并提交的祖先中，但不在 first-parent 路径上
            # 说明它来自已合并的分支，应该删除
            if tag_version == base_version_str:
                log_info(
                    f"   ✅ Tag {tag} ({tag_commit}) 版本 {tag_version} 匹配当前版本 {base_version_str} "
                    f"且来自已合并分支，将删除"
                )
            else:
                log_info(f"   ✅ Tag {tag} ({tag_commit}) 来自已合并分支，将删除")
            tags_to_delete.append(tag)
        else:
            # Tag 不在合并提交的祖先中，检查版本号是否匹配
            if tag_version == base_version_str:
                log_warning(
                    f"   ⚠️  Tag {tag} ({tag_commit}) 版本 {tag_version} 匹配当前版本 {base_version_str} "
                    f"但 commit 不在合并祖先中"
                )
                log_info("   💡 由于版本号匹配，考虑删除...")
                tags_to_delete.append(tag)
            else:
                log_info(f"   ⏭️  Tag {tag} ({tag_commit}) 与此合并无关，保留")

    if not tags_to_delete:
        log_break()
        log_success("没有需要删除的 alpha tags")
        if args.ci:
            _output_ci_result(0)
        return

    log_break()
    log_info(f"删除 {len(tags_to_delete)} 个 alpha tags...")

    # 删除本地 tag
    for tag in tags_to_delete:
        log_info(f"删除本地 tag: {tag}")
        try:
            delete_local_tag(tag)
        except Exception as e:
            log_warning(f"   删除本地 tag 失败: {e} (可能不存在)")

    # 删除远程 tag
    log_break()
    log_info("删除远程 tags...")
    deleted_count = 0
    for tag in tags_to_delete:
        log_info(f"删除远程 tag: {tag}")
        try:
            delete_remote_tag(tag)
            deleted_count += 1
        except Exception as e:
            log_warning(f"   删除远程 tag 失败: {e} (可能不存在或已删除)")

    log_break()
    log_success(f"清理完成: 删除了 {deleted_count} 个 alpha tag(s)")

    if args.ci:
        _output_ci_result(deleted_count)


def _output_ci_result(deleted_count: int) -> None:
    """输出 CI 模式结果到 GITHUB_OUTPUT"""
    output_file = os.environ.get('GITHUB_OUTPUT')
    if output_file:
        try:
            with open(output_file, 'a') as f:
                f.write(f'deleted_count={deleted_count}\n')
        except IOError as e:
            log_error(f"Failed to write to GITHUB_OUTPUT: {e}")


def main():
    """CLI 入口（可以直接运行此脚本）"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='cleanup',
        description='Cleanup alpha tags'
    )
    parser.add_argument('--merge-commit', required=True, help='Merge commit SHA')
    parser.add_argument('--version', required=True, help='Current version')
    parser.add_argument('--ci', action='store_true', help='CI mode')

    args = parser.parse_args()
    cleanup(args)


if __name__ == '__main__':
    main()

