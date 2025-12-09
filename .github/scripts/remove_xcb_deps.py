#!/usr/bin/env python3
"""
从 Cargo.lock 中移除 clipboard 和 xcb 相关依赖的脚本

用于 Linux ARM64 交叉编译时，避免编译 xcb 库（因为 XCB 库在 Ubuntu 源中不可用）。
"""

import re
import sys
from typing import List, Set, Optional

# 要移除的包名
PACKAGES_TO_REMOVE: Set[str] = {'clipboard', 'x11-clipboard', 'xcb', 'clipboard-win'}


def is_package_block_start(line: str) -> bool:
    """检查是否是 package 块开始"""
    return line.strip() == '[[package]]'


def extract_package_name(line: str) -> Optional[str]:
    """从 name = "package-name" 行中提取包名"""
    match = re.search(r'name\s*=\s*"([^"]+)"', line)
    return match.group(1) if match else None


def is_dependency_line(line: str, packages: Set[str]) -> bool:
    """检查是否是我们要移除的依赖行"""
    stripped = line.strip()
    # 匹配格式: "package-name" 或 "package-name version" 或 "package-name version source"
    for pkg in packages:
        # 匹配以包名开头的依赖项（可能在引号内）
        pattern = rf'^\s*"{re.escape(pkg)}"'
        if re.match(pattern, stripped):
            return True
    return False


def remove_packages_from_cargo_lock(content: str) -> str:
    """从 Cargo.lock 内容中移除指定的包及其依赖引用"""
    lines = content.split('\n')
    result: List[str] = []
    in_dependencies = False
    bracket_depth = 0
    removed_packages = set()
    removed_deps_count = 0

    # 当前 package 块的内容（用于缓冲）
    current_package_lines: List[str] = []
    current_package_name: Optional[str] = None
    skip_current_package = False

    i = 0
    while i < len(lines):
        line = lines[i]

        # 检测新的 package 块开始
        if is_package_block_start(line):
            # 处理之前的 package 块
            if current_package_lines and not skip_current_package:
                # 将之前的 package 块添加到结果中
                result.extend(current_package_lines)

            # 开始新的 package 块
            current_package_lines = [line]
            current_package_name = None
            skip_current_package = False
            in_dependencies = False
            bracket_depth = 0
            i += 1
            continue

        # 如果当前 package 块应该被跳过，继续跳过直到下一个 package
        if skip_current_package:
            i += 1
            while i < len(lines):
                if is_package_block_start(lines[i]):
                    i -= 1  # 回退，让外层循环处理
                    break
                i += 1
            continue

        # 检测 package 名称
        if current_package_name is None:
            pkg_name = extract_package_name(line)
            if pkg_name:
                current_package_name = pkg_name
                if pkg_name in PACKAGES_TO_REMOVE:
                    skip_current_package = True
                    removed_packages.add(pkg_name)
                    current_package_lines = []  # 清空缓冲，不添加这个包
                    i += 1
                    continue

        # 检测 dependencies 数组开始（在 package 块内）
        if not in_dependencies and line.strip().startswith('dependencies = ['):
            in_dependencies = True
            bracket_depth = 1

        # 处理 dependencies 数组内容
        if in_dependencies:
            # 计算括号深度（处理嵌套数组）
            bracket_depth += line.count('[') - line.count(']')

            # 检查是否是我们要移除的依赖
            if is_dependency_line(line, PACKAGES_TO_REMOVE):
                removed_deps_count += 1
                # 不将这一行添加到缓冲中（移除依赖）
                i += 1
                # 如果括号深度回到 0 或以下，dependencies 数组结束
                if bracket_depth <= 0:
                    in_dependencies = False
                    bracket_depth = 0
                continue

            # 如果括号深度回到 0 或以下，dependencies 数组结束
            if bracket_depth <= 0:
                in_dependencies = False
                bracket_depth = 0

        # 将当前行添加到 package 块缓冲中（如果不在跳过状态）
        current_package_lines.append(line)
        i += 1

    # 处理最后一个 package 块
    if current_package_lines and not skip_current_package:
        result.extend(current_package_lines)

    # 验证结果
    if removed_packages:
        print(f"✅ Removed {len(removed_packages)} package(s): {', '.join(sorted(removed_packages))}")
    if removed_deps_count > 0:
        print(f"✅ Removed {removed_deps_count} dependency reference(s)")

    return '\n'.join(result)


def validate_cargo_lock(content: str) -> bool:
    """验证 Cargo.lock 格式是否正确"""
    # 检查是否至少有一个 package 块
    if '[[package]]' not in content:
        return False

    # 检查是否有基本的 TOML 结构
    if content.count('[') < content.count(']'):
        return False

    # 验证每个 [[package]] 块都有 name 字段
    lines = content.split('\n')
    in_package = False
    has_name = False

    for line in lines:
        if line.strip() == '[[package]]':
            if in_package and not has_name:
                # 前一个 package 块没有 name 字段
                return False
            in_package = True
            has_name = False
        elif in_package:
            if extract_package_name(line):
                has_name = True
            # 检查是否到了下一个 package 块（简单检查，不处理嵌套）
            if line.strip().startswith('[') and not line.strip().startswith('[['):
                in_package = False
                if not has_name:
                    return False

    # 检查最后一个 package 块
    if in_package and not has_name:
        return False

    return True


def main() -> int:
    """主函数"""
    try:
        # 读取文件
        with open('Cargo.lock', 'r', encoding='utf-8') as f:
            original_content = f.read()

        if not original_content.strip():
            print("⚠️  Cargo.lock is empty", file=sys.stderr)
            return 1

        # 处理内容
        modified_content = remove_packages_from_cargo_lock(original_content)

        # 验证结果
        if not validate_cargo_lock(modified_content):
            print("❌ Modified Cargo.lock appears to be invalid", file=sys.stderr)
            return 1

        # 检查是否有实际修改
        if original_content == modified_content:
            print("ℹ️  No changes needed (packages may not be in Cargo.lock)")
        else:
            # 写入文件
            with open('Cargo.lock', 'w', encoding='utf-8') as f:
                f.write(modified_content)
            print("✅ Successfully updated Cargo.lock")

        return 0
    except FileNotFoundError:
        print("❌ Cargo.lock not found", file=sys.stderr)
        return 1
    except PermissionError:
        print("❌ Permission denied when accessing Cargo.lock", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"❌ Error processing Cargo.lock: {type(e).__name__}: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        return 1


if __name__ == '__main__':
    sys.exit(main())
