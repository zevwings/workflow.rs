# Workflow CLI 配置迁移脚本：1.5.6 → 1.5.7
#
# 将 jira-status.toml 和 jira-users.toml 合并为 jira.toml
#
# 使用方法:
#   .\1.5.6-to-1.5.7.ps1 [-DryRun] [-Cleanup]
#
# 参数:
#   -DryRun      预览模式，不实际修改文件
#   -Cleanup     迁移完成后删除旧配置文件

param(
    [switch]$DryRun,
    [switch]$Cleanup
)

# 检查 Python 是否可用
$python = Get-Command python3 -ErrorAction SilentlyContinue
if (-not $python) {
    $python = Get-Command python -ErrorAction SilentlyContinue
}

if (-not $python) {
    Write-Error "错误: 需要 Python 3.6+ 环境"
    Write-Error "请安装 Python 3: https://www.python.org/downloads/"
    exit 1
}

# 检查并安装 toml 库
$tomlCheck = & $python.Name -c "import toml" 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "ℹ 检测到 toml 库未安装，正在自动安装..."
    $installResult = & $python.Name -m pip install --quiet --user toml 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ toml 库安装成功"
    } else {
        Write-Error "错误: 无法自动安装 toml 库"
        Write-Error "请手动安装: pip install toml"
        exit 1
    }
}

# 构建参数数组（用于 Python sys.argv）
$pythonArgList = @()
if ($DryRun) {
    $pythonArgList += "--dry-run"
}
if ($Cleanup) {
    $pythonArgList += "--cleanup"
}

# 执行内嵌的 Python 代码
$pythonScript = @'
import argparse
import os
import shutil
import sys
from pathlib import Path
from typing import Dict

try:
    import toml
except ImportError:
    print("错误: 需要安装 toml 库", file=sys.stderr)
    print("安装命令: pip install toml", file=sys.stderr)
    sys.exit(1)


def get_config_dir() -> Path:
    """获取配置文件目录"""
    home = Path.home()

    # 检查 macOS iCloud 路径
    if sys.platform == "darwin":
        icloud_path = home / "Library" / "Mobile Documents" / "com~apple~CloudDocs" / ".workflow"
        if icloud_path.exists():
            return icloud_path / "config"

    # 默认路径
    return home / ".workflow" / "config"


def log_info(message: str) -> None:
    """输出信息"""
    print(f"ℹ {message}")


def log_success(message: str) -> None:
    """输出成功信息"""
    print(f"✓ {message}")


def log_warning(message: str) -> None:
    """输出警告信息"""
    print(f"⚠ {message}", file=sys.stderr)


def log_error(message: str) -> None:
    """输出错误信息"""
    print(f"✗ {message}", file=sys.stderr)


def read_toml_file(path: Path) -> Dict:
    """读取 TOML 文件"""
    if not path.exists():
        return {}

    try:
        with open(path, "r", encoding="utf-8") as f:
            return toml.load(f)
    except Exception as e:
        log_error(f"读取文件失败 {path}: {e}")
        sys.exit(1)


def write_toml_file(path: Path, data: Dict, dry_run: bool = False) -> None:
    """写入 TOML 文件"""
    if dry_run:
        log_info(f"[预览] 将写入文件: {path}")
        # 预览模式下也显示配置内容
        try:
            output = toml.dumps(data)
            log_info("配置内容预览:")
            print(output)
        except Exception:
            pass  # 忽略预览错误
        return

    try:
        # 确保目录存在
        path.parent.mkdir(parents=True, exist_ok=True)

        # 使用 toml.dumps 然后写入，以便更好地控制格式
        output = toml.dumps(data)
        with open(path, "w", encoding="utf-8") as f:
            f.write(output)
    except Exception as e:
        log_error(f"写入文件失败 {path}: {e}")
        sys.exit(1)


def backup_file(path: Path, dry_run: bool = False) -> None:
    """备份文件"""
    if not path.exists():
        return

    backup_path = path.with_suffix(path.suffix + ".bak")

    if dry_run:
        log_info(f"[预览] 将备份文件: {path} -> {backup_path}")
        return

    try:
        shutil.copy2(path, backup_path)
        log_success(f"已备份: {backup_path}")
    except Exception as e:
        log_error(f"备份文件失败 {path}: {e}")
        sys.exit(1)


