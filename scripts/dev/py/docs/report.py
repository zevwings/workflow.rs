"""文档检查报告生成实现"""
import os
import sys
from datetime import datetime
from pathlib import Path

# 添加父目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_info


def _read_integrity_results() -> tuple[bool, int, int]:
    """从 GITHUB_OUTPUT 读取完整性检查结果"""
    # 尝试从环境变量读取（如果检查命令已经运行）
    passed = os.environ.get('docs_integrity_passed', 'true').lower() == 'true'
    missing = int(os.environ.get('docs_missing_architecture', '0'))
    invalid = int(os.environ.get('docs_invalid_timestamps', '0'))
    return passed, missing, invalid


def _read_links_results() -> tuple[bool, int]:
    """从 GITHUB_OUTPUT 读取链接检查结果"""
    # 尝试从环境变量读取（如果检查命令已经运行）
    passed = os.environ.get('docs_links_passed', 'true').lower() == 'true'
    broken = int(os.environ.get('docs_broken_links', '0'))
    return passed, broken


def _read_github_output() -> dict[str, str]:
    """从 GITHUB_OUTPUT 文件读取所有输出"""
    output_file = os.environ.get('GITHUB_OUTPUT')
    results = {}

    if output_file and Path(output_file).exists():
        try:
            content = Path(output_file).read_text()
            for line in content.split('\n'):
                if '=' in line:
                    key, value = line.split('=', 1)
                    results[key] = value
        except Exception:
            pass

    return results


def generate(args) -> str:
    """生成文档检查报告"""
    # 确定输出文件路径
    if args.output:
        report_file = args.output
    else:
        timestamp = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
        report_file = f"report/doc-check-{timestamp}.md"

    # 确保输出目录存在
    report_path = Path(report_file)
    report_path.parent.mkdir(parents=True, exist_ok=True)

    # 从 GITHUB_OUTPUT 读取结果
    github_output = _read_github_output()

    # 读取完整性检查结果
    integrity_passed = github_output.get('docs_integrity_passed', 'true').lower() == 'true'
    missing_architecture = int(github_output.get('docs_missing_architecture', '0'))
    invalid_timestamps = int(github_output.get('docs_invalid_timestamps', '0'))

    # 读取链接检查结果
    links_passed = github_output.get('docs_links_passed', 'true').lower() == 'true'
    broken_links = int(github_output.get('docs_broken_links', '0'))

    # 获取仓库信息
    repository = os.environ.get('GITHUB_REPOSITORY', 'unknown/repo')

    # 生成报告内容
    check_date = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    update_date = datetime.now().strftime("%Y-%m-%d")
    check_type = args.check_type or "定期审查"

    content = f"""# 文档检查报告

**检查日期**：{check_date}
**检查类型**：{check_type}

## 检查结果

### 文档链接检查

"""

    if links_passed:
        content += "✅ 已完成文档链接有效性检查，所有链接有效。\n\n"
    else:
        content += f"⚠️  已完成文档链接有效性检查，发现 {broken_links} 个无效链接。\n\n"

    content += """### 架构文档存在性检查

"""

    if integrity_passed and missing_architecture == 0:
        content += "✅ 已完成架构文档存在性检查，所有模块都有对应的架构文档。\n\n"
    else:
        content += f"⚠️  已完成架构文档存在性检查，发现 {missing_architecture} 个缺失的架构文档。\n\n"

    content += """### 文档时间戳格式检查

"""

    if integrity_passed and invalid_timestamps == 0:
        content += "✅ 已完成文档时间戳格式检查，所有文档都有正确的时间戳格式。\n\n"
    else:
        content += f"⚠️  已完成文档时间戳格式检查，发现 {invalid_timestamps} 个无效的时间戳格式。\n\n"

    # 问题汇总
    content += "## 问题汇总\n\n"
    total_issues = missing_architecture + invalid_timestamps + broken_links
    if total_issues == 0:
        content += "✅ 未发现任何问题，所有检查均通过。\n\n"
    else:
        content += f"发现 {total_issues} 个问题：\n"
        if missing_architecture > 0:
            content += f"- {missing_architecture} 个缺失的架构文档\n"
        if invalid_timestamps > 0:
            content += f"- {invalid_timestamps} 个无效的时间戳格式\n"
        if broken_links > 0:
            content += f"- {broken_links} 个无效的链接\n"
        content += "\n请查看上方的检查输出以了解详细问题。\n\n"

    # 改进建议
    content += """## 改进建议

1. 确保所有模块都有对应的架构文档
2. 确保所有文档都有正确的时间戳格式
3. 确保所有文档链接都有效

参考文档：
"""
    content += f"- [架构文档审查指南](https://github.com/{repository}/blob/main/docs/guidelines/development/references/review-architecture-consistency.md)\n"
    content += f"- [文档更新检查清单](https://github.com/{repository}/blob/main/docs/guidelines/development/code-review.md)\n\n"

    content += "---\n\n"
    content += f"**最后更新**: {update_date}\n"

    # 写入文件
    report_path.write_text(content)
    log_info(f"📄 Report generated: {report_file}")

    # 如果设置了 GITHUB_OUTPUT，输出报告文件路径
    output_file = os.environ.get('GITHUB_OUTPUT')
    if output_file:
        try:
            with open(output_file, 'a') as f:
                f.write(f'report_file={report_file}\n')
        except IOError as e:
            log_info(f"Failed to write to GITHUB_OUTPUT: {e}")

    return report_file


def main():
    """CLI 入口（可以直接运行此脚本）"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='report',
        description='Generate document check report'
    )
    parser.add_argument('--output', help='Output report file path')
    parser.add_argument('--check-type', help='Check type (default: 定期审查)')

    args = parser.parse_args()
    generate(args)


if __name__ == '__main__':
    main()

