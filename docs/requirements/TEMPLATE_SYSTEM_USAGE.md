# 模板系统使用分析文档

## 📋 概述

本文档分析模板系统如何与现有命令配合使用，包括具体的使用场景、集成点和实现流程。

**创建日期**: 2025-01-27
**状态**: 📋 分析中

---

## 🎯 模板系统集成点

### 1. 分支命名模板集成

#### 1.1 集成命令
- `workflow branch create --from PROJ-123` (待实现)
- `workflow pr create --jira PROJ-123` (已实现，需要增强)

#### 1.2 当前实现
**位置**: `src/lib/pr/helpers.rs::generate_branch_name()`

**当前逻辑**:
```rust
// 当前是硬编码的分支名生成逻辑
// 如果有 Jira ticket，添加到分支名前缀
if let Some(ticket) = jira_ticket {
    branch_name = format!("{}-{}", ticket, branch_name);
}
```

#### 1.3 模板系统集成方案

**步骤 1**: 从 JIRA ticket 获取信息
```rust
// 在 branch create 或 pr create 命令中
let jira_info = if let Some(ticket) = &jira_ticket {
    Jira::get_ticket_info(ticket)?  // 获取 ticket 详细信息
} else {
    None
};
```

**步骤 2**: 使用模板引擎生成分支名
```rust
// 使用模板引擎（handlebars 或 tera）
let template = load_branch_template(jira_info.as_ref().map(|i| i.ticket_type))?;
let branch_name = template_engine.render(&template, &template_vars)?;
```

**步骤 3**: 清理和规范化分支名
```rust
// 自动清理特殊字符、空格等
let branch_name = sanitize_branch_name(branch_name)?;
```

#### 1.4 配置示例
```toml
[branch.templates]
default = "{{jira_key}}-{{summary_slug}}"
feature = "feature/{{jira_key}}-{{summary_slug}}"
bugfix = "bugfix/{{jira_key}}-{{summary_slug}}"
hotfix = "hotfix/{{jira_key}}-{{summary_slug}}"
```

#### 1.5 使用流程
```bash
# 用户执行命令
workflow branch create --from PROJ-123

# 系统流程：
# 1. 获取 JIRA ticket 信息（PROJ-123）
# 2. 根据 ticket type 选择模板（feature/bugfix/hotfix）
# 3. 使用模板引擎渲染分支名
# 4. 清理和规范化（去除特殊字符、转换为小写等）
# 5. 创建分支
```

---

### 2. Commit 消息模板集成

#### 2.1 集成命令
- `workflow commit` (待实现)
- `workflow pr create` (已实现，需要增强 commit 消息生成)

#### 2.2 当前实现
**位置**: `src/commands/pr/create.rs::generate_commit_title_and_branch_name()`

**当前逻辑**:
- 使用 LLM 生成 commit 标题
- 或使用默认方法生成

#### 2.3 模板系统集成方案

**步骤 1**: 交互式收集 commit 信息
```rust
// 使用 dialoguer 收集信息
let commit_type = Select::new()
    .with_prompt("Commit type")
    .items(&["feat", "fix", "docs", "style", "refactor", "test", "chore"])
    .interact()?;

let scope = Input::new()
    .with_prompt("Scope (optional)")
    .allow_empty(true)
    .interact()?;

let subject = Input::new()
    .with_prompt("Subject")
    .interact()?;
```

**步骤 2**: 从 JIRA ticket 或分支名提取信息
```rust
// 自动从当前分支名提取 JIRA ID
let jira_key = extract_jira_from_branch(&current_branch)?;
// 或从 JIRA ticket 获取
let jira_info = if let Some(ticket) = jira_key {
    Jira::get_ticket_info(&ticket)?
} else {
    None
};
```

**步骤 3**: 使用模板引擎生成 commit 消息
```rust
let template = load_commit_template()?;
let commit_message = template_engine.render(&template, &template_vars)?;
```

#### 2.4 配置示例
```toml
[commit.templates]
default = """
{{type}}({{scope}}): {{subject}}

{{body}}

Closes {{jira_key}}
"""
```

