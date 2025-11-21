# 配置管理命令模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的配置管理模块架构，包括：
- 交互式配置设置和配置查看功能
- GitHub 账号管理（多账号支持）
- 日志级别管理
- 配置管理辅助函数
- 环境检查功能（Git 仓库状态、网络连接）

这些命令负责管理 Workflow CLI 的所有 TOML 配置文件，使用统一的 `ConfigManager` 进行配置更新，并提供环境诊断和验证功能。

---

## 📁 相关文件

### CLI 入口层

```
src/main.rs
```

### 命令封装层

```
src/commands/config/
├── setup.rs        # 初始化设置命令（~654 行）
├── show.rs         # 配置查看命令（~52 行）
├── github.rs       # GitHub 账号管理命令（~442 行）
├── log.rs          # 日志级别管理命令（~108 行）
├── helpers.rs      # 配置管理辅助函数（~164 行）
└── check.rs        # 环境检查命令（~68 行）
```

### 依赖模块（简要说明）

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：
- **`lib/base/settings/`**：TOML 配置管理（`Settings`、`Paths`）
- **`lib/git/`**：Git 操作（`GitRepo`、`GitCommit`）
- **`lib/base/http/`**：HTTP 客户端（`HttpClient`）
- **`lib/jira/config.rs`**：配置管理器（`ConfigManager`）
- **`lib/git/config.rs`**：Git 配置管理（`GitConfig`）
- **`lib/base/util/`**：工具函数（`LogLevel`、`mask_sensitive_value()`）

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
main.rs (CLI 入口，参数解析)
  ↓
