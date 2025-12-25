# 文档编写指南

> 本文档提供统一的架构文档编写指南和模板，用于创建新的模块架构文档。

---

## 📋 模板说明

### 使用场景

- 创建新的 Lib 层模块架构文档
- 创建新的命令层架构文档
- 更新现有文档时参考

### 章节说明

- **必选章节**：所有文档必须包含
- **推荐章节**：建议包含（出现率 > 80%）
- **可选章节**：根据模块特性选择性添加

---

## 📝 模板说明

项目提供两种类型的文档模板：

### 模板类型

#### 1. 架构文档模板 (`architecture.template`)

**统一架构模板** (`docs/templates/architecture/architecture.template`)
   - 适用于：同时包含 `lib/` 和 `commands/` 的模块架构文档
   - 特点：统一描述 Lib 层和 Commands 层的架构、设计、集成关系
   - 示例：`pr.md`、`jira.md`、`branch.md`、`alias.md`
   - 位置：`docs/templates/architecture/architecture.template`

**注意**：现在所有架构文档都统一使用此模板，因为实际文档已经合并了 Lib 层和 Commands 层的内容。

#### 2. 指南文档模板 (`guideline.template`)

**指南文档模板** (`docs/templates/development/guideline.template`)
   - 适用于：开发规范、配置指南、使用指南等指南类文档
   - 特点：提供统一的文档结构，包含目录、概述、章节、故障排除等
   - 示例：`testing/README.md`、`github-setup.md`、`pr-platform.md`
   - 位置：`docs/templates/development/guideline.template`

#### 3. 需求文档模板 (`requirement.template`)

**需求文档模板** (`docs/templates/requirements/requirement.template`)
   - 适用于：待办事项、需求分析、实施计划等需求类文档
   - 特点：提供统一的文档结构，包含状态标记、任务清单、实施计划等
   - 示例：`jira.md`、`coverage-improvement.md`、`gix-migration.md`、`integration.md`
   - 位置：`docs/templates/requirements/requirement.template`

#### 4. 检查工作流文档模板 (`review-workflow.template`)

**检查工作流文档模板** (`docs/templates/review/review-workflow.template`)
   - 适用于：检查工作流文档，如提交前检查、综合深入检查等
   - 特点：包含 Mermaid 流程图、快速导航、检查步骤、进度指示器、报告生成等
   - 示例：`pre-commit.md`、`review.md`、`README.md`（工作流索引）
   - 位置：`docs/templates/review/review-workflow.template`
   - **注意**：这是 AI 工作流文档模板，专为 AI 助手设计

#### 5. 检查指南模板 (`review-guide.template`)

**检查指南模板** (`docs/templates/review/review-guide.template`)
   - 适用于：专门检查指南和快速参考文档
   - 特点：提供统一的检查指南结构，包含核心原则、检查目标、检查流程、检查方法、检查清单等，灵活支持详细指南和快速参考两种模式
   - 示例：`review-cli.md`、`review-code.md`、`review-test-case.md`、`review-document-completeness.md`、`review-architecture-consistency.md`、`test-coverage-check.md`、`quick-reference.md`
   - 位置：`docs/templates/review/review-guide.template`

#### 6. 核心规范文档模板 (`development-core.template`)

**核心规范文档模板** (`docs/templates/development/development-core.template`)
   - 适用于：核心开发规范文档（日常必读）
   - 特点：提供核心规范文档结构，包含快速参考、规则、最佳实践等
   - 示例：`code-style.md`、`error-handling.md`、`naming.md`、`module-organization.md`
   - 位置：`docs/templates/development/development-core.template`

#### 7. 开发工作流文档模板 (`development-workflow.template`)

**开发工作流文档模板** (`docs/templates/development/development-workflow.template`)
   - 适用于：开发工作流文档（开发流程）
   - 特点：提供开发工作流结构，包含工作流步骤、检查清单、验证步骤等
   - 示例：`new-feature.md`、`refactoring.md`、`add-dependency.md`
   - 位置：`docs/templates/development/development-workflow.template`
   - **注意**：这是 AI 工作流文档模板，专为 AI 助手设计

