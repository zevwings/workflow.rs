# Template 模块架构文档

## 📋 概述

Template 模块（`lib/template/`）是 Workflow CLI 的核心库模块，提供模板渲染功能，支持分支命名模板、PR body 模板、Commit 消息模板等。使用 Handlebars 模板引擎，支持从全局配置和项目级配置加载模板。

**注意**：本文档仅描述 `lib/template/` 模块的架构。关于模板配置的详细内容，请参考 [Repo 模块架构文档](./REPO_ARCHITECTURE.md)。

**模块统计：**
- 总代码行数：约 488 行
- 文件数量：4 个
- 主要组件：`TemplateEngine`、`TemplateConfig`、`TemplateEngineType`、模板变量结构体
- 支持功能：模板加载、模板渲染、多级配置（全局+项目级）

---

## 📁 模块结构

### 核心模块文件

```
src/lib/template/
├── mod.rs          # Template 模块声明和导出 (15行)
├── config.rs       # 模板配置管理 (306行)
├── engine.rs       # 模板引擎封装 (88行)
└── vars.rs         # 模板变量定义 (82行)
```

### 依赖模块

- **`lib/base/settings/`**：路径管理
  - `Paths::project-_config()` - 获取项目配置文件路径
  - `Paths::global-_config()` - 获取全局配置文件路径
- **`handlebars`**：Handlebars 模板引擎（第三方库）

### 模块集成

- **`lib/branch/`**：分支命名
  - 使用 `TemplateConfig::load-_branch-_template()` 加载分支模板
  - 使用 `TemplateEngine` 渲染分支名
- **`lib/pr/`**：PR 创建
  - 使用 `TemplateConfig::load-_pull-_request-_template()` 加载 PR 模板
  - 使用 `TemplateEngine` 渲染 PR body
- **`commands/commit/`**：提交管理
  - 使用 `TemplateConfig::load-_commit-_template()` 加载提交模板
  - 使用 `TemplateEngine` 渲染提交消息
- **`commands/repo/`**：仓库配置管理
  - 使用 `TemplateConfig::load()` 加载模板配置用于显示

---

## 🏗️ 架构设计

### 设计原则

1. **职责单一**：专注于模板渲染功能
2. **配置驱动**：模板从配置文件加载，支持多级配置
3. **类型安全**：使用结构化的模板变量，确保类型安全
4. **可扩展性**：支持添加新的模板类型和变量

### 核心组件

#### 1. TemplateEngine 结构体

**职责**：提供模板渲染的统一接口

**主要方法**：
- `new()` - 创建新的模板引擎实例
- `register-_template()` - 注册模板
- `render()` - 渲染已注册的模板
- `render-_string()` - 直接渲染模板字符串（无需注册）

**关键特性**：
- 使用 Handlebars 作为底层引擎
- 禁用严格模式（允许未定义的变量）
- 禁用 HTML 转义（模板输出为纯文本）
- 支持临时模板渲染（`render-_string`）

**使用场景**：
- 分支名生成：渲染分支命名模板
- PR body 生成：渲染 PR 模板
- 提交消息生成：渲染提交模板

#### 2. TemplateConfig 结构体

**职责**：管理模板配置的加载

**主要方法**：
- `load()` - 加载模板配置（全局+项目级合并）
- `load-_branch-_template()` - 加载分支模板（根据分支类型）
- `load-_branch-_template-_by-_type()` - 根据分支类型加载模板
- `load-_commit-_template()` - 加载提交模板
- `load-_pull-_request-_template()` - 加载 PR 模板

**关键特性**：
- 支持多级配置（全局配置 + 项目级配置）
- 项目级配置优先于全局配置
- 支持默认模板（如果未配置，使用内置默认值）
- 支持类型特定模板（如 feature、bugfix 等分支类型）

**配置结构**：
- `engine: String` - 模板引擎类型（默认：`"handlebars"`）
- `branch: BranchTemplates` - 分支模板配置
- `commit: CommitTemplates` - 提交模板配置
- `pull-_requests: PullRequestsTemplates` - PR 模板配置

#### 3. 模板变量结构体

**职责**：定义模板变量的数据结构

**主要类型**：
- `BranchTemplateVars` - 分支模板变量
- `CommitTemplateVars` - 提交模板变量
- `PullRequestTemplateVars` - PR 模板变量
- `ChangeTypeItem` - 变更类型项（用于 PR 模板）

**关键特性**：
- 使用 `serde::Serialize` 支持序列化
- 使用 `skip-_serializing-_if` 控制可选字段的序列化
- 类型安全，确保模板变量与模板匹配

---

## 🔄 核心功能

### 1. 模板配置加载 (`TemplateConfig::load()`)

**功能**：加载模板配置（合并全局和项目级配置）

