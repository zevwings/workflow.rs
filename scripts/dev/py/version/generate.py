"""版本号生成实现"""
import os
import re
import sys
from datetime import datetime
from pathlib import Path
from typing import Optional, Tuple

# 添加父目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_info, log_success, log_warning, log_error
from utils.git import (
    get_last_commit_sha, list_tags, get_tags_at_commit,
    get_commits_between, get_branch_commits, run_git_command
)


class VersionIncrementType:
    """版本递增类型"""
    MAJOR = "major"
    MINOR = "minor"
    PATCH = "patch"


def parse_version(version_str: str) -> Tuple[int, int, int]:
    """解析版本号字符串为 (major, minor, patch)"""
    parts = version_str.split('.')
    major = int(parts[0]) if len(parts) > 0 and parts[0].isdigit() else 0
    minor = int(parts[1]) if len(parts) > 1 and parts[1].isdigit() else 0
    patch = int(parts[2]) if len(parts) > 2 and parts[2].isdigit() else 0
    return major, minor, patch


def get_latest_version() -> Tuple[str, str]:
    """获取最新版本号

    返回: (version, tag) 例如 ("1.6.0", "v1.6.0")
    """
    tags = list_tags()
    version_regex = re.compile(r'^v(\d+)\.(\d+)\.(\d+)$')

    version_tags = []
    for tag in tags:
        match = version_regex.match(tag)
        if match:
            major, minor, patch = int(match.group(1)), int(match.group(2)), int(match.group(3))
            version_tags.append((tag, (major, minor, patch)))

    if version_tags:
        # 按版本号排序（从高到低）
        version_tags.sort(key=lambda x: x[1], reverse=True)
        latest_tag = version_tags[0][0]
        version = latest_tag.lstrip('v')
        log_success(f"Latest standard version from git tags: {latest_tag} ({version})")
        return version, latest_tag

    # 如果没有找到标准版本 tag，使用默认版本
    version = "0.0.0"
    tag = "v0.0.0"
    log_warning(f"No standard version tag found, using default: {version}")
    return version, tag


def determine_version_increment(commits: list[str], current_patch: int) -> str:
    """确定版本递增类型

    优先级：BREAKING CHANGE > patch >= 9 > feat: > 其他
    """
    has_breaking = False
    has_feat = False

    for commit_msg in commits:
        # 检查 BREAKING CHANGE 或 BREAKING:
        if "BREAKING CHANGE" in commit_msg or "BREAKING:" in commit_msg:
            has_breaking = True

        # 检查 ! 标记（BREAKING CHANGE 的简写）
        if '!' in commit_msg and ':' in commit_msg:
            colon_pos = commit_msg.find(':')
            if colon_pos > 0:
                before_colon = commit_msg[:colon_pos]
                if before_colon.endswith('!'):
                    has_breaking = True

        # 检查 feat: 或 feature:
        if commit_msg.startswith("feat:") or commit_msg.startswith("feature:"):
            has_feat = True

    if has_breaking:
        return VersionIncrementType.MAJOR

    # 规则：如果 patch 版本达到 9，自动递增 minor 版本
    if current_patch >= 9:
        log_warning("Patch version reached 9, incrementing MINOR version")
        return VersionIncrementType.MINOR

    if has_feat:
        return VersionIncrementType.MINOR

    return VersionIncrementType.PATCH


def generate_master_version(latest_version: str, latest_tag: str) -> Tuple[str, str, bool]:
    """生成 master 分支版本号"""
    major, minor, patch = parse_version(latest_version)

    # 检查当前 commit 是否已经有标准版本 tag 指向它
    current_sha = get_last_commit_sha()
    if current_sha:
        tags_at_head = get_tags_at_commit(current_sha)
        version_regex = re.compile(r'^v(\d+)\.(\d+)\.(\d+)$')

        for tag in tags_at_head:
            if version_regex.match(tag):
                version = tag.lstrip('v')
                log_success(f"Found existing tag {tag} on current commit, reusing it")
                return version, tag, False

    # 获取提交消息
    if latest_tag and run_git_command(['rev-parse', latest_tag], check=False).returncode == 0:
        commits = get_commits_between(latest_tag, 'HEAD')
    else:
        commits = get_branch_commits(10)

    # 确定版本递增类型
    increment_type = determine_version_increment(commits, patch)

    # 应用版本递增
    if increment_type == VersionIncrementType.MAJOR:
        major += 1
        minor = 0
        patch = 0
        log_info("🔴 Detected BREAKING CHANGE, incrementing MAJOR version")
    elif increment_type == VersionIncrementType.MINOR:
        minor += 1
        patch = 0
        log_info("🟢 Detected feat: commit, incrementing MINOR version")
    else:
        patch += 1
        log_info("🔵 No feat: or BREAKING CHANGE detected, incrementing PATCH version")

    version = f"{major}.{minor}.{patch}"
    tag = f"v{version}"

    log_success(f"Version increment type: {increment_type}")
    log_success(f"Generated version {version} ({tag}) based on Conventional Commits")

    return version, tag, True