#### 8. 参考文档模板 (`development-reference.template`)

**参考文档模板** (`docs/templates/development/development-reference.template`)
   - 适用于：开发规范参考文档（详细指南）
   - 特点：提供参考文档结构，包含详细说明、最佳实践、故障排除等
   - 示例：`references/logging.md`、`references/documentation.md`、`references/refactoring.md`
   - 位置：`docs/templates/development/development-reference.template`

### 快速使用

#### 架构文档

```bash
# 复制统一架构模板
cp docs/templates/architecture/architecture.template docs/architecture/{module}.md

# 然后编辑文件，替换所有 {占位符}
```

#### 指南文档

```bash
# 复制指南文档模板（通用，适用于开发规范、配置指南、使用指南等）
cp docs/templates/development/guideline.template docs/guidelines/{topic}.md

# 复制核心规范文档模板（适用于日常必读的核心规范文档）
cp docs/templates/development/development-core.template docs/guidelines/development/{topic}.md

# 然后编辑文件，替换所有 {占位符}
```

#### 需求文档

```bash
# 复制需求文档模板
cp docs/templates/requirements/requirement.template docs/requirements/{topic}.md

# 然后编辑文件，替换所有 {占位符}
```

#### 检查工作流文档

```bash
# 复制检查工作流文档模板
cp docs/templates/review/review-workflow.template docs/guidelines/workflows/{topic}.md

# 然后编辑文件，替换所有 {占位符}
```

#### 检查指南文档

```bash
# 复制检查指南模板
cp docs/templates/review/review-guide.template docs/guidelines/development/references/{topic}.md

# 然后编辑文件，替换所有 {占位符}
# 对于快速参考文档，可以省略详细检查方法章节，保持简洁
```

#### 开发规范文档

```bash
# 复制指南文档模板（通用，适用于开发规范文档）
cp docs/templates/development/guideline.template docs/guidelines/development/{topic}.md

# 复制核心规范文档模板（适用于日常必读的核心规范文档）
cp docs/templates/development/development-core.template docs/guidelines/development/{topic}.md

# 然后编辑文件，替换所有 {占位符}
```

#### 开发工作流文档

```bash
# 复制开发工作流文档模板
cp docs/templates/development/development-workflow.template docs/guidelines/development/workflows/{workflow-name}.md

# 然后编辑文件，替换所有 {占位符}
```

#### 参考文档

```bash
# 复制参考文档模板
cp docs/templates/development/development-reference.template docs/guidelines/development/references/{topic}.md

# 然后编辑文件，替换所有 {占位符}
```

**对于 AI 助手（Cursor 等）**：
- **架构文档**：参考模板结构和格式生成架构文档，根据模块实际情况填写 Lib 层和 Commands 层的内容
- **指南文档**：参考模板结构和格式生成指南文档，根据文档类型调整章节内容
- **需求文档**：参考模板结构和格式生成需求文档，根据需求类型调整任务清单和实施计划
- **工作流文档**：参考模板结构和格式生成工作流文档，包含流程图、快速导航、检查步骤、进度指示器等（注意：工作流文档模板已添加 AI 工作流文档声明）
- **检查指南文档**：参考模板结构和格式生成检查指南，根据文档类型选择详细模式（检查指南）或简洁模式（快速参考）
- 如果模块只有 Lib 层或只有 Commands 层，可以省略对应部分
- 根据文档类型选择合适的模板（参考下面的模板选择流程图）

### 模板选择流程图

```
创建新文档
  ↓
是架构文档？
  ├─ 是 → architecture.template
  │
  └─ 否 → 是工作流文档？
           ├─ 是 → 开发工作流？ → development-workflow.template（AI 工作流文档）
           │        └─ 检查工作流？ → review-workflow.template（AI 工作流文档）
           │
           └─ 否 → 是指南文档？
                    ├─ 是 → 检查指南？ → review-guide.template
                    │        └─ 参考文档？ → development-reference.template
                    │             └─ 核心规范（日常必读）？ → development-core.template
                    │                  └─ 通用指南？ → guideline.template
                    │
                    └─ 否 → 是需求文档？ → requirement.template
```

