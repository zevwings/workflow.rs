# 迁移脚本

用于在不同版本之间迁移 Workflow CLI 配置的脚本。

## 脚本列表

| 脚本 | 说明 | 版本范围 |
|------|------|----------|
| `1.5.6-to-1.5.7.sh` | 将 jira-status.toml 和 jira-users.toml 合并为 jira.toml | 1.5.6 → 1.5.7 |
| `1.5.6-to-1.5.7.ps1` | PowerShell 版本（Windows） | 1.5.6 → 1.5.7 |

## 使用方法

**Linux/macOS**:

```bash
curl -fsSL https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/migrate/1.5.6-to-1.5.7.sh | bash -s -- [--dry-run] [--cleanup]
```

**Windows**:

```powershell
.\scripts\migrate\1.5.6-to-1.5.7.ps1 [-DryRun] [-Cleanup]
```

**选项**:
- `--dry-run` / `-DryRun`: 预览模式，不实际修改文件
- `--cleanup` / `-Cleanup`: 迁移完成后删除旧配置文件

## 相关文档

- [迁移文档](../../docs/migration/README.md)
