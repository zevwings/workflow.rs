# Repo 模块架构文档

## 📋 概述

Repo 模块（`lib/repo/`）是 Workflow CLI 的核心库模块，提供仓库级配置管理功能，包括配置检查、加载、保存等功能。配置存储在项目根目录的 `.workflow/config.toml` 文件中，支持分支配置、模板配置等。

**注意**：本文档仅描述 `lib/repo/` 模块的架构。关于 Repo 命令层的详细内容，请参考 [Repo 命令模块架构文档](../commands/REPO_COMMAND_ARCHITECTURE.md)。

**模块统计：**
- 总代码行数：约 358 行
- 文件数量：2 个
- 主要组件：`RepoConfig`、`ProjectConfig`、`ProjectBranchConfig`
- 支持功能：配置检查、加载、保存、分支前缀获取、忽略分支列表获取

---

## 📁 模块结构

### 核心模块文件

```
src/lib/repo/
├── mod.rs          # Repo 模块声明和导出 (8行)
└── config.rs       # 仓库配置管理 (358行)
```

### 依赖模块

- **`lib/base/settings/`**：路径管理
  - `Paths::project_config()` - 获取项目配置文件路径
- **`lib/git/`**：Git 操作
  - `GitRepo::is_git_repo()` - 检查是否是 Git 仓库

### 模块集成

- **`commands/repo/`**：Repo 命令层
  - `setup.rs` - 使用 `RepoConfig` 进行配置加载和保存
  - `show.rs` - 使用 `RepoConfig` 进行配置加载和显示
- **`commands/branch/`**：分支管理命令
  - 使用 `RepoConfig::get_branch_prefix()` 获取分支前缀
  - 使用 `RepoConfig::get_ignore_branches()` 获取忽略分支列表
- **`commands/commit/`**、**`commands/pr/`**：其他命令
  - 使用 `RepoSetupCommand::ensure()` 确保配置存在

---

## 🏗️ 架构设计

### 设计原则

1. **职责单一**：专注于仓库级配置管理
2. **无状态设计**：所有方法都是静态方法，不维护状态
3. **错误传播**：使用 `Result` 类型传播错误，由调用层处理
4. **可测试性**：业务逻辑与文件操作分离，便于单元测试

### 核心组件

#### 1. RepoConfig 结构体

**职责**：提供仓库配置管理的静态方法

**主要方法**：
- `exists()` - 检查配置是否存在且完整
- `load()` - 加载项目配置
- `save()` - 保存项目配置
- `get_branch_prefix()` - 获取分支前缀
- `get_ignore_branches()` - 获取忽略分支列表

**关键特性**：
- 配置文件路径：`.workflow/config.toml`（项目根目录）
- 自动创建目录结构（如果不存在）
- 支持配置合并（保存时合并现有配置）
- 向后兼容（支持旧格式配置）

**使用场景**：
- `repo setup` 命令：加载和保存配置
- `repo show` 命令：加载配置用于显示
- `branch` 命令：获取分支前缀和忽略列表
- 其他命令：检查配置是否存在

#### 2. ProjectConfig 结构体

**职责**：表示项目级配置结构

**字段**：
- `template_commit: Map<String, Value>` - 提交模板配置
- `template_branch: Map<String, Value>` - 分支模板配置
- `template_pull_requests: Map<String, Value>` - PR 模板配置
- `branch: ProjectBranchConfig` - 分支配置
- `auto_accept_change_type: Option<bool>` - 自动接受变更类型选择

**关键特性**：
- 使用 `toml::Value` 存储模板配置，支持灵活配置
- 分支配置使用结构化类型 `ProjectBranchConfig`
- 支持可选字段（使用 `Option`）

#### 3. ProjectBranchConfig 结构体

**职责**：表示项目级分支配置

**字段**：
- `prefix: Option<String>` - 分支前缀（可选）
- `ignore: Vec<String>` - 忽略分支列表

**关键特性**：
- 使用 `serde` 进行序列化/反序列化
- 支持 TOML 格式
- 可选字段使用 `skip_serializing_if` 控制序列化

---

## 🔄 核心功能

### 1. 配置检查 (`exists()`)

**功能**：检查仓库配置是否存在且完整

**流程**：
1. 检查是否在 Git 仓库中（如果不是，返回 `true`，跳过检查）
2. 检查配置文件是否存在
3. 检查配置是否有必需的节（`[template.commit]` 或 `[branch]`）

**返回值**：
- `Ok(true)` - 配置存在且完整
- `Ok(false)` - 配置不存在或不完整
- `Err(_)` - 检查过程中出错

### 2. 配置加载 (`load()`)

**功能**：从文件加载项目配置

**流程**：
1. 获取配置文件路径（`.workflow/config.toml`）
2. 如果文件不存在，返回默认配置
3. 读取并解析 TOML 文件
4. 解析各个配置节：
   - `[template.commit]` - 提交模板配置
   - `[template.branch]` - 分支模板配置
   - `[template.pull_requests]` - PR 模板配置
   - `[branch]` - 分支配置
   - `[pr]` 或顶层 `auto_accept_change_type` - PR 配置

**关键特性**：
- 文件不存在时返回默认配置（不报错）
- 支持向后兼容（手动解析旧格式）
- 使用 `toml::Value` 灵活处理模板配置

### 3. 配置保存 (`save()`)

**功能**：保存项目配置到文件

**流程**：
1. 获取配置文件路径
2. 确保目录存在（创建 `.workflow/` 目录）
3. 读取现有配置（如果存在）
4. 合并新配置与现有配置
5. 写入文件

**关键特性**：
- 自动创建目录结构
- 配置合并（保留现有配置，只更新提供的字段）
- 使用 `toml::to_string_pretty()` 格式化输出