**模板选择说明**：

1. **架构文档**：模块架构文档 → `architecture.template`
2. **开发工作流**：新功能开发、重构、添加依赖等 → `development-workflow.template`（AI 工作流文档）
3. **检查工作流**：提交前检查、综合深入检查 → `review-workflow.template`（AI 工作流文档）
4. **检查指南**：专门检查指南、快速参考 → `review-guide.template`
5. **参考文档**：详细参考文档 → `development-reference.template`
6. **核心规范**：日常必读的核心规范 → `development-core.template`
7. **通用指南**：开发规范、配置指南、使用指南等 → `guideline.template`
8. **需求文档**：待办事项、需求分析、实施计划 → `requirement.template`

### 模板预览

<details>
<summary>点击展开查看完整模板</summary>

```markdown
# {模块名} 模块架构文档

## 📋 概述

{模块功能描述，包括模块定位、主要功能、设计目标等}

**注意**：{可选，如果有需要说明的注意事项}

**模块统计：**
- 总代码行数：约 XXX 行
- 文件数量：X 个
- 主要组件：X 个（...）
- {其他统计信息}

---

## 📁 模块结构

### 核心模块文件

```
{文件树结构，包含文件路径和行数}
```

### 依赖模块

- **`lib/xxx/`**：{依赖说明}
- **`lib/yyy/`**：{依赖说明}

### 模块集成

- **`lib/xxx/`**：{集成说明}
  - `Method::xxx()` - {方法说明}
- **`lib/yyy/`**：{集成说明}
  - `Method::yyy()` - {方法说明}

---

## 🏗️ 架构设计

### 设计原则

1. **原则1**：{说明}
2. **原则2**：{说明}
3. **原则3**：{说明}
...

### 核心组件

#### 1. {组件名} ({文件路径})

**职责**：{职责说明}

**主要方法**：
- `method1()` - {说明}
- `method2()` - {说明}

**关键特性**：
- {特性1}
- {特性2}

**使用场景**：
- {场景1}
- {场景2}

#### 2. {组件名} ({文件路径})

{同上}

### 设计模式

#### 1. {模式名}

{模式说明}

**优势**：
- {优势1}
- {优势2}

#### 2. {模式名}

{同上}

### 错误处理

#### 分层错误处理

1. **层级1**：{错误类型}
2. **层级2**：{错误类型}

#### 容错机制

- **场景1**：{处理方式}
- **场景2**：{处理方式}

---

## 🔄 调用流程与数据流

### 整体架构流程

```
{流程图或文字描述}
```

或使用 Mermaid 图表：

```mermaid
flowchart LR
    A[起点] --> B[处理]
    B --> C[终点]
```

### 典型调用示例

#### 1. {场景1}

```
{调用流程}
```

#### 2. {场景2}

```
{调用流程}
```

### 数据流

```
{数据流向图或 Mermaid 图表}
```

---

## 📝 扩展性（可选）

### 添加新功能

1. {步骤1}
2. {步骤2}

**示例**：
```rust
// 示例代码
pub fn new-_feature() -> Result<()> {
    // 实现
}
```

---

## 📚 相关文档

- [主架构文档](../architecture/architecture.md)
- `{module}.md` - Lib 层模块架构文档（示例：`pr.md`、`jira.md`）
- `{MODULE}.md` - 命令层模块架构文档（示例：`pr.md`、`config.md`）

---

## 📋 使用示例

### 基本使用

```rust
use workflow::{Module};

// 基本使用示例
let result = Module::method()?;
```

### {场景}使用

```rust
// 场景使用示例
```

---

## ✅ 总结

{模块名}模块采用清晰的{设计模式}设计：

1. **特性1**：{说明}
2. **特性2**：{说明}

**设计优势**：
- ✅ {优势1}
- ✅ {优势2}

{可选：当前实现状态、配置说明等}
```