def migrate_config(dry_run: bool = False, cleanup: bool = False) -> int:
    """执行迁移"""
    config_dir = get_config_dir()

    old_status_path = config_dir / "jira-status.toml"
    old_users_path = config_dir / "jira-users.toml"
    new_config_path = config_dir / "jira.toml"

    # 检查是否需要迁移
    if not old_status_path.exists() and not old_users_path.exists():
        log_info("没有需要迁移的配置文件")
        return 0

    if new_config_path.exists():
        log_warning(f"新配置文件已存在: {new_config_path}")
        log_warning("如果继续，将合并现有配置")
        response = input("是否继续? (y/N): ")
        if response.lower() != "y":
            log_info("迁移已取消")
            return 0

    if dry_run:
        log_info("=== 预览模式（不会实际修改文件）===")

    # 读取旧配置
    old_status = read_toml_file(old_status_path)
    old_users = read_toml_file(old_users_path)

    # 读取新配置（如果存在）
    new_config = read_toml_file(new_config_path) if new_config_path.exists() else {}

    # 初始化新配置结构
    if "users" not in new_config:
        new_config["users"] = []
    if "status" not in new_config:
        new_config["status"] = {}

    # 合并 users
    users_added = 0
    if "users" in old_users and old_users["users"]:
        existing_emails = {user.get("email") for user in new_config["users"] if user.get("email")}

        for user in old_users["users"]:
            email = user.get("email")
            if email and email not in existing_emails:
                new_config["users"].append(user)
                users_added += 1
                existing_emails.add(email)

        if users_added > 0:
            log_success(f"合并了 {users_added} 个用户")

    # 合并 status（构建嵌套字典结构）
    status_dict = {}

    # 如果新配置中已有 status，先复制
    if "status" in new_config and isinstance(new_config["status"], dict):
        status_dict = new_config["status"].copy()

    # 合并旧配置中的状态
    status_added = 0
    for project_key, project_status in old_status.items():
        if project_key == "users":  # 跳过可能的 users 键
            continue

        if project_key not in status_dict:
            status_dict[project_key] = project_status
            status_added += 1
            log_success(f"迁移了项目状态配置: {project_key}")
        else:
            log_warning(f"项目 {project_key} 的状态配置已存在，跳过")

    # 构建最终配置
    final_config = {}

    # 添加 users
    if new_config.get("users"):
        final_config["users"] = new_config["users"]

    # 添加 status（嵌套表格式）
    if status_dict:
        final_config["status"] = status_dict

    # 备份旧文件
    if old_status_path.exists():
        backup_file(old_status_path, dry_run)
    if old_users_path.exists():
        backup_file(old_users_path, dry_run)

    # 写入新配置
    write_toml_file(new_config_path, final_config, dry_run)
    if not dry_run:
        log_success(f"已创建/更新配置文件: {new_config_path}")

    # 清理旧文件
    if cleanup:
        if old_status_path.exists():
            if dry_run:
                log_info(f"[预览] 将删除文件: {old_status_path}")
            else:
                old_status_path.unlink()
                log_success(f"已删除: {old_status_path}")

        if old_users_path.exists():
            if dry_run:
                log_info(f"[预览] 将删除文件: {old_users_path}")
            else:
                old_users_path.unlink()
                log_success(f"已删除: {old_users_path}")
    else:
        log_info("旧文件已保留（使用 --cleanup 选项可删除）")

    if not dry_run:
        log_success("迁移完成！")

    return 0


def main() -> int:
    """主函数"""
    parser = argparse.ArgumentParser(
        description="Workflow CLI 配置迁移脚本：1.5.6 → 1.5.7",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="预览模式，不实际修改文件",
    )
    parser.add_argument(
        "--cleanup",
        action="store_true",
        help="迁移完成后删除旧配置文件",
    )

    args = parser.parse_args()

    return migrate_config(dry_run=args.dry_run, cleanup=args.cleanup)


if __name__ == "__main__":
    sys.exit(main())
'@

# 将参数传递给 Python 脚本
# 构建 Python 命令，将参数作为 sys.argv 传递
$argList = @()
foreach ($arg in $pythonArgList) {
    $argList += "'$arg'"
}
$argString = if ($argList.Count -gt 0) {
    $argList -join ', '
} else {
    ''
}

$pythonCode = @"
import sys
sys.argv = ['script'] + [$argString]
$pythonScript
"@

& $python.Name -c $pythonCode
exit $LASTEXITCODE