commands/config/* (命令封装层)
  ↓
lib/* (通过 API 调用，具体实现见相关模块文档)
  ↓
~/.workflow/config/workflow.toml 和 llm.toml (配置文件)
```

---

## 1. 初始化设置命令 (`setup.rs`)

### 相关文件

```
src/commands/setup.rs
```

### 调用流程

```
main.rs::Commands::Setup
  ↓
commands/setup.rs::SetupCommand::run()
  ↓
  1. Settings::get()                          # 加载现有配置（从 TOML）
  2. load_existing_config()                   # 转换为 CollectedConfig
  3. collect_config()                          # 收集配置信息（交互式）
     ├─ 用户配置（EMAIL）
     ├─ Jira 配置（地址、Token）
     ├─ GitHub 配置（Token、分支前缀）
     ├─ 日志配置（文件夹、删除策略）
     ├─ 代理配置（是否禁用检查）
     ├─ LLM 配置（提供商、API Key）
     └─ Codeup 配置（项目 ID、CSRF Token、Cookie）
  4. save_config()                             # 保存配置到 TOML 文件
     ├─ workflow.toml (主配置)
     └─ llm.toml (LLM 配置，如果存在)
  5. verify_config()                            # 验证配置（可选）
```

### 功能说明

1. **智能配置处理**：
   - 自动检测现有配置（从 TOML 配置文件）
   - 支持保留现有值（按 Enter 跳过）
   - 支持覆盖现有值（输入新值）

2. **配置分组**：
   - **必填项**：用户配置（EMAIL）、Jira 配置、GitHub 配置
   - **可选项**：日志、代理、LLM、Codeup 配置

3. **交互式输入**：
   - 使用 `dialoguer` 库提供友好的交互界面
   - 输入验证（邮箱格式、URL 格式等）
   - 敏感信息掩码显示

4. **配置保存**：
   - 保存到 TOML 配置文件（`~/.workflow/config/workflow.toml`）
   - LLM 配置单独保存到 `~/.workflow/config/llm.toml`（如果配置了 LLM）
   - 使用 `toml` crate 进行序列化
   - 配置文件不存在时自动创建目录

### 关键步骤说明

1. **配置收集**：
   - 按逻辑分组收集配置项
   - 每个配置项都有默认值和验证逻辑
   - 支持跳过可选配置
   - 从 `Settings::get()` 读取现有配置作为默认值

2. **配置验证**：
   - 邮箱格式验证
   - URL 格式验证
   - 必填项检查
   - 配置保存后可选验证（Jira、GitHub、Codeup）

3. **配置保存**：
   - 使用 `ConfigPaths` 获取配置文件路径
   - 使用 `toml::to_string_pretty()` 序列化为 TOML 格式
   - 主配置保存到 `workflow.toml`
   - LLM 配置单独保存到 `llm.toml`（如果存在）

---

## 2. 配置查看命令 (`show.rs`)

### 相关文件

```
src/commands/config/show.rs
```

### 调用流程

```
main.rs::Commands::Config
  ↓
commands/config/show.rs::ConfigCommand::show()
  ↓
  1. Paths::workflow_config()                # 获取 workflow.toml 路径
  2. Settings::load()                        # 加载配置（从 TOML，不使用缓存）
  3. Settings::verify()                      # 验证并打印所有配置
     ├─ 敏感信息掩码（Token、Key 等）
     ├─ 布尔值转换（Yes/No）
     └─ 按逻辑分组显示
```

### 功能说明

1. **配置加载**：
   - 从 TOML 配置文件加载（`workflow.toml` 和 `llm.toml`）
   - 使用 `Settings::load()` 方法（不使用缓存，获取最新配置）
   - 配置文件不存在时使用默认值

2. **配置显示**：
   - 按逻辑分组和顺序显示
   - 敏感信息自动掩码（Token、Key 等）
   - 布尔值转换为可读格式（Yes/No）
   - 显示配置文件路径

3. **配置分组**：
   - 用户配置
   - Jira 配置
   - GitHub 配置
   - 日志配置
   - 代理配置
   - LLM 配置
   - Codeup 配置

### 关键步骤说明

1. **敏感信息掩码**：
   - 使用 `mask_sensitive_value()` 函数
   - 只显示前 4 个字符和后 4 个字符，中间用 `***` 替代

2. **配置验证**：
   - 使用 `Settings::verify()` 方法统一验证和显示配置
   - 自动处理空配置情况

---

## 3. GitHub 账号管理命令 (`github.rs`)

### 相关文件

```
src/commands/config/github.rs
```

### 调用流程

```
main.rs::Commands::GitHub { subcommand }
  ↓
commands/config/github.rs::GitHubCommand::{list|current|add|remove|switch|update}()
  ↓
  1. Settings::load()                        # 加载配置
  2. 执行相应的账号管理操作
  3. ConfigManager::<Settings>::update()    # 更新配置
  4. GitConfig::set_global_user()            # 更新 Git 全局配置（如需要）
```

### 功能说明

GitHub 账号管理命令提供多账号管理功能，支持以下子命令：

1. **`list`** - 列出所有 GitHub 账号
   - 显示所有已配置的 GitHub 账号
   - 标记当前激活的账号
   - 显示账号的详细信息（名称、邮箱、Token、分支前缀）

2. **`current`** - 显示当前激活的 GitHub 账号
   - 显示当前正在使用的 GitHub 账号信息
   - 如果没有激活的账号，提示用户添加

3. **`add`** - 添加新的 GitHub 账号
   - 交互式收集账号信息（名称、邮箱、API Token、分支前缀）
   - 检查账号名称是否已存在
   - 如果这是第一个账号，自动设置为当前账号
   - 自动更新 Git 全局配置（user.name 和 user.email）

4. **`remove`** - 删除 GitHub 账号
   - 交互式选择要删除的账号
   - 如果删除的是当前账号，自动切换到第一个账号
   - 如果删除后没有账号了，清空 current 字段
   - 更新 Git 全局配置（如需要）

5. **`switch`** - 切换当前 GitHub 账号
   - 交互式选择要切换到的账号
   - 更新 `Settings.github.current` 字段
   - 自动更新 Git 全局配置（user.name 和 user.email）

6. **`update`** - 更新 GitHub 账号信息
   - 交互式选择要更新的账号
   - 使用现有值作为默认值，支持部分更新
   - 如果更新的是当前账号，且 email 或 name 改变了，更新 Git 全局配置
   - 如果账号名称改变了，且是当前账号，更新 current 字段

### 关键步骤说明

1. **账号信息收集**：
   - 使用 `helpers::collect_github_account()` 收集新账号信息
   - 使用 `helpers::collect_github_account_with_defaults()` 更新现有账号信息
   - 验证输入（账号名称不能为空、邮箱格式验证等）

2. **配置更新**：
   - 使用 `ConfigManager::<Settings>::update()` 方法更新配置
   - 原子性更新，确保配置一致性

3. **Git 配置同步**：
   - 当添加、切换或更新账号时，自动更新 Git 全局配置
   - 使用 `GitConfig::set_global_user()` 设置 user.name 和 user.email

4. **当前账号管理**：
   - `Settings.github.current` 字段存储当前激活的账号名称
   - 如果没有设置 current，第一个账号被视为当前账号
   - 删除或切换账号时自动维护 current 字段

---

## 4. 日志级别管理命令 (`log.rs`)

### 相关文件

```
src/commands/config/log.rs
```

### 调用流程

```
main.rs::Commands::Log { subcommand }
  ↓
commands/config/log.rs::LogCommand::{set|check}()
  ↓
  1. LogLevel::get_level()                   # 获取当前日志级别
  2. 执行相应操作（设置或检查）
  3. ConfigManager::<Settings>::update()    # 保存到配置文件（如需要）
```

### 功能说明

日志级别管理命令提供日志输出级别的设置和检查功能：

1. **`set`** - 设置日志级别（交互式选择）
   - 显示当前日志级别
   - 提供日志级别选项：none, error, warn, info, debug
   - 交互式选择新的日志级别
   - 立即生效（内存中设置）
   - 保存到配置文件（`workflow.toml`）

2. **`check`** - 检查当前日志级别
   - 显示当前日志级别
   - 显示默认日志级别（基于构建模式）
   - 显示配置文件中的日志级别
   - 显示是否手动设置
   - 列出所有可用的日志级别

### 关键步骤说明

1. **日志级别设置**：
   - 使用 `LogLevel::set_level()` 在内存中设置日志级别
   - 使用 `ConfigManager::<Settings>::update()` 保存到配置文件
   - 设置后立即生效，无需重启

2. **日志级别层次**：
   - **none** - 不输出任何日志
   - **error** - 只输出错误消息
   - **warn** - 输出警告和错误消息
   - **info** - 输出信息、警告和错误消息（默认）
   - **debug** - 输出所有日志消息（包括调试信息）

3. **配置持久化**：
   - 日志级别保存在 `workflow.toml` 的 `[log]`  section 中
   - 使用 `Settings.log.level` 字段存储

---

## 5. 配置管理辅助函数 (`helpers.rs`)

### 相关文件

```
src/commands/config/helpers.rs
```

### 功能说明

配置管理辅助函数模块提供配置管理命令的共享逻辑，减少代码重复：

1. **`collect_github_account()`** - 收集 GitHub 账号信息
   - 交互式收集账号名称（必填）
   - 交互式收集邮箱（必填，需包含 @）
   - 交互式收集 API Token（必填）
   - 交互式收集分支前缀（可选）
   - 输入验证（非空验证、邮箱格式验证等）

2. **`collect_github_account_with_defaults()`** - 收集 GitHub 账号信息（带默认值）
   - 与 `collect_github_account()` 类似
   - 使用现有账号信息作为默认值
   - 用户可以直接按 Enter 保留现有值
   - 支持部分更新账号信息

### 关键步骤说明

1. **输入验证**：
   - 账号名称：不能为空
   - 邮箱：不能为空，必须包含 @ 符号
   - API Token：不能为空
   - 分支前缀：可选，可以为空

2. **数据清理**：
   - 自动去除输入值的前后空格
   - 空字符串转换为 `None`（对于可选字段）

3. **使用场景**：
   - `collect_github_account()` 用于添加新账号（`github.rs::add()`）
   - `collect_github_account_with_defaults()` 用于更新现有账号（`github.rs::update()`）
   - 也被 `setup.rs` 使用，用于初始化配置

---

## 6. 环境检查命令 (`check.rs`)

### 相关文件

```
src/commands/config/check.rs
```

### 调用流程

```
main.rs::Commands::Check
  ↓
commands/config/check.rs::CheckCommand::run_all()
  ↓
  1. GitRepo::is_git_repo()                  # 检查是否在 Git 仓库中
  2. GitCommit::status()                     # 检查 Git 状态
  3. HttpClient::global().stream()          # 检查网络连接（GitHub）
```

### 功能说明

环境检查命令提供工作环境的诊断和验证功能：

1. **Git 仓库检查**：
   - 检查当前目录是否是 Git 仓库
   - 检查 Git 状态（是否有未提交的更改）
   - 使用 `GitRepo::is_git_repo()` 检测仓库
   - 使用 `GitCommit::status()` 获取状态信息

2. **网络连接检查**：
   - 检查到 GitHub 的网络连接
   - 使用 `HttpClient::global()` 发送 HTTP 请求
   - 超时时间设置为 10 秒
   - 验证响应状态码

### 关键步骤说明

1. **Git 检查**：
   - 如果不在 Git 仓库中，直接失败并提示错误
   - 如果有未提交的更改，显示 Git 状态信息
   - 如果仓库干净（无未提交更改），显示成功信息

2. **网络检查**：
   - 使用 `HttpClient::stream()` 方法发送 GET 请求到 `https://github.com`
   - 如果请求失败，提供详细的错误信息和建议
   - 如果响应状态码不是成功状态，报告错误
   - 如果检查通过，显示成功信息

3. **检查顺序**：
   - 按顺序执行检查（Git 检查 → 网络检查）
   - 如果任何检查失败，整个命令失败
   - 所有检查通过后，显示总体成功信息

### 错误处理

- **Git 检查失败**：
  - 如果不在 Git 仓库中，提供清晰的错误提示
  - 如果 Git 状态检查失败，提供上下文错误信息

- **网络检查失败**：
  - 提供详细的错误信息（网络问题、代理设置、防火墙限制等）
  - 给出可能的解决建议

---

## 📊 数据流

### 配置管理数据流

```
用户输入（交互式）
  ↓
命令层处理（验证、格式化）
  ↓
Settings 管理（读取/写入 TOML 配置文件）
  ├── workflow.toml (主配置)
  └── llm.toml (LLM 配置)
  ↓
配置缓存（OnceLock，单次加载）
  ↓
应用使用配置
```

### 配置文件结构

```
~/.workflow/
└── config/
    ├── workflow.toml      # 主配置文件
    ├── llm.toml           # LLM 配置（可选）
    ├── jira-status.toml   # Jira 状态配置
    └── jira-users.toml    # Jira 用户缓存
```

---

## 🔗 与其他模块的集成

命令层通过调用 `lib/` 模块提供的 API 实现功能，主要使用的接口包括：

- **`lib/base/settings/`**：`Settings::get()`、`Settings::load()`、`Paths::workflow_config()`
- **`lib/jira/config.rs`**：`ConfigManager::<Settings>::update()` - 原子性更新配置
- **`lib/git/config.rs`**：`GitConfig::set_global_user()` - 同步 Git 全局配置
- **`lib/base/util/`**：`LogLevel::get_level()`、`LogLevel::set_level()`、`mask_sensitive_value()`
- **`lib/git/`**：`GitRepo::is_git_repo()`、`GitCommit::status()` - 环境检查
- **`lib/base/http/`**：`HttpClient::global().stream()` - 网络检查

详细实现请参考相关模块架构文档。

### 配置文件位置

- **主配置**：`~/.workflow/config/workflow.toml`
- **LLM 配置**：`~/.workflow/config/llm.toml`（可选）
- **Jira 状态配置**：`~/.workflow/config/jira-status.toml`
- **Jira 用户缓存**：`~/.workflow/config/jira-users.toml`

---

## 🎯 设计模式

### 1. 命令模式

每个命令都是一个独立的结构体，实现统一的方法接口。

### 2. 配置管理模式

使用 `Settings` 结构体和 `ConfigPaths` 统一管理 TOML 配置文件的读写：
- **单例模式**：使用 `OnceLock` 实现配置的单次加载和缓存
- **路径管理**：使用 `ConfigPaths` 统一管理所有配置文件路径
- **默认值**：使用 `#[serde(default)]` 和 `Default` trait 提供默认值
- **分离存储**：主配置和 LLM 配置分别存储在不同的文件中

---

## 🔍 错误处理

### 分层错误处理

1. **CLI 层**：参数验证错误
2. **命令层**：用户交互错误、业务逻辑错误
3. **工具层**：文件操作错误、配置读写错误

### 容错机制

- **配置加载失败**：使用默认值或提示用户运行 `setup`
- **文件操作失败**：提供清晰的错误提示和手动操作建议
- **配置文件不存在**：自动使用默认值，不影响程序运行
- **TOML 解析失败**：使用默认值，并记录错误（如果启用日志）
- **环境检查失败**：提供详细的错误信息和解决建议（Git 检查、网络检查）

---

## 📝 扩展性

### 添加新配置项

1. 在 `lib/base/settings/settings.rs` 中添加新的配置结构体（参考相关模块文档）
2. 在 `setup.rs` 的 `CollectedConfig` 结构体中添加字段
3. 在 `setup.rs` 的 `collect_config()` 方法中添加配置收集逻辑
4. 在 `setup.rs` 的 `save_config()` 方法中添加保存逻辑
5. 在 `show.rs` 中，`Settings::verify()` 会自动显示新配置项

### 添加新的配置管理子命令

1. 在 `src/commands/config/` 中创建新的命令文件（如 `xxx.rs`）
2. 实现命令结构体和方法（参考 `github.rs` 或 `log.rs`）
3. 在 `src/commands/config/mod.rs` 中声明模块
4. 在 `src/main.rs` 中添加命令枚举和分发逻辑
5. 如果需要共享逻辑，在 `helpers.rs` 中添加辅助函数
6. 使用 `ConfigManager::<Settings>::update()` 更新配置（推荐）

### 添加新的环境检查项

1. 在 `check.rs` 的 `run_all()` 方法中添加新的检查步骤
2. 调用相应的工具函数或库函数
3. 提供清晰的错误提示和解决建议
4. 更新检查步骤的编号和日志输出

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [生命周期管理命令模块架构文档](./LIFECYCLE_COMMAND_ARCHITECTURE.md)
- [Git 模块架构文档](../lib/GIT_ARCHITECTURE.md) - Git 操作和环境检查相关
- [HTTP 模块架构文档](../lib/HTTP_ARCHITECTURE.md) - HTTP 客户端和网络检查相关
- [Jira 模块架构文档](../lib/JIRA_ARCHITECTURE.md) - ConfigManager 使用说明

