"""Homebrew Formula 更新实现"""
import os
import re
import sys
import subprocess
from pathlib import Path
from typing import Optional

# 添加父目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_info, log_success, log_warning, log_error
from utils.git import run_git_command
import hashlib


def _get_repo() -> str:
    """获取仓库名称（owner/repo）"""
    repo = os.environ.get('GITHUB_REPOSITORY', '')
    if repo:
        return repo

    # 从 Git remote 获取
    try:
        result = run_git_command(['remote', 'get-url', 'origin'])
        url = result.stdout.strip()
        if 'github.com' in url:
            if url.startswith('https://'):
                parts = url.replace('https://github.com/', '').replace('.git', '').split('/')
            elif url.startswith('git@'):
                parts = url.replace('git@github.com:', '').replace('.git', '').split('/')
            else:
                parts = url.split('/')

            if len(parts) >= 2:
                return f"{parts[-2]}/{parts[-1].replace('.git', '')}"
    except Exception:
        pass

    return 'unknown/repo'


def _generate_from_template(template_path: str, output_path: str, version: str, tag: str, sha256: str) -> None:
    """从模板生成 Formula 文件"""
    template_content = Path(template_path).read_text()

    # 替换模板变量
    content = template_content.replace('{{VERSION}}', version)
    content = content.replace('{{TAG}}', tag)
    content = content.replace('{{SHA256}}', sha256)

    Path(output_path).write_text(content)


def _update_existing_file(formula_path: str, version: str, tag: str, repo: str, sha256: str) -> None:
    """更新现有 Formula 文件"""
    content = Path(formula_path).read_text()

    # 更新版本号
    version_regex = re.compile(r'version\s+"[^"]+"')
    content = version_regex.sub(f'version "{version}"', content)

    # 更新下载 URL
    url_pattern = re.escape(repo).replace('/', r'\/')
    url_regex = re.compile(fr'url\s+"https://github\.com/{url_pattern}/releases/download/[^"]+"')
    download_url = f"https://github.com/{repo}/releases/download/{tag}/workflow-{version}-x86_64-apple-darwin.tar.gz"
    content = url_regex.sub(f'url "{download_url}"', content)

    # 更新 SHA256
    sha256_regex = re.compile(r'sha256\s+"[^"]+"')
    content = sha256_regex.sub(f'sha256 "{sha256}"', content)

    Path(formula_path).write_text(content)


def _validate_formula(formula_path: str) -> None:
    """验证 Formula 文件语法"""
    log_info("🔍 Validating Formula file structure...")

    # 尝试使用 ruby -c 验证语法
    try:
        result = subprocess.run(
            ['ruby', '-c', formula_path],
            capture_output=True,
            text=True,
            check=True
        )
        log_success("Formula file syntax is valid")
    except subprocess.CalledProcessError as e:
        log_error("Formula file has syntax errors")
        log_error(e.stderr)
        sys.exit(1)
    except FileNotFoundError:
        log_warning("Ruby not found, skipping syntax validation")


def _git_operations(formula_path: str, version: str, tag: str, push: bool) -> None:
    """Git 操作（配置、提交、推送）"""
    # 配置 Git
    run_git_command(['config', 'user.name', 'github-actions[bot]'])
    run_git_command(['config', 'user.email', 'github-actions[bot]@users.noreply.github.com'])

    # 添加文件
    run_git_command(['add', formula_path])

    # 检查是否有更改
    result = run_git_command(['diff', '--staged', '--quiet'], check=False)
    if result.returncode == 0:
        log_info("No changes to commit. Formula file is already up to date.")
        return

    # 验证 Formula 文件格式（可选）
    try:
        subprocess.run(['brew', '--version'], capture_output=True, check=True)
        result = subprocess.run(
            ['brew', 'audit', '--strict', formula_path],
            capture_output=True,
            text=True,
            check=False
        )
        if result.returncode != 0:
            log_warning("brew audit failed, but continuing...")
            log_warning(result.stderr)
    except FileNotFoundError:
        pass  # brew 未安装，跳过

    # 提交更改
    commit_message = f"Update workflow to {tag}"
    run_git_command(['commit', '-m', commit_message])
    log_success(f"Committed changes: {commit_message}")

    # 推送到远程
    if push:
        result = run_git_command(['branch', '--show-current'])
        current_branch = result.stdout.strip()
        log_info(f"Pushing to branch: {current_branch}")

        run_git_command(['push', 'origin', current_branch])
        log_success(f"Successfully pushed to {current_branch} branch")


def update(args) -> None:
    """更新 Formula 文件"""
    formula_path = args.formula_path or "Formula/workflow.rb"
    template_path = args.template_path
    repo = args.repo or _get_repo()

    log_info(f"📝 Updating Homebrew Formula")
    log_info(f"   Version: {args.version}")
    log_info(f"   Tag: {args.tag}")
    log_info(f"   Formula path: {formula_path}")
    log_info(f"   Repo: {repo}")

    # 计算 SHA256（如果需要）
    sha256 = args.sha256
    if not sha256:
        # 尝试从下载的归档文件计算
        archive_name = f"workflow-{args.version}-x86_64-apple-darwin.tar.gz"
        if Path(archive_name).exists():
            log_info(f"Calculating SHA256 for {archive_name}...")
            sha256_hash = hashlib.sha256()
            with open(archive_name, 'rb') as f:
                for chunk in iter(lambda: f.read(4096), b''):
                    sha256_hash.update(chunk)
            sha256 = sha256_hash.hexdigest()
            log_info(f"SHA256: {sha256}")
        else:
            log_warning("Archive file not found, SHA256 will need to be set manually")
            sha256 = "PLACEHOLDER_SHA256"

    # 备份原始文件
    formula_file = Path(formula_path)
    if formula_file.exists():
        backup_path = f"{formula_path}.bak"
        import shutil
        shutil.copy(formula_path, backup_path)
        log_info(f"📝 Backed up Formula file to {backup_path}")

    # 从模板生成或更新现有文件
    if template_path and Path(template_path).exists():
        log_info("📝 Generating Formula file from template...")
        _generate_from_template(template_path, formula_path, args.version, args.tag, sha256)
        log_success("Formula file generated from template")
    else:
        if template_path:
            log_warning(f"Template file not found: {template_path}, updating existing file")
        log_info("📝 Updating version in Formula file...")
        _update_existing_file(formula_path, args.version, args.tag, repo, sha256)
        log_success("Formula file updated")

    # 验证文件结构
    _validate_formula(formula_path)

    # 显示生成的 Formula 文件
    log_info("\n📄 Generated Formula file:")
    log_info(f"--- {formula_path} ---")
    content = Path(formula_path).read_text()
    log_info(content)

    # Git 操作
    if args.commit:
        _git_operations(formula_path, args.version, args.tag, args.push)


def main():
    """CLI 入口（可以直接运行此脚本）"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='update',
        description='Update Homebrew Formula'
    )
    parser.add_argument('--version', required=True, help='Version number')
    parser.add_argument('--tag', required=True, help='Tag name')
    parser.add_argument('--formula-path', help='Formula file path (default: Formula/workflow.rb)')
    parser.add_argument('--template-path', help='Template file path')
    parser.add_argument('--repo', help='Repository (owner/repo)')
    parser.add_argument('--sha256', help='SHA256 checksum (will calculate if archive exists)')
    parser.add_argument('--commit', action='store_true', help='Commit changes')
    parser.add_argument('--push', action='store_true', help='Push to remote')

    args = parser.parse_args()
    update(args)


if __name__ == '__main__':
    main()

