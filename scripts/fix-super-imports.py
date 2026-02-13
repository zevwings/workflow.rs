#!/usr/bin/env python3
"""将 super:: 替换为 crate:: 以统一模块路径（方案 A）

规则：crate 内统一用 crate::，super:: 仅保留测试模块内的 use super::*。
排除：use super::*、$crate::

用法:
  python scripts/fix-super-imports.py          # 处理剩余待转换文件
  python scripts/fix-super-imports.py --dry-run # 仅预览，不写入
  python scripts/fix-super-imports.py --auto    # 自动发现所有含 super:: 的文件
"""
import argparse
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).parent.parent

# 剩余待转换文件（方案 A 第二阶段，42 个）
REMAINING_FILES = [
    # App (7)
    "crates/app/src/commands/branch/cli.rs",
    "crates/app/src/commands/commit/cli.rs",
    "crates/app/src/commands/jira/cli.rs",
    "crates/app/src/commands/pr/cli.rs",
    "crates/app/src/commands/tag/cli.rs",
    "crates/app/src/commands/update/download.rs",
    "crates/app/src/commands/update/version.rs",
    # Storage (8，constants.rs 需手动处理，见 EXCLUDE)
    "crates/storage/src/git/services/branch.rs",
    "crates/storage/src/git/services/blame.rs",
    "crates/storage/src/git/services/diff.rs",
    "crates/storage/src/git/services/merge.rs",
    "crates/storage/src/git/services/stash.rs",
    "crates/storage/src/git/services/tag.rs",
    "crates/storage/src/git/services/hooks/tool_executor.rs",
    "crates/storage/src/git/services/hooks/script_executor.rs",
    # Http (4)
    "crates/http/src/error.rs",
    "crates/http/src/multipart.rs",
    "crates/http/src/request.rs",
    "crates/http/src/response.rs",
    # Prompt (3)
    "crates/prompt/src/backend/terminal.rs",
    "crates/prompt/src/backend/mock.rs",
    "crates/prompt/src/dialog/input/validator.rs",
    # Services (11)
    "crates/services/src/branch/service.rs",
    "crates/services/src/commit/message/service.rs",
    "crates/services/src/commit/message/conversation.rs",
    "crates/services/src/summary/summary/service.rs",
    "crates/services/src/summary/service.rs",
    "crates/services/src/summary/logic/service.rs",
    "crates/services/src/summary/batch/service.rs",
    "crates/services/src/summary/config/service.rs",
    "crates/services/src/summary/test_analyze/service.rs",
    "crates/services/src/summary/classify/service.rs",
    "crates/services/src/testing/builders.rs",
    # Toolkit (7)
    "crates/toolkit/src/util/fs/archive.rs",
    "crates/toolkit/src/util/traits/clipboard_ext.rs",
    "crates/toolkit/src/util/traits/browser_ext.rs",
    "crates/toolkit/src/util/traits/truncate_ext.rs",
    "crates/toolkit/src/util/traits/size_ext.rs",
    "crates/toolkit/src/terminal/layer.rs",
    "crates/toolkit/src/shell/config.rs",
    # Domain (1)
    "crates/domain/src/summary/markdown.rs",
]

# 含嵌套模块的 super::，脚本无法正确解析，需手动修复
EXCLUDE = {
    "crates/storage/src/git/services/hooks/constants.rs",  # pre_commit_hooks 内 super::git_hooks
}


def get_module_paths(rel_path: str) -> tuple[str, str]:
    """返回 (parent_path, grandparent_path)。如 config/global/verification_service.rs -> (config::global, config)"""
    # crates/domain/src/config/global/verification_service.rs
    parts = rel_path.replace("crates/", "").split("/")
    idx = parts.index("src")
    path_parts = parts[idx + 1 : -1]  # 去掉文件名
    if not path_parts:
        return ("", "")
    parent = "::".join(path_parts)
    grandparent = "::".join(path_parts[:-1]) if len(path_parts) > 1 else ""
    return (parent, grandparent)


def needs_conversion(content: str) -> bool:
    """检查文件是否含需转换的 super::（排除仅 use super::* 的情况）"""
    if "super::" not in content:
        return False
    # 移除 use super::* 后，若仍有 super:: 则需转换
    without_star = re.sub(r"use super::\s*\*", "", content)
    return "super::" in without_star


def get_files_to_process(mode: str) -> list[str]:
    """获取待处理文件列表"""
    if mode == "auto":
        files = []
        for rs in (ROOT / "crates").rglob("*.rs"):
            rel = str(rs.relative_to(ROOT)).replace("\\", "/")
            if rel in EXCLUDE:
                continue
            if needs_conversion(rs.read_text(encoding="utf-8")):
                files.append(rel)
        return sorted(files)
    return [f for f in REMAINING_FILES if f not in EXCLUDE]


def process_file(
    filepath: Path, parent_path: str, grandparent_path: str, dry_run: bool
) -> bool:
    content = filepath.read_text(encoding="utf-8")
    if "super::" not in content:
        return False
    lines = content.split("\n")
    new_lines = []
    changed = False
    for line in lines:
        if "use super::*" in line or "use super:: *" in line:
            new_lines.append(line)
            continue
        if "$crate::" in line:
            new_lines.append(line)
            continue
        if "super::" not in line:
            new_lines.append(line)
            continue
        new_line = line
        if "super::super::" in new_line and grandparent_path:
            new_line = new_line.replace(
                "super::super::", f"crate::{grandparent_path}::"
            )
            changed = True
        if "super::" in new_line:
            prefix = f"crate::{parent_path}::" if parent_path else "crate::"
            new_line = new_line.replace("super::", prefix)
            changed = True
        new_lines.append(new_line)
    if changed and not dry_run:
        filepath.write_text("\n".join(new_lines), encoding="utf-8")
    return changed


def main():
    parser = argparse.ArgumentParser(description="super:: → crate:: 统一替换")
    parser.add_argument("--dry-run", action="store_true", help="仅预览，不写入")
    parser.add_argument("--auto", action="store_true", help="自动发现需转换文件")
    parser.add_argument("--check", action="store_true", help="完成后执行 cargo check")
    args = parser.parse_args()

    mode = "auto" if args.auto else "list"
    files = get_files_to_process(mode)

    if args.dry_run:
        print(f"[dry-run] 将处理 {len(files)} 个文件:\n")
        for rel in files:
            fp = ROOT / rel
            if fp.exists():
                parent, grand = get_module_paths(rel)
                content = fp.read_text(encoding="utf-8")
                if process_file(fp, parent, grand, dry_run=True):
                    print(f"  {rel}")
        print(f"\n共 {len(files)} 个文件需转换。去掉 --dry-run 执行实际替换。")
        return

    count = 0
    for rel in files:
        fp = ROOT / rel
        if not fp.exists():
            print(f"Skip (not found): {rel}")
            continue
        parent_path, grandparent_path = get_module_paths(rel)
        if process_file(fp, parent_path, grandparent_path, dry_run=False):
            print(f"Updated: {rel}")
            count += 1

    print(f"Done. {count} files updated.")

    if args.check and count > 0:
        print("\nRunning cargo check...")
        result = subprocess.run(["cargo", "check"], cwd=ROOT)
        if result.returncode != 0:
            print("Warning: cargo check failed. Review changes and fix manually.")
            raise SystemExit(1)


if __name__ == "__main__":
    main()