### 4. 分支前缀获取 (`get_branch_prefix()`)

**功能**：获取当前仓库的分支前缀

**流程**：
1. 加载项目配置
2. 返回 `config.branch.prefix`

**返回值**：
- `Some(String)` - 如果配置了分支前缀
- `None` - 如果未配置或加载失败

### 5. 忽略分支列表获取 (`get_ignore_branches()`)

**功能**：获取当前仓库的忽略分支列表

**流程**：
1. 加载项目配置
2. 返回 `config.branch.ignore`

**返回值**：
- `Vec<String>` - 忽略分支列表（如果未配置或加载失败，返回空列表）

---

## 📝 配置文件格式

### 配置文件位置

```
项目根目录/.workflow/config.toml
```

### 配置格式示例

```toml
[branch]
prefix = "feature"
ignore = [
    "main",
    "master",
    "develop",
    "zw/important-feature",
]

[template.commit]
use_scope = false
default = "{{#if jira_key}}{{jira_key}}: {{subject}}{{else}}# {{subject}}{{/if}}"

[template.branch]
default = "{{jira_key}}-{{summary_slug}}"
feature = "feature/{{jira_key}}-{{summary_slug}}"

[template.pull_requests]
default = "## Description\n\n{{jira_summary}}\n\n..."

[pr]
auto_accept_change_type = true
```

### 配置节说明

#### `[branch]` 节

- `prefix` (可选) - 分支前缀，用于生成分支名时自动添加前缀
- `ignore` (可选) - 忽略分支列表，分支清理时会自动排除这些分支

#### `[template.commit]` 节

- `use_scope` (可选) - 是否使用 scope（Conventional Commits 格式）
- `default` (可选) - 默认提交模板（Handlebars 格式）

#### `[template.branch]` 节

- `default` (可选) - 默认分支模板
- `feature` (可选) - Feature 分支模板
- `bugfix` (可选) - Bugfix 分支模板
- `hotfix` (可选) - Hotfix 分支模板
- `refactoring` (可选) - Refactoring 分支模板
- `chore` (可选) - Chore 分支模板

#### `[template.pull_requests]` 节

- `default` (可选) - 默认 PR 模板（Handlebars 格式）

#### `[pr]` 节

- `auto_accept_change_type` (可选) - 是否自动接受变更类型选择

---

## 🔍 错误处理

### 错误类型

1. **文件系统错误**：
   - 目录创建失败
   - 文件读取失败
   - 文件写入失败

2. **配置解析错误**：
   - TOML 格式错误
   - 配置结构不匹配

3. **Git 仓库错误**：
   - 不在 Git 仓库中
   - 无法提取仓库名

### 错误处理策略

- **文件不存在**：返回默认配置（不报错）
- **配置解析失败**：返回错误，提示用户检查配置文件
- **Git 仓库检测失败**：返回错误，提示用户在 Git 仓库中运行

---

## 📚 使用示例

### 检查配置是否存在

```rust
use workflow::repo::RepoConfig;

if !RepoConfig::exists()? {
    println!("Configuration not found");
}
```

### 加载配置

```rust
use workflow::repo::RepoConfig;

let config = RepoConfig::load()?;
println!("Branch prefix: {:?}", config.branch.prefix);
```

### 保存配置

```rust
use workflow::repo::{ProjectConfig, RepoConfig};

let mut config = ProjectConfig::default();
config.branch.prefix = Some("feature".to_string());
RepoConfig::save(&config)?;
```

### 获取分支前缀

```rust
use workflow::repo::RepoConfig;

if let Some(prefix) = RepoConfig::get_branch_prefix() {
    println!("Branch prefix: {}", prefix);
}
```

### 获取忽略分支列表

```rust
use workflow::repo::RepoConfig;

let ignore_list = RepoConfig::get_ignore_branches();
println!("Ignored branches: {:?}", ignore_list);
```

---

## 🔄 与其他模块的集成

### 与 Template 模块的集成

- Repo 模块管理模板配置的存储和加载
- Template 模块使用 Repo 模块加载的配置进行模板渲染
- 配置存储在项目级配置文件中，支持团队共享

### 与 Branch 模块的集成

- Branch 模块使用 Repo 模块获取分支前缀和忽略列表
- 分支创建时自动应用配置的分支前缀
- 分支清理时自动排除配置的忽略列表

### 与命令层的集成

- `repo setup` 命令使用 Repo 模块进行配置保存
- `repo show` 命令使用 Repo 模块进行配置加载
- `branch` 命令使用 Repo 模块获取配置
- 其他命令使用 `RepoSetupCommand::ensure()` 确保配置存在

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [Repo 命令模块架构文档](../commands/REPO_COMMAND_ARCHITECTURE.md) - 命令层
- [Template 模块架构文档](./TEMPLATE_ARCHITECTURE.md) - 模板系统
- [Branch 模块架构文档](./BRANCH_ARCHITECTURE.md) - 分支管理

---

## ✅ 总结

Repo 模块采用清晰的设计原则：

1. **职责单一**：专注于仓库级配置管理
2. **无状态设计**：所有方法都是静态方法
3. **灵活配置**：支持多种配置类型和可选字段
4. **团队协作**：配置存储在项目根目录，可提交到 Git

**设计优势**：
- ✅ 职责清晰，易于维护
- ✅ 配置灵活，支持多种场景
- ✅ 团队协作友好，配置可版本控制
- ✅ 向后兼容，支持旧格式配置

**当前实现状态**：
- ✅ 配置检查功能完整实现
- ✅ 配置加载功能完整实现
- ✅ 配置保存功能完整实现
- ✅ 分支配置获取功能完整实现
