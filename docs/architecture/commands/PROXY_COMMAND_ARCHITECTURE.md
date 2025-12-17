# 代理管理命令模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的代理管理命令模块架构，包括：
- 代理启用功能
- 代理禁用功能
- 代理状态检查功能

代理管理命令提供完整的代理配置管理功能，支持从系统设置读取代理配置，并通过环境变量启用或禁用代理。支持临时模式（仅当前 shell）和持久模式（写入配置文件）。

**定位**：命令层专注于用户交互、参数解析和输出格式化，核心业务逻辑由 `lib/proxy/` 模块提供。

**注意**：虽然 `lib/proxy/` 有 `PROXY_ARCHITECTURE.md` 文档，但本文档专注于命令层的实现和使用。

---

## 📁 相关文件

### CLI 入口层

代理管理命令现在作为 `workflow` 主命令的子命令，通过 `src/main.rs` 中的 `Commands::Proxy` 枚举定义。

```
src/main.rs
```
- **职责**：`workflow` 主命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将 `workflow proxy` 子命令分发到对应的命令处理函数

### 命令封装层

```
src/commands/proxy/
├── mod.rs          # Proxy 命令模块声明（3 行）
└── proxy.rs        # 代理管理命令（~190 行）
```

**职责**：
- 解析命令参数
- 处理用户交互
- 格式化输出
- 调用核心业务逻辑层 (`lib/proxy/`) 的功能

### 依赖模块

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：
- **`lib/proxy/`**：代理管理模块（`ProxyManager`、`SystemProxyReader`、`ProxyConfigGenerator`）
  - `ProxyManager::enable()` - 启用代理
  - `ProxyManager::disable()` - 禁用代理
  - `ProxyManager::check_env_proxy()` - 检查环境变量中的代理设置
  - `ProxyManager::is_proxy_configured()` - 检查代理是否已正确配置
  - `SystemProxyReader::read()` - 读取系统代理设置
- **`lib/base/shell/`**：Shell 配置管理（`ShellConfigManager`）
  - `ShellConfigManager::load_env_vars()` - 加载 shell 配置文件中的环境变量
- **`lib/base/util/`**：工具函数（`Clipboard`）
  - `Clipboard::copy()` - 复制到剪贴板

详细架构文档：参见 [代理管理模块架构文档](../lib/PROXY_ARCHITECTURE.md)

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
src/main.rs (workflow 主命令，参数解析)
  ↓
commands/proxy/proxy.rs (命令封装层)
  ↓