</details>

---

## 🎯 可选章节

根据模块特性，可以添加以下可选章节：

### 使用场景

```markdown
## 🔄 使用场景

### {场景名}

1. **场景描述**：
   - {步骤1}
   - {步骤2}
```

### 设计决策

```markdown
## 💡 设计决策

### 为什么{设计选择}？

- **原因**：{原因说明}
- **好处**：{好处说明}
- **代价**：{代价说明}
```

### 代码质量特性

```markdown
## 🎨 代码质量特性

### 已实现的优化

1. **优化1**：
   - {说明}
```

### 安全性考虑

```markdown
## 🔒 安全性考虑

### {安全方面}

- **{方面1}**：{说明}
```

### 实现细节

```markdown
## 🔧 实现细节

### {细节名称}

{详细说明}
```

### 模块依赖关系

```markdown
## 📊 模块依赖关系

```
{依赖关系图}
```
```

---

## 📋 章节检查清单

创建或更新文档时，使用以下清单：

### 统一架构文档（architecture.template）

#### 必选章节

- [ ] 📋 概述（包含 Lib 层和 Commands 层的统一描述、模块统计）
- [ ] 📁 Lib 层架构（如果存在，包含模块结构、依赖和模块集成）
- [ ] 🏗️ Lib 层架构设计（如果存在，包含设计原则、核心组件、设计模式、错误处理）
- [ ] 📁 Commands 层架构（如果存在，包含 CLI 入口层、命令封装层、依赖模块）
- [ ] 🔄 集成关系（包含 Lib 层和 Commands 层的协作、调用流程、数据流）
- [ ] 📚 相关文档（至少 2-3 个链接）
- [ ] ✅ 总结（包含设计优势）

#### 推荐章节

- [ ] 🎯 核心功能（包含主要功能的详细说明）
- [ ] 🏗️ Commands 层架构设计（如果存在，包含设计模式、错误处理）
- [ ] 📋 使用示例（至少 2 个命令的示例）
- [ ] 📝 扩展性（包含添加新功能或新命令的步骤）

#### 可选章节

- [ ] 其他扩展场景说明

### 指南文档（guideline.template）

#### 必选章节

- [ ] 📋 目录（包含所有主要章节的链接）
- [ ] 📋 概述（说明文档目的、适用范围、核心概念）
- [ ] 主要章节（根据文档类型包含相应章节）
- [ ] **最后更新**时间戳（格式：`**最后更新**: YYYY-MM-DD`）

#### 推荐章节

- [ ] 🔍 故障排除（常见问题和解决方案）
- [ ] 📚 相关文档（至少 2-3 个相关文档链接）
- [ ] ✅ 检查清单（使用本指南时的检查项）

#### 可选章节

- [ ] 使用场景（具体的使用场景和步骤）
- [ ] 配置示例（配置文件的示例）
- [ ] 代码示例（代码使用示例）
- [ ] 最佳实践（推荐的做法）

### 需求文档（requirement.template）

#### 必选章节

- [ ] 📋 目录（包含所有主要章节的链接）
- [ ] 📋 概述（说明文档目的、当前状态、目标）
- [ ] 当前状态（状态标记、实现度、优先级、分类）
- [ ] 任务清单（待实现功能列表）
- [ ] **最后更新**时间戳（格式：`**最后更新**: YYYY-MM-DD`）

#### 推荐章节

- [ ] 📊 任务统计（已完成、进行中、待实施的数量统计）
- [ ] 实施计划（分阶段的实施步骤）
- [ ] 📚 相关文档（至少 2-3 个相关文档链接）
- [ ] ✅ 检查清单（实施本需求时的检查项）

#### 可选章节

- [ ] 已完成项（已完成的功能列表）
- [ ] 待实现项（待实现的功能列表）
- [ ] 优先级说明（高/中/低优先级分类）
- [ ] 时间估算（各阶段的时间估算）
- [ ] 依赖关系（与其他需求的依赖关系）
- [ ] 🔍 故障排除（实施过程中的问题和解决方案）