**流程**：
1. 加载全局配置（`~/.workflow/config/workflow.toml`）
2. 加载项目级配置（`.workflow/config.toml`）
3. 合并配置（项目级配置优先）
4. 应用默认值（如果未配置）

**配置优先级**：
1. 项目级配置（`.workflow/config.toml`）
2. 全局配置（`~/.workflow/config/workflow.toml`）
3. 内置默认值

### 2. 分支模板加载 (`load-_branch-_template()`)

**功能**：根据分支类型加载分支模板

**流程**：
1. 加载模板配置
2. 根据分支类型选择模板：
   - `feature` → `config.branch.feature` 或 `config.branch.default`
   - `bugfix` → `config.branch.bugfix` 或 `config.branch.default`
   - `hotfix` → `config.branch.hotfix` 或 `config.branch.default`
   - `refactoring` → `config.branch.refactoring` 或 `config.branch.default`
   - `chore` → `config.branch.chore` 或 `config.branch.default`
   - 其他 → `config.branch.default`
3. 返回模板字符串

### 3. 模板渲染 (`TemplateEngine::render-_string()`)

**功能**：渲染模板字符串

**流程**：
1. 创建临时模板名称（基于时间戳）
2. 注册模板
3. 渲染模板（使用提供的变量）
4. 返回渲染结果

**关键特性**：
- 无需预先注册模板
- 自动处理临时模板的注册和清理
- 支持 Handlebars 语法

### 4. 模板变量准备

**功能**：准备模板变量结构体

**分支模板变量** (`BranchTemplateVars`)：
- `jira-_key: Option<String>` - JIRA ticket ID
- `jira-_summary: Option<String>` - JIRA ticket 摘要
- `summary-_slug: Option<String>` - 摘要的 slug 格式
- `jira-_type: Option<String>` - JIRA ticket 类型

**提交模板变量** (`CommitTemplateVars`)：
- `commit-_type: String` - 提交类型（feat、fix 等）
- `scope: Option<String>` - 提交范围
- `subject: String` - 提交主题
- `body: Option<String>` - 提交正文
- `jira-_key: Option<String>` - JIRA ticket ID
- `use-_scope: bool` - 是否使用 scope

**PR 模板变量** (`PullRequestTemplateVars`)：
- `jira-_key: Option<String>` - JIRA ticket ID
- `jira-_summary: Option<String>` - JIRA ticket 摘要
- `jira-_description: Option<String>` - JIRA ticket 描述
- `jira-_type: Option<String>` - JIRA ticket 类型
- `jira-_service-_address: Option<String>` - JIRA 服务地址
- `change-_types: Vec<ChangeTypeItem>` - 变更类型列表
- `short-_description: Option<String>` - 简短描述
- `dependency: Option<String>` - 依赖信息

---

## 📝 模板语法

### Handlebars 语法

模板使用 Handlebars 语法，支持以下特性：

#### 变量插值

```handlebars
{{jira-_key}}-{{summary-_slug}}
```

#### 条件语句

```handlebars
{{#if jira-_key}}
  {{jira-_key}}: {{subject}}
{{else}}
  # {{subject}}
{{/if}}
```

#### 嵌套条件

```handlebars
{{#if jira-_key}}
  {{jira-_key}}: {{subject}}
{{else}}
  {{#if use-_scope}}
    {{commit-_type}}({{scope}}): {{subject}}
  {{else}}
    # {{subject}}
  {{/if}}
{{/if}}
```

#### 循环

```handlebars
{{#each change-_types}}
  {{#if selected}}
    - {{name}}
  {{/if}}
{{/each}}
```

### 默认模板示例

#### 分支模板

```handlebars
{{jira-_key}}-{{summary-_slug}}
```

或带前缀：

```handlebars
feature/{{jira-_key}}-{{summary-_slug}}
```

#### 提交模板

```handlebars
{{#if jira-_key}}{{jira-_key}}: {{subject}}{{else}}{{#if use-_scope}}{{commit-_type}}{{#if scope}}({{scope}}){{/if}}: {{subject}}{{else}}# {{subject}}{{/if}}{{/if}}

{{#if body}}{{body}}{{/if}}

{{#if jira-_key}}Closes {{jira-_key}}{{/if}}
```

#### PR 模板

```handlebars
## Description

{{jira-_summary}}

{{#if jira-_description}}
{{jira-_description}}
{{/if}}

## Change Types

{{#each change-_types}}
  {{#if selected}}
  - {{name}}
  {{/if}}
{{/each}}

{{#if short-_description}}
## Short Description

{{short-_description}}
{{/if}}

{{#if dependency}}
## Dependencies

{{dependency}}
{{/if}}

{{#if jira-_key}}
## Related Ticket

[{{jira-_key}}]({{jira-_service-_address}}/browse/{{jira-_key}})
{{/if}}
```

---

## 🔍 错误处理

### 错误类型

