"""文档完整性检查实现"""
import os
import re
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


def _check_architecture_docs() -> List[Tuple[str, Path]]:
    """检查架构文档存在性"""
    missing_docs = []

    # 检查 lib 层模块
    log_info("检查所有 lib 层模块...")
    lib_dir = Path("src/lib")
    if lib_dir.exists():
        for module_dir in lib_dir.iterdir():
            if module_dir.is_dir():
                module = module_dir.name
                doc_path = Path("docs/architecture") / f"{module}.md"
                if not doc_path.exists():
                    missing_docs.append((module, doc_path))
                    log_warning(f"  Missing: {doc_path} (module: {module})")
                else:
                    log_success(f"  {module} -> {doc_path}")

    log_break()

    # 检查 commands 层模块
    log_info("检查所有 commands 层模块...")
    cmd_dir = Path("src/commands")
    if cmd_dir.exists():
        for module_dir in cmd_dir.iterdir():
            if module_dir.is_dir():
                module = module_dir.name
                doc_path = Path("docs/architecture") / f"{module}.md"
                if not doc_path.exists():
                    missing_docs.append((module, doc_path))
                    log_warning(f"  Missing: {doc_path} (module: {module})")
                else:
                    log_success(f"  {module} -> {doc_path}")

    return missing_docs


def _check_timestamp_format() -> List[Path]:
    """检查文档时间戳格式"""
    invalid_files = []
    timestamp_pattern = re.compile(r'\*\*最后更新\*\*: \d{4}-\d{2}-\d{2}')

    docs_dir = Path("docs")
    if not docs_dir.exists():
        return invalid_files

    # 查找所有 markdown 文件
    doc_files = []
    for md_file in docs_dir.rglob("*.md"):
        if not _should_skip_file(md_file):
            doc_files.append(md_file)

    checked_count = 0
    for file_path in doc_files:
        checked_count += 1
        try:
            content = file_path.read_text()
            lines = content.split('\n')

            # 检查最后5行
            last_lines = lines[-5:] if len(lines) >= 5 else lines

            has_valid_timestamp = any(timestamp_pattern.search(line) for line in last_lines)

            if not has_valid_timestamp:
                invalid_files.append(file_path)
        except Exception as e:
            log_warning(f"Failed to read {file_path}: {e}")

    log_info(f"检查了 {checked_count} 个文档")
    return invalid_files


def _output_ci_result(missing_architecture: int, invalid_timestamps: int, has_issues: bool) -> None:
    """输出 CI 模式结果到 GITHUB_OUTPUT"""
    output_file = os.environ.get('GITHUB_OUTPUT')
    if output_file:
        try:
            with open(output_file, 'a') as f:
                f.write(f'docs_integrity_passed={str(not has_issues).lower()}\n')
                f.write(f'docs_missing_architecture={missing_architecture}\n')
                f.write(f'docs_invalid_timestamps={invalid_timestamps}\n')
        except IOError as e:
            log_error(f"Failed to write to GITHUB_OUTPUT: {e}")


def check(args) -> None:
    """检查文档完整性"""
    log_break('=')
    log_info("文档完整性检查")
    log_break('=')
    log_break()

    has_issues = False

    # 如果未指定具体检查项，则检查所有项
    check_architecture = args.architecture if hasattr(args, 'architecture') else True
    check_timestamps = args.timestamps if hasattr(args, 'timestamps') else True

    missing_architecture_docs = []
    invalid_timestamps = []

    # 检查架构文档存在性
    if check_architecture:
        log_info("📝 检查架构文档存在性...")
        missing_architecture_docs = _check_architecture_docs()

        if missing_architecture_docs:
            has_issues = True
            log_break()
            log_info(f"📋 发现 {len(missing_architecture_docs)} 个缺失的架构文档:")
            for module, doc_path in missing_architecture_docs:
                log_warning(f"  模块 '{module}' 缺少架构文档: {doc_path}")
        else:
            log_success("所有模块都有架构文档")
        log_break()

    # 检查文档时间戳格式
    if check_timestamps:
        log_info("📅 检查文档时间戳格式...")
        invalid_timestamps = _check_timestamp_format()

        if invalid_timestamps:
            has_issues = True
            log_break()
            log_info(f"📋 发现 {len(invalid_timestamps)} 个文档的时间戳格式无效:")
            for file_path in invalid_timestamps:
                log_warning(f"  无效的时间戳格式: {file_path}")
        else:
            log_success("所有文档都有有效的时间戳格式")
        log_break()

    # CI 模式：输出到 GITHUB_OUTPUT
    if args.ci:
        _output_ci_result(len(missing_architecture_docs), len(invalid_timestamps), has_issues)
        return

    # 本地模式：如果有问题则退出
    if has_issues:
        sys.exit(1)

    log_success("文档完整性检查完成")


def main():
    """CLI 入口（可以直接运行此脚本）"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='integrity',
        description='Check document integrity'
    )
    parser.add_argument('--architecture', action='store_true', help='Check architecture docs')
    parser.add_argument('--timestamps', action='store_true', help='Check timestamp format')
    parser.add_argument('--ci', action='store_true', help='CI mode')

    args = parser.parse_args()
    check(args)


if __name__ == '__main__':
    main()

