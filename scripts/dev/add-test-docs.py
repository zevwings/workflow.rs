#!/usr/bin/env python3
"""为测试函数添加标准文档注释"""

import re
import sys
from pathlib import Path

def has_full_doc_comment(content, test_start_pos):
    """检查测试函数前是否有完整的文档注释"""
    # 向前查找最多50行
    lines_before = content[:test_start_pos].split('\n')
    start_idx = max(0, len(lines_before) - 50)
    context = '\n'.join(lines_before[start_idx:])

    # 检查是否有完整的文档注释（包含"测试目的"、"测试场景"或"预期结果"）
    return bool(re.search(r'///.*?(测试目的|测试场景|预期结果|## 测试目的|## 测试场景|## 预期结果)', context, re.MULTILINE))

def generate_doc_comment(test_name, existing_comment):
    """根据测试函数名称和现有注释生成标准文档注释"""

    # 从现有注释中提取描述
    desc = existing_comment.strip().replace("///", "").strip()
    if not desc.startswith("测试"):
        desc = f"测试{desc}"

    # 根据测试名称提取功能描述
    func_name = test_name.replace("test_", "").replace("_", " ")

    doc = f"""/// {desc}
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误"""

    return doc

def add_docs_to_file(file_path):
    """为文件中的测试函数添加文档注释"""
    file_path = Path(file_path)

    if not file_path.exists():
        print(f"⚠️  文件不存在: {file_path}")
        return 0

    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # 查找所有测试函数
    # 匹配模式: /// 注释 ... #[test] ... fn test_xxx
    pattern = r'(///\s+[^\n]*)\n(\s*#\[test\].*?\n\s*fn\s+(test_\w+))'

    matches = list(re.finditer(pattern, content, re.MULTILINE | re.DOTALL))

    modified_count = 0
    new_content = content
    offset = 0

    for match in reversed(matches):  # 从后往前处理，避免位置偏移
        comment_start = match.start(1)
        test_start = match.start(2)
        existing_comment = match.group(1)
        test_decl = match.group(2)
        test_name = match.group(3)

        # 检查是否已有完整文档注释
        if has_full_doc_comment(content, comment_start):
            continue

        # 生成新的文档注释
        new_doc = generate_doc_comment(test_name, existing_comment)

        # 替换注释
        replacement = f"{new_doc}\n{test_decl}"
        new_content = new_content[:match.start()] + replacement + new_content[match.end():]
        modified_count += 1

    if modified_count > 0:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(new_content)
        print(f"✅ 已更新文件: {file_path} ({modified_count}个测试函数)")

    return modified_count

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("用法: python3 add-test-docs.py <test_file>")
        sys.exit(1)

    file_path = sys.argv[1]
    count = add_docs_to_file(file_path)
    print(f"总共更新了 {count} 个测试函数的文档注释")

