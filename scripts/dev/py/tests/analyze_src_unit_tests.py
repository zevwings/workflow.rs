"""分析 src/ 目录下的单元测试注释完整性"""
import re
import sys
from pathlib import Path
from typing import Dict, List, Tuple
from dataclasses import dataclass

# 添加父目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_info, log_success, log_warning, log_error, log_break


@dataclass
class TestFunction:
    """测试函数信息"""
    name: str
    line_number: int
    has_doc_comment: bool
    doc_comment: str
    has_detailed_comment: bool


@dataclass
class FileAnalysis:
    """文件分析结果"""
    file_path: str
    relative_path: str
    test_functions: List[TestFunction]
    total_tests: int
    commented_tests: int
    detailed_commented_tests: int


def is_detailed_comment(doc_comment: str) -> bool:
    """判断注释是否是详细的中文注释（包含测试目的、场景等）"""
    if not doc_comment:
        return False

    # 检查是否包含"测试目的"、"测试场景"、"预期结果"等关键词
    detailed_keywords = [
        "测试目的",
        "测试场景",
        "预期结果",
        "## 测试目的",
        "## 测试场景",
        "## 预期结果"
    ]

    # 检查是否有至少一个关键词
    for keyword in detailed_keywords:
        if keyword in doc_comment:
            return True

    # 如果注释很长（超过3行），也认为是详细注释
    lines = doc_comment.strip().split('\n')
    if len(lines) >= 3:
        return True

    return False


def extract_doc_comment(lines: List[str], test_line_idx: int) -> Tuple[bool, str]:
    """提取测试函数的文档注释"""
    doc_lines = []
    idx = test_line_idx - 1
    in_multiline_attr = False

    # 向上查找文档注释
    while idx >= 0:
        line = lines[idx]
        stripped = line.strip()

        # 检查是否在多行属性中
        if in_multiline_attr:
            # 检查是否找到属性开始（#[ 开头）
            if stripped.startswith('#['):
                in_multiline_attr = False
                idx -= 1
                continue
            # 继续跳过多行属性的内容
            idx -= 1
            continue

        # 跳过属性（如 #[test], #[tokio::test], #[rstest] 等）
        if stripped.startswith('#['):
            # 检查是否是单行属性（以 ] 结尾）
            if stripped.endswith(']'):
                idx -= 1
                continue

            # 多行属性：标记状态，继续向上跳过
            in_multiline_attr = True
            idx -= 1
            continue

        # 检查是否是多行属性的结束部分（包含 )]）
        # 注意：)] 可能在行中间，后面可能有注释（如 )] // 注释）
        if ')]' in stripped:
            # 是多行属性的结束部分，标记为在多行属性中
            in_multiline_attr = True
            idx -= 1
            continue

        # 如果是文档注释
        if stripped.startswith('///'):
            doc_lines.insert(0, stripped[3:].strip())
            idx -= 1
        else:
            # 遇到非注释行，停止
            break

    doc_comment = '\n'.join(doc_lines)
    has_doc = len(doc_lines) > 0

    return has_doc, doc_comment


def find_test_functions(file_path: Path) -> List[TestFunction]:
    """查找文件中的所有测试函数"""
    try:
        content = file_path.read_text(encoding='utf-8')
        lines = content.split('\n')
    except Exception as e:
        log_error(f"读取文件失败 {file_path}: {e}")
        return []

    test_functions = []
    in_test_module = False

    # 正则表达式匹配测试函数
    test_fn_pattern = re.compile(r'^\s*fn\s+(test_\w+)\s*\(')
    test_mod_pattern = re.compile(r'^\s*mod\s+tests\s*\{')
    cfg_test_pattern = re.compile(r'^\s*#\[cfg\(test\)\]')
    test_attr_pattern = re.compile(r'^\s*#\[test\]')

    i = 0
    while i < len(lines):
        line = lines[i]

        # 检查是否进入测试模块
        if cfg_test_pattern.match(line):
            # 下一行可能是 mod tests
            if i + 1 < len(lines) and test_mod_pattern.match(lines[i + 1]):
                in_test_module = True
                i += 2
                continue

        # 检查是否是测试函数
        test_fn_match = test_fn_pattern.match(line)
        if test_fn_match:
            fn_name = test_fn_match.group(1)

            # 提取文档注释
            has_doc, doc_comment = extract_doc_comment(lines, i)
            has_detailed = is_detailed_comment(doc_comment)

            test_func = TestFunction(
                name=fn_name,
                line_number=i + 1,
                has_doc_comment=has_doc,
                doc_comment=doc_comment,
                has_detailed_comment=has_detailed
            )
            test_functions.append(test_func)

        i += 1

    return test_functions


