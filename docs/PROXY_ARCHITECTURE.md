# 代理管理模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的代理管理模块架构，包括代理的开启、关闭和检查功能。该模块负责从系统设置读取代理配置，并管理代理相关的环境变量。

---

## 📁 相关文件

### CLI 入口层

```
src/main.rs
```

### 命令封装层

```
src/commands/proxy.rs    # 代理管理命令（210 行）
```

### 依赖模块

- **`lib/utils/proxy.rs`**：代理信息管理
- **`lib/utils/env.rs`**：环境变量读写管理
- **`lib/utils/clipboard.rs`**：剪贴板操作

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
main.rs (CLI 入口，参数解析)
  ↓
commands/proxy.rs (命令封装层)
  ↓
lib/utils/proxy.rs (核心业务逻辑层)
```

---

## 代理命令 (`proxy.rs`)

### 相关文件

```
src/commands/proxy.rs
```

### 调用流程

**开启代理** (`on`):
```
main.rs::Commands::Proxy::On
  ↓
commands/proxy.rs::ProxyCommand::on()
  ↓
  1. Proxy::get_system_proxy()                # 获取系统代理设置
  2. Proxy::is_proxy_configured()             # 检查代理是否已配置
  3. Proxy::generate_env_vars()               # 生成环境变量
  4. EnvFile::set_multiple()                  # 保存到 shell 配置文件
  5. Clipboard::copy()                         # 复制代理命令到剪贴板
```

**关闭代理** (`off`):
```
main.rs::Commands::Proxy::Off
  ↓
commands/proxy.rs::ProxyCommand::off()
  ↓
  1. Proxy::check_env_proxy()                 # 检查当前代理环境变量
  2. EnvFile::load()                          # 加载配置文件
  3. EnvFile::save()                          # 从配置块中移除代理配置
  4. EnvFile::remove_from_file()              # 从整个文件中移除代理配置
  5. Clipboard::copy()                        # 复制 unset 命令到剪贴板
```

**检查代理** (`check`):
```
main.rs::Commands::Proxy::Check
  ↓
commands/proxy.rs::ProxyCommand::check()
  ↓
  1. Proxy::get_system_proxy()                # 获取系统代理设置
  2. Proxy::check_env_proxy()                 # 检查环境变量
  3. EnvFile::load()                          # 加载配置文件
  4. 显示系统代理设置
  5. 显示环境变量设置
  6. Proxy::is_proxy_configured()            # 检查代理是否已正确配置
```

### 功能说明

1. **系统代理读取**：
   - 从 macOS 系统设置读取代理配置
   - 支持 HTTP、HTTPS、SOCKS 代理
   - 使用 `Proxy::get_system_proxy()` 方法

2. **环境变量管理**：
   - 开启代理：设置 `http_proxy`、`https_proxy`、`all_proxy` 环境变量
   - 关闭代理：移除所有代理相关的环境变量
   - 保存到 shell 配置文件，持久化配置

3. **代理检查**：
   - 显示系统代理设置
   - 显示当前环境变量设置
   - 检查代理是否已正确配置

### 关键步骤说明

1. **开启代理**：
   - 从系统设置读取代理配置
   - 生成环境变量并保存到配置文件
   - 复制代理命令到剪贴板（用于当前 shell 会话）

2. **关闭代理**：
   - 从配置块中移除代理配置
   - 从整个文件中移除所有代理相关的 export 语句
   - 生成 unset 命令并复制到剪贴板

3. **代理检查**：
   - 显示系统代理设置（HTTP、HTTPS、SOCKS）
   - 显示环境变量设置（区分当前会话和配置文件）
   - 检查代理配置是否正确

---

## 📊 数据流

### 代理管理数据流

```
系统代理设置（macOS）
  ↓
命令层处理（读取、生成环境变量）
  ↓
EnvFile 管理（保存到配置文件）
  ↓
剪贴板操作（复制命令）
```

---

## 🔗 与其他模块的集成

### 工具模块集成

- **`lib/utils/proxy.rs`**：代理信息管理
- **`lib/utils/env.rs`**：环境变量读写管理
- **`lib/utils/clipboard.rs`**：剪贴板操作

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

- **系统代理读取失败**：提示用户检查系统设置
- **文件操作失败**：提供清晰的错误提示和手动操作建议

---

## 📝 扩展性

### 添加新代理类型

1. 在 `lib/utils/proxy.rs` 中添加新的代理类型支持
2. 在 `proxy.rs` 中添加相应的处理逻辑

---

## 📚 相关文档

- [主架构文档](./ARCHITECTURE.md)
- [配置管理模块架构文档](./CONFIG_ARCHITECTURE.md)

