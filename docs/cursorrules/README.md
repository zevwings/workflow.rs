# Cursor AI 规则

> 本目录包含 Workflow CLI 项目的 Cursor AI 规则文件。这些规则指导 AI 助手（如 Cursor）理解项目结构、开发标准和最佳实践。

---

## 📋 概述

本目录包含 Cursor IDE 自动加载的规则文件。每个规则文件专注于项目的特定方面。

**重要**：本目录下的所有规则文件必须与其英文版本 `.cursor/rules/` 保持同步。

## 📂 规则文件

| 文件 | 说明 | 类型 |
|------|------|------|
| `document.mdc` | 文档生成、分类和存储规则 | `always` |
| `overview.mdc` | 项目概述、架构、修改规则、文档索引/删除规则和一般注意事项 | `always` |
| `development.mdc` | 开发规范和指南 | `always` |
| `sync.mdc` | 保持中英文版本同步的规则 | `always` |

## 🔄 同步规则

**关键**：`.cursor/rules/` 目录下的每个文件都有对应的同名文件在 `docs/cursorrules/` 目录中。这些文件必须始终保持同步。

### 同步规则

- 修改 `.cursor/rules/` 中的任何文件时，立即更新 `docs/cursorrules/` 中对应的文件
- 修改 `docs/cursorrules/` 中的任何文件时，立即更新 `.cursor/rules/` 中对应的文件
- 保持章节结构、内容和时间戳在两个版本之间一致

详细的同步规则，请参考 `sync.mdc`。

## 📖 使用说明

Cursor IDE 会自动加载本目录下的所有 `.mdc` 文件。这些文件使用 MDC（Markdown Component）格式，包含 YAML front matter 元数据来定义规则类型和行为。

**规则类型**：
- `always`: 始终包含在模型上下文中（核心规则的默认类型）
- `auto-attach`: 当引用匹配 glob 模式的文件时自动附加
- `agent-request`: 可供 AI 根据需要包含（需要提供描述）
- `manual`: 只有在使用 `@ruleName` 明确引用时才会包含

**注意**：本项目使用 `.mdc` 格式（Cursor 推荐），而不是传统的 `.cursorrules` 文件格式。

## 📚 相关文档

- [文档模板](../../docs/templates/cursorrules/README.md) - 创建新规则文件的模板
- [文档编写指南](../../docs/guidelines/document.md) - 通用文档编写标准
- [开发指南](../../docs/guidelines/development/README.md) - 完整的开发指南

---

**最后更新**: 2025-12-25

