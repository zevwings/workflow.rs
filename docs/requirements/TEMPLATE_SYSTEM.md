# 模板系统需求文档

## 📋 需求概述

本文档描述工作流自动化中的模板系统需求，包括 PR 模板、Commit 模板和分支命名模板。

**状态**: 📋 需求分析中
**分类**: 工作流自动化
**优先级**: 高优先级
**来源**: 从 `docs/todo/WORKFLOW_TODO.md` 迁移

---

## 🎯 需求目标

实现一个灵活的模板系统，能够根据 JIRA ticket 信息自动生成标准化的：
1. PR 描述模板
2. Commit 消息模板
3. 分支命名模板

---

## 📝 详细需求

### 1. PR 模板（根据 JIRA ticket 自动生成）

#### 1.1 功能描述
根据 JIRA ticket 自动生成 PR 描述模板。

#### 1.2 功能要求
- 从 JIRA ticket 提取信息（summary、description、labels 等）
- 使用模板引擎（如 `handlebars`）生成 PR 描述
- 支持自定义模板（配置文件）

#### 1.3 配置示例
```toml
[pr.templates]
default = """
## Description
{{jira_summary}}

## Related Ticket
{{jira_key}}

## Changes
- [ ] Feature
- [ ] Bug fix
- [ ] Documentation
"""
```

#### 1.4 实现建议
- 使用模板引擎（`handlebars` 或 `tera`）
- 支持模板变量替换
- 提供默认模板，支持用户自定义

---

### 2. Commit 模板（标准化格式）

#### 2.1 功能描述
标准化 commit 消息格式。

#### 2.2 功能要求
- 支持 Conventional Commits 格式
- 自动提取 JIRA ID
- 支持交互式填写

#### 2.3 配置示例
```toml
[commit.templates]
default = """
{{type}}({{scope}}): {{subject}}

{{body}}

Closes {{jira_key}}
"""
```

#### 2.4 实现建议
- 遵循 Conventional Commits 规范
- 提供交互式 CLI 界面填写模板变量
- 自动从当前分支或 JIRA ticket 提取相关信息

---

### 3. 分支命名模板（根据 JIRA ticket 自动生成）

#### 3.1 功能描述
根据 JIRA ticket 自动生成分支名。

#### 3.2 功能要求
- 支持模板变量（`{{jira_key}}`、`{{jira_type}}`、`{{summary}}` 等）
- 自动清理和规范化分支名（去除特殊字符、空格等）

#### 3.3 配置示例
```toml
[branch.templates]
default = "{{jira_key}}-{{summary_slug}}"
feature = "feature/{{jira_key}}-{{summary_slug}}"
bugfix = "bugfix/{{jira_key}}-{{summary_slug}}"
```

#### 3.4 实现建议
- 支持多种分支类型模板（feature、bugfix、hotfix 等）
- 自动将 summary 转换为 URL 友好的 slug 格式
- 验证生成的分支名是否符合 Git 分支命名规范

---

## 🔧 技术实现

### 模板引擎选择
- **推荐**: `handlebars` 或 `tera`
- 需要支持变量替换、条件判断等基本功能

### 模板变量
所有模板支持以下变量：
- `{{jira_key}}`: JIRA ticket key（如 `PROJ-123`）
- `{{jira_summary}}`: JIRA ticket summary
- `{{jira_description}}`: JIRA ticket description
- `{{jira_type}}`: JIRA ticket type（如 `Feature`、`Bug`）
- `{{summary_slug}}`: Summary 的 URL 友好格式（小写、连字符分隔）

### 配置管理
- 模板配置存储在项目配置文件中
- 支持全局配置和项目级配置
- 支持模板继承和覆盖

---

## ✅ 验收标准

### PR 模板
- [ ] 能够从 JIRA ticket 提取信息
- [ ] 能够使用模板引擎生成 PR 描述
- [ ] 支持自定义模板配置
- [ ] 生成的 PR 描述格式正确

### Commit 模板
- [ ] 支持 Conventional Commits 格式
- [ ] 能够自动提取 JIRA ID
- [ ] 提供交互式填写界面
- [ ] 生成的 commit 消息符合规范

### 分支命名模板
- [ ] 能够根据 JIRA ticket 生成分支名
- [ ] 支持多种分支类型模板
- [ ] 自动清理和规范化分支名
- [ ] 生成的分支名符合 Git 规范

---

## 📚 相关文档

- [JIRA 模块待办事项](../todo/JIRA_TODO.md)
- [Git 工作流需求文档](./GIT_WORKFLOW.md)

---

**创建日期**: 2025-01-27
**最后更新**: 2025-01-27