def analyze_src_tests(src_dir: Path) -> List[FileAnalysis]:
    """分析 src/ 目录下的所有测试文件"""
    analyses = []

    # 查找所有 .rs 文件
    for rs_file in src_dir.rglob('*.rs'):
        # 跳过 main.rs 和 lib.rs
        if rs_file.name in ['main.rs', 'lib.rs']:
            continue

        # 查找测试函数
        test_functions = find_test_functions(rs_file)

        # 如果没有测试函数，跳过
        if not test_functions:
            continue

        # 统计注释情况
        total_tests = len(test_functions)
        commented_tests = sum(1 for tf in test_functions if tf.has_doc_comment)
        detailed_commented_tests = sum(1 for tf in test_functions if tf.has_detailed_comment)

        relative_path = rs_file.relative_to(src_dir.parent)

        analysis = FileAnalysis(
            file_path=str(rs_file),
            relative_path=str(relative_path),
            test_functions=test_functions,
            total_tests=total_tests,
            commented_tests=commented_tests,
            detailed_commented_tests=detailed_commented_tests
        )
        analyses.append(analysis)

    return analyses


def generate_report(analyses: List[FileAnalysis], output_path: Path) -> None:
    """生成分析报告"""
    from datetime import datetime

    # 计算总体统计
    total_files = len(analyses)
    total_tests = sum(a.total_tests for a in analyses)
    total_commented = sum(a.commented_tests for a in analyses)
    total_detailed = sum(a.detailed_commented_tests for a in analyses)

    # 按完成度分类
    no_comment_files = [a for a in analyses if a.commented_tests == 0]
    partial_comment_files = [a for a in analyses if 0 < a.commented_tests < a.total_tests]
    full_comment_files = [a for a in analyses if a.commented_tests == a.total_tests]

    # 按详细注释分类
    no_detailed_files = [a for a in analyses if a.detailed_commented_tests == 0]
    partial_detailed_files = [a for a in analyses if 0 < a.detailed_commented_tests < a.total_tests]
    full_detailed_files = [a for a in analyses if a.detailed_commented_tests == a.total_tests]

    # 生成报告
    report = f"""# src/ 目录单元测试注释分析报告

**生成时间**: {datetime.now().strftime("%Y-%m-%d %H:%M:%S")}
**分析范围**: src/ 目录下的所有单元测试

---

## 📊 总体统计

### 基础统计

- **总文件数**: {total_files} 个文件包含单元测试
- **总测试数**: {total_tests} 个测试函数
- **有注释的测试**: {total_commented} 个 ({total_commented/total_tests*100:.1f}%)
- **有详细注释的测试**: {total_detailed} 个 ({total_detailed/total_tests*100:.1f}%)
- **缺失注释的测试**: {total_tests - total_commented} 个 ({(total_tests - total_commented)/total_tests*100:.1f}%)

### 文件分类（基础注释）

| 分类 | 文件数 | 测试数 | 占比 |
|-----|-------|-------|-----|
| 完全无注释 (0%) | {len(no_comment_files)} | {sum(a.total_tests for a in no_comment_files)} | {len(no_comment_files)/total_files*100:.1f}% |
| 部分有注释 (1-99%) | {len(partial_comment_files)} | {sum(a.total_tests for a in partial_comment_files)} | {len(partial_comment_files)/total_files*100:.1f}% |
| 完全有注释 (100%) | {len(full_comment_files)} | {sum(a.total_tests for a in full_comment_files)} | {len(full_comment_files)/total_files*100:.1f}% |

### 文件分类（详细注释）

| 分类 | 文件数 | 测试数 | 占比 |
|-----|-------|-------|-----|
| 完全无详细注释 (0%) | {len(no_detailed_files)} | {sum(a.total_tests for a in no_detailed_files)} | {len(no_detailed_files)/total_files*100:.1f}% |
| 部分有详细注释 (1-99%) | {len(partial_detailed_files)} | {sum(a.total_tests for a in partial_detailed_files)} | {len(partial_detailed_files)/total_files*100:.1f}% |
| 完全有详细注释 (100%) | {len(full_detailed_files)} | {sum(a.total_tests for a in full_detailed_files)} | {len(full_detailed_files)/total_files*100:.1f}% |

---

## 🔴 优先级 1：完全缺失注释的文件

这些文件的测试函数完全没有文档注释，需要立即补充。

"""

    # 详细列出完全缺失注释的文件
    for idx, analysis in enumerate(sorted(no_comment_files, key=lambda a: a.total_tests, reverse=True), 1):
        report += f"""
### {idx}. `{analysis.relative_path}` - {analysis.total_tests} 个测试

| 测试函数 | 行号 | 状态 |
|---------|-----|------|
"""
        for tf in analysis.test_functions[:10]:  # 只显示前10个
            report += f"| `{tf.name}()` | {tf.line_number} | ❌ 无注释 |\n"

        if len(analysis.test_functions) > 10:
            report += f"\n... 还有 {len(analysis.test_functions) - 10} 个测试函数\n"

    # 部分有注释的文件
    if partial_comment_files:
        report += "\n---\n\n## 🟡 优先级 2：部分有注释的文件\n\n"
        report += "这些文件的部分测试函数有注释，需要补充剩余的测试注释。\n\n"

        for idx, analysis in enumerate(sorted(partial_comment_files, key=lambda a: a.commented_tests/a.total_tests), 1):
            completion = analysis.commented_tests / analysis.total_tests * 100
            missing = analysis.total_tests - analysis.commented_tests

            report += f"""
### {idx}. `{analysis.relative_path}` - {completion:.1f}% 完成

- **总测试数**: {analysis.total_tests}
- **有注释**: {analysis.commented_tests} 个
- **缺失注释**: {missing} 个

**缺失注释的测试函数**:

"""
            missing_tests = [tf for tf in analysis.test_functions if not tf.has_doc_comment]
            for tf in missing_tests[:5]:  # 只显示前5个
                report += f"- `{tf.name}()` (行 {tf.line_number})\n"

            if len(missing_tests) > 5:
                report += f"- ... 还有 {len(missing_tests) - 5} 个\n"

    # 完全有基础注释但缺失详细注释的文件
    needs_detailed = [a for a in analyses if a.commented_tests == a.total_tests and a.detailed_commented_tests < a.total_tests]
    if needs_detailed:
        report += "\n---\n\n## 🟢 优先级 3：需要补充详细注释的文件\n\n"
        report += "这些文件的测试都有基础注释，但缺少详细的中文注释（测试目的、场景、预期结果）。\n\n"

        for idx, analysis in enumerate(sorted(needs_detailed, key=lambda a: a.detailed_commented_tests/a.total_tests), 1):
            detailed_completion = analysis.detailed_commented_tests / analysis.total_tests * 100
            missing_detailed = analysis.total_tests - analysis.detailed_commented_tests

            report += f"""
### {idx}. `{analysis.relative_path}` - {detailed_completion:.1f}% 详细注释

- **总测试数**: {analysis.total_tests}
- **有详细注释**: {analysis.detailed_commented_tests} 个
- **需要补充**: {missing_detailed} 个

**需要补充详细注释的测试函数**:

"""
            simple_comment_tests = [tf for tf in analysis.test_functions if tf.has_doc_comment and not tf.has_detailed_comment]
            for tf in simple_comment_tests[:5]:  # 只显示前5个
                report += f"- `{tf.name}()` (行 {tf.line_number})\n"
                if tf.doc_comment:
                    # 显示当前的简单注释
                    first_line = tf.doc_comment.split('\n')[0][:60]
                    report += f"  - 当前注释: \"{first_line}...\"\n"

            if len(simple_comment_tests) > 5:
                report += f"- ... 还有 {len(simple_comment_tests) - 5} 个\n"

    # 已完成的文件
    if full_detailed_files:
        report += "\n---\n\n## ✅ 已完成：注释完整的文件\n\n"
        report += "这些文件的所有测试都有详细的中文注释。\n\n"

        for analysis in sorted(full_detailed_files, key=lambda a: a.total_tests, reverse=True):
            report += f"- `{analysis.relative_path}` - {analysis.total_tests} 个测试 ✅\n"

    # 注释格式规范
    report += """

---

## 📋 注释格式规范

所有测试函数应该遵循以下格式：

```rust
/// 测试描述
///
/// ## 测试目的
/// 说明测试要验证什么功能
///
/// ## 测试场景
/// 1. 场景1
/// 2. 场景2
///
/// ## 预期结果
/// - 预期结果1
/// - 预期结果2
#[test]
fn test_xxx() {
    // Arrange: 准备测试数据
    // Act: 执行操作
    // Assert: 验证结果
}
```

---

## 🎯 行动建议

### 立即行动

1. 优先补充**完全缺失注释**的文件（优先级 1）
2. 关注测试数量多的文件，优先补充
3. 补充**部分有注释**的文件中缺失的测试注释（优先级 2）

### 后续行动

1. 为已有基础注释的测试补充详细注释（优先级 3）
2. 确保所有新增测试都有详细的中文注释

---

## 📚 参考

- [测试编写规范](docs/guidelines/testing/writing.md)
- [测试注释完整性分析](analysis/test_comment_completeness_analysis.md)
- [测试注释迁移分析](analysis/test_comment_migration_analysis.md)

---

**最后更新**: {check_date.split()[0]}
"""

    # 保存报告
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(report, encoding='utf-8')

    log_success(f"报告已保存到: {output_path}")