### 检查工作流文档（review-workflow.template）

#### 必选章节

- [ ] 📖 相关指南/检查指南体系（指南类型对照表）
- [ ] 📋 快速导航（核心检查、详细检查、报告和帮助）
- [ ] 🚀 检查步骤（检查流程图、详细检查步骤）
- [ ] 🎯 检查概述/检查目标（检查目标表格、检查范围表格）
- [ ] **最后更新**时间戳（格式：`**最后更新**: YYYY-MM-DD`）

### 指南文档（guideline.template）

#### 必选章节

- [ ] 📋 目录（包含所有主要章节的链接）
- [ ] 📋 概述（说明文档目的、适用范围、核心概念）
- [ ] 主要章节（根据文档类型包含相应章节）
- [ ] **最后更新**时间戳（格式：`**最后更新**: YYYY-MM-DD`）

#### 推荐章节

- [ ] 🔍 故障排除（常见问题和解决方案）
- [ ] 📚 相关文档（至少 2-3 个相关文档链接）
- [ ] ✅ 检查清单（使用本指南时的检查项）

**注意**：`development.template` 已合并到 `guideline.template`，现在统一使用 `guideline.template` 作为通用指南文档模板。

### 核心规范文档（development-core.template）

#### 必选章节

- [ ] 📋 目录（包含所有主要章节的链接）
- [ ] 📋 概述（核心原则、使用场景、快速参考）
- [ ] 规范章节（规则、示例、最佳实践）
- [ ] **最后更新**时间戳（格式：`**最后更新**: YYYY-MM-DD`）

#### 推荐章节

- [ ] 🔍 故障排除（常见问题和解决方案）
- [ ] 📚 相关文档（至少 2-3 个相关文档链接）
- [ ] ✅ 检查清单（使用本规范时的检查项）

### 开发工作流文档（development-workflow.template）

#### 必选章节

- [ ] 📖 相关指南（指南类型对照表）
- [ ] 📋 快速导航（核心步骤、详细步骤、验证和完成）
- [ ] 🚀 工作流步骤（工作流程图、详细步骤）
- [ ] 🎯 工作流概述（工作流目标、范围、时间投入）
- [ ] ⚡ 快速检查清单
- [ ] **最后更新**时间戳（格式：`**最后更新**: YYYY-MM-DD`）

#### 推荐章节

- [ ] ✅ 验证步骤（验证清单、验证方法、验证标准）
- [ ] 📄 后续步骤（完成后的操作）
- [ ] ❓ 常见问题（故障排除）
- [ ] 📚 相关文档（至少 2-3 个相关文档链接）

### 参考文档（development-reference.template）

#### 必选章节

- [ ] 📋 目录（包含所有主要章节的链接）
- [ ] 📋 概述（文档目的、适用范围、核心概念、快速参考）
- [ ] 详细章节（规则、实现要求、使用场景）
- [ ] 最佳实践（实践说明、适用场景、实现方法）
- [ ] **最后更新**时间戳（格式：`**最后更新**: YYYY-MM-DD`）

#### 推荐章节

- [ ] 🔍 故障排除（常见问题和解决方案）
- [ ] 📚 相关文档（至少 2-3 个相关文档链接）
- [ ] ✅ 检查清单（使用本指南时的检查项）

#### 推荐章节

- [ ] ⏱️ 时间投入规划（检查类型、时间投入、适用场景表格）
- [ ] 📊 检查进度指示器（ASCII 进度条）
- [ ] ⏰ 何时进行深入检查/检查场景（场景对照表、详细场景指南）
- [ ] 📄 生成检查报告（报告生成步骤、报告模板、报告文件位置）
- [ ] 📚 相关文档（至少 2-3 个相关文档链接）

#### 可选章节

- [ ] 🔗 跨领域问题关联分析（问题关联分析）
- [ ] 📅 定期审查工作流（检查频率建议、检查计划）
- [ ] ❓ 常见问题/故障排除（常见问题和解决方案）
- [ ] ✅ 检查清单（使用本指南时的检查项）