#### 2.5 使用流程
```bash
# 用户执行命令
workflow commit

# 系统流程：
# 1. 检测当前分支，提取 JIRA ID（如果有）
# 2. 交互式收集 commit 信息（type, scope, subject, body）
# 3. 获取 JIRA ticket 信息（如果存在）
# 4. 使用模板引擎渲染 commit 消息
# 5. 显示预览，确认后提交
```

---

### 3. PR 描述模板集成

#### 3.1 集成命令
- `workflow pr create` (已实现，需要增强)
- `workflow pr update` (已实现，可能需要增强)

#### 3.2 当前实现
**位置**: `src/lib/pr/helpers.rs::generate_pull_request_body()`

**当前逻辑**:
```rust
// 硬编码的 PR body 生成逻辑
let mut body = String::from("\n# PR Ready\n\n## Types of changes\n\n");
// ... 生成变更类型复选框
// ... 添加简短描述
// ... 添加 Jira 链接
```

#### 3.3 模板系统集成方案

**步骤 1**: 从 JIRA ticket 获取完整信息
```rust
// 在 pr create 命令中
let jira_info = if let Some(ticket) = &jira_ticket {
    Jira::get_ticket_info(ticket)?  // 获取 summary, description, labels, type 等
} else {
    None
};
```

**步骤 2**: 准备模板变量
```rust
let template_vars = TemplateVars {
    jira_key: jira_info.as_ref().map(|i| i.key.clone()),
    jira_summary: jira_info.as_ref().and_then(|i| i.summary.clone()),
    jira_description: jira_info.as_ref().and_then(|i| i.description.clone()),
    jira_type: jira_info.as_ref().map(|i| i.ticket_type.clone()),
    summary_slug: jira_info.as_ref()
        .and_then(|i| i.summary.clone())
        .map(|s| slugify(&s)),
    change_types: selected_change_types,
    short_description: short_description.clone(),
    // ... 其他变量
};
```

**步骤 3**: 使用模板引擎生成 PR body
```rust
let template = load_pr_template()?;
let pr_body = template_engine.render(&template, &template_vars)?;
```

#### 3.4 配置示例
```toml
[pr.templates]
default = """
## Description
{{jira_summary}}

{{#if jira_description}}
## Details
{{jira_description}}
{{/if}}

## Related Ticket
{{jira_key}}

## Changes
{{#each change_types}}
- [{{#if this}}x{{else}} {{/if}}] {{this}}
{{/each}}

{{#if short_description}}
#### Short description:
{{short_description}}
{{/if}}
"""
```

#### 3.5 使用流程
```bash
# 用户执行命令
workflow pr create --jira PROJ-123

# 系统流程：
# 1. 获取 JIRA ticket 信息（PROJ-123）
# 2. 收集用户输入（标题、描述、变更类型等）
# 3. 准备模板变量
# 4. 使用模板引擎渲染 PR body
# 5. 创建 PR
```

---

## 🔄 完整工作流示例

### 场景：从 JIRA ticket 创建 PR

```bash
# 1. 创建分支（使用分支命名模板）
workflow branch create --from PROJ-123
# 系统：
#   - 获取 PROJ-123 信息：type=Feature, summary="Add user authentication"
#   - 使用模板：feature/{{jira_key}}-{{summary_slug}}
#   - 生成：feature/PROJ-123-add-user-authentication
#   - 创建并切换到该分支

# 2. 进行开发工作...
git add .
git commit -m "WIP: Add login form"

# 3. 提交代码（使用 commit 模板）
workflow commit
# 系统：
#   - 检测分支名，提取 PROJ-123
#   - 交互式收集：type=feat, scope=auth, subject="Add login form"
#   - 使用模板生成：
#     feat(auth): Add login form
#
#     Closes PROJ-123
#   - 确认后提交

# 4. 创建 PR（使用 PR 模板）
workflow pr create
# 系统：
#   - 检测分支名，提取 PROJ-123
#   - 获取 JIRA ticket 完整信息
#   - 使用模板生成 PR body（包含 summary, description, link 等）
#   - 创建 PR
#   - 自动更新 JIRA ticket 状态
```

---

## 🏗️ 实现架构

