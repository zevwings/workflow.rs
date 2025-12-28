# Branch 模块架构文档

## 📋 概述

Branch 模块是 Workflow CLI 的核心模块，提供完整的分支命名和管理功能。该模块采用分层架构设计，包括：

- **Lib 层**（`lib/branch/`）：提供分支命名服务、分支类型定义、LLM 翻译功能，支持从 JIRA ticket、标题文本、模板系统等多种方式生成分支名
- **Commands 层**（`commands/branch/`）：提供 CLI 命令封装，包括分支创建、切换、重命名、清理和忽略列表管理

Branch 模块支持智能的分支名生成（模板系统 → LLM → 简单回退的多层策略）、分支类型定义、非英文翻译、分支配置管理等功能。

**模块统计：**
- Lib 层代码行数：约 700+ 行
- Commands 层代码行数：约 1000+ 行
- 命令数量：5 个（create, switch, rename, clean, ignore）
- 主要组件：`BranchNaming`、`BranchType`、`BranchLLM`
- 支持功能：分支名生成（模板系统、LLM、简单回退）、分支类型定义、非英文翻译、分支管理

**注意**：分支配置管理已迁移到 `lib/repo/config.rs`，使用 `RepoConfig` 和 `ProjectBranchConfig` 进行管理。

---

## 📁 Lib 层架构（核心业务逻辑）

Branch 模块（`lib/branch/`）是 Workflow CLI 的核心库模块，提供分支命名和管理功能。该模块专注于分支名称生成、分支前缀管理、分支配置管理以及分支类型定义，支持从 JIRA ticket、标题文本、模板系统等多种方式生成分支名，并提供智能的分支名称清理和翻译功能。

### 模块结构

```
src/lib/branch/
├── mod.rs          # Branch 模块声明和导出 (46行)
├── naming.rs       # 分支命名服务（从 JIRA ticket、标题、类型生成）(455行)
├── types.rs        # 分支类型定义（feature/bugfix/refactoring/hotfix/chore）(178行)
├── llm.rs          # Branch LLM 服务（非英文翻译）(56行)
└── sync.rs         # 分支同步功能
```

**注意**：分支配置管理已迁移到 `lib/repo/config.rs`，使用 `RepoConfig` 和 `ProjectBranchConfig` 进行管理。

### 依赖模块

- **`lib/git/`**：Git 操作（获取分支列表、提取仓库名）
  - `GitBranch::get-_all-_branches()` - 获取所有分支（用于 LLM 生成）
  - `GitRepo::extract-_repo-_name()` - 提取仓库名（用于配置管理）
- **`lib/jira/`**：JIRA 集成（JIRA ticket 信息获取）
  - 通过命令层传入 JIRA ticket ID 和 summary
- **`lib/template/`**：模板系统（分支名模板渲染）
  - `load-_branch-_template()` - 加载分支模板
  - `load-_branch-_template-_by-_type()` - 按类型加载模板
  - `TemplateEngine` - 模板渲染引擎
- **`lib/base/llm/`**：LLM 客户端（分支名生成、翻译）
  - `LLMClient` - LLM API 调用
- **`lib/base/dialog/`**：用户交互（分支类型选择）
  - `SelectDialog` - 选择对话框
  - `InputDialog` - 输入对话框
- **`lib/base/settings/`**：配置管理（配置文件路径）
  - `Paths::project-_config()` - 项目级配置文件路径
- **`lib/pr/llm/`**：PR LLM 服务（分支名生成回退）
  - `PullRequestLLM::generate()` - 生成分支名（作为回退方案）

### 模块集成

- **`commands/branch/`**：分支命令层
  - `create.rs` - 使用 `BranchNaming`、`BranchType` 创建分支
  - `switch.rs` - 分支切换命令（使用 `GitBranch` 进行分支操作）
  - `rename.rs` - 分支重命名命令（使用 `GitBranch` 进行分支操作）
  - `ignore.rs` - 使用 `RepoConfig` 管理忽略列表
- **`commands/pr/`**：PR 命令层
  - `create.rs` - 使用 `BranchNaming`、`BranchType` 生成分支名
  - `pick.rs` - 使用 `BranchNaming`、`BranchType` 生成分支名
  - `helpers.rs` - 提供 stash 处理辅助函数（`handle-_stash-_pop-_result()`），被 `branch switch` 命令使用