lib/proxy/* (通过 Proxy API 调用，具体实现见相关模块文档)
  ↓
系统代理设置 / Shell 配置文件 / 环境变量
```

### 命令分发流程

```
src/main.rs::main()
  ↓
Cli::parse() (解析命令行参数)
  ↓
match cli.subcommand {
  ProxySubcommand::On { temporary } => ProxyCommand::on(temporary)
  ProxySubcommand::Off => ProxyCommand::off()
  ProxySubcommand::Check => ProxyCommand::check()
}
```

---

## 1. 启用代理命令 (`on`)

### 调用流程

```
src/main.rs::ProxySubcommand::On { temporary }
  ↓
commands/proxy/proxy.rs::ProxyCommand::on(temporary)
  ↓
  1. SystemProxyReader::read() (读取系统代理设置)
  2. ProxyManager::enable(temporary) (启用代理)
  3. 显示代理命令
  4. Clipboard::copy() (复制代理命令到剪贴板)
  5. 显示使用说明
```

### 功能说明

启用代理功能，支持两种模式：

1. **临时模式** (`--temporary` / `-t`)：
   - 只在当前 shell 启用代理
   - 不写入 shell 配置文件
   - 关闭 shell 后失效

2. **持久模式**（默认）：
   - 写入 shell 配置文件（`~/.zshrc` 或 `~/.bash_profile`）
   - 新打开的 shell 自动启用代理
   - 持久化配置

### 关键步骤说明

1. **系统代理读取**：
   - 使用 `SystemProxyReader::read()` 读取系统代理设置
   - 支持 HTTP、HTTPS、SOCKS 代理

2. **代理启用**：
   - 使用 `ProxyManager::enable(temporary)` 启用代理
   - 生成代理命令（`export HTTP_PROXY=...` 等）
   - 根据模式决定是否写入配置文件

3. **用户提示**：
   - 显示生成的代理命令
   - 复制命令到剪贴板
   - 提供使用说明（如何在当前 shell 启用）

---

## 2. 禁用代理命令 (`off`)

### 调用流程

```
src/main.rs::ProxySubcommand::Off
  ↓
commands/proxy/proxy.rs::ProxyCommand::off()
  ↓
  1. ProxyManager::disable() (禁用代理)
  2. 显示从配置文件移除的结果
  3. 显示当前环境变量中的代理设置
  4. 生成并显示 unset 命令
  5. Clipboard::copy() (复制 unset 命令到剪贴板)
```

### 功能说明

禁用代理功能，同时从 shell 配置文件和当前 shell 环境变量中移除代理设置：

1. **配置文件清理**：
   - 从 shell 配置文件中移除代理设置
   - 新打开的 shell 不会启用代理

2. **环境变量清理**：
   - 生成 `unset` 命令
   - 用户需要在当前 shell 中运行以禁用代理

3. **用户提示**：
   - 显示从配置文件移除的结果
   - 显示当前环境变量中的代理设置
   - 复制 unset 命令到剪贴板

---

## 3. 检查代理状态命令 (`check`)

### 调用流程

```
src/main.rs::ProxySubcommand::Check
  ↓
commands/proxy/proxy.rs::ProxyCommand::check()
  ↓
  1. SystemProxyReader::read() (读取系统代理设置)
  2. ProxyManager::check_env_proxy() (检查环境变量)
  3. ShellConfigManager::load_env_vars() (加载配置文件中的环境变量)
  4. 显示系统代理设置
  5. 显示环境变量设置（当前 session 和配置文件）
  6. ProxyManager::is_proxy_configured() (检查代理是否已正确配置)
```

### 功能说明

检查代理状态和配置：

1. **系统代理设置**：
   - 显示系统代理设置（HTTP、HTTPS、SOCKS）
   - 显示代理地址和端口

2. **环境变量设置**：
   - 显示当前 shell 环境变量中的代理设置
   - 显示 shell 配置文件中的代理设置
   - 区分当前 session 和配置文件中的设置

3. **配置状态**：
   - 检查代理是否已正确配置
   - 提供配置建议（如果未配置）

### 关键步骤说明

1. **多源检查**：
   - 系统代理设置（macOS System Preferences）
   - 当前 shell 环境变量
   - Shell 配置文件中的环境变量

2. **状态显示**：
   - 当前 shell：启用/禁用
   - Shell 配置文件：启用/禁用（将在新 shell 中生效）

3. **配置验证**：
   - 使用 `ProxyManager::is_proxy_configured()` 验证配置
   - 提供配置建议（如果未配置）

---

## 🏗️ 架构设计

### 设计模式

#### 1. 命令模式

每个命令都是一个独立的结构体，实现统一的方法接口：
- `ProxyCommand::on()` - 启用代理
- `ProxyCommand::off()` - 禁用代理
- `ProxyCommand::check()` - 检查代理状态

#### 2. 工具函数模式

将代理管理逻辑封装到 `lib/proxy/` 中的工具函数，命令层只负责调用和展示：
- `ProxyManager` - 代理管理
- `SystemProxyReader` - 系统代理读取
- `ProxyConfigGenerator` - 代理配置生成

#### 3. 模式选择

支持两种模式：
- **临时模式**：仅当前 shell，不持久化
- **持久模式**：写入配置文件，持久化

---

## 🔍 错误处理

### 分层错误处理

1. **CLI 层**：参数验证错误
   - `clap` 自动处理参数验证和错误提示

2. **命令层**：用户交互错误、业务逻辑错误
   - 系统代理未配置：显示警告，提供配置建议

3. **库层**：系统操作错误、文件操作错误
   - 通过 `ProxyManager`、`SystemProxyReader` API 返回的错误信息
   - 系统代理读取失败、配置文件写入失败等

### 容错机制

- **系统代理未配置**：显示警告，但不中断执行
- **配置文件写入失败**：返回错误，提供手动配置建议
- **剪贴板复制失败**：记录警告，但不影响主要功能
  - 注意：Linux ARM64 和 musl 静态链接版本不支持剪贴板功能（详见 [工具函数模块架构文档](../lib/TOOLS_ARCHITECTURE.md)）

---

## 📝 扩展性

### 添加新的代理类型

1. 在 `lib/proxy/` 中添加新的代理类型支持（参考相关模块文档）
2. `ProxyManager` 会自动支持新的代理类型

### 添加新的代理操作

1. 在 `proxy.rs` 中添加新的命令方法
2. 在 `src/main.rs` 中添加新的子命令枚举
3. 在命令分发逻辑中添加处理代码

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [代理管理模块架构文档](../lib/PROXY_ARCHITECTURE.md) - 代理管理核心逻辑
- [Shell 检测与管理模块架构文档](../lib/SHELL_ARCHITECTURE.md) - Shell 配置管理相关

---

## 📋 使用示例

### On 命令

```bash
# 启用代理（持久模式，写入配置文件）
workflow proxy on

# 启用代理（临时模式，仅当前 shell）
workflow proxy on --temporary
# 或
workflow proxy on -t

# 在当前 shell 中启用（使用 eval）
eval $(workflow proxy on)
```

### Off 命令

```bash
# 禁用代理
workflow proxy off

# 在当前 shell 中禁用（使用 eval）
eval $(workflow proxy off)
```

### Check 命令

```bash
# 检查代理状态
workflow proxy check
```

---

## ✅ 总结

代理管理命令层采用清晰的代理管理设计：

1. **双模式支持**：临时模式和持久模式，满足不同场景需求
2. **系统集成**：自动读取系统代理设置，无需手动配置
3. **用户友好**：自动复制命令到剪贴板，提供清晰的使用说明

**设计优势**：
- ✅ **灵活性**：支持临时和持久两种模式
- ✅ **自动化**：自动读取系统设置，减少手动配置
- ✅ **易用性**：自动复制命令，提供清晰的使用说明
- ✅ **可扩展性**：易于添加新的代理类型和操作

---

**最后更新**: 2025-12-16
