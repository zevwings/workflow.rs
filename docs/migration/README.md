# 迁移文档索引

Workflow CLI 各版本间的配置迁移说明。

> **注意**：迁移版本号独立于软件版本号。迁移版本（如 `v1.0.0`）仅在配置格式变化时产生。

## 迁移文档

（暂无）

## 执行迁移

```bash
# 预览（推荐）
workflow migrate --dry-run

# 执行
workflow migrate

# 执行并清理旧文件
workflow migrate cleanup
```

## 相关链接

- [迁移脚本说明](../../scripts/migrate/README.md)
- 分支配置已迁移至 `.workflow/config.toml`，使用 `workflow repo setup` 设置
