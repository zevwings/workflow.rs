# 配置管理模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的配置管理模块架构，包括交互式配置设置和配置查看功能。这些命令负责管理 Workflow CLI 的所有环境变量配置。

---

## 📁 相关文件

### CLI 入口层

```
src/main.rs
```

### 命令封装层

```
src/commands/
├── setup.rs        # 初始化设置命令（513 行）
└── config.rs       # 配置查看命令（125 行）
```

### 依赖模块

- **`lib/utils/env.rs`**：环境变量读写管理
- **`lib/utils/shell.rs`**：Shell 检测和配置管理
- **`lib/settings/`**：配置管理（环境变量读取）

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
lib/utils/env.rs (核心业务逻辑层)
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
  1. EnvFile::load_merged()                  # 加载现有配置
  2. collect_config()                         # 收集配置信息（交互式）
     ├─ 用户配置（EMAIL）
     ├─ Jira 配置（地址、Token）
     ├─ GitHub 配置（Token、分支前缀）
     ├─ 日志配置（文件夹、删除策略）
     ├─ 代理配置（是否禁用检查）
     ├─ LLM 配置（提供商、API Key）
     └─ Codeup 配置（项目 ID、CSRF Token、Cookie）
  3. EnvFile::save()                          # 保存配置到 shell 配置文件
  4. std::env::set_var()                      # 更新当前进程环境变量
  5. Shell::reload_config()                   # 重新加载 shell 配置
```

### 功能说明

1. **智能配置处理**：
   - 自动检测现有配置（从 shell 配置文件和当前环境变量）
   - 支持保留现有值（按 Enter 跳过）
   - 支持覆盖现有值（输入新值）

2. **配置分组**：
   - **必填项**：用户配置（EMAIL）、Jira 配置
   - **可选项**：GitHub、日志、代理、LLM、Codeup 配置

3. **交互式输入**：
   - 使用 `dialoguer` 库提供友好的交互界面
   - 输入验证（邮箱格式、URL 格式等）
   - 敏感信息掩码显示

4. **配置保存**：
   - 保存到 shell 配置文件（`~/.zshrc` 或 `~/.bashrc`）
   - 使用 `EnvFile::save()` 方法统一管理
   - 自动更新当前进程环境变量

### 关键步骤说明

1. **配置收集**：
   - 按逻辑分组收集配置项
   - 每个配置项都有默认值和验证逻辑
   - 支持跳过可选配置

2. **配置验证**：
   - 邮箱格式验证
   - URL 格式验证
   - 必填项检查

3. **配置保存**：
   - 统一保存到 shell 配置文件
   - 使用 Workflow 配置块管理
   - 自动重新加载配置

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
  1. EnvFile::get_shell_config_path()        # 获取配置文件路径
  2. EnvFile::load_merged()                  # 加载合并后的配置
  3. print_all_config()                       # 打印所有配置
     ├─ 敏感信息掩码（Token、Key 等）
     ├─ 布尔值转换（Yes/No）
     └─ 按逻辑顺序显示
```

### 功能说明

1. **配置加载**：
   - 从多个来源加载配置（当前环境变量 > shell 配置文件）
   - 使用 `EnvFile::load_merged()` 方法

2. **配置显示**：
   - 按逻辑分组和顺序显示
   - 敏感信息自动掩码（Token、Key 等）
   - 布尔值转换为可读格式（Yes/No）

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
EnvFile 管理（读取/写入 shell 配置文件）
  ↓
环境变量更新（当前进程）
  ↓
Shell 配置重新加载
```

---

## 🔗 与其他模块的集成

### 工具模块集成

- **`lib/utils/env.rs`**：环境变量读写管理
- **`lib/utils/shell.rs`**：Shell 检测和配置管理
- **`lib/settings/`**：配置管理（环境变量读取）

---

## 🎯 设计模式

### 1. 命令模式

每个命令都是一个独立的结构体，实现统一的方法接口。

### 2. 配置管理模式

使用 `EnvFile` 统一管理环境变量的读写，支持从多个来源加载和合并。

---

## 🔍 错误处理

### 分层错误处理

1. **CLI 层**：参数验证错误
2. **命令层**：用户交互错误、业务逻辑错误
3. **工具层**：文件操作错误、配置读写错误

### 容错机制

- **配置加载失败**：使用默认值或提示用户运行 `setup`
- **文件操作失败**：提供清晰的错误提示和手动操作建议

---

## 📝 扩展性

### 添加新配置项

1. 在 `setup.rs` 的 `collect_config()` 方法中添加配置收集逻辑
2. 在 `config.rs` 的 `print_all_config()` 方法中添加配置显示逻辑
3. 在 `lib/utils/env.rs` 的 `get_workflow_env_keys()` 方法中添加环境变量键

---

## 📚 相关文档

- [主架构文档](./ARCHITECTURE.md)
- [安装/卸载模块架构文档](./INSTALL_ARCHITECTURE.md)

