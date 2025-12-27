"""文档链接检查实现"""
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import List, Tuple

# 添加父目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_info, log_success, log_warning, log_error, log_break


def _should_skip_file(path: Path) -> bool:
    """判断是否应该跳过文件"""
    path_str = str(path)
    return '/templates/' in path_str or path.name == 'README.md'


def _resolve_link_path(file_path: Path, link: str) -> Path:
    """解析链接路径"""
    # 移除锚点部分（#anchor）
    link_without_anchor = link.split('#')[0]

    if link_without_anchor.startswith('/'):
        # 绝对路径（从项目根目录开始）
        return Path(link_without_anchor.lstrip('/'))
    else:
        # 相对路径
        file_dir = file_path.parent
        return file_dir / link_without_anchor


def _check_internal_links() -> Tuple[List[Tuple[Path, str, Path]], int]:
    """检查内部链接"""
    broken_links = []
    link_count = 0
    link_pattern = re.compile(r'\]\(([^)]+)\)')

    docs_dir = Path("docs")
    if not docs_dir.exists():
        return broken_links, link_count

    # 查找所有 markdown 文件
    doc_files = []
    for md_file in docs_dir.rglob("*.md"):
        if not _should_skip_file(md_file):
            doc_files.append(md_file)

    for file_path in doc_files:
        try:
            content = file_path.read_text()

            # 提取所有链接
            for match in link_pattern.finditer(content):
                link = match.group(1)
                link_count += 1

                # 跳过空链接
                if not link:
                    continue

                # 跳过外部链接
                if link.startswith(('http://', 'https://')):
                    continue

                # 跳过锚点链接（只检查文件存在性）
                if link.startswith('#'):
                    continue

                # 解析链接路径
                target_file = _resolve_link_path(file_path, link)

                # 检查文件是否存在
                if not target_file.exists():
                    broken_links.append((file_path, link, target_file))
        except Exception as e:
            log_warning(f"Failed to process {file_path}: {e}")

    return broken_links, link_count


def _check_external_links() -> bool:
    """检查外部链接（使用 lychee）"""
    try:
        result = subprocess.run(
            ['lychee', '--output', 'json', 'docs/'],
            capture_output=True,
            text=True,
            check=False
        )

        if result.returncode == 0:
            log_success("所有外部链接有效")
            return True
        else:
            log_warning("发现无效的外部链接")
            log_info(result.stdout)
            return False
    except FileNotFoundError:
        log_warning("lychee 未安装，跳过外部链接检查")
        log_info("   安装方法: cargo install lychee")
        return True
    except Exception as e:
        log_warning(f"外部链接检查失败: {e}")
        return True


def _output_ci_result(broken_links: List[Tuple[Path, str, Path]]) -> None:
    """输出 CI 模式结果到 GITHUB_OUTPUT"""
    output_file = os.environ.get('GITHUB_OUTPUT')
    if output_file:
        try:
            with open(output_file, 'a') as f:
                f.write(f'docs_links_passed={str(len(broken_links) == 0).lower()}\n')
                f.write(f'docs_broken_links={len(broken_links)}\n')
        except IOError as e:
            log_error(f"Failed to write to GITHUB_OUTPUT: {e}")


def check(args) -> None:
    """检查文档链接"""
    log_break('=')
    log_info("文档链接有效性检查")
    log_break('=')
    log_break()

    broken_links = []
    internal_link_count = 0

    # 检查内部链接
    log_info("📋 检查内部链接...")
    broken_links, internal_link_count = _check_internal_links()

    # 显示断链信息
    if broken_links:
        log_break()
        log_info("发现的断链:")
        for file_path, link, target_file in broken_links:
            log_error(f"  断链: {file_path} -> {link} (目标文件: {target_file})")
        log_break()

    log_info(f"检查了 {internal_link_count} 个内部链接")
    if not broken_links:
        log_success("所有内部链接有效")
    else:
        log_error(f"发现 {len(broken_links)} 个断链")

    # 检查外部链接（如果指定）
    if args.external:
        log_break()
        log_info("📋 检查外部链接...")
        _check_external_links()
    else:
        log_break()
        log_info("跳过外部链接检查（使用 --external 启用）")
        log_info("   安装方法: cargo install lychee")

    log_break()
    log_success("链接检查完成")

    # CI 模式：输出到 GITHUB_OUTPUT
    if args.ci:
        _output_ci_result(broken_links)
        return

    # 本地模式：如果有断链则退出
    if broken_links:
        sys.exit(1)


def main():
    """CLI 入口（可以直接运行此脚本）"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='links',
        description='Check document links'
    )
    parser.add_argument('--external', action='store_true', help='Check external links')
    parser.add_argument('--ci', action='store_true', help='CI mode')

    args = parser.parse_args()
    check(args)


if __name__ == '__main__':
    main()

