"""PR 创建实现"""
import sys
from pathlib import Path
from typing import Optional

# 添加父目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_info, log_success, log_warning, log_error, log_break
from utils.git import checkout_branch, add_files, commit, push_branch
from utils.github import create_pull_request, _get_owner_and_repo, _find_existing_pr


def _output_ci_result(pr_number: Optional[str], pr_url: Optional[str]) -> None:
    """输出 CI 模式结果到 GITHUB_OUTPUT"""
    import os

    output_file = os.environ.get('GITHUB_OUTPUT')
    if output_file:
        try:
            with open(output_file, 'a') as f:
                if pr_number:
                    f.write(f'pr_number={pr_number}\n')
                if pr_url:
                    f.write(f'pr_url={pr_url}\n')
        except IOError as e:
            log_error(f"Failed to write to GITHUB_OUTPUT: {e}")


def create(args) -> None:
    """创建版本更新 PR"""
    log_break('=')
    log_info("创建版本更新 PR")
    log_break('=')
    log_break()

    version = args.version
    branch_name = args.branch or f"bump-version-{version}"
    base_branch = args.base or "master"

    log_info(f"版本: {version}")
    log_info(f"分支: {branch_name}")
    log_break()

    # 第一步：创建新分支
    log_info(f"创建分支 {branch_name}...")
    checkout_branch(branch_name, create=True)
    log_success("分支创建/切换成功")

    # 第二步：添加文件
    log_break()
    log_info("暂存 Cargo.toml 和 Cargo.lock...")
    add_files(['Cargo.toml', 'Cargo.lock'])
    log_success("文件已暂存")

    # 第三步：提交更改
    log_break()
    log_info("提交更改...")
    commit_message = f"chore: bump version to {version}"
    commit(commit_message)
    log_success("提交成功")

    # 第四步：推送到新分支
    log_break()
    log_info("推送分支到远程...")
    push_branch(branch_name, force=False)
    log_success("分支推送成功")

    # 第五步：创建 Pull Request
    log_break()
    log_info("创建 Pull Request...")
    title = f"chore: bump version to {version}"
    body = (
        f"Automated version bump to {version}\n\n"
        f"This PR was created automatically by the release workflow.\n\n"
        f"**Changes:**\n"
        f"- Updated version in Cargo.toml to {version}\n"
        f"- Updated version in Cargo.lock to {version}"
    )

    try:
        pr_url, pr_number = create_pull_request(title, body, branch_name, base_branch)
        log_success("PR 创建成功")
        log_info(f"   URL: {pr_url}")
        log_info(f"   PR 编号: {pr_number}")

        if args.ci:
            _output_ci_result(pr_number, pr_url)
    except Exception as e:
        # 如果是 422 错误（已存在 PR），尝试查找现有的 PR
        if '422' in str(e):
            log_warning("PR 可能已存在，尝试查找现有 PR...")
            try:
                owner, repo = _get_owner_and_repo()
                pr_url, pr_number = _find_existing_pr(f"{owner}:{branch_name}", base_branch)
                log_success("找到现有 PR")
                log_info(f"   URL: {pr_url}")
                log_info(f"   PR 编号: {pr_number}")
                if args.ci:
                    _output_ci_result(pr_number, pr_url)
                return
            except Exception as find_error:
                log_warning(f"查找现有 PR 失败: {find_error}")
        log_error(f"Failed to create PR: {e}")
        sys.exit(1)

    log_break()
    log_success("PR 创建流程完成")
    log_break()


def main():
    """CLI 入口（可以直接运行此脚本）"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='create',
        description='Create version bump PR'
    )
    parser.add_argument('--version', required=True, help='Version number')
    parser.add_argument('--branch', help='Branch name (default: bump-version-{version})')
    parser.add_argument('--base', help='Base branch (default: master)')
    parser.add_argument('--ci', action='store_true', help='CI mode')

    args = parser.parse_args()
    create(args)


if __name__ == '__main__':
    main()