---

## 🏗️ Lib 层架构设计

### 设计原则

1. **策略模式**：分支名生成采用模板系统 → LLM → 简单回退的多层策略
2. **配置分离**：分支配置按仓库分组，支持不同仓库不同配置
3. **智能前缀**：自动检测并避免重复前缀（JIRA ticket 前缀、仓库前缀）
4. **类型安全**：使用枚举定义分支类型，提供类型转换和验证
5. **容错设计**：LLM 失败时自动回退到简单方法，翻译失败时使用原始输入

### 核心组件

#### 1. 分支命名服务 (`naming.rs`)

**职责**：提供从多种来源生成分支名的功能

**主要方法**：
- `from-_jira-_ticket()` - 从 JIRA ticket 生成分支名（模板系统 → LLM → 简单回退）
- `from-_type-_and-_slug()` - 从分支类型和 slug 生成分支名（使用模板系统）
- `from-_title()` - 从标题生成分支名
- `sanitize()` - 清理字符串为分支名格式（仅保留 ASCII 字母数字）
- `slugify()` - 转换为 slug 格式（保留更多字符）
- `sanitize-_and-_translate-_branch-_name()` - 清理并翻译分支名（处理非英文输入）

**关键特性**：
- 支持三种生成策略：模板系统优先，LLM 次之，简单方法最后
- 自动处理非英文字符（使用 LLM 翻译）
- 支持两种格式：`prefix/ticket-slug` 和 `ticket--slug`
- 模板系统自动处理前缀（JIRA ticket 前缀和仓库前缀）

**使用场景**：
- `branch create` 命令：从 JIRA ticket 创建分支
- `pr create` 命令：生成 PR 分支名
- `pr pick` 命令：从源 PR 生成新分支名

#### 2. 分支类型定义 (`types.rs`)

**职责**：定义分支类型枚举和提供选择功能

**核心枚举**：
- `BranchType` - 分支类型（Feature、Bugfix、Refactoring、Hotfix、Chore）

**主要方法**：
- `all()` - 获取所有分支类型
- `as-_str()` - 转换为字符串（用于模板选择）
- `display-_name()` - 获取显示名称（带描述）
- `from-_str()` - 从字符串解析
- `prompt-_selection()` - 交互式选择分支类型

**关键特性**：
- 支持多种字符串格式（如 "bug"、"fix" 都映射到 Bugfix）
- 提供中英文显示名称
- 支持交互式选择（使用 `SelectDialog`）

**使用场景**：
- `branch create` 命令：选择分支类型
- `pr create` 命令：选择分支类型
- 模板系统：根据分支类型选择模板

#### 3. Branch LLM 服务 (`llm.rs`)

**职责**：提供使用 LLM 处理分支名称的功能

**主要方法**：
- `translate-_to-_english()` - 将非英文文本翻译为英文

**关键特性**：
- 使用统一的 LLM 客户端（支持多种提供商）
- 使用翻译系统提示词
- 自动清理响应（去除引号、多余空格）

**使用场景**：
- `BranchNaming::sanitize-_and-_translate-_branch-_name()` - 处理非英文输入

### 设计模式

#### 1. 策略模式（分支名生成）

分支名生成采用三层策略：
1. **模板系统**：优先使用模板系统生成分支名（如果可用）
2. **LLM 生成**：模板系统不可用时，使用 LLM 生成
3. **简单回退**：LLM 失败时，使用简单的 slugify 方法

**优势**：
- 灵活性强：支持多种生成方式
- 容错性好：自动回退到简单方法
- 可扩展：易于添加新的生成策略

#### 2. 配置分组模式

分支配置按仓库分组，每个仓库有独立的配置：
- 分支前缀（可选）
- 忽略列表
- 提示标记（内部字段）

**优势**：
- 支持多仓库：不同仓库可以有不同的配置
- 配置隔离：仓库之间互不影响
- 易于管理：配置结构清晰

#### 3. 智能前缀检测

自动检测并避免重复前缀：
- 检测 JIRA ticket 前缀是否已存在
- 检测仓库前缀是否已存在
- 检测仓库前缀是否已作为分支类型使用

**优势**：
- 避免重复：不会生成 `feature/feature/...` 这样的分支名
- 智能合并：自动处理各种前缀组合情况

### 错误处理

