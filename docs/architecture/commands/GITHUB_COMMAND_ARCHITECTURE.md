# GitHub 账号管理命令模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 GitHub 账号管理命令模块架构，包括：
- 多账号管理功能
- 账号切换功能
- 账号配置管理功能

GitHub 账号管理命令提供完整的 GitHub 多账号管理功能，支持配置多个 GitHub 账号，并在不同账号之间切换。切换账号时会自动更新 Git 全局配置（user.name 和 user.email）。

**定位**：命令层专注于用户交互、参数解析和输出格式化，核心业务逻辑由 `lib/base/settings/` 和 `lib/git/` 模块提供。

---

## 📁 相关文件

### CLI 入口层

GitHub 账号管理命令现在作为 `workflow` 主命令的子命令，通过 `src/main.rs` 中的 `Commands::GitHub` 枚举定义。

```
src/main.rs
```
- **职责**：`workflow` 主命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将 `workflow github` 子命令分发到对应的命令处理函数

### 命令封装层

```
src/commands/github/
├── mod.rs          # GitHub 命令模块声明（3 行）
├── github.rs       # GitHub 账号管理命令（~444 行）
└── helpers.rs      # 辅助函数（账号收集、验证等，~163 行）
```

**职责**：
- 解析命令参数
- 处理用户交互（输入、选择等）
- 格式化输出
- 调用核心业务逻辑层 (`lib/base/settings/`、`lib/git/`) 的功能
- 管理 GitHub 账号配置（`workflow.toml` 中的 `github.accounts` 数组）

### 依赖模块

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：
- **`lib/base/settings/`**：配置管理（`Settings`、`Paths`）
  - `Settings::load()` - 加载配置
  - `Paths::workflow_config()` - 获取配置文件路径
- **`lib/git/`**：Git 配置管理（`GitConfig`）
  - `GitConfig::set_global_user()` - 设置 Git 全局用户配置
- **`lib/jira/config.rs`**：配置管理器（`ConfigManager`）
  - `ConfigManager::<Settings>::update()` - 更新配置
- **`lib/base/util/`**：工具函数（`confirm()`、`mask_sensitive_value()`）
- **`dialoguer`**：交互式输入（`Input`、`Select`）

详细架构文档：参见 [Settings 模块架构文档](../lib/SETTINGS_ARCHITECTURE.md) 和 [Git 模块架构文档](../lib/GIT_ARCHITECTURE.md)

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
src/main.rs (workflow 主命令，参数解析)
  ↓
