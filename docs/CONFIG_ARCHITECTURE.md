# 配置管理模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的配置管理模块架构，包括交互式配置设置和配置查看功能。这些命令负责管理 Workflow CLI 的所有 TOML 配置文件。

---

## 📁 相关文件

### CLI 入口层

```
src/main.rs
```

### 命令封装层

```
src/commands/
├── setup.rs        # 初始化设置命令（~686 行）
└── config.rs       # 配置查看命令（~131 行）
```

### 依赖模块

- **`lib/settings/`**：TOML 配置管理
  - `settings.rs`：配置结构体和加载逻辑
  - `paths.rs`：配置文件路径管理
  - `defaults.rs`：默认值辅助函数
- **`lib/utils/shell.rs`**：Shell 检测和配置管理（仅用于 completion）

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
main.rs (CLI 入口，参数解析)
  ↓
commands/setup.rs 或 commands/config.rs (命令封装层)
  ↓
lib/settings/ (TOML 配置管理)
  ├── paths.rs (配置文件路径)
  ├── settings.rs (配置加载和结构体)
  └── defaults.rs (默认值)
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

## 2. 配置查看命令 (`config.rs`)

### 相关文件

```
src/commands/config.rs
```

### 调用流程

```
main.rs::Commands::Config
  ↓
commands/config.rs::ConfigCommand::show()
  ↓
  1. ConfigPaths::workflow_config()          # 获取 workflow.toml 路径
  2. ConfigPaths::llm_config()               # 获取 llm.toml 路径
  3. Settings::get()                         # 加载配置（从 TOML）
  4. print_all_config()                       # 打印所有配置
     ├─ 敏感信息掩码（Token、Key 等）
     ├─ 布尔值转换（Yes/No）
     └─ 按逻辑分组显示
```

### 功能说明

1. **配置加载**：
   - 从 TOML 配置文件加载（`workflow.toml` 和 `llm.toml`）
   - 使用 `Settings::get()` 方法（带缓存，使用 `OnceLock`）
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

2. **配置排序**：
   - 按逻辑分组排序
   - 显示其他未列出的配置项

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

### 工具模块集成

- **`lib/settings/`**：TOML 配置管理
  - `settings.rs`：配置结构体定义和加载逻辑
  - `paths.rs`：统一管理配置文件路径
  - `defaults.rs`：默认值辅助函数
- **`lib/utils/shell.rs`**：Shell 检测和配置管理（仅用于 completion）

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

---

## 📝 扩展性

### 添加新配置项

1. 在 `lib/settings/settings.rs` 中添加新的 `XXSettings` 结构体
2. 在 `Settings` 结构体中添加新字段（使用 `#[serde(default)]`）
3. 在 `setup.rs` 的 `CollectedConfig` 结构体中添加字段
4. 在 `setup.rs` 的 `collect_config()` 方法中添加配置收集逻辑
5. 在 `setup.rs` 的 `save_config()` 方法中添加保存逻辑
6. 在 `config.rs` 的 `print_all_config()` 方法中添加配置显示逻辑
7. 如果需要默认值，在 `lib/settings/defaults.rs` 中添加辅助函数

---

## 📚 相关文档

- [主架构文档](./ARCHITECTURE.md)
- [安装/卸载模块架构文档](./INSTALL_ARCHITECTURE.md)

