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
    total_lines = len(lines)
    print(f"📄 Processing Cargo.lock ({total_lines} lines)...", flush=True)

    result: List[str] = []
    removed_packages = set()
    removed_deps_count = 0
    processed_packages = 0

    i = 0
    while i < len(lines):
        line = lines[i]

        # 检测新的 package 块开始
        if is_package_block_start(line):
            # 开始新的 package 块
            package_name = None
            in_dependencies = False
            bracket_depth = 0
            package_lines: List[str] = []
            skip_package = False

            i += 1
            # 处理整个 package 块
            while i < len(lines):
                current_line = lines[i]

                # 如果遇到下一个 package 块，停止处理当前块
                if is_package_block_start(current_line):
                    break

                # 检测 package 名称
                if package_name is None:
                    pkg_name = extract_package_name(current_line)
                    if pkg_name:
                        package_name = pkg_name
                        if pkg_name in PACKAGES_TO_REMOVE:
                            skip_package = True
                            removed_packages.add(pkg_name)
                            # 跳过整个 package 块，找到下一个 package 块
                            i += 1
                            while i < len(lines) and not is_package_block_start(lines[i]):
                                i += 1
                            # 不继续处理当前 package，直接跳到外层循环处理下一个 package
                            break

                # 检测 dependencies 数组开始
                if not in_dependencies and current_line.strip().startswith('dependencies = ['):
                    in_dependencies = True
                    bracket_depth = 1
                    package_lines.append(current_line)
                    i += 1
                    continue

                # 处理 dependencies 数组内容
                if in_dependencies:
                    # 计算括号深度（处理嵌套数组）
                    bracket_depth += current_line.count('[') - current_line.count(']')

                    # 检查是否是我们要移除的依赖
                    if is_dependency_line(current_line, PACKAGES_TO_REMOVE):
                        removed_deps_count += 1
                        # 不添加这一行（移除依赖）
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

                # 将当前行添加到 package 块
                package_lines.append(current_line)
                i += 1

            # 如果不需要跳过，添加 package 块到结果
            if not skip_package:
                package_lines.insert(0, line)  # 添加 [[package]] 行
                result.extend(package_lines)
                processed_packages += 1
                # 每处理 100 个包输出一次进度
                if processed_packages % 100 == 0:
                    progress = (i / total_lines) * 100
                    print(f"  ⏳ Processed {processed_packages} packages ({progress:.1f}% of file)...", flush=True)

            # 如果跳过了 package，i 已经指向下一个 package 块，继续外层循环处理它
            # 如果没跳过，i 也指向下一个 package 块或文件末尾，继续外层循环
            continue
        else:
            # 如果不是 package 块开始，可能是文件开头的元数据，直接添加
            result.append(line)
            i += 1

    print(f"📊 Processing complete: {processed_packages} packages processed", flush=True)

    # 验证结果
    if removed_packages:
        print(f"✅ Removed {len(removed_packages)} package(s): {', '.join(sorted(removed_packages))}", flush=True)
    if removed_deps_count > 0:
        print(f"✅ Removed {removed_deps_count} dependency reference(s)", flush=True)
    if not removed_packages and removed_deps_count == 0:
        print("ℹ️  No packages to remove found", flush=True)

    return '\n'.join(result)


def validate_cargo_lock(content: str) -> bool:
    """验证 Cargo.lock 格式是否正确（快速验证）"""
    # 检查是否至少有一个 package 块
    if '[[package]]' not in content:
        return False

    # 检查是否有基本的 TOML 结构（括号匹配）
    open_brackets = content.count('[')
    close_brackets = content.count(']')
    if open_brackets < close_brackets:
        return False

    # 快速验证：检查是否有 name 字段（每个 package 块都应该有）
    # 使用简单的计数方法，不逐行解析
    package_count = content.count('[[package]]')
    name_count = content.count('name = "')

    # 每个 package 块应该至少有一个 name 字段
    # 允许一些容差（某些包可能有多个 name 字段，或者有注释等）
    if name_count < package_count:
        return False

    return True


def main() -> int:
    """主函数"""
    try:
        print("🚀 Starting Cargo.lock processing...", flush=True)

        # 读取文件
        print("📖 Reading Cargo.lock...", flush=True)
        with open('Cargo.lock', 'r', encoding='utf-8') as f:
            original_content = f.read()

        if not original_content.strip():
            print("⚠️  Cargo.lock is empty", file=sys.stderr)
            return 1

        file_size = len(original_content)
        print(f"📏 Cargo.lock size: {file_size:,} bytes", flush=True)

        # 处理内容
        modified_content = remove_packages_from_cargo_lock(original_content)

        # 验证结果
        print("🔍 Validating modified Cargo.lock...", flush=True)
        if not validate_cargo_lock(modified_content):
            print("❌ Modified Cargo.lock appears to be invalid", file=sys.stderr)
            return 1

        # 检查是否有实际修改
        if original_content == modified_content:
            print("ℹ️  No changes needed (packages may not be in Cargo.lock)", flush=True)
        else:
            # 写入文件
            print("💾 Writing modified Cargo.lock...", flush=True)
            with open('Cargo.lock', 'w', encoding='utf-8') as f:
                f.write(modified_content)
            new_size = len(modified_content)
            size_diff = file_size - new_size
            print(f"✅ Successfully updated Cargo.lock (reduced by {size_diff:,} bytes)", flush=True)

        print("✨ Processing completed successfully!", flush=True)
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