1. **配置加载错误**：
   - 配置文件不存在
   - 配置文件格式错误
   - 配置解析失败

2. **模板渲染错误**：
   - 模板语法错误
   - 变量未定义（在严格模式下）
   - 模板注册失败

### 错误处理策略

- **配置文件不存在**：使用默认配置（不报错）
- **模板语法错误**：返回错误，提示用户检查模板
- **变量未定义**：在非严格模式下，未定义变量渲染为空字符串

---

## 📚 使用示例

### 加载并渲染分支模板

```rust
use workflow::template::{TemplateConfig, TemplateEngine, BranchTemplateVars};

// 加载模板
let template = TemplateConfig::load-_branch-_template(Some("feature"))?;

// 准备变量
let vars = BranchTemplateVars {
    jira-_key: Some("PROJ-123".to-_string()),
    jira-_summary: Some("Add new feature".to-_string()),
    summary-_slug: Some("add-new-feature".to-_string()),
    jira-_type: Some("Feature".to-_string()),
};

// 渲染模板
let engine = TemplateEngine::new();
let branch-_name = engine.render-_string(&template, &vars)?;
println!("Branch name: {}", branch-_name);
```

### 加载并渲染提交模板

```rust
use workflow::template::{TemplateConfig, TemplateEngine, CommitTemplateVars};

// 加载模板
let template = TemplateConfig::load-_commit-_template()?;

// 准备变量
let vars = CommitTemplateVars {
    commit-_type: "feat".to-_string(),
    scope: Some("api".to-_string()),
    subject: "Add user authentication".to-_string(),
    body: Some("Implement JWT-based authentication".to-_string()),
    jira-_key: Some("PROJ-123".to-_string()),
    use-_scope: true,
};

// 渲染模板
let engine = TemplateEngine::new();
let commit-_message = engine.render-_string(&template, &vars)?;
println!("Commit message:\n{}", commit-_message);
```

### 加载并渲染 PR 模板

```rust
use workflow::template::{TemplateConfig, TemplateEngine, PullRequestTemplateVars, ChangeTypeItem};

// 加载模板
let template = TemplateConfig::load-_pull-_request-_template()?;

// 准备变量
let vars = PullRequestTemplateVars {
    jira-_key: Some("PROJ-123".to-_string()),
    jira-_summary: Some("Add new feature".to-_string()),
    jira-_description: Some("Detailed description...".to-_string()),
    jira-_type: Some("Feature".to-_string()),
    jira-_service-_address: Some("https://jira.example.com".to-_string()),
    change-_types: vec![
        ChangeTypeItem { name: "Feature".to-_string(), selected: true },
        ChangeTypeItem { name: "Bug Fix".to-_string(), selected: false },
    ],
    short-_description: Some("Brief description".to-_string()),
    dependency: None,
};

// 渲染模板
let engine = TemplateEngine::new();
let pr-_body = engine.render-_string(&template, &vars)?;
println!("PR body:\n{}", pr-_body);
```

---

## 🔄 与其他模块的集成

### 与 Repo 模块的集成

- Template 模块从 Repo 模块管理的配置文件中加载模板
- 支持全局配置和项目级配置
- 项目级配置优先于全局配置

### 与 Branch 模块的集成

- Branch 模块使用 Template 模块渲染分支名
- 支持从 JIRA ticket 信息生成分支名
- 支持不同类型分支的模板（feature、bugfix 等）

### 与 PR 模块的集成

- PR 模块使用 Template 模块渲染 PR body
- 支持从 JIRA ticket 信息和用户输入生成 PR body
- 支持变更类型列表的渲染

### 与 Commit 模块的集成

- Commit 模块使用 Template 模块渲染提交消息
- 支持 Conventional Commits 格式
- 支持 JIRA ticket 集成

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [Repo 模块架构文档](./REPO_ARCHITECTURE.md) - 配置管理
- [Branch 模块架构文档](./BRANCH_ARCHITECTURE.md) - 分支管理
- [PR 模块架构文档](./PR_ARCHITECTURE.md) - PR 管理

---

## ✅ 总结

Template 模块采用清晰的设计原则：

1. **配置驱动**：模板从配置文件加载，支持多级配置
2. **类型安全**：使用结构化的模板变量，确保类型安全
3. **灵活扩展**：支持添加新的模板类型和变量
4. **统一接口**：提供统一的模板渲染接口

**设计优势**：
- ✅ 配置灵活，支持全局和项目级配置
- ✅ 类型安全，减少运行时错误
- ✅ 易于扩展，支持新模板类型
- ✅ 统一接口，便于使用和维护

**当前实现状态**：
- ✅ 模板配置加载功能完整实现
- ✅ 模板渲染功能完整实现
- ✅ 分支模板支持完整实现
- ✅ 提交模板支持完整实现
- ✅ PR 模板支持完整实现

---

**最后更新**: 2025-12-16