### 检查指南文档（review-guide.template）

#### 必选章节

- [ ] 🎯 检查目标（主要目标、检查范围、检查原则）
- [ ] 🔄 检查流程（详细检查步骤）
- [ ] 📋 检查清单（各类型检查清单）
- [ ] **最后更新**时间戳（格式：`**最后更新**: YYYY-MM-DD`）

#### 推荐章节（详细检查指南）

- [ ] 🎯 核心原则（检查重点、关键目标）
- [ ] 📋 目录（包含所有主要章节的链接）
- [ ] ✅ 详细检查章节（检查方法、检查清单、示例分析）
- [ ] 🛠️ 工具和测试（自动化检查工具、手动检查工具）
- [ ] 📚 参考文档（至少 2-3 个相关文档链接）

#### 推荐章节（快速参考）

- [ ] 📊 检查结果汇总/问题优先级（问题优先级表格）
- [ ] 📁 报告文件位置/常用工具函数（报告位置表格、工具函数示例）
- [ ] 🆘 故障排除（常见问题快速解决表格）

#### 可选章节

- [ ] 📊 检查结果统计（检查结果统计表格）
- [ ] 🔄 更新历史（更新历史表格）
- [ ] ✅ 检查清单（使用本指南时的检查项）

### 格式检查（适用于所有文档）

- [ ] 使用统一的 emoji 图标
- [ ] 使用统一的代码块格式
- [ ] 使用统一的列表格式
- [ ] 使用统一的分隔线（`---`）
- [ ] 添加"最后更新"时间戳（格式：`**最后更新**: YYYY-MM-DD`）
- [ ] 目录中的链接与实际章节标题一致

---

## 📅 文档时间戳

### 问题说明

在生成文档时，AI 助手（如 Cursor）可能会在文档末尾添加"最后更新"时间戳，但时间可能不正确（时区、格式等问题）。

### 解决方案

项目提供了专门的日期时间工具函数，用于生成正确格式的文档时间戳。

### 使用方法

#### 在 Rust 代码中使用

```rust
use workflow::base::util::date::format-_last-_updated;

// 生成"最后更新"时间戳（格式：YYYY-MM-DD）
let timestamp = format-_last-_updated();
// 输出示例：2025-12-09
```

#### 在文档中使用（推荐格式）

在 Markdown 文档末尾添加时间戳时，使用以下格式：

```markdown
---

**最后更新**: 2025-12-09
```

或者：

```markdown
---

*最后更新: 2025-12-09*
```

### 对于 AI 助手（Cursor/Claude 等）

**重要提示**：当生成或更新文档时，请在文档末尾添加"最后更新"时间戳。

#### 推荐做法

1. **使用当前日期**：使用当前本地日期（格式：`YYYY-MM-DD`）
2. **格式统一**：使用以下格式之一：
   - `**最后更新**: YYYY-MM-DD`
   - `*最后更新: YYYY-MM-DD*`
3. **位置**：放在文档末尾，分隔线之后

#### 获取当前日期的方法

**命令行方式**：

- **Unix/macOS/Linux**：
  ```bash
  date +%Y-%m-%d
  # 输出：2025-12-09
  ```

- **Windows PowerShell**：
  ```powershell
  Get-Date -Format "yyyy-MM-dd"
  # 输出：2025-12-09
  ```

- **Windows CMD**：
  ```cmd
  powershell -Command "Get-Date -Format 'yyyy-MM-dd'"
  ```

**代码方式**：

在 Rust 代码中使用：
```rust
use workflow::base::util::date::format-_last-_updated;

let timestamp = format-_last-_updated();
// 输出：2025-12-09
```

### 高级用法

如果需要更多控制，可以使用其他函数：

```rust
use workflow::base::util::date::{
    format-_document-_timestamp, format-_last-_updated-_with-_time, DateFormat, Timezone,
};

// 只生成日期（本地时区）
let date = format-_document-_timestamp(DateFormat::DateOnly, Timezone::Local);

// 生成日期和时间（本地时区）
let datetime = format-_last-_updated-_with-_time();

// 使用 UTC 时区
let utc-_date = format-_document-_timestamp(DateFormat::DateOnly, Timezone::Utc);

// ISO 8601 格式
let iso = format-_document-_timestamp(DateFormat::Iso8601, Timezone::Local);
```

