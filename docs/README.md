# Workflow CLI 文档索引

## 📚 文档概览

本文档目录包含 Workflow CLI 的参考文档和使用说明。

---

## 📚 参考文档

### [架构设计](./architecture.md)

项目整体架构设计文档，包含：
- Workspace 与 Crate 结构（app、domain、storage、services、toolkit、prompt、registry）
- 模块职责与数据流向
- 数据存储说明

### [开发规范](./development.md)

开发规范和最佳实践，包含：
- 代码风格规范（格式化、Clippy、命名约定）
- 错误处理规范（`anyhow::Result`、错误处理模式）
- 命名规范（文件、函数、类型、常量、CLI 参数）
- 模块组织规范（workspace 与 crate 职责）
- 文档规范（代码文档注释、文档命名和格式）
- 提交规范（Conventional Commits）
- 检查流程（pre-commit、review）

### [测试规范](./testing.md)

测试规范和最佳实践，包含：
- 测试类型（单元测试、集成测试）
- 测试组织结构
- 测试编写规范（AAA 模式）
- 测试命令参考
- 测试工具使用（pretty_assertions、rstest、insta、mockito）

### [迁移文档](./migration/)

版本迁移指南，包含：
- 各版本之间的配置迁移说明
- 迁移步骤和注意事项
- 迁移前后配置对比

**注意**：分支配置已迁移到项目级配置（`.workflow/config.toml`）。请使用 `workflow repo setup` 设置项目级配置。

---

## 📖 快速导航

### 整体架构
- 想了解整体架构？ → [架构设计](./architecture.md)

### 版本迁移
- 需要升级版本？ → [迁移文档索引](./migration/README.md)
- 从 1.5.6 升级到 1.5.7？ → [1.5.6 → 1.5.7 迁移指南](./migration/1.5.6-to-1.5.7.md)

### 开发规范
- 想了解代码风格？ → [开发规范 - 代码风格](./development.md#-代码风格规范)
- 想了解错误处理？ → [开发规范 - 错误处理](./development.md#-错误处理规范)
- 想了解命名规范？ → [开发规范 - 命名规范](./development.md#-命名规范)
- 想了解模块组织？ → [开发规范 - 模块组织](./development.md#-模块组织规范)
- 想了解提交规范？ → [开发规范 - 提交规范](./development.md#-提交规范)
- 想了解检查流程？ → [开发规范 - 检查流程](./development.md#-检查流程)

### 测试规范
- 想了解测试组织？ → [测试规范 - 测试组织结构](./testing.md#-测试组织结构)
- 想了解测试编写？ → [测试规范 - 测试编写规范](./testing.md#-测试编写规范)
- 想了解测试命令？ → [测试规范 - 测试命令参考](./testing.md#-测试命令参考)
- 想了解测试工具？ → [测试规范 - 测试工具](./testing.md#-测试工具)

### API 文档

运行以下命令生成并查看完整的 API 文档：

```bash
cargo doc --open
```

---

## 📝 文档说明

### 参考文档（长期维护）

以下文档是项目的参考文档，需要长期维护：

- **`architecture.md`** - 架构设计文档
- **`development.md`** - 开发规范文档
- **`testing.md`** - 测试规范文档
- **`migration/`** - 迁移文档目录

### 临时文档（不做参考）

以下目录存放临时文档，用于开发过程中的分析和记录，**不做参考**：

- **`requirements/`** - 未完成的需求文档
- **`analysis/`** - 临时分析文档

**说明**：
- 临时文档可以随时删除
- 不需要索引到主文档
- 不需要长期维护
- 不需要遵循命名规范
- 重要的设计提案应移动到参考文档

---

**最后更新**: 2025-01-27
