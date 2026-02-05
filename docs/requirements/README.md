# 需求分析文档

本目录用于存放 Workflow CLI 项目的需求分析、功能说明、设计方案和实施计划文档。

## 📋 说明

需求文档用于记录和说明项目的功能需求、实施计划、设计方案等关键信息，帮助开发者理解需求背景、实施步骤和预期目标。

## 📝 文档规范

### 文档类型

本目录包含以下类型的需求文档：

- **需求分析**：功能需求分析、非功能需求分析、约束条件分析
- **设计方案**：技术方案设计、架构设计、实现思路
- **实施计划**：分阶段实施计划、任务清单、时间估算
- **设计提案**：未实现的架构设计、功能设计提案
- **待办事项**：功能待办事项、改进计划、实施指南

### 文档命名

需求文档应使用描述性的名称命名，例如：

- `{topic}.md` - 需求分析文档（例如：`jira.md`）
- `{topic}-requirement.md` - 需求文档（例如：`config-sync-requirement.md`）
- `{topic}-design.md` - 设计提案文档（例如：`prompt-design.md`）

### 文档性质

**⚠️ 重要**：本目录下的文档是**临时分析文档**，不是参考文档。

- **可删除性**：这些文档可以随时删除，不需要长期保留
- **保留期限**：实施完成后 1 个月内可以清理，或转为参考文档
- **索引规则**：这些文档**不应该**被索引到 `docs/README.md` 中
- **转为参考文档**：当设计提案需要长期保留时，应移动到 `docs/architecture/` 目录

### 文档状态分类

需求文档按状态分类：

- ✅ **已实现** - 功能已完成，文档可归档或删除
- 🚧 **实施中** - 功能部分完成，文档仍有参考价值
- ⏳ **待实施** - 功能未开始，文档为规划参考
- 📚 **参考文档** - 需要长期保留的设计提案，应移动到 `docs/architecture/`

---

## 📚 当前需求文档

### 📋 文档列表

#### 1. [jira.md](./jira.md)
- **状态**: 🚧 部分完成
- **实现度**: ~40%
- **分类**: JIRA 模块
- **已完成**:
  - ✅ `jira info` - 显示 ticket 基本信息（支持多种输出格式）
  - ✅ `jira changelog` - 显示变更历史（支持字段过滤、多种输出格式）
  - ✅ `jira comments` - 显示评论（支持分页、过滤、多种输出格式）
  - ✅ `jira attachments` - 下载附件
  - ✅ `jira clean` - 清理本地数据
  - ✅ JIRA API：`transition`、`assign`、`add_comment`（已实现，待封装为命令）
  - ✅ PR 创建和合并时自动更新 JIRA 状态
- **待实现**:
  - `jira info` 增强功能（显示更多字段）
  - 新增 JIRA 命令（assign、comment、create、list、watch）
  - JIRA 集成增强（批量操作、自定义工作流规则）
- **优先级**: 高优先级（命令封装、info 增强）

#### 2. [test-architecture-improvement.md](./test-architecture-improvement.md)
- **状态**: 🔄 进行中
- **实现度**: 0%
- **分类**: 测试架构改进
- **内容**:
  - 测试覆盖率监控系统 (cargo-tarpaulin)
  - 测试数据管理架构 (工厂模式 + Mock服务器)
  - 系统化性能测试 (Criterion基准测试)
  - CI/CD流水线集成
- **优先级**: 高优先级（测试质量提升）

#### 3. [integration.md](./integration.md)
- **状态**: ⏳ 待实施
- **实现度**: 0%
- **分类**: 集成与扩展
- **内容**:
  - 更多平台支持（GitLab、Bitbucket）
  - 通知系统（桌面通知、邮件通知）
- **优先级**: 中优先级

#### 4. [doc-check.md](./doc-check.md)
- **状态**: ⏳ 待实施
- **实现度**: 0%
- **分类**: 文档检查工具
- **内容**:
  - 文档路径验证脚本
  - 模块统计验证脚本
  - 综合检查脚本（可选）
  - CI 集成（可选）
- **优先级**: P2（可选，但建议实施以提升效率）