### 支持的格式

| 格式 | 函数 | 输出示例 |
|------|------|----------|
| 日期（YYYY-MM-DD） | `format-_last-_updated()` | `2025-12-09` |
| 日期时间（YYYY-MM-DD HH:MM:SS） | `format-_last-_updated-_with-_time()` | `2025-12-09 14:30:00` |
| ISO 8601 | `format-_document-_timestamp(DateFormat::Iso8601, ...)` | `2025-12-09T14:30:00+08:00` |

### 注意事项

1. **时区**：默认使用本地时区（`Timezone::Local`）
2. **格式一致性**：建议在项目中统一使用 `YYYY-MM-DD` 格式
3. **自动更新**：时间戳不会自动更新，需要在编辑文档时手动更新

---

## 📊 文档长度参考

### 统一架构文档

- **最小文档**：包含必选章节 + 部分推荐章节，约 300-450 行
- **标准文档**：包含必选章节 + 所有推荐章节，约 500-800 行
- **完整文档**：包含所有章节 + 可选章节，约 800-1200 行

**注意**：如果模块只有 Lib 层或只有 Commands 层，文档长度会相应减少。

### 指南文档

- **最小文档**：包含必选章节，约 100-200 行
- **标准文档**：包含必选章节 + 推荐章节，约 300-600 行
- **完整文档**：包含所有章节 + 可选章节，约 600-1000 行

**注意**：文档长度取决于文档类型和内容复杂度。配置指南类文档通常较短（200-400 行），开发规范类文档通常较长（500-1000+ 行）。

### 需求文档

- **最小文档**：包含必选章节，约 150-300 行
- **标准文档**：包含必选章节 + 推荐章节，约 300-600 行
- **完整文档**：包含所有章节 + 可选章节，约 600-1000+ 行

**注意**：文档长度取决于需求复杂度。简单的待办事项文档通常较短（200-400 行），复杂的需求分析和实施计划文档通常较长（500-1000+ 行）。

### 工作流文档

- **最小文档**：包含必选章节，约 200-400 行
- **标准文档**：包含必选章节 + 推荐章节，约 500-1000 行
- **完整文档**：包含所有章节 + 可选章节，约 1000-1500+ 行

**注意**：文档长度取决于工作流复杂度。简单的快速检查工作流通常较短（300-600 行），复杂的综合深入检查工作流通常较长（1000-1500+ 行）。

### 检查指南文档

- **最小文档（快速参考）**：包含必选章节 + 快速参考推荐章节，约 100-200 行
- **标准文档（详细检查指南）**：包含必选章节 + 详细检查指南推荐章节，约 400-800 行
- **完整文档（详细检查指南）**：包含所有章节 + 可选章节，约 800-1200+ 行

**注意**：文档长度取决于文档类型。快速参考文档通常较短（100-200 行），详细的检查指南文档通常较长（500-1200+ 行）。

---

## 🔗 相关文档

- [统一架构模板文件](../templates/architecture/architecture.template) - 统一架构文档模板（Lib 层 + Commands 层）
- [指南文档模板文件](../templates/development/guideline.template) - 指南文档模板（开发规范、配置指南等，已合并 development.template）
- [核心规范文档模板文件](../templates/development/development-core.template) - 核心规范文档模板（日常必读的核心规范）
- [开发工作流文档模板文件](../templates/development/development-workflow.template) - 开发工作流文档模板（AI 工作流文档）
- [检查工作流文档模板文件](../templates/review/review-workflow.template) - 检查工作流文档模板（AI 工作流文档）
- [参考文档模板文件](../templates/development/development-reference.template) - 参考文档模板（详细参考文档）
- [检查指南模板文件](../templates/review/review-guide.template) - 检查指南模板（专门检查指南、快速参考等）
- [需求文档模板文件](../templates/requirements/requirement.template) - 需求文档模板（待办事项、需求分析等）
- [主架构文档](../architecture/architecture.md) - 总体架构设计文档
- [文档时间戳维护指南](./document-timestamp.md) - 文档更新时间维护指南

