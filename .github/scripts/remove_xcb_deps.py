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
    skip_package = False
    in_dependencies = False
    bracket_depth = 0
    removed_packages = set()
    removed_deps_count = 0

    i = 0
    while i < len(lines):
        line = lines[i]

        # 检测新的 package 块开始
        if is_package_block_start(line):
            skip_package = False
            in_dependencies = False
            bracket_depth = 0
            result.append(line)
            i += 1
            continue

        # 检测 package 名称（只在非跳过状态下检查）
        if not skip_package:
            pkg_name = extract_package_name(line)
            if pkg_name and pkg_name in PACKAGES_TO_REMOVE:
                skip_package = True
                removed_packages.add(pkg_name)
                # 跳过整个 package 块，直到下一个 [[package]] 或文件结束
                i += 1
                while i < len(lines):
                    if is_package_block_start(lines[i]):
                        i -= 1  # 回退一行，让外层循环处理下一个 package
                        break
                    i += 1
                continue

        # 如果正在跳过这个包，直接跳过所有行（避免重复检查）
        if skip_package:
            i += 1
            continue

        # 检测 dependencies 数组开始
        if not in_dependencies and line.strip().startswith('dependencies = ['):
            in_dependencies = True
            bracket_depth = 1
            result.append(line)
            i += 1
            continue

        # 处理 dependencies 数组内容
        if in_dependencies:
            # 计算括号深度（处理嵌套数组）
            bracket_depth += line.count('[') - line.count(']')

            # 检查是否是我们要移除的依赖
            if is_dependency_line(line, PACKAGES_TO_REMOVE):
                removed_deps_count += 1
                i += 1
                # 如果括号深度回到 0 或以下，dependencies 数组结束
                if bracket_depth <= 0:
                    in_dependencies = False
                    bracket_depth = 0
                continue

            result.append(line)

            # 如果括号深度回到 0 或以下，dependencies 数组结束
            if bracket_depth <= 0:
                in_dependencies = False
                bracket_depth = 0
        else:
            result.append(line)

        i += 1

    # 验证结果
    if removed_packages:
        print(f"✅ Removed {len(removed_packages)} package(s): {', '.join(sorted(removed_packages))}")
    if removed_deps_count > 0:
        print(f"✅ Removed {removed_deps_count} dependency reference(s)")

    return '\n'.join(result)


def validate_cargo_lock(content: str) -> bool:
    """简单验证 Cargo.lock 格式是否正确"""
    # 检查是否至少有一个 package 块
    if '[[package]]' not in content:
        return False
    # 检查是否有基本的 TOML 结构
    if content.count('[') < content.count(']'):
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
