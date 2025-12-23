# 模板配置指南

> 本文档描述了 Workflow CLI 的模板系统配置和使用方法，包括分支命名模板、提交消息模板和 PR 正文模板。

---

## 📋 目录

- [概述](#-概述)
- [配置位置](#-配置位置)
- [模板类型](#-模板类型)
- [模板引擎](#-模板引擎)
- [配置示例](#-配置示例)
- [模板变量](#-模板变量)
- [Handlebars 语法](#-handlebars-语法)
- [配置优先级](#-配置优先级)
- [使用场景](#-使用场景)
- [故障排除](#-故障排除)

---

## 📋 概述

Workflow CLI 使用模板系统来生成：
- **分支名称**：根据 JIRA ticket 信息自动生成分支名
- **提交消息**：根据 PR 信息生成符合规范的提交消息
- **PR 正文**：根据变更类型和 JIRA 信息生成 PR 描述

### 设计原则

1. **灵活性**：支持全局配置和项目级配置
2. **可扩展性**：使用 Handlebars 模板引擎，支持条件判断和循环
3. **向后兼容**：提供默认模板，无需配置即可使用
4. **优先级**：项目级配置优先于全局配置

---

## 📁 配置位置

模板配置可以存储在两个位置：

### 1. 全局配置

**路径**：`~/.workflow/config/workflow.toml`（macOS/Linux）或 `%APPDATA%\workflow\config\workflow.toml`（Windows）

**用途**：适用于所有项目的默认模板配置

**示例**：

```toml
[template]
engine = "handlebars"

[template.branch]
default = "{{jira-_key}}-{{summary-_slug}}"
feature = "feature/{{jira-_key}}-{{summary-_slug}}"
bugfix = "bugfix/{{jira-_key}}-{{summary-_slug}}"
hotfix = "hotfix/{{jira-_key}}-{{summary-_slug}}"
refactoring = "refactoring/{{jira-_key}}-{{summary-_slug}}"
chore = "chore/{{jira-_key}}-{{summary-_slug}}"

[template.commit]
default = """{{#if jira-_key}}{{jira-_key}}: {{subject}}{{else}}{{#if use-_scope}}{{commit-_type}}{{#if scope}}({{scope}}){{/if}}: {{subject}}{{else}}# {{subject}}{{/if}}{{/if}}

{{#if body}}{{body}}{{/if}}

{{#if jira-_key}}Closes {{jira-_key}}{{/if}}"""
use-_scope = false

[template.pull-_requests]
default = """
# PR Ready

## Types of changes

{{#each change-_types}}
- [{{#if this.selected}}x{{else}} {{/if}}] {{this.name}}
{{/each}}

{{#if short-_description}}
#### Short description:

{{short-_description}}
{{/if}}

{{#if jira-_key}}
{{#if jira-_service-_address}}
#### Jira Link:

{{jira-_service-_address}}/browse/{{jira-_key}}
{{/if}}
{{/if}}

{{#if dependency}}
#### Dependency

{{dependency}}
{{/if}}
"""
```

### 2. 项目级配置

**路径**：`.workflow/config.toml`（项目根目录）

**用途**：覆盖全局配置，适用于特定项目的模板配置

**优先级**：项目级配置优先于全局配置

**示例**：

```toml
[template]
engine = "handlebars"

[template.branch]
default = "{{jira-_key}}-{{summary-_slug}}"
feature = "feat/{{jira-_key}}-{{summary-_slug}}"

[template.commit]
use-_scope = true
```

---

## 🎨 模板类型

### 1. 分支命名模板 (`[template.branch]`)

用于根据 JIRA ticket 信息生成分支名称。

#### 支持的模板类型

- **`default`**：默认模板（必需）
- **`feature`**：功能分支模板（可选）
- **`bugfix`**：Bug 修复分支模板（可选）
- **`hotfix`**：热修复分支模板（可选）
- **`refactoring`**：重构分支模板（可选）
- **`chore`**：杂务分支模板（可选）

#### 分支类型映射

- JIRA 类型 `Feature`、`Story`、`Epic` → 使用 `feature` 模板
- JIRA 类型 `Bug` → 使用 `bugfix` 模板
- JIRA 类型 `Hotfix` → 使用 `hotfix` 模板
- 分支类型 `refactoring` → 使用 `refactoring` 模板
- 分支类型 `chore` → 使用 `chore` 模板
- 其他情况 → 使用 `default` 模板

#### 默认模板

```toml
[template.branch]
default = "{{jira-_key}}-{{summary-_slug}}"
feature = "feature/{{jira-_key}}-{{summary-_slug}}"
bugfix = "bugfix/{{jira-_key}}-{{summary-_slug}}"
hotfix = "hotfix/{{jira-_key}}-{{summary-_slug}}"
refactoring = "refactoring/{{jira-_key}}-{{summary-_slug}}"
chore = "chore/{{jira-_key}}-{{summary-_slug}}"
```

### 2. 提交消息模板 (`[template.commit]`)

用于生成符合规范的提交消息。

#### 配置项

- **`default`**：提交消息模板（必需）
- **`use-_scope`**：是否使用 Conventional Commits 格式的 scope（可选，默认：`false`）

#### `use-_scope` 说明

- **`false`**（默认）：当没有 JIRA ticket 时，使用简单格式 `# {title}`
- **`true`**：当没有 JIRA ticket 时，使用 Conventional Commits 格式 `{commit-_type}({scope}): {title}`

#### 默认模板

```toml
[template.commit]
default = """{{#if jira-_key}}{{jira-_key}}: {{subject}}{{else}}{{#if use-_scope}}{{commit-_type}}{{#if scope}}({{scope}}){{/if}}: {{subject}}{{else}}# {{subject}}{{/if}}{{/if}}

{{#if body}}{{body}}{{/if}}

{{#if jira-_key}}Closes {{jira-_key}}{{/if}}"""
use-_scope = false
```

### 3. PR 正文模板 (`[template.pull-_requests]`)

用于生成 PR 描述正文。

#### 配置项

- **`default`**：PR 正文模板（必需）

#### 默认模板

```toml
[template.pull-_requests]
default = """
# PR Ready

## Types of changes

{{#each change-_types}}
- [{{#if this.selected}}x{{else}} {{/if}}] {{this.name}}
{{/each}}

{{#if short-_description}}
#### Short description:

{{short-_description}}
{{/if}}

{{#if jira-_key}}
{{#if jira-_service-_address}}
#### Jira Link:

{{jira-_service-_address}}/browse/{{jira-_key}}
{{/if}}
{{/if}}

{{#if dependency}}
#### Dependency

{{dependency}}
{{/if}}
"""
```

---

## ⚙️ 模板引擎

### 引擎类型

当前支持 **Handlebars** 模板引擎（默认）。

**配置**：

```toml
[template]
engine = "handlebars"
```

### Handlebars 特性

- ✅ 变量插值：`{{variable}}`
- ✅ 条件判断：`{{#if condition}}...{{/if}}`
- ✅ 循环：`{{#each items}}...{{/each}}`
- ✅ 嵌套条件：支持多层嵌套
- ✅ 转义：默认不转义 HTML（适合 Markdown）

---

## 📝 配置示例

### 完整配置示例

```toml
[template]
engine = "handlebars"

# 分支命名模板
[template.branch]
default = "{{jira-_key}}-{{summary-_slug}}"
feature = "feature/{{jira-_key}}-{{summary-_slug}}"
bugfix = "bugfix/{{jira-_key}}-{{summary-_slug}}"
hotfix = "hotfix/{{jira-_key}}-{{summary-_slug}}"
refactoring = "refactoring/{{jira-_key}}-{{summary-_slug}}"
chore = "chore/{{jira-_key}}-{{summary-_slug}}"

# 提交消息模板
[template.commit]
default = """{{#if jira-_key}}{{jira-_key}}: {{subject}}{{else}}{{#if use-_scope}}{{commit-_type}}{{#if scope}}({{scope}}){{/if}}: {{subject}}{{else}}# {{subject}}{{/if}}{{/if}}

{{#if body}}{{body}}{{/if}}

{{#if jira-_key}}Closes {{jira-_key}}{{/if}}"""
use-_scope = false

# PR 正文模板
[template.pull-_requests]
default = """
# PR Ready

## Types of changes

{{#each change-_types}}
- [{{#if this.selected}}x{{else}} {{/if}}] {{this.name}}
{{/each}}

{{#if short-_description}}
#### Short description:

{{short-_description}}
{{/if}}

{{#if jira-_key}}
{{#if jira-_service-_address}}
#### Jira Link:

{{jira-_service-_address}}/browse/{{jira-_key}}
{{/if}}
{{/if}}

{{#if dependency}}
#### Dependency

{{dependency}}
{{/if}}
"""
```

### 最小配置示例

如果只需要自定义部分模板，可以只配置需要的部分：

```toml
[template.branch]
feature = "feat/{{jira-_key}}-{{summary-_slug}}"

[template.commit]
use-_scope = true
```

未配置的部分将使用默认值。

---

## 🔧 模板变量

### 分支命名模板变量 (`BranchTemplateVars`)

| 变量名 | 类型 | 说明 | 示例 |
|--------|------|------|------|
| `jira-_key` | `Option<String>` | JIRA ticket ID | `"PROJ-123"` |
| `jira-_summary` | `Option<String>` | JIRA ticket 摘要 | `"Add user authentication"` |
| `summary-_slug` | `Option<String>` | JIRA ticket 摘要的 URL 友好格式 | `"add-user-authentication"` |
| `jira-_type` | `Option<String>` | JIRA ticket 类型 | `"Feature"` |

**使用示例**：

```handlebars
{{jira-_key}}-{{summary-_slug}}
```

**输出示例**：`PROJ-123-add-user-authentication`

### 提交消息模板变量 (`CommitTemplateVars`)

| 变量名 | 类型 | 说明 | 示例 |
|--------|------|------|------|
| `commit-_type` | `String` | 提交类型 | `"feat"`, `"fix"`, `"docs"` |
| `scope` | `Option<String>` | 提交范围 | `"auth"`, `"api"` |
| `subject` | `String` | 提交主题 | `"Add user authentication"` |
| `body` | `Option<String>` | 提交正文 | `"Implement OAuth2 flow"` |
| `jira-_key` | `Option<String>` | JIRA ticket ID | `"PROJ-123"` |
| `use-_scope` | `bool` | 是否使用 scope（来自配置） | `true`, `false` |

**使用示例**：

```handlebars
{{#if jira-_key}}{{jira-_key}}: {{subject}}{{else}}{{commit-_type}}{{#if scope}}({{scope}}){{/if}}: {{subject}}{{/if}}
```

**输出示例**：
- 有 JIRA ticket：`PROJ-123: Add user authentication`
- 无 JIRA ticket：`feat(auth): Add user authentication`

### PR 正文模板变量 (`PullRequestTemplateVars`)

| 变量名 | 类型 | 说明 | 示例 |
|--------|------|------|------|
| `jira-_key` | `Option<String>` | JIRA ticket ID | `"PROJ-123"` |
| `jira-_summary` | `Option<String>` | JIRA ticket 摘要 | `"Add user authentication"` |
| `jira-_description` | `Option<String>` | JIRA ticket 描述 | `"Implement OAuth2..."` |
| `jira-_type` | `Option<String>` | JIRA ticket 类型 | `"Feature"` |
| `jira-_service-_address` | `Option<String>` | JIRA 服务地址 | `"https://jira.example.com"` |
| `change-_types` | `Vec<ChangeTypeItem>` | 变更类型列表 | 见下方说明 |
| `short-_description` | `Option<String>` | 简短描述 | `"Add OAuth2 support"` |
| `dependency` | `Option<String>` | 依赖信息 | `"Depends on #456"` |

**`ChangeTypeItem` 结构**：

```rust
{
    name: String,      // 变更类型名称，如 "Bug fix"
    selected: bool     // 是否选中
}
```

**使用示例**：

```handlebars
{{#each change-_types}}
- [{{#if this.selected}}x{{else}} {{/if}}] {{this.name}}
{{/each}}
```

**输出示例**：

```markdown
- [x] Bug fix
- [ ] New feature
- [ ] Breaking change
```

---

## 📖 Handlebars 语法

### 基本语法

#### 1. 变量插值

```handlebars
{{variable-_name}}
```

**示例**：

```handlebars
{{jira-_key}}-{{summary-_slug}}
```

#### 2. 条件判断

```handlebars
{{#if condition}}
  <!-- 条件为真时显示 -->
{{else}}
  <!-- 条件为假时显示 -->
{{/if}}
```

**示例**：

```handlebars
{{#if jira-_key}}
  {{jira-_key}}: {{subject}}
{{else}}
  # {{subject}}
{{/if}}
```

#### 3. 循环

```handlebars
{{#each items}}
  <!-- 循环体 -->
  {{this.property}}
{{/each}}
```

**示例**：

```handlebars
{{#each change-_types}}
- [{{#if this.selected}}x{{else}} {{/if}}] {{this.name}}
{{/each}}
```

#### 4. 嵌套条件

```handlebars
{{#if condition1}}
  {{#if condition2}}
    <!-- 嵌套条件 -->
  {{/if}}
{{/if}}
```

**示例**：

```handlebars
{{#if jira-_key}}
  {{#if jira-_service-_address}}
    {{jira-_service-_address}}/browse/{{jira-_key}}
  {{/if}}
{{/if}}
```

### 常用模式

#### 可选字段显示

```handlebars
{{#if optional-_field}}
#### Field Name:

{{optional-_field}}
{{/if}}
```

#### 条件格式

```handlebars
{{#if condition}}
  Format A
{{else}}
  Format B
{{/if}}
```

#### 列表渲染

```handlebars
{{#each items}}
- {{this.name}}
{{/each}}
```

---

## 🔄 配置优先级

模板配置的加载顺序（优先级从高到低）：

1. **项目级配置** (`.workflow/config.toml`)
   - 如果存在，优先使用
   - 只覆盖配置文件中指定的部分
   - 未配置的部分使用全局配置或默认值

2. **全局配置** (`~/.workflow/config/workflow.toml`)
   - 如果项目级配置不存在，使用全局配置
   - 适用于所有项目

3. **默认配置**（代码中的默认值）
   - 如果配置文件不存在，使用默认模板
   - 确保系统始终可以正常工作

### 配置合并规则

- 项目级配置会**部分覆盖**全局配置
- 未在项目级配置中指定的字段，使用全局配置的值
- 如果全局配置也不存在，使用默认值

**示例**：

**全局配置** (`~/.workflow/config/workflow.toml`)：

```toml
[template.branch]
default = "{{jira-_key}}-{{summary-_slug}}"
feature = "feature/{{jira-_key}}-{{summary-_slug}}"
bugfix = "bugfix/{{jira-_key}}-{{summary-_slug}}"
```

**项目级配置** (`.workflow/config.toml`)：

```toml
[template.branch]
feature = "feat/{{jira-_key}}-{{summary-_slug}}"
```

**最终生效的配置**：

```toml
[template.branch]
default = "{{jira-_key}}-{{summary-_slug}}"        # 来自全局配置
feature = "feat/{{jira-_key}}-{{summary-_slug}}"  # 来自项目级配置（覆盖）
bugfix = "bugfix/{{jira-_key}}-{{summary-_slug}}" # 来自全局配置
```

---

## 🎯 使用场景

### 场景 1：自定义分支命名格式

**需求**：使用 `feat/` 前缀而不是 `feature/`

**配置**：

```toml
[template.branch]
feature = "feat/{{jira-_key}}-{{summary-_slug}}"
```

### 场景 2：启用 Conventional Commits 格式

**需求**：提交消息使用 `feat(scope): title` 格式

**配置**：

```toml
[template.commit]
use-_scope = true
```

### 场景 3：自定义 PR 模板

**需求**：添加更多字段到 PR 模板

**配置**：

```toml
[template.pull-_requests]
default = """
# PR Ready

## Types of changes

{{#each change-_types}}
- [{{#if this.selected}}x{{else}} {{/if}}] {{this.name}}
{{/each}}

{{#if short-_description}}
#### Short description:

{{short-_description}}
{{/if}}

{{#if jira-_key}}
{{#if jira-_service-_address}}
#### Jira Link:

{{jira-_service-_address}}/browse/{{jira-_key}}
{{/if}}
{{/if}}

{{#if dependency}}
#### Dependency

{{dependency}}
{{/if}}

## Testing

- [ ] Unit tests added
- [ ] Integration tests added
- [ ] Manual testing completed
"""
```

### 场景 4：项目特定配置

**需求**：某个项目需要特殊的提交消息格式

**配置**（`.workflow/config.toml`）：

```toml
[template.commit]
default = """[{{jira-_key}}] {{subject}}

{{#if body}}{{body}}{{/if}}"""
```

---

## 🔍 故障排除

### 问题 1：模板未生效

**症状**：修改了配置文件，但模板没有变化

**解决方案**：

1. 检查配置文件路径是否正确
2. 检查 TOML 语法是否正确（可以使用在线 TOML 验证器）
3. 检查配置是否在正确的 `[template]` 部分下
4. 确认项目级配置优先级（项目级配置会覆盖全局配置）

### 问题 2：Handlebars 语法错误

**症状**：模板渲染失败，提示语法错误

**解决方案**：

1. 检查 Handlebars 语法是否正确
2. 确保所有 `{{#if}}` 都有对应的 `{{/if}}`
3. 确保所有 `{{#each}}` 都有对应的 `{{/each}}`
4. 检查变量名是否正确（区分大小写）

### 问题 3：变量未定义

**症状**：模板中使用了不存在的变量

**解决方案**：

1. 检查变量名是否正确（参考[模板变量](#-模板变量)部分）
2. 使用 `{{#if variable}}` 检查变量是否存在
3. 对于可选变量，始终使用条件判断

### 问题 4：多行字符串格式错误

**症状**：TOML 解析失败

**解决方案**：

在 TOML 中，多行字符串需要使用三重引号：

```toml
default = """
多行内容
可以包含换行
"""
```

或者使用字面量字符串：

```toml
default = '''
多行内容
可以包含换行
'''
```

### 问题 5：配置优先级问题

**症状**：项目级配置没有覆盖全局配置

**解决方案**：

1. 确认项目级配置文件路径：`.workflow/config.toml`（项目根目录）
2. 确认配置在 `[template]` 部分下
3. 检查是否有语法错误导致配置未正确加载
4. 使用 `workflow repo show` 命令查看当前生效的配置

---

## 📚 相关文档

- [开发规范文档](./development.md) - 代码风格和开发规范
- [分支管理架构文档](../architecture/branch.md) - 分支命名实现细节
- [PR 模块架构文档](../architecture/lib/pr.md) - PR 生成实现细节
- [设置模块架构文档](../architecture/lib/SETTINGS_architecture.md) - 配置加载实现细节

---

## 🔗 参考资源

- [Handlebars 官方文档](https://handlebarsjs.com/)
- [TOML 规范](https://toml.io/)

---

**最后更新**: 2025-12-12