def analyze(args) -> None:
    """分析 src/ 目录的单元测试注释"""
    log_break('=')
    log_info("分析 src/ 目录的单元测试注释")
    log_break('=')
    log_break()

    # 获取项目根目录
    script_path = Path(__file__).resolve()
    project_root = script_path.parent.parent.parent.parent.parent
    src_dir = project_root / 'src'

    if not src_dir.exists():
        log_error(f"src/ 目录不存在: {src_dir}")
        sys.exit(1)

    log_info(f"项目根目录: {project_root}")
    log_info(f"src/ 目录: {src_dir}")
    log_break()

    # 分析测试文件
    log_info("正在扫描 src/ 目录...")
    analyses = analyze_src_tests(src_dir)

    if not analyses:
        log_warning("未找到任何包含单元测试的文件")
        return

    log_success(f"找到 {len(analyses)} 个包含单元测试的文件")
    log_break()

    # 输出统计信息
    total_tests = sum(a.total_tests for a in analyses)
    total_commented = sum(a.commented_tests for a in analyses)
    total_detailed = sum(a.detailed_commented_tests for a in analyses)

    log_info("统计信息:")
    log_info(f"  文件数: {len(analyses)}")
    log_info(f"  测试数: {total_tests}")
    log_info(f"  有注释: {total_commented} ({total_commented/total_tests*100:.1f}%)")
    log_info(f"  详细注释: {total_detailed} ({total_detailed/total_tests*100:.1f}%)")
    log_info(f"  缺失注释: {total_tests - total_commented} ({(total_tests - total_commented)/total_tests*100:.1f}%)")
    log_break()

    # 生成报告
    output_path = project_root / args.output
    log_info(f"生成报告: {output_path}")
    generate_report(analyses, output_path)

    log_break()
    log_success("分析完成")
    log_break()


def main():
    """CLI 入口"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='analyze_src_unit_tests',
        description='Analyze unit test comments in src/ directory'
    )
    parser.add_argument(
        '--output',
        default='analysis/src_unit_test_comments_analysis.md',
        help='Output report file path (default: analysis/src_unit_test_comments_analysis.md)'
    )

    args = parser.parse_args()
    analyze(args)


if __name__ == '__main__':
    main()