#### 5. [gix-migration.md](./gix-migration.md)
- **状态**: ⏳ 待实施
- **实现度**: 0%
- **分类**: 依赖迁移
- **内容**: Git 库迁移到 gix

#### 6. [cursor-rules-enhancement.md](./cursor-rules-enhancement.md)
- **状态**: ⏳ 待实施
- **实现度**: 0%
- **分类**: 开发工具配置
- **内容**:
  - 快速决策树机制（白名单/黑名单检查流程）
  - 每次消息独立判断规则
  - 规则优先级和执行保证
  - 详细的文档分类和存储规范
  - 文档命名规范和注意事项
- **优先级**: 高优先级（确保规则正确执行）

#### 7. [logger-enhancement.md](./logger-enhancement.md)
- **状态**: ⏳ 待实施
- **实现度**: 0%
- **分类**: 日志系统增强
- **内容**:
  - 模块级日志分离（每个模块输出到独立文件）
  - 自动模块识别（通过调用栈或显式指定）
  - 日志轮转（大小限制、备份管理、自动压缩）
  - 统一错误日志收集（`error.log`）
  - 结构化日志字段支持（`WithField/WithFields/WithError`）
  - JSON 格式输出支持
- **优先级**: 中优先级（功能增强，非阻塞性）

#### 8. [git-hooks-implementation.md](./git-hooks-implementation.md)
- **状态**: ⏳ 待实施
- **实现度**: 0%
- **分类**: Git 功能增强
- **内容**:
  - Git hooks 发现和执行机制
  - 支持标准 Git hooks（pre-commit、pre-push、post-commit 等）
  - Hook 执行器、发现器、配置管理
  - 在 CommitService 和 RemoteService 中集成 hooks
  - 跳过机制（--no-verify、环境变量）
- **优先级**: P0（高优先级，保持与标准 Git 兼容性）

---

## 📊 当前统计

| 状态 | 文档数量 | 说明 |
|-----|---------|------|
| 🚧 部分完成 | 1 个 | JIRA 模块已有基础实现 |
| 🔄 进行中 | 1 个 | 测试架构改进 |
| ⏳ 待实施 | 6 个 | 集成扩展、文档检查工具、依赖迁移、Cursor 规则增强、日志系统增强、Git Hooks 实现 |
| **总计** | **7 个** | - |

---

## 🎯 使用指南

### 创建新需求文档

1. 根据需求类型选择合适的文件名（使用 `kebab-case`）
2. 在文档中明确标注需求状态和实现度
3. 提供清晰的需求描述、设计方案和实施计划
4. 使用模板创建新文档（参考 `docs/document/templates/requirements/requirement.template`）

### 更新现有需求文档

当需求状态发生变化时，应及时更新对应的需求文档：

- 实施进度更新
- 已完成任务标记
- 新增或调整待实现任务
- 更新任务统计

### 文档维护

- **定期审查**：定期审查文档状态，确保信息准确
- **及时归档**：实施完成后及时归档或删除
- **转为参考文档**：重要的设计提案应移动到 `docs/architecture/` 目录
- **保持整洁**：保持目录整洁，只保留活跃的待办事项

---

## 🔗 快速导航

### 按模块查找

- **JIRA 相关** → [jira.md](./jira.md) 🚧 部分完成
- **测试架构改进** → [test-architecture-improvement.md](./test-architecture-improvement.md) 🔄 进行中
- **集成与扩展** → [integration.md](./integration.md) ⏳ 待实施
- **文档检查工具** → [doc-check.md](./doc-check.md) ⏳ 待实施
- **依赖迁移** → [gix-migration.md](./gix-migration.md) ⏳ 待实施
- **Cursor 规则增强** → [cursor-rules-enhancement.md](./cursor-rules-enhancement.md) ⏳ 待实施
- **日志系统增强** → [logger-enhancement.md](./logger-enhancement.md) ⏳ 待实施
- **Git Hooks 实现** → [git-hooks-implementation.md](./git-hooks-implementation.md) ⏳ 待实施

---

## 📚 相关文档

- [开发规范](../development/README.md)
- [文档编写指南](../document/README.md#-文档编写指南)
- [架构文档](../architecture/architecture.md)
- [需求文档模板](../guidelines/templates/requirements/requirement.template)

---

**最后更新**: 2025-02-05
