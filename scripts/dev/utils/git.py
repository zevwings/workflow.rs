"""Git 操作工具（使用 subprocess，零依赖）"""
import subprocess
import sys
from pathlib import Path
from typing import Optional, List, Tuple

# 添加父目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_error


def run_git_command(args: List[str], cwd: Optional[str] = None, check: bool = True) -> subprocess.CompletedProcess:
    """运行 Git 命令"""
    try:
        result = subprocess.run(
            ['git'] + args,
            capture_output=True,
            text=True,
            check=check,
            cwd=cwd
        )
        return result
    except subprocess.CalledProcessError as e:
        log_error(f"Git command failed: git {' '.join(args)}")
        log_error(f"Error: {e.stderr}")
        raise
    except FileNotFoundError:
        log_error("Git not found. Please install Git.")
        sys.exit(1)


def get_current_branch() -> Optional[str]:
    """获取当前分支名"""
    try:
        result = run_git_command(['branch', '--show-current'], check=False)
        if result.returncode == 0:
            branch = result.stdout.strip()
            return branch if branch else None
    except Exception:
        pass
    return None


def get_last_commit_sha() -> Optional[str]:
    """获取最后一次提交的 SHA"""
    try:
        result = run_git_command(['rev-parse', 'HEAD'])
        return result.stdout.strip()
    except Exception:
        return None


def list_tags() -> List[str]:
    """列出所有本地 tag"""
    try:
        result = run_git_command(['tag', '--list'])
        tags = [tag.strip() for tag in result.stdout.strip().split('\n') if tag.strip()]
        return tags
    except Exception:
        return []


def get_tags_at_commit(commit_sha: str) -> List[str]:
    """获取指定 commit 的所有 tag"""
    try:
        result = run_git_command(['tag', '--points-at', commit_sha])
        tags = [tag.strip() for tag in result.stdout.strip().split('\n') if tag.strip()]
        return tags
    except Exception:
        return []


def get_commits_between(from_ref: str, to_ref: str) -> List[str]:
    """获取两个引用之间的提交消息"""
    try:
        result = run_git_command(['log', '--format=%s', f'{from_ref}..{to_ref}'])
        commits = [msg.strip() for msg in result.stdout.strip().split('\n') if msg.strip()]
        return commits
    except Exception:
        return []


def get_branch_commits(limit: int = 10) -> List[str]:
    """获取当前分支的最近提交消息"""
    try:
        result = run_git_command(['log', '--format=%s', '-n', str(limit)])
        commits = [msg.strip() for msg in result.stdout.strip().split('\n') if msg.strip()]
        return commits
    except Exception:
        return []


def tag_exists(tag_name: str) -> Tuple[bool, bool]:
    """检查 tag 是否存在（本地，远程）"""
    local_exists = False
    remote_exists = False

    try:
        # 检查本地
        result = run_git_command(['rev-parse', tag_name], check=False)
        local_exists = result.returncode == 0

        # 检查远程
        result = run_git_command(['ls-remote', '--tags', 'origin', tag_name], check=False)
        remote_exists = tag_name in result.stdout
    except Exception:
        pass

    return local_exists, remote_exists


def create_tag(tag_name: str, commit_sha: Optional[str] = None) -> None:
    """创建 tag"""
    args = ['tag', tag_name]
    if commit_sha:
        args.append(commit_sha)
    run_git_command(args)


def delete_local_tag(tag_name: str) -> None:
    """删除本地 tag"""
    run_git_command(['tag', '-d', tag_name])


def delete_remote_tag(tag_name: str) -> None:
    """删除远程 tag"""
    run_git_command(['push', 'origin', '--delete', tag_name])


def push_tag(tag_name: str) -> None:
    """推送 tag 到远程"""
    run_git_command(['push', 'origin', tag_name])


def checkout_branch(branch_name: str, create: bool = True) -> None:
    """切换或创建分支"""
    if create:
        # 尝试创建并切换
        result = run_git_command(['checkout', '-b', branch_name], check=False)
        if result.returncode != 0:
            # 如果创建失败，尝试切换
            run_git_command(['checkout', branch_name])
    else:
        run_git_command(['checkout', branch_name])


def add_files(files: List[str]) -> None:
    """暂存文件"""
    run_git_command(['add'] + files)


def commit(message: str) -> None:
    """提交更改"""
    run_git_command(['commit', '-m', message])


def push_branch(branch_name: str, force: bool = False) -> None:
    """推送分支"""
    args = ['push', 'origin', branch_name]
    if force:
        args.append('--force')
    run_git_command(args)