---

*模板版本：3.4*
*最后更新: 2025-12-23*

---

## 📝 更新说明

### v3.3 工作流和检查指南模板更新

新增工作流和检查指南模板，主要改进：

1. **新增检查工作流文档模板**：
   - 创建 `docs/guidelines/templates/review-workflow.template`
   - 适用于检查工作流文档，如提交前检查、综合深入检查等
   - 包含 Mermaid 流程图、快速导航、检查步骤、进度指示器、报告生成等

2. **新增检查指南模板**：
   - 创建 `docs/guidelines/templates/review-guide.template`
   - 适用于专门检查指南和快速参考文档
   - 灵活支持详细检查指南和快速参考两种模式

3. **文档编写指南更新**：
   - 添加检查工作流文档模板的使用说明
   - 添加检查指南模板的使用说明
   - 添加检查工作流文档和检查指南文档的章节检查清单
   - 添加检查工作流文档和检查指南文档的长度参考
   - 更新相关文档链接

4. **模板统一**：
   - 将所有指南类模板统一到 `docs/guidelines/templates/` 目录
   - 架构文档模板：`docs/architecture/templates/architecture.template`
   - 指南文档模板：`docs/guidelines/templates/guideline.template`
   - 需求文档模板：`docs/requirements/templates/requirement.template`
   - 检查工作流文档模板：`docs/guidelines/templates/review-workflow.template`
   - 检查指南模板：`docs/guidelines/templates/review-guide.template`

### v3.2 需求文档模板更新

新增需求文档模板，主要改进：

1. **新增需求文档模板**：
   - 创建 `docs/requirements/templates/requirement.template`
   - 适用于待办事项、需求分析、实施计划等需求类文档
   - 提供统一的文档结构，包含状态标记、任务清单、实施计划等

2. **文档编写指南更新**：
   - 添加需求文档模板的使用说明
   - 添加需求文档的章节检查清单
   - 添加需求文档的长度参考
   - 更新相关文档链接

3. **模板分类**：
   - 架构文档模板：`docs/architecture/templates/architecture.template`
   - 指南文档模板：`docs/guidelines/templates/guideline.template`
   - 需求文档模板：`docs/requirements/templates/requirement.template`

### v3.1 指南文档模板更新

新增指南文档模板，主要改进：

1. **新增指南文档模板**：
   - 创建 `docs/guidelines/templates/guideline.template`
   - 适用于开发规范、配置指南、使用指南等指南类文档
   - 提供统一的文档结构，包含目录、概述、章节、故障排除等

2. **文档编写指南更新**：
   - 添加指南文档模板的使用说明
   - 添加指南文档的章节检查清单
   - 添加指南文档的长度参考
   - 更新相关文档链接

3. **模板分类**：
   - 架构文档模板：`docs/architecture/templates/architecture.template`
   - 指南文档模板：`docs/guidelines/templates/guideline.template`

### v3.0 统一模板更新

模板已统一，主要改进：

1. **统一架构文档**：
   - 合并 Lib 层和 Commands 层的模板为统一模板
   - 模板迁移到 `docs/architecture/templates/` 目录
   - 模板名称改为 `architecture.template`

2. **结构优化**：
   - 概述部分同时描述 Lib 层和 Commands 层
   - 添加"集成关系"章节，说明两层协作方式
   - 支持模块只有 Lib 层或只有 Commands 层的情况

3. **与实际文档对齐**：
   - 模板结构与现有文档（`pr.md`、`jira.md`、`branch.md` 等）保持一致
   - 反映实际文档已经合并 Lib 层和 Commands 层的现状

### v2.0 简化更新（已废弃）

v2.0 版本的模板已废弃，现在统一使用 v3.0+ 的统一模板。

---

**最后更新**: 2025-12-23
