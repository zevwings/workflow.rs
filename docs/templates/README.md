# 文档模板索引

> 本目录包含 Workflow CLI 项目的所有文档模板，用于标准化不同类型文档的创建。

---

## 📋 目录

- [概述](#-概述)
- [模板分类](#-模板分类)
  - [架构文档模板](#-架构文档模板)
  - [开发指南模板](#-开发指南模板)
  - [测试文档模板](#-测试文档模板)
  - [需求文档模板](#-需求文档模板)
  - [迁移文档模板](#-迁移文档模板)
  - [代码审查模板](#-代码审查模板)
- [如何使用](#-如何使用)
- [模板路径速查](#-模板路径速查)

---

## 📋 概述

本目录统一管理项目所有文档模板，按照文档类型分类存储，便于查找和使用。

### 模板组织原则

- **分类清晰**：按文档类型分为 6 大类
- **结构统一**：所有模板遵循统一的章节结构
- **易于扩展**：添加新模板类型只需创建新的子目录
- **路径稳定**：所有模板在 `docs/templates/` 下，便于引用

---

## 📂 模板分类

### 🏗 架构文档模板

**目录**: `architecture/`

架构文档模板用于描述模块或系统的架构设计。

| 模板文件 | 用途 | 包含章节 |
|---------|------|---------|
| `architecture.template` | 模块架构文档 | 概述、Lib层架构、Commands层架构、工作流程、依赖关系、代码示例、测试、相关文档 |

**适用场景**：
- 新增模块的架构设计文档
- 现有模块的架构重构文档
- 系统架构的详细说明

---

### 📘 开发指南模板

**目录**: `development/`

开发指南模板用于定义开发规范、最佳实践和工作流程。

| 模板文件 | 用途 | 包含章节 |
|---------|------|---------|
| `development-core.template` | 核心开发规范 | 概述、代码风格、错误处理、模块组织、测试、相关文档 |
| `development-reference.template` | 参考文档 | 概述、主要内容、最佳实践、常见问题、相关文档 |
| `development-workflow.template` | 开发工作流 | 概述、前置条件、工作流步骤、检查清单、常见问题、相关文档 |
| `guideline.template` | 通用指南 | 概述、核心原则、使用场景、具体规范、相关文档 |

**适用场景**：
- 定义新的开发规范
- 编写技术参考文档
- 制定开发工作流程
- 创建通用开发指南

---

### 🧪 测试文档模板

**目录**: `testing/`

测试文档模板用于测试用例、Mock 测试和忽略测试的文档编写。

| 模板文件 | 用途 | 包含章节 |
|---------|------|---------|
| `test-case.template` | 测试用例文档 | 测试描述、测试代码、预期结果 |
| `mock-test.template` | Mock 测试文档 | 测试描述、Mock 设置、测试代码、注意事项 |
| `ignored-test.template` | 忽略测试文档 | 测试描述、忽略原因、恢复计划、临时解决方案 |

**适用场景**：
- 编写测试用例文档
- 记录 Mock 测试实现
- 标注被忽略的测试及原因

---

### 📝 需求文档模板

**目录**: `requirements/`

需求文档模板用于需求分析、功能规划和实施计划。

| 模板文件 | 用途 | 包含章节 |
|---------|------|---------|
| `requirement.template` | 需求文档/TODO | 概述、当前状态、目标、需求分析、实施计划、测试策略、相关文档 |

**适用场景**：
- 新功能需求分析
- TODO 事项跟踪
- 功能实施计划
- 技术方案设计

---

### 🔄 迁移文档模板

**目录**: `migration/`

迁移文档模板用于版本升级、配置迁移和重构指南。

| 模板文件 | 用途 | 包含章节 |
|---------|------|---------|
| `migration.template` | 迁移指南 | 概述、变更说明、迁移步骤、兼容性、常见问题、回滚方案、相关文档 |

**适用场景**：
- 版本升级指南
- 配置格式迁移
- API 重构说明
- 破坏性变更指南

---

### 🔍 代码审查模板

**目录**: `review/`

代码审查模板用于定义审查流程和审查指南。

| 模板文件 | 用途 | 包含章节 |
|---------|------|---------|
| `review-guide.template` | 审查指南 | 概述、审查目标、审查重点、审查清单、常见问题、相关文档 |
| `review-workflow.template` | 审查工作流 | 概述、审查阶段、执行步骤、输出规范、相关文档 |

**适用场景**：
- 制定代码审查标准
- 定义审查工作流程
- 创建审查清单

---

## 📖 如何使用

### 1. 选择合适的模板

根据你要创建的文档类型，从上述分类中选择合适的模板：

```bash
# 例如：创建新模块的架构文档
cp docs/templates/architecture/architecture.template docs/architecture/my-module.md

# 例如：创建需求文档
cp docs/templates/requirements/requirement.template docs/requirements/my-feature.md

# 例如：创建测试文档
cp docs/templates/testing/test-case.template docs/guidelines/testing/test-my-feature.md
```

### 2. 填写模板内容

打开复制的文档，按照模板中的占位符进行填写：

- `{xxx}` - 需要替换的内容
- `{xxx / yyy}` - 需要选择其中一个的内容
- `{可选}` - 可选内容，根据实际情况决定是否保留

### 3. 遵循文档规范

确保文档符合项目规范：

- **章节结构**：保持模板定义的章节结构
- **命名规范**：使用 `snake_case` 命名文件（例如：`test-coverage.md`）
- **时间戳**：在文档末尾添加 "最后更新" 时间戳（参考 `docs/guidelines/document-timestamp.md`）
- **索引更新**：如有需要，更新相应的文档索引（`docs/README.md` 或 `docs/requirements/README.md`）

### 4. 文档位置

根据文档类型，将创建的文档放到正确的目录：

| 文档类型 | 存放目录 | 是否需要索引 |
|---------|---------|-------------|
| 架构文档 | `docs/architecture/` | 是（`docs/README.md`） |
| 开发指南 | `docs/guidelines/development/` | 是（`docs/README.md`） |
| 测试指南 | `docs/guidelines/testing/` | 是（`docs/README.md`） |
| 需求文档 | `docs/requirements/` | 是（`docs/requirements/README.md`） |
| 迁移指南 | `docs/migration/` | 是（`docs/README.md`） |
| 分析报告 | `analysis/` | 否（临时文档） |
| 审查报告 | `report/` | 否（临时文档） |

**注意**：
- 分析报告（`analysis/`）和审查报告（`report/`）是**临时文档**，不需要建立索引
- 需求文档只需在 `docs/requirements/README.md` 中索引，不需要在 `docs/README.md` 中索引

---

## 🗺 模板路径速查

### 按模板类型查询

```bash
# 架构文档模板
docs/templates/architecture/architecture.template

# 开发指南模板
docs/templates/development/development-core.template
docs/templates/development/development-reference.template
docs/templates/development/development-workflow.template
docs/templates/development/guideline.template

# 测试文档模板
docs/templates/testing/test-case.template
docs/templates/testing/mock-test.template
docs/templates/testing/ignored-test.template

# 需求文档模板
docs/templates/requirements/requirement.template

# 迁移文档模板
docs/templates/migration/migration.template

# 代码审查模板
docs/templates/review/review-guide.template
docs/templates/review/review-workflow.template
```

### 按使用场景查询

| 使用场景 | 推荐模板 |
|---------|---------|
| 我要设计一个新模块 | `architecture/architecture.template` |
| 我要定义一个新的开发规范 | `development/guideline.template` |
| 我要编写一个技术参考文档 | `development/development-reference.template` |
| 我要定义一个开发工作流程 | `development/development-workflow.template` |
| 我要编写测试用例文档 | `testing/test-case.template` |
| 我要记录 Mock 测试 | `testing/mock-test.template` |
| 我要标注忽略的测试 | `testing/ignored-test.template` |
| 我要分析一个新功能需求 | `requirements/requirement.template` |
| 我要编写版本升级指南 | `migration/migration.template` |
| 我要制定代码审查标准 | `review/review-guide.template` |
| 我要定义审查工作流程 | `review/review-workflow.template` |

---

## 📚 相关文档

- [文档编写指南](../guidelines/document.md) - 文档写作规范和模板使用说明
- [文档时间戳规范](../guidelines/document-timestamp.md) - "最后更新" 时间戳格式要求
- [开发指南索引](../guidelines/development/README.md) - 开发规范和最佳实践
- [测试指南索引](../guidelines/testing/README.md) - 测试规范和测试工具
- [文档索引](../README.md) - 项目所有参考文档索引

---

**最后更新**: 2025-12-25