### 模板引擎模块结构

```
src/lib/template/
├── mod.rs              # 模板模块入口
├── engine.rs           # 模板引擎封装（handlebars/tera）
├── vars.rs             # 模板变量定义
├── loader.rs           # 模板加载器（从配置加载）
└── sanitizer.rs        # 分支名清理和规范化
```

### 配置结构

```toml
[template]
engine = "handlebars"  # 或 "tera"

[branch.templates]
default = "{{jira_key}}-{{summary_slug}}"
feature = "feature/{{jira_key}}-{{summary_slug}}"
bugfix = "bugfix/{{jira_key}}-{{summary_slug}}"

[commit.templates]
default = """
{{type}}({{scope}}): {{subject}}

{{body}}

Closes {{jira_key}}
"""

[pr.templates]
default = """
## Description
{{jira_summary}}

## Related Ticket
{{jira_key}}

## Changes
{{#each change_types}}
- [{{#if this}}x{{else}} {{/if}}] {{this}}
{{/each}}
"""
```

---

## 🔗 命令集成点总结

### 已实现的命令（需要增强）

1. **`workflow pr create`**
   - ✅ 已有分支名生成逻辑 → 需要替换为模板系统
   - ✅ 已有 PR body 生成逻辑 → 需要替换为模板系统
   - ✅ 已有 commit 标题生成 → 可以增强为模板系统

### 待实现的命令（需要集成模板）

1. **`workflow branch create --from PROJ-123`**
   - ❌ 需要实现分支命名模板集成

2. **`workflow commit`**
   - ❌ 需要实现 commit 消息模板集成

3. **`workflow branch create`** (通用版本)
   - ❌ 需要实现分支命名模板集成

---

## 📝 实现优先级

### 高优先级
1. **PR 模板集成** - 替换现有的 `generate_pull_request_body()`
2. **分支命名模板集成** - 在 `branch create` 和 `pr create` 中使用

### 中优先级
1. **Commit 模板集成** - 实现 `workflow commit` 命令
2. **模板配置管理** - 支持全局和项目级配置

### 低优先级
1. **模板继承和覆盖** - 支持模板继承机制
2. **模板验证** - 验证模板语法和变量

---

## 🔧 技术实现要点

### 1. 模板引擎选择
- **推荐**: `handlebars` (Rust 实现，功能完整)
- **备选**: `tera` (类似 Jinja2，语法熟悉)

### 2. 变量提取
- 从 JIRA ticket 提取：key, summary, description, type, labels
- 从分支名提取：JIRA ID（正则匹配）
- 从 Git 提取：当前分支、提交历史等

### 3. 分支名清理
- 转换为小写
- 替换空格为连字符
- 移除特殊字符
- 限制长度（Git 分支名限制）

### 4. 配置加载
- 支持全局配置（`~/.workflow/config.toml`）
- 支持项目级配置（`.workflow/config.toml`）
- 项目配置覆盖全局配置

---

## ✅ 验收标准

### 分支命名模板
- [ ] `workflow branch create --from PROJ-123` 能使用模板生成分支名
- [ ] 支持根据 ticket type 选择不同模板
- [ ] 自动清理和规范化分支名
- [ ] 生成的分支名符合 Git 规范

### Commit 模板
- [ ] `workflow commit` 能使用模板生成 commit 消息
- [ ] 支持交互式填写模板变量
- [ ] 自动从分支名提取 JIRA ID
- [ ] 生成的 commit 消息符合 Conventional Commits 规范

### PR 模板
- [ ] `workflow pr create` 能使用模板生成 PR body
- [ ] 从 JIRA ticket 自动提取信息填充模板
- [ ] 支持自定义模板配置
- [ ] 生成的 PR body 格式正确且完整

---

## 📚 相关文档

- [模板系统需求文档](./TEMPLATE_SYSTEM.md)
- [Git 工作流需求文档](./GIT_WORKFLOW.md)
- [JIRA 命令需求文档](./JIRA_COMMANDS.md)
- [PR 命令架构文档](../architecture/commands/PR_COMMAND_ARCHITECTURE.md)

---

**最后更新**: 2025-01-27
