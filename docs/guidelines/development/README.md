# 开发规范

> 本文档是 Workflow CLI 项目开发规范的索引和总览，包含所有开发相关的规范和最佳实践。

---

## 📋 目录

- [概述](#-概述)
- [核心规范](#-核心规范)
- [流程规范](#-流程规范)
- [参考文档](#-参考文档)
- [开发工作流](#-开发工作流)
- [相关文档](#-相关文档)

---

## 📋 概述

本文档目录包含所有开发规范文档，分为以下类别：

- **核心规范**：日常开发必须遵循的规范（代码风格、错误处理、命名、模块组织）
- **流程规范**：开发流程相关的规范（Git 工作流、提交规范、代码审查）
- **参考文档**：详细的参考指南（日志、文档、API 设计、配置管理等，以及检查参考指南）
- **开发工作流**：具体的开发流程指南（新功能开发、重构、添加依赖等）
- **检查工作流**：代码质量检查流程指南（提交前检查、综合深入检查）

### 快速导航

| 类别 | 文档 | 说明 |
|------|------|------|
| **核心规范** | [代码风格规范](./code-style.md) | 代码格式化、Lint 检查、命名约定 |
| | [错误处理规范](./error-handling.md) | 错误类型、错误信息格式、错误处理模式 |
| | [命名规范](./naming.md) | 文件、函数、结构体、常量、CLI 参数命名 |
| | [模块组织规范](./module-organization.md) | 目录结构、模块职责、依赖规则 |
| **流程规范** | [Git 工作流规范](./git-workflow.md) | 分支策略、分支命名、工作流程 |
| | [提交规范](./commit.md) | Conventional Commits 格式、提交类型 |
| | [代码审查规范](./code-review.md) | 审查清单、审查重点 |
| **参考文档** | [日志和调试规范](./references/logging.md) | 日志系统架构、日志级别、敏感信息过滤 |
| | [文档规范](./references/documentation.md) | 公共 API 文档、文档注释格式、文档同步要求 |
| | [配置管理规范](./references/configuration.md) | 配置验证、配置迁移、默认值管理 |
| | [安全性规则](./references/security.md) | API Token 处理、输入验证、依赖安全检查 |
| | [依赖管理规范](./references/dependency-management.md) | 添加依赖、依赖更新、安全漏洞处理 |
| | [性能优化规范](./references/performance.md) | 性能分析、优化策略 |
| | [开发工具规范](./references/development-tools.md) | 开发工具使用指南 |
| | [发布前检查清单](./references/pre-release-checklist.md) | 发布前的检查清单 |
| | [定期检查机制](./references/periodic-review.md) | 定期检查计划和检查记录 |
| | [快速参考指南](./references/quick-reference.md) | 快速查找命令和清单 |
| | [样式规范指南](./references/style.md) | 统一文档格式和术语标准 |
| | [CLI 检查指南](./references/review-cli.md) | CLI 命令结构、补全脚本检查 |
| | [代码检查指南](./references/review-code.md) | 重复代码、工具复用、第三方库检查 |
| | [测试用例检查指南](./references/review-test-case.md) | 测试覆盖、合理性、缺失测试检查 |
| | [测试覆盖检查机制指南](./references/test-coverage-check.md) | 测试覆盖检查机制、定期检查流程 |
| | [文档完整性检查指南](./references/review-document-completeness.md) | README、架构文档、CHANGELOG 完整性检查 |
| | [架构文档与代码一致性检查指南](./references/review-architecture-consistency.md) | 架构文档与代码一致性检查 |
| **开发工作流** | [开发工作流索引](./workflows/README.md) | 开发工作流文档索引 |
| **检查工作流** | [提交前检查](./workflows/pre-commit.md) | 代码质量检查流程（5-15分钟） |
| | [综合深入检查](./workflows/review.md) | 综合深入检查流程（2-4小时） |

---

## 核心规范

核心规范是日常开发必须遵循的规范，建议优先阅读：

### [代码风格规范](./code-style.md)

定义代码格式化、Lint 检查和代码组织规范。

**关键内容**：
- 代码格式化（`cargo fmt`）
- Lint 检查（`cargo clippy`）
- Rust 命名约定
- 代码组织（导入顺序、模块声明）

**快速参考**：
```bash
# 格式化代码
cargo fmt

# Lint 检查
cargo clippy -- -D warnings
```

### [错误处理规范](./error-handling.md)

定义错误处理规范和最佳实践。

**关键内容**：
- color-eyre 配置要求
- 错误类型（统一使用 `color_eyre::Result<T>`）
- 错误信息格式规范
- 错误处理模式（`WrapErr`、`bail!`、`ensure!` 等）
- 分层错误处理

**快速参考**：
```rust
use color_eyre::{eyre::WrapErr, Result};

let result = operation()
    .wrap_err_with(|| format!("Failed to perform operation with id: {}", id))?;
```

### [命名规范](./naming.md)

定义文件、函数、结构体、常量和 CLI 参数的命名规范。

**关键内容**：
- 文件命名（模块文件、测试文件、文档文件）
- 函数命名（动作函数、查询函数、检查函数）
- 结构体命名
- 常量命名（`SCREAMING_SNAKE_CASE`）
- CLI 参数命名规范

**快速参考**：
- 模块文件：`snake_case.rs`
- 函数名：`snake_case`
- 类型名：`PascalCase`
- 常量名：`SCREAMING_SNAKE_CASE`

### [模块组织规范](./module-organization.md)

定义模块组织规范和最佳实践。

**关键内容**：
- 目录结构（三层架构）
- 模块职责（commands/、lib/、bin/）
- 模块依赖规则
- 平台特定代码组织

**快速参考**：
```
src/
├── commands/    # 命令封装层
├── lib/         # 核心业务逻辑层
└── bin/         # 独立可执行文件
```

---

## 流程规范

流程规范定义开发流程相关的规范：

### [Git 工作流规范](./git-workflow.md)

定义 Git 工作流规范，包括分支策略和工作流程。

**关键内容**：
- 分支策略（`master`、`feature/*`、`fix/*`、`hotfix/*`）
- 分支命名规范
- 工作流程（创建分支 → 开发 → 提交 → 推送 → 创建 PR → 代码审查 → 合并）

### [提交规范](./commit.md)

定义提交规范，使用 Conventional Commits 格式。

**关键内容**：
- Conventional Commits 格式
- 提交类型（`feat`、`fix`、`docs`、`style`、`refactor` 等）
- 提交信息要求

**快速参考**：
```bash
feat(jira): add attachments download command

Add new command to download all attachments from a JIRA ticket.
The command supports filtering by file type and size.

Closes #123
```

### [代码审查规范](./code-review.md)

定义代码审查规范和审查清单。

**关键内容**：
- 审查清单（代码格式化、Clippy 检查、测试通过、文档更新等）
- 审查重点（功能正确性、代码质量、错误处理、性能、安全性、可维护性、文档完整性）

---

## 参考文档

参考文档提供详细的参考指南，按需查阅：

### [日志和调试规范](./references/logging.md)

定义日志系统架构、日志级别使用规则和敏感信息过滤规则。

**关键内容**：
- 日志系统架构（Commands 层使用 `log_*!`，Lib 层使用 `trace_*!`）
- 日志级别使用规则
- 敏感信息过滤规则
- 日志配置

### [文档规范](./references/documentation.md)

定义文档规范和文档同步要求。

**关键内容**：
- 公共 API 文档（`///` 文档注释）
- 文档注释格式
- 文档同步要求（代码变更必须同步更新文档）

### [配置管理规范](./references/configuration.md)

定义配置验证、配置迁移和默认值管理规范。

**关键内容**：
- 配置验证规则
- 配置迁移规则
- 配置默认值管理规则

### [安全性规则](./references/security.md)

定义安全性规则，包括 API Token 处理、输入验证和依赖安全检查。

**关键内容**：
- API Token 和敏感信息处理
- 输入验证和清理
- 依赖安全检查

### [依赖管理规范](./references/dependency-management.md)

定义依赖管理规范，包括添加依赖、依赖更新和安全漏洞处理。

**关键内容**：
- 添加依赖原则
- 依赖更新频率要求
- 安全漏洞依赖的紧急更新流程

### [性能优化规范](./references/performance.md)

定义性能优化规范和最佳实践。

**关键内容**：
- 性能分析工具
- 性能优化策略

### [开发工具规范](./references/development-tools.md)

定义开发工具使用指南。

**关键内容**：
- 开发工具安装和使用
- 常用命令和工具

### [发布前检查清单](./references/pre-release-checklist.md)

定义发布前的检查清单。

**关键内容**：
- 代码质量检查
- 版本管理检查
- 文档检查
- 测试检查
- 构建检查

### [定期检查机制](./references/periodic-review.md)

定义定期检查计划和检查记录。

**关键内容**：
- 检查频率（每次发布前、每月、每季度）
- 检查责任人
- 检查方法
- 检查记录

---

## 开发工作流

开发工作流提供具体的开发流程指南：

### [开发工作流索引](./workflows/README.md)

开发工作流文档索引，包含：
- [新功能开发流程](./workflows/new-feature.md) - 从需求分析到功能实现的完整流程
- [重构流程](./workflows/refactoring.md) - 代码重构的标准流程
- [添加依赖流程](./workflows/add-dependency.md) - 添加新依赖的标准流程
- [添加模块流程](./workflows/add-module.md) - 添加新模块的标准流程
- [Bug 修复流程](./workflows/bug-fix.md) - Bug 修复的标准流程

## 检查工作流

检查工作流提供代码质量检查流程指南：

### [提交前检查](./workflows/pre-commit.md)

提交前快速检查流程（5-15分钟），包含：
- 代码质量检查（格式化、Clippy、编译）
- 测试执行
- 文档更新验证
- CLI 和 Completion 检查
- 版本管理检查

### [综合深入检查](./workflows/review.md)

综合深入检查流程（2-4小时），包含：
- CLI 检查
- 代码检查
- 测试检查
- 文档检查
- 架构文档与代码一致性检查
- 跨领域问题关联分析

---

## 📚 相关文档

### 架构文档

- [架构文档总览](../../architecture/architecture.md) - 项目架构总览

### 其他指南

- [文档编写指南](../document.md) - 文档编写规范和模板
- [测试规范指南](../testing/README.md) - 测试规范和最佳实践
  - 基本测试命令说明
  - 覆盖率测试指南
  - 性能测试指南
  - 编写测试最佳实践
  - 测试数据管理最佳实践
  - Mock 服务器使用指南
  - 覆盖率提升技巧
  - 基本测试命令说明
  - 覆盖率测试指南
  - 性能测试指南
  - 编写测试最佳实践
  - 测试数据管理最佳实践
  - Mock 服务器使用指南
  - 覆盖率提升技巧

---

## ✅ 快速检查清单

开始开发前，请确保：

- [ ] 已阅读核心规范（代码风格、错误处理、命名、模块组织）
- [ ] 已了解流程规范（Git 工作流、提交规范、代码审查）
- [ ] 已查阅相关参考文档（如需要）
- [ ] 已了解开发工作流（如需要）

---

**最后更新**: 2025-12-23

