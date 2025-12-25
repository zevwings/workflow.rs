# 测试规范

> 本文档是 Workflow CLI 项目测试规范的索引和总览，包含所有测试相关的规范和最佳实践。

---

## 📋 目录

- [概述](#-概述)
- [核心规范](#-核心规范)
- [参考文档](#-参考文档)
- [测试模板](#-测试模板)
- [快速开始](#-快速开始)
- [相关文档](#-相关文档)

---

## 📋 概述

本文档目录包含所有测试规范文档，分为以下类别：

- **核心规范**：日常测试必须遵循的规范（测试组织、测试编写、测试命令）
- **参考文档**：详细的参考指南（工具使用、Mock服务器、数据工厂、覆盖率、性能测试等）
- **测试模板**：常用的测试模板和代码片段

### 快速导航

| 类别 | 文档 | 说明 |
|------|------|------|
| **核心规范** | [测试组织规范](./organization.md) | 测试类型、目录结构、命名约定、共享工具 |
| | [测试编写规范](./writing.md) | AAA模式、命名规范、独立性、断言最佳实践 |
| | [测试命令参考](./commands.md) | 常用测试命令、调试命令、Makefile命令 |
| **参考文档** | [测试工具指南](./references/tools.md) | pretty_assertions、rstest、mockito |
| | [测试环境工具指南](./references/environments.md) | TestIsolation、CliTestEnv、GitTestEnv |
| | [测试辅助工具指南](./references/helpers.md) | CliCommandBuilder、TestDataGenerator |
| | [Mock服务器使用指南](./references/mock-server.md) | MockServer 使用、端点配置、错误模拟 |
| | [测试数据工厂指南](./references/data-factory.md) | Builder模式、数据生成、模板扩展 |
| | [被忽略测试规范](./references/ignored-tests.md) | 文档格式、测试类型模板、最佳实践 |
| | [覆盖率测试指南](./references/coverage.md) | 覆盖率工具、报告生成、提升技巧 |
| | [性能测试指南](./references/performance.md) | 基准测试、性能要求、优化建议 |
| | [集成测试指南](./references/integration.md) | 环境配置、数据隔离、清理机制 |
| **测试模板** | [标准测试模板](./templates/test-case.template) | 标准测试用例模板 |
| | [被忽略测试模板](./templates/ignored-test.template) | 被忽略测试的完整模板 |
| | [Mock测试模板](./templates/mock-test.template) | Mock 测试模板 |

---

## 核心规范

核心规范是日常测试必须遵循的规范，建议优先阅读：

### [测试组织规范](./organization.md)

定义测试组织结构、命名约定和共享工具使用规范。

**关键内容**：
- 测试类型（单元测试、集成测试、文档测试）
- 测试组织结构（目录结构、模块对应）
- 测试文件命名约定
- 共享测试工具（common 模块）
- 测试数据管理（Fixtures 目录）

**快速参考**：
```
tests/
├── base/              # Base 模块测试
├── cli/               # CLI 命令层测试
├── common/            # 共享测试工具
└── fixtures/          # 测试数据
```

### [测试编写规范](./writing.md)

定义测试编写规范和最佳实践。

**关键内容**：
- 测试结构（AAA 模式：Arrange-Act-Assert）
- 测试命名规范
- 测试独立性原则
- 测试覆盖原则
- 断言最佳实践
- 错误处理测试
- 边界条件测试

**快速参考**：
```rust
#[test]
fn test_parse_ticket_id_with_valid_input() {
    // Arrange: 准备测试数据
    let input = "PROJ-123";

    // Act: 执行被测试的功能
    let result = parse_ticket_id(input);

    // Assert: 验证结果
    assert_eq!(result, Some("PROJ-123"));
}
```

### [测试命令参考](./commands.md)

提供常用测试命令的快速参考。

**关键内容**：
- 基本测试命令
- 测试类型命令（单元、集成、文档）
- Makefile 测试命令
- 测试调试命令
- 常用命令速查

**快速参考**：
```bash
# 运行所有测试
cargo test

# 运行被忽略的测试
cargo test -- --ignored

# 生成覆盖率报告
make coverage
```

---

## 参考文档

参考文档提供详细的参考指南，按需查阅：

### [测试工具指南](./references/tools.md)

介绍常用测试工具的使用方法。

**包含工具**：
- `pretty_assertions` - 彩色 diff 断言
- `rstest` - 参数化测试
- `mockito` - HTTP Mock 测试
- 测试环境工具（TestIsolation、CliTestEnv、GitTestEnv）
- 测试辅助工具（CliCommandBuilder、TestDataGenerator）

### [Mock服务器使用指南](./references/mock-server.md)

详细说明 Mock 服务器的使用方法。

**关键内容**：
- MockServer 基本使用
- 创建 Mock 端点
- 模拟错误情况
- 验证 Mock 调用
- Mock 最佳实践

### [测试数据工厂指南](./references/data-factory.md)

介绍测试数据工厂的使用和扩展。

**关键内容**：
- Builder 模式架构
- 支持的数据类型（Git Commit、GitHub PR、Jira Issue、Config）
- 使用示例和最佳实践
- 模板文件管理
- 扩展 Builder

### [被忽略测试规范](./references/ignored-tests.md)

定义被忽略测试的文档规范。

**关键内容**：
- 统一文档格式（5个必需部分）
- 6种测试类型模板
- 文档编写最佳实践
- 文档维护清单

### [覆盖率测试指南](./references/coverage.md)

介绍测试覆盖率的检查和提升方法。

**关键内容**：
- 覆盖率工具安装和使用
- 生成覆盖率报告
- 覆盖率分析方法
- 覆盖率提升技巧

### [性能测试指南](./references/performance.md)

介绍性能测试和基准测试的方法。

**关键内容**：
- 性能测试要求
- 基准测试（Criterion）
- 性能测试报告
- 性能优化建议

### [集成测试指南](./references/integration.md)

介绍集成测试的环境配置和最佳实践。

**关键内容**：
- 集成测试环境配置
- 数据隔离规则
- 清理机制
- 临时文件管理

---

## 测试模板

项目提供了常用的测试模板，帮助快速创建标准化的测试代码：

### [标准测试模板](./templates/test-case.template)

标准测试用例的模板，包含完整的 AAA 结构。

### [被忽略测试模板](./templates/ignored-test.template)

被忽略测试的完整模板，包含5个必需的文档部分。

### [Mock测试模板](./templates/mock-test.template)

使用 MockServer 的测试模板。

---

## 快速开始

### 新手开发者学习路径

1. **阅读核心规范**（~1小时）
   - [测试组织规范](./organization.md) - 了解测试结构
   - [测试编写规范](./writing.md) - 学习编写规范
   - [测试命令参考](./commands.md) - 掌握常用命令

2. **实践编写测试**（~2小时）
   - 使用[标准测试模板](./templates/test-case.template)
   - 参考[测试工具指南](./references/tools.md)
   - 运行测试并查看结果

3. **深入学习**（按需）
   - 需要隔离的测试环境 → 阅读 [测试环境工具指南](./references/environments.md)
   - 需要 CLI 命令测试辅助 → 阅读 [测试辅助工具指南](./references/helpers.md)
   - 需要 Mock 外部 API → 阅读 [Mock服务器使用指南](./references/mock-server.md)
   - 需要生成测试数据 → 阅读 [测试数据工厂指南](./references/data-factory.md)
   - 需要提升覆盖率 → 阅读 [覆盖率测试指南](./references/coverage.md)

### 快速检查清单

开始编写测试前，请确保：

- [ ] 已阅读[测试组织规范](./organization.md)和[测试编写规范](./writing.md)
- [ ] 了解[测试命令](./commands.md)的基本用法
- [ ] 测试文件放在正确的目录（`tests/` 或 `#[cfg(test)]` 模块）
- [ ] 测试命名遵循规范（`test_` 前缀，描述性名称）
- [ ] 测试使用 AAA 模式（Arrange-Act-Assert）
- [ ] 测试之间相互独立，不共享状态

### 常用命令速查

```bash
# 运行所有测试
cargo test

# 运行特定模块的测试
cargo test --lib module_name

# 运行被忽略的测试
cargo test -- --ignored

# 显示详细输出
cargo test -- --nocapture

# 生成覆盖率报告
make coverage

# 打开覆盖率报告
make coverage-open
```

---

## 📚 相关文档

### 开发规范

- [开发规范总览](../development/README.md) - 开发规范索引
- [代码风格规范](../development/code-style.md) - 代码格式化和 Lint
- [错误处理规范](../development/error-handling.md) - 错误处理最佳实践

### 测试审查

- [测试用例检查指南](../development/references/review-test-case.md) - AI 助手测试审查指南
- [测试覆盖检查机制](../development/references/test-coverage-check.md) - 测试覆盖检查流程

### 其他指南

- [文档编写指南](../document.md) - 文档编写规范和模板
- [PR 平台指南](../pr-platform.md) - PR 平台测试相关

---

## 测试覆盖率目标

- **总体覆盖率**：> 80%
- **关键业务逻辑**：> 90%
- **工具函数**：> 70%
- **CLI 命令层**：> 75%

---

## ✅ 测试质量标准

一个高质量的测试应该满足：

1. **清晰性** - 测试名称和代码清晰表达测试意图
2. **独立性** - 测试之间相互独立，不共享状态
3. **完整性** - 覆盖成功路径、错误路径和边界条件
4. **可维护性** - 代码简洁，易于理解和修改
5. **快速性** - 单元测试 < 100ms，集成测试 < 1s
6. **稳定性** - 测试结果可重复，不受外部因素影响

---

**最后更新**: 2025-12-25

---

## 📝 变更历史

### 2025-12-25
- **新增测试环境工具文档**：添加 `environments.md` 和 `helpers.md` 文档
- **更新Mock服务器文档**：更新 `mock-server.md`，添加高级方法和预设Mock端点说明
- **更新测试工具指南**：更新 `tools.md`，添加测试环境工具和测试辅助工具的链接

