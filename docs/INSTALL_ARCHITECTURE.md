# 安装/卸载模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的安装和卸载模块架构，包括 shell completion 脚本的生成、安装和清理，以及二进制文件和配置的卸载功能。

---

## 📁 相关文件

### CLI 入口层

```
src/bin/install.rs (独立可执行文件入口)
src/main.rs (卸载命令入口)
```

### 命令封装层

```
src/commands/
├── install.rs      # 安装命令（254 行）
└── uninstall.rs    # 卸载命令（198 行）
```

### 依赖模块

- **`lib/utils/shell.rs`**：Shell 检测和配置管理
- **`lib/utils/completion.rs`**：Completion 脚本生成和配置
- **`lib/utils/env.rs`**：环境变量读写管理
- **`lib/utils/uninstall.rs`**：卸载工具函数

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
bin/install.rs 或 main.rs (CLI 入口，参数解析)
  ↓
commands/install.rs 或 commands/uninstall.rs (命令封装层)
  ↓
lib/utils/* (核心业务逻辑层)
```

---

## 1. 安装命令 (`install.rs`)

### 相关文件

```
src/commands/install.rs
src/bin/install.rs (独立可执行文件入口)
```

### 调用流程

```
bin/install.rs::main()
  ↓
commands/install.rs::InstallCommand::install()
  ↓
  1. install_completions()                    # 安装 shell completion
     ├─ Shell::detect()                      # 检测 shell 类型
     ├─ generate_completions()               # 生成 completion 脚本
     │   ├─ generate_workflow_completion()    # 生成 workflow 命令 completion
     │   ├─ generate_pr_completion()         # 生成 pr 命令 completion
     │   └─ generate_qk_completion()         # 生成 qk 命令 completion
     └─ Completion::configure_shell_config() # 配置 shell 配置文件
```

### 功能说明

1. **Shell 检测**：
   - 自动检测当前 shell 类型（zsh/bash）
   - 确定 completion 目录和配置文件路径

2. **Completion 生成**：
   - 为 `workflow`、`pr`、`qk` 三个命令生成 completion 脚本
   - 支持多种 shell（zsh, bash, fish, powershell, elvish）

3. **配置文件管理**：
   - 自动在 shell 配置文件中添加 completion 加载代码
   - 支持 zsh 和 bash

### 关键步骤说明

1. **生成 Completion 脚本**：
   - 使用 `clap_complete` 生成 completion 脚本
   - 根据 shell 类型生成对应的文件（如 zsh 的 `_workflow`、`_pr`、`_qk`）

2. **配置 Shell 配置文件**：
   - 在 `~/.zshrc` 或 `~/.bashrc` 中添加 completion 加载代码
   - 使用 `Completion::configure_shell_config()` 方法

---

## 2. 卸载命令 (`uninstall.rs`)

### 相关文件

```
src/commands/uninstall.rs
```

### 调用流程

```
main.rs::Commands::Uninstall
  ↓
commands/uninstall.rs::UninstallCommand::run()
  ↓
  1. 显示卸载信息（确认提示）
  2. 第一步确认：是否删除二进制文件和 completion 脚本
  3. 第二步确认：是否删除环境变量配置
  4. remove_binaries()                      # 删除二进制文件
     └─ Uninstall::remove_binaries()       # 调用工具函数
  5. remove_completion_files()              # 删除 completion 脚本
     └─ Completion::remove_completion_files()
  6. uninstall_all()                         # 删除配置（如果确认）
     └─ Uninstall::uninstall_all()          # 调用工具函数
  7. Shell::reload_config()                  # 重新加载 shell 配置
```

### 功能说明

1. **两步确认机制**：
   - 第一步：确认是否删除二进制文件和 completion 脚本
   - 第二步：确认是否删除环境变量配置（可选）

2. **二进制文件删除**：
   - 删除 `workflow`、`pr`、`qk`、`install` 二进制文件
   - 自动处理需要 sudo 权限的文件

3. **Completion 清理**：
   - 删除 completion 脚本文件
   - 从 shell 配置文件中移除 completion 加载代码

4. **配置清理**：
   - 从 shell 配置文件中移除所有 Workflow 相关的环境变量
   - 使用 `Uninstall::uninstall_all()` 方法

### 关键步骤说明

1. **二进制文件删除**：
   - 检查文件是否存在
   - 尝试直接删除，如果失败则使用 sudo
   - 提供清晰的错误提示

2. **配置清理**：
   - 从 shell 配置文件中移除 Workflow 配置块
   - 支持部分清理（只删除二进制，保留配置）

---

## 📊 数据流

### 安装数据流

```
Shell 检测
  ↓
Completion 生成
  ↓
文件系统操作（创建文件）
  ↓
Shell 配置更新
```

### 卸载数据流

```
用户确认
  ↓
二进制文件删除
  ↓
Completion 清理
  ↓
配置清理（可选）
  ↓
Shell 配置重新加载
```

---

## 🔗 与其他模块的集成

### 工具模块集成

- **`lib/utils/shell.rs`**：Shell 检测和配置管理
- **`lib/utils/completion.rs`**：Completion 脚本生成和配置
- **`lib/utils/env.rs`**：环境变量读写管理
- **`lib/utils/uninstall.rs`**：卸载工具函数

---

## 🎯 设计模式

### 1. 命令模式

每个命令都是一个独立的结构体，实现统一的方法接口。

### 2. 工具函数模式

将复杂的操作封装到 `lib/utils/` 中的工具函数，命令层只负责调用和交互。

---

## 🔍 错误处理

### 分层错误处理

1. **CLI 层**：参数验证错误
2. **命令层**：用户交互错误、业务逻辑错误
3. **工具层**：文件操作错误、配置读写错误

### 容错机制

- **文件操作失败**：提供清晰的错误提示和手动操作建议
- **权限不足**：自动使用 sudo 尝试删除
- **配置清理失败**：提供手动清理步骤

---

## 📝 扩展性

### 添加新的 Shell 支持

1. 在 `lib/utils/completion.rs` 中添加新的 shell 类型支持
2. 在 `install.rs` 的 `generate_completions()` 方法中添加对应的生成逻辑

### 添加新的二进制文件

1. 在 `lib/utils/uninstall.rs` 的 `get_binary_paths()` 方法中添加新的二进制路径

---

## 📚 相关文档

- [主架构文档](./ARCHITECTURE.md)
- [配置管理模块架构文档](./CONFIG_ARCHITECTURE.md)