def generate_prerelease_version(latest_version: str, latest_tag: str) -> Tuple[str, str, bool]:
    """生成预发布版本号"""
    major, minor, patch = parse_version(latest_version)

    # 获取提交消息
    if latest_tag and run_git_command(['rev-parse', latest_tag], check=False).returncode == 0:
        commits = get_commits_between(latest_tag, 'HEAD')
    else:
        commits = get_branch_commits(10)

    # 确定版本递增类型
    increment_type = determine_version_increment(commits, patch)

    # 应用版本递增
    if increment_type == VersionIncrementType.MAJOR:
        major += 1
        minor = 0
        patch = 0
        log_info("🔴 Detected BREAKING CHANGE, incrementing MAJOR version")
    elif increment_type == VersionIncrementType.MINOR:
        minor += 1
        patch = 0
        log_info("🟢 Detected feat: commit, incrementing MINOR version")
    else:
        patch += 1
        log_info("🔵 No feat: or BREAKING CHANGE detected, incrementing PATCH version")

    base_version = f"{major}.{minor}.{patch}"

    # 使用时间戳格式确保唯一性：YYYYMMDDHHmmssSSS
    now = datetime.utcnow()
    timestamp = now.strftime("%Y%m%d%H%M%S") + f"{now.microsecond // 1000:03d}"
    version = f"{base_version}.alpha-{timestamp}"
    tag = f"v{version}"

    log_success(f"Non-master branch: Generated pre-release version {version} ({tag})")
    log_info("   Timestamp format: YYYYMMDDHHmmssSSS")
    log_info("   Example: v1.6.1.alpha-20251216101712000")

    return version, tag, False


def update_cargo_files(version: str) -> None:
    """更新 Cargo.toml 和 Cargo.lock"""
    cargo_toml_path = Path("Cargo.toml")
    if not cargo_toml_path.exists():
        log_error("Cargo.toml not found")
        sys.exit(1)

    # 读取 Cargo.toml
    content = cargo_toml_path.read_text()

    # 更新版本号
    version_regex = re.compile(r'version\s*=\s*"[^"]+"')
    updated_content = version_regex.sub(f'version = "{version}"', content)

    # 写入文件
    cargo_toml_path.write_text(updated_content)
    log_success(f"Updated Cargo.toml to version {version}")

    # 运行 cargo update 更新 Cargo.lock
    try:
        run_git_command(['cargo', 'update', '--workspace'])
        log_success("Updated Cargo.lock")
    except Exception as e:
        log_error(f"Failed to update Cargo.lock: {e}")
        sys.exit(1)


def output_github_actions(version: str, tag: str, needs_increment: bool) -> None:
    """输出到 GitHub Actions GITHUB_OUTPUT"""
    output_file = os.environ.get('GITHUB_OUTPUT')
    if not output_file:
        log_error("GITHUB_OUTPUT not set")
        sys.exit(1)

    try:
        with open(output_file, 'a') as f:
            f.write(f'version={version}\n')
            f.write(f'tag={tag}\n')
            f.write(f'needs_increment={str(needs_increment).lower()}\n')
        log_success("Output version info to GITHUB_OUTPUT")
    except IOError as e:
        log_error(f"Failed to write to GITHUB_OUTPUT: {e}")
        sys.exit(1)


def generate(args) -> Tuple[str, str, bool]:
    """生成版本号"""
    # 获取最新版本
    latest_version, latest_tag = get_latest_version()
    log_info("📋 Version generation inputs:")
    log_info(f"   LATEST_VERSION: {latest_version}")
    log_info(f"   IS_MASTER: {args.master}")

    # 生成版本号
    if args.master:
        version, tag, needs_increment = generate_master_version(latest_version, latest_tag)
    else:
        version, tag, needs_increment = generate_prerelease_version(latest_version, latest_tag)

    log_success(f"Generated version {version} ({tag})")

    # 更新 Cargo 文件
    if args.update:
        update_cargo_files(version)

    # 输出到 GitHub Actions
    if args.ci:
        output_github_actions(version, tag, needs_increment)

    return version, tag, needs_increment


def main():
    """CLI 入口（可以直接运行此脚本）"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='generate',
        description='Generate version number'
    )
    parser.add_argument('--master', action='store_true', help='Generate version for master branch')
    parser.add_argument('--update', action='store_true', help='Update Cargo.toml and Cargo.lock')
    parser.add_argument('--ci', action='store_true', help='CI mode (output to GITHUB_OUTPUT)')

    args = parser.parse_args()
    generate(args)


if __name__ == '__main__':
    main()