commands/github/*.rs (命令封装层，处理交互)
  ↓
lib/base/settings/* 和 lib/git/* (通过 API 调用，具体实现见相关模块文档)
  ↓
~/.workflow/config/workflow.toml (配置文件)
```

### 命令分发流程

```
src/main.rs::main()
  ↓
Cli::parse() (解析命令行参数)
  ↓
match cli.subcommand {
  GitHubSubcommand::List => GitHubCommand::list()
  GitHubSubcommand::Current => GitHubCommand::current()
  GitHubSubcommand::Add => GitHubCommand::add()
  GitHubSubcommand::Remove => GitHubCommand::remove()
  GitHubSubcommand::Switch => GitHubCommand::switch()
  GitHubSubcommand::Update => GitHubCommand::update()
}
```

---

## 1. 列出所有账号命令 (`list`)

### 调用流程

```
src/main.rs::GitHubSubcommand::List
  ↓
commands/github/github.rs::GitHubCommand::list()
  ↓
  1. Settings::load() (加载配置)
  2. 检查账号列表是否为空
  3. 遍历所有账号，格式化显示
  4. 标记当前账号
```

### 功能说明

列出所有配置的 GitHub 账号，显示每个账号的详细信息：
- 账号名称
- 邮箱
- API Token（掩码显示）
- 分支前缀（如果有）

**当前账号标记**：
- 如果设置了 `github.current`，标记对应的账号
- 如果没有设置 `github.current`，第一个账号被视为当前账号

---

## 2. 显示当前账号命令 (`current`)

### 调用流程

```
src/main.rs::GitHubSubcommand::Current
  ↓
commands/github/github.rs::GitHubCommand::current()
  ↓
  1. Settings::load() (加载配置)
  2. Settings::github.get_current_account() (获取当前账号)
  3. 格式化显示当前账号信息
```

### 功能说明

显示当前激活的 GitHub 账号信息：
- 账号名称
- 邮箱
- API Token（掩码显示）
- 分支前缀（如果有）

如果没有任何账号或当前账号未设置，显示警告信息。

---

## 3. 添加账号命令 (`add`)

### 调用流程

```
src/main.rs::GitHubSubcommand::Add
  ↓
commands/github/github.rs::GitHubCommand::add()
  ↓
  1. collect_github_account() (交互式收集账号信息)
  2. 检查账号名称是否已存在
  3. 判断是否为第一个账号
  4. ConfigManager::update() (保存到配置文件)
  5. 如果是第一个账号，自动设置为当前账号并更新 Git 配置
  6. 如果不是第一个账号，询问是否设为当前账号
```

### 功能说明

添加新的 GitHub 账号到配置：

1. **交互式收集信息**：
   - 账号名称（必填）
   - 邮箱（必填，需包含 @）
   - API Token（必填）
   - 分支前缀（可选）

2. **自动处理**：
   - 如果是第一个账号，自动设置为当前账号
   - 自动更新 Git 全局配置（user.name 和 user.email）

3. **用户选择**：
   - 如果不是第一个账号，询问是否将新账号设为当前账号

### 关键步骤说明

1. **账号信息收集**：
   - 使用 `helpers::collect_github_account()` 交互式收集
   - 输入验证（名称非空、邮箱格式、Token 非空）

2. **配置更新**：
   - 使用 `ConfigManager::<Settings>::update()` 更新配置
   - 添加到 `github.accounts` 数组
   - 如果是第一个账号，设置 `github.current`

3. **Git 配置更新**：
   - 使用 `GitConfig::set_global_user()` 更新 Git 全局配置
   - 确保 Git 提交使用正确的用户信息

---

## 4. 删除账号命令 (`remove`)

### 调用流程

```
src/main.rs::GitHubSubcommand::Remove
  ↓
commands/github/github.rs::GitHubCommand::remove()
  ↓
  1. Settings::load() (加载配置)
  2. 检查账号列表是否为空
  3. 交互式选择要删除的账号
  4. 确认删除
  5. 检查删除的是否为当前账号
  6. ConfigManager::update() (从配置中移除)
  7. 如果删除后还有账号，更新当前账号（如果需要）
  8. 更新 Git 配置（如果需要）
```

### 功能说明

删除指定的 GitHub 账号：

1. **交互式选择**：
   - 列出所有账号
   - 当前账号作为默认选中项
   - 用户选择要删除的账号

2. **确认机制**：
   - 删除前需要用户确认

3. **自动处理**：
   - 如果删除的是当前账号，自动设置第一个账号为当前账号
   - 如果删除后没有账号了，清空 `github.current`
   - 如果需要，更新 Git 全局配置

### 关键步骤说明

1. **当前账号处理**：
   - 如果删除的是当前账号，需要选择新的当前账号
   - 如果删除后还有账号，自动设置第一个账号为当前账号

2. **Git 配置更新**：
   - 如果删除的是当前账号，或当前账号不在列表中，更新 Git 配置
   - 使用新当前账号的信息更新 Git 全局配置

---

## 5. 切换账号命令 (`switch`)

### 调用流程

```
src/main.rs::GitHubSubcommand::Switch
  ↓
commands/github/github.rs::GitHubCommand::switch()
  ↓
  1. Settings::load() (加载配置)
  2. 检查账号列表（至少需要 2 个账号）
  3. 交互式选择要切换到的账号
  4. ConfigManager::update() (更新当前账号)
  5. GitConfig::set_global_user() (更新 Git 配置)
```

### 功能说明

切换到指定的 GitHub 账号：

1. **前置检查**：
   - 至少需要 2 个账号才能切换
   - 如果只有 1 个账号，显示警告

2. **交互式选择**：
   - 列出所有账号
   - 当前账号作为默认选中项
   - 用户选择要切换到的账号

3. **自动更新**：
   - 更新 `github.current` 字段
   - 更新 Git 全局配置（user.name 和 user.email）

---

## 6. 更新账号命令 (`update`)

### 调用流程

```
src/main.rs::GitHubSubcommand::Update
  ↓
commands/github/github.rs::GitHubCommand::update()
  ↓
  1. Settings::load() (加载配置)
  2. 检查账号列表是否为空
  3. 交互式选择要更新的账号
  4. 显示当前账号信息
  5. collect_github_account_with_defaults() (使用现有值作为默认值收集新信息)
  6. 检查账号名称是否冲突
  7. 检查是否为当前账号
  8. ConfigManager::update() (更新配置)
  9. 如果更新的是当前账号且名称或邮箱改变，更新 Git 配置
```

### 功能说明

更新指定 GitHub 账号的信息：

1. **交互式选择**：
   - 列出所有账号
   - 当前账号作为默认选中项
   - 用户选择要更新的账号

2. **信息更新**：
   - 使用现有值作为默认值
   - 用户可以直接按 Enter 保留现有值
   - 或输入新值覆盖

3. **自动处理**：
   - 如果账号名称改变且是当前账号，更新 `github.current`
   - 如果更新的是当前账号且名称或邮箱改变，更新 Git 配置

### 关键步骤说明

1. **默认值处理**：
   - 使用 `helpers::collect_github_account_with_defaults()` 收集信息
   - 所有字段都使用现有值作为默认值

2. **名称冲突检查**：
   - 如果账号名称改变，检查新名称是否与其他账号冲突
   - 排除当前正在更新的账号

3. **当前账号处理**：
   - 如果更新的是当前账号，且名称改变，需要更新 `github.current`
   - 如果名称或邮箱改变，需要更新 Git 配置

---

## 7. 辅助函数 (`helpers.rs`)

### 功能说明

辅助函数模块提供账号信息收集的共享逻辑：

1. **`collect_github_account()`**：
   - 交互式收集新的 GitHub 账号信息
   - 所有字段都是必填（除了分支前缀）

2. **`collect_github_account_with_defaults()`**：
   - 交互式收集 GitHub 账号信息，使用现有值作为默认值
   - 用于更新账号信息

### 输入验证

- **账号名称**：非空验证
- **邮箱**：非空验证，必须包含 @
- **API Token**：非空验证
- **分支前缀**：可选，可以为空

---

## 🏗️ 架构设计

### 设计模式

#### 1. 命令模式

每个命令都是一个独立的结构体，实现统一的方法接口：
- `GitHubCommand::list()` - 列出所有账号
- `GitHubCommand::current()` - 显示当前账号
- `GitHubCommand::add()` - 添加账号
- `GitHubCommand::remove()` - 删除账号
- `GitHubCommand::switch()` - 切换账号
- `GitHubCommand::update()` - 更新账号

#### 2. 配置管理模式

使用 `ConfigManager` 统一管理配置文件：
- 配置文件路径：`~/.workflow/config/workflow.toml`
- 配置结构：`github.accounts` 数组和 `github.current` 字段
- 自动更新 Git 配置，确保一致性

#### 3. 交互式输入模式

使用 `dialoguer` 库提供友好的交互界面：
- `Input` - 文本输入
- `Select` - 选择列表

---

## 🔍 错误处理

### 分层错误处理

1. **CLI 层**：参数验证错误
   - `clap` 自动处理参数验证和错误提示

2. **命令层**：用户交互错误、业务逻辑错误
   - 账号名称冲突：返回错误，提示用户
   - 账号不存在：返回错误或警告

3. **库层**：配置读写错误、Git 操作错误
   - 通过 `ConfigManager`、`GitConfig` API 返回的错误信息
   - 配置文件读写失败、Git 配置更新失败等

### 容错机制

- **配置文件不存在**：自动创建空配置
- **账号列表为空**：显示警告，提示用户添加账号
- **Git 配置更新失败**：返回错误，但不影响配置文件的更新

---

## 📝 扩展性

### 添加新的账号字段

1. 在 `lib/base/settings/settings.rs` 的 `GitHubAccount` 结构体中添加新字段
2. 在 `helpers.rs` 的收集函数中添加新字段的输入逻辑
3. 在 `github.rs` 的显示函数中添加新字段的显示逻辑

### 添加新的账号操作

1. 在 `github.rs` 中添加新的命令方法
2. 在 `src/main.rs` 中添加新的子命令枚举
3. 在命令分发逻辑中添加处理代码

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [Settings 模块架构文档](../lib/SETTINGS_ARCHITECTURE.md) - 配置管理相关
- [Git 模块架构文档](../lib/GIT_ARCHITECTURE.md) - Git 配置相关
- [配置管理命令模块架构文档](./CONFIG_COMMAND_ARCHITECTURE.md) - 配置管理相关

---

## 📋 使用示例

### List 命令

```bash
# 列出所有 GitHub 账号
workflow github list
```

### Current 命令

```bash
# 显示当前 GitHub 账号
workflow github current
```

### Add 命令

```bash
# 添加新的 GitHub 账号
workflow github add
```

### Remove 命令

```bash
# 删除 GitHub 账号
workflow github remove
```

### Switch 命令

```bash
# 切换 GitHub 账号
workflow github switch
```

### Update 命令

```bash
# 更新 GitHub 账号信息
workflow github update
```

---

## ✅ 总结

GitHub 账号管理命令层采用清晰的多账号管理设计：

1. **多账号支持**：支持配置和管理多个 GitHub 账号
2. **自动同步**：切换账号时自动更新 Git 配置
3. **交互友好**：使用交互式输入和选择，用户友好

**设计优势**：
- ✅ **灵活性**：支持多账号管理，满足不同场景需求
- ✅ **一致性**：自动同步 Git 配置，确保提交信息正确
- ✅ **易用性**：交互式操作，清晰的提示和反馈
- ✅ **可扩展性**：易于添加新的账号字段和操作