#### 分层错误处理

1. **配置层错误**：配置加载/保存失败时，返回错误但不会中断流程（`check-_and-_prompt-_prefix()`）
2. **LLM 层错误**：LLM 调用失败时，自动回退到简单方法
3. **模板层错误**：模板渲染失败时，自动回退到 LLM 或简单方法
4. **验证层错误**：分支名验证失败时，返回明确的错误信息

#### 容错机制

- **LLM 失败**：自动回退到简单 slugify 方法
- **翻译失败**：使用原始输入，让 `sanitize()` 处理
- **配置加载失败**：使用默认配置（空配置）
- **用户取消输入**：标记为已提示，避免重复提示

---

## 📁 Commands 层架构（命令封装）

分支管理命令模块提供完整的分支生命周期管理功能，包括分支创建、切换、重命名、清理和忽略列表管理。

> **架构说明**：本模块遵循项目的三层架构设计，详见 [architecture.md](./architecture.md#三层架构设计)

### 相关文件

#### CLI 入口层

分支管理命令现在作为 `workflow` 主命令的子命令，通过 `src/main.rs` 中的 `Commands::Branch` 枚举定义。

```
src/main.rs
```
- **职责**：`workflow` 主命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将 `workflow branch` 子命令分发到对应的命令处理函数

#### 命令封装层

```
src/commands/branch/
├── mod.rs          # 分支命令模块声明
├── create.rs       # 分支创建命令（~347 行）
├── switch.rs       # 分支切换命令（~104 行）
├── rename.rs       # 分支重命名命令（~357 行）
├── clean.rs        # 分支清理命令（~195 行）
├── ignore.rs       # 分支忽略列表管理命令（~199 行）
└── helpers.rs      # 辅助函数（分支选择等，~260 行）
```

**职责**：
- 解析命令参数
- 处理用户交互（确认、预览等）
- 格式化输出
- 调用核心业务逻辑层 (`lib/git/`) 的功能
- 管理项目级分支配置文件（`.workflow/config.toml`）

### 依赖模块

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：
- **`lib/git/`**：Git 操作（`GitBranch`、`GitRepo`）
  - `GitBranch::current-_branch()` - 获取当前分支
  - `GitBranch::get-_default-_branch()` - 获取默认分支
  - `GitBranch::get-_all-_branches()` - 获取所有本地分支
  - `GitBranch::delete()` - 删除分支
  - `GitRepo::extract-_repo-_name()` - 提取仓库名
  - `GitRepo::prune-_remote()` - 清理远端引用
- **`commands/check/`**：环境检查（`CheckCommand::run-_all()`）
- **`lib/base/util/`**：工具函数（`confirm()`）
- **`lib/jira/config.rs`**：配置管理器（`ConfigManager`）

---

## 🔄 集成关系

### Lib 层和 Commands 层的协作

Branch 模块采用清晰的分层架构，Lib 层和 Commands 层通过以下方式协作：

1. **分支命名**：Commands 层调用 `BranchNaming` 生成分支名，Lib 层负责实现命名策略
2. **分支类型**：Commands 层使用 `BranchType` 进行交互式选择，Lib 层提供类型定义和选择功能
3. **Git 操作**：Commands 层调用 `GitBranch` 进行实际的分支操作，Lib 层专注于命名逻辑
4. **配置管理**：Commands 层通过 `RepoConfig` 管理配置，Lib 层读取配置用于命名

### 数据流向

#### 创建分支数据流

```
用户输入 (workflow branch create [JIRA_ID])
  ↓
Commands 层 (解析参数、处理交互)
  ↓
Lib 层 (BranchNaming::from-_jira-_ticket())
  ↓
模板系统/LLM/简单方法
  ↓
返回分支名
  ↓
Commands 层 (GitBranch::create())
```

#### 切换分支数据流

```
用户输入 (workflow branch switch [BRANCH_NAME])
  ↓
Commands 层 (分支选择、stash 处理)
  ↓
Lib 层 (GitBranch::checkout-_branch())
  ↓
Git 操作
```

---

## 📋 Commands 层命令详情

### 1. 分支创建命令 (`create.rs`)

分支创建命令提供智能的分支创建功能：

1. **JIRA 集成**：
   - 支持从 JIRA ticket ID 创建分支
   - 使用 LLM 从 ticket 信息自动生成分支名
   - 自动验证 JIRA ticket 格式

2. **分支类型确定**：
   - 优先使用仓库配置的分支前缀作为分支类型
   - 如果未配置，则交互式选择（feature/bugfix/refactoring/hotfix/chore）

3. **分支名生成**：
   - 从 JIRA ticket：使用 LLM 生成分支名 slug
   - 手动输入：支持非英文输入，自动转换为 slug
   - 格式化：`{type}/{jira-ticket}-{branch-name}`

4. **基础分支选择**：
   - `--from-default`：从默认分支（main/master）创建
   - 默认：从当前分支创建（可选拉取最新更改）

5. **安全机制**：
   - Dry-run 模式：预览将要创建的分支名
   - 自动处理未提交的更改（stash）
   - 自动拉取最新更改（可选）

### 2. 分支切换命令 (`switch.rs`)

分支切换命令提供快速、智能的分支切换功能：

1. **直接切换**：
   - 支持直接指定分支名切换
   - 如果分支不存在，使用 `ConfirmDialog` 询问是否创建

2. **交互式选择**：
   - 不带参数时自动进入交互式选择
   - 显示所有可用分支（本地 + 远程，已去重）
   - 标记当前分支（显示 "[current]"）

3. **智能搜索**：
   - 分支数量 > 25：自动启用 fuzzy filter，支持输入关键词实时过滤
   - 分支数量 <= 25：使用普通 selector，通过方向键浏览

4. **自动处理未提交更改**：
   - 自动检测未提交的更改
   - 切换前自动 stash
   - 切换后自动恢复 stash

### 3. 分支重命名命令 (`rename.rs`)

分支重命名命令提供完整的交互式分支重命名功能：

1. **完全交互式流程**：
   - 所有操作都通过交互式提示完成，无需记忆复杂参数
   - 清晰的步骤引导，每一步都有明确提示

2. **分支选择**：
   - 优先询问是否重命名当前分支（最常用场景）
   - 如果否，从分支列表选择

3. **新分支名验证**：
   - 验证分支名格式（Git 规范）
   - 检查本地是否存在（如果存在则提示错误，重新输入）

4. **预览和确认**：
   - 检查是否是默认分支（需要额外警告）
   - 显示完整的预览信息
   - 最终确认

5. **远程分支处理**：
   - 自动检测远程分支是否存在
   - 如果存在：显示警告信息，询问是否更新远程分支
   - 如果用户选择更新：二次确认后执行远程分支重命名

### 4. 分支清理命令 (`clean.rs`)

分支清理命令提供智能的分支清理功能：

1. **前置检查**：运行所有环境检查（git status、network 等）
2. **分支分类**：
   - 自动排除：当前分支、默认分支（main/master）、develop 分支
   - 配置文件排除：从项目级配置（`.workflow/config.toml`）读取忽略列表
   - 合并状态分类：区分已合并和未合并的分支
3. **安全机制**：
   - 预览模式：显示将要删除的分支列表
   - Dry-run 模式：只预览，不实际删除
   - 确认机制：删除前需要用户确认
   - 未合并分支：需要额外确认才能强制删除
4. **清理远端引用**：自动清理已删除的远端分支引用

### 5. 分支忽略列表管理命令 (`ignore.rs`)

分支忽略列表管理命令提供分支忽略列表的完整管理功能：

1. **添加分支到忽略列表**：
   - 自动检测当前仓库名
   - 检查分支是否已在列表中
   - 按仓库分组存储

2. **从忽略列表移除分支**：
   - 自动检测当前仓库名
   - 如果列表为空，自动清理仓库配置

3. **列出忽略分支**：
   - 显示当前仓库的所有忽略分支
   - 格式化输出，显示总数

---

## 🔄 调用流程与数据流

### 整体架构流程

```
调用者（命令层）
  ↓
lib/branch/naming.rs (BranchNaming)
  ↓
策略选择：
  1. 模板系统 (lib/template/) - 自动处理前缀
  2. LLM 生成 (lib/pr/llm/ 或 lib/base/llm/)
  3. 简单回退 (sanitize/slugify)
  ↓
模板系统自动处理前缀（JIRA ticket 前缀和仓库前缀）
  ↓
最终分支名（带前缀）
```

### 命令分发流程

```
src/main.rs::main()
  ↓
Cli::parse() (解析命令行参数)
  ↓
match cli.subcommand {
  BranchSubcommand::Create { jira-_id, from-_default, dry-_run } => CreateCommand::execute()
  BranchSubcommand::Switch { branch-_name } => SwitchCommand::execute()
  BranchSubcommand::Rename => BranchRenameCommand::execute()
  BranchSubcommand::Clean { dry-_run } => BranchCleanCommand::clean()
  BranchSubcommand::Ignore { subcommand } => match subcommand {
    IgnoreSubcommand::Add { branch-_name } => BranchIgnoreCommand::add()
    IgnoreSubcommand::Remove { branch-_name } => BranchIgnoreCommand::remove()
    IgnoreSubcommand::List => BranchIgnoreCommand::list()
  }
}
```

---

## 📋 使用示例

### Create 命令

```bash
# 从 JIRA ticket 创建分支（交互式选择分支类型）
workflow branch create PROJ-123

# 从 JIRA ticket 创建分支，从默认分支创建
workflow branch create PROJ-123 --from-default

# 手动输入创建分支（交互式输入分支名和类型）
workflow branch create

# 预览模式（不实际创建）
workflow branch create PROJ-123 --dry-run
```

### Switch 命令

```bash
# 直接指定分支名切换（不存在时询问是否创建）
workflow branch switch feature/new-feature

# 交互式选择分支（分支数量 > 25 时自动启用搜索）
workflow branch switch
```

### Rename 命令

```bash
# 交互式重命名分支（完全交互式，支持本地和远程分支）
workflow branch rename
```

### Clean 命令

```bash
# 预览将要删除的分支（dry-run）
workflow branch clean --dry-run

# 清理分支（需要确认）
workflow branch clean
```

### Ignore 命令

```bash
# 添加分支到忽略列表
workflow branch ignore add feature-branch

# 从忽略列表移除分支
workflow branch ignore remove feature-branch

# 列出忽略的分支
workflow branch ignore list
```

---

## 📝 扩展性

### 添加新的分支类型

1. 在 `lib/branch/types.rs` 的 `BranchType` 枚举中添加新变体
2. 在 `as-_str()` 方法中添加字符串映射
3. 在 `display-_name()` 方法中添加显示名称
4. 在 `from-_str()` 方法中添加解析逻辑
5. 在 `all()` 方法中添加新类型

### 添加新的分支名生成策略

1. 在 `lib/branch/naming.rs` 的 `from-_jira-_ticket()` 方法中添加新策略
2. 实现新的生成函数
3. 在策略链中插入新策略（按优先级顺序）

---

## 📚 相关文档

- [主架构文档](./architecture.md)
- [Git 模块架构文档](./git.md) - Git 操作详情
- [Jira 模块架构文档](./jira.md) - Jira 集成详情
- [LLM 模块架构文档](./llm.md) - AI 功能详情
- [PR 模块架构文档](./pr.md) - PR 相关功能（使用分支模块）

---

## ✅ 总结

Branch 模块采用清晰的分层架构和策略模式设计：

1. **分支命名服务**：提供多种生成策略（模板系统 → LLM → 简单回退）
2. **分支前缀管理**：统一处理前缀逻辑，智能避免重复
3. **分支类型定义**：类型安全的分支类型枚举
4. **LLM 集成**：支持非英文翻译和智能生成
5. **命令封装**：提供完整的分支生命周期管理功能

**设计优势**：
- ✅ **策略模式**：灵活的分支名生成策略，易于扩展
- ✅ **智能前缀**：自动检测并避免重复前缀
- ✅ **配置集中**：分支配置统一通过 `lib/repo/config.rs` 管理
- ✅ **容错设计**：多层回退机制，确保总能生成分支名
- ✅ **类型安全**：使用枚举和类型系统保证类型安全
- ✅ **用户友好**：自动提示配置，支持非英文输入翻译
- ✅ **智能切换**：支持直接切换和交互式选择，自动处理未提交更改
- ✅ **安全重命名**：完全交互式流程，多重验证和确认

通过分层架构和策略模式，实现了灵活、可扩展、容错性强的分支命名和管理功能。命令层（`commands/branch/` 和 `commands/pr/`）使用本模块提供的接口，实现了完整的分支生命周期管理功能。

---

**最后更新**: 2025-12-16

