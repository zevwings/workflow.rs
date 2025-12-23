# Proxy 模块架构文档

## 📋 概述

Proxy 模块是 Workflow CLI 的核心模块，提供代理管理功能。该模块采用分层架构设计，包括：

- **Lib 层**（`lib/proxy/`）：提供代理管理的核心业务逻辑，包括从 macOS 系统设置读取代理配置、生成代理命令和环境变量、管理代理的开启和关闭等功能
- **Commands 层**（`commands/proxy/`）：提供 CLI 命令封装，处理用户交互，包括代理启用、禁用和状态检查功能

Proxy 模块负责从 macOS 系统设置读取代理配置，并管理代理相关的环境变量，支持临时模式（仅当前 shell）和持久模式（写入配置文件）。

**模块统计：**
- Lib 层代码行数：约 600 行
- Commands 层代码行数：约 190 行
- 命令数量：3 个（on, off, check）
- 文件数量：Lib 层 5 个，Commands 层 2 个
- 主要组件：`SystemProxyReader`、`ProxyConfigGenerator`、`ProxyManager`、`ProxyInfo`、`ProxyType`

---

## 📁 Lib 层架构（核心业务逻辑）

代理管理模块（`lib/proxy/`）是 Workflow CLI 的核心库模块，提供代理管理的核心业务逻辑，包括从 macOS 系统设置读取代理配置、生成代理命令和环境变量、管理代理的开启和关闭等功能。

### 模块结构

```
src/lib/proxy/
├── mod.rs                  # 模块声明和导出
├── proxy.rs                # 类型定义（ProxyType, ProxyInfo, ProxyConfig, 结果类型）
├── system-_reader.rs        # 系统代理读取器（从 macOS 系统设置读取）
├── config-_generator.rs     # 代理配置生成器（生成命令和环境变量）
└── manager.rs              # 代理管理器（协调其他组件，提供高级功能）
```

### 依赖模块

- **`lib/base/shell/config.rs`**：Shell 配置文件管理（`ShellConfigManager`）
- **`lib/base/util/clipboard.rs`**：剪贴板操作（`Clipboard`）

---

## 🏗️ Lib 层架构设计

### 设计原则

1. **单一职责原则**：每个组件只负责一个明确的功能
2. **迭代器模式**：使用 `ProxyType::all()` 迭代器统一处理所有代理类型
3. **策略模式**：通过 `temporary` 参数控制代理启用策略
4. **职责分离**：系统读取、配置生成、管理协调分离

### 核心组件

#### 1. ProxyType（枚举）

**职责**：定义代理类型（HTTP、HTTPS、SOCKS）

**位置**：`src/lib/proxy/proxy.rs`

**关键方法**：
- `all()` - 返回所有代理类型的迭代器
- `env-_key()` - 返回对应的环境变量键名
- `url-_scheme()` - 返回对应的 URL 协议方案

**设计优势**：
- 消除硬编码的环境变量名
- 统一处理所有代理类型
- 易于扩展新的代理类型

#### 2. ProxyInfo（结构体）

**职责**：存储代理配置信息

**位置**：`src/lib/proxy/proxy.rs`

**设计**：使用 `HashMap<ProxyType, ProxyConfig>` 存储，消除字段重复

**关键方法**：
- `new()` - 创建新实例
- `get-_config(proxy-_type)` - 获取指定代理类型的配置
- `get-_config-_mut(proxy-_type)` - 获取可变引用
- `get-_proxy-_url(proxy-_type)` - 获取代理 URL

**设计优势**：
- 消除字段重复（从 9 个字段减少到 1 个 HashMap）
- 易于扩展新的代理类型
- 使用 `ProxyType` 作为键，类型安全

#### 3. SystemProxyReader

**职责**：从 macOS 系统设置读取代理配置

**位置**：`src/lib/proxy/system-_reader.rs`

**关键方法**：
- `read()` - 从系统设置读取代理配置

**关键特性**：
- 使用 `scutil --proxy` 命令读取系统代理设置
- 使用映射表简化解析逻辑
- 消除重复的 match 分支

#### 4. ProxyConfigGenerator

**职责**：生成代理命令和环境变量

**位置**：`src/lib/proxy/config-_generator.rs`

**关键方法**：
- `generate-_command(proxy-_info)` - 生成 `export` 命令字符串
- `generate-_env-_vars(proxy-_info)` - 生成环境变量 HashMap

**关键特性**：
- 提取公共逻辑（`generate-_proxy-_pairs()`）
- 减少代码重复
- 统一处理所有代理类型

#### 5. ProxyManager

**职责**：协调其他组件，提供高级代理管理功能

**位置**：`src/lib/proxy/manager.rs`

**关键方法**：
- `check-_env-_proxy()` - 检查环境变量中的代理设置
- `is-_proxy-_configured(proxy-_info)` - 检查代理设置是否匹配
- `enable(temporary)` - 开启代理（支持临时模式和持久化模式）
- `disable()` - 关闭代理（同时从配置文件和当前 shell 移除）
- `ensure-_proxy-_enabled()` - 确保代理已启用（如果系统代理已启用，自动设置环境变量）**注意：此函数已不再自动调用，需手动使用**

**关键特性**：
- 协调 `SystemProxyReader` 和 `ProxyConfigGenerator`
- 支持临时模式和持久化模式
- 使用 `ProxyType` 迭代器统一处理

#### 6. ShellConfigManager

**职责**：通用的 Shell 配置文件管理

**位置**：`src/lib/base/shell/config.rs`

**关键方法**：
- `load-_env-_vars()` - 从配置块加载环境变量
- `save-_env-_vars(env-_vars)` - 保存环境变量到配置块
- `set-_env-_vars(env-_vars)` - 批量设置环境变量
- `remove-_env-_vars(keys)` - 从文件中移除指定的 export 语句
- `add-_source(source-_path, comment)` - 添加 source 语句
- `remove-_source(source-_path)` - 移除 source 语句

**关键特性**：
- 通用的 Shell 配置文件管理工具
- 供 Proxy 和 Completion 模块共用
- 支持环境变量和 source 语句管理

### 设计模式

#### 1. 迭代器模式

使用 `ProxyType::all()` 迭代器统一处理所有代理类型：

```rust
for proxy-_type in ProxyType::all() {
    // 统一处理所有代理类型
}
```

**优势**：
- 消除代码重复
- 易于扩展新的代理类型
- 统一处理逻辑

#### 2. 策略模式

通过 `temporary` 参数控制代理启用策略：
- **临时模式**（`temporary = true`）：不写入配置文件，只在当前 shell 生效
- **持久模式**（`temporary = false`）：写入 shell 配置文件，新开 shell 自动启用

**优势**：
- 灵活的策略选择
- 代码复用
- 易于扩展新策略

---

## 📁 Commands 层架构（命令封装）

代理管理命令层是 Workflow CLI 的命令接口，提供代理启用、禁用和状态检查功能。该层采用命令模式设计，通过调用 `lib/proxy/` 模块提供的 API 实现业务功能。

### 相关文件

#### CLI 入口层

代理管理命令现在作为 `workflow` 主命令的子命令，通过 `src/main.rs` 中的 `Commands::Proxy` 枚举定义。

```
src/main.rs
```
- **职责**：`workflow` 主命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将 `workflow proxy` 子命令分发到对应的命令处理函数

#### 命令封装层

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
  - `ProxyManager::check-_env-_proxy()` - 检查环境变量中的代理设置
  - `ProxyManager::is-_proxy-_configured()` - 检查代理是否已正确配置
  - `SystemProxyReader::read()` - 读取系统代理设置
- **`lib/base/shell/`**：Shell 配置管理（`ShellConfigManager`）
  - `ShellConfigManager::load-_env-_vars()` - 加载 shell 配置文件中的环境变量
- **`lib/base/util/`**：工具函数（`Clipboard`）
  - `Clipboard::copy()` - 复制到剪贴板

---

## 🔄 集成关系

### Lib 层和 Commands 层的协作

Proxy 模块采用清晰的分层架构，Lib 层和 Commands 层通过以下方式协作：

1. **代理管理**：Commands 层调用 `ProxyManager` 的方法进行代理管理
2. **系统读取**：Commands 层使用 `SystemProxyReader` 读取系统代理设置
3. **配置生成**：Commands 层使用 `ProxyConfigGenerator` 生成代理命令
4. **用户交互**：Commands 层负责格式化输出和用户提示

### 调用流程

#### 整体架构流程

```
调用者（命令层或其他模块）
  ↓
ProxyManager (协调层)
  ↓
SystemProxyReader / ProxyConfigGenerator / ShellConfigManager (功能层)
```

#### 命令分发流程

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

## 📋 Commands 层命令详情

### 1. 启用代理命令 (`on`)

启用代理功能，支持两种模式：

1. **临时模式** (`--temporary` / `-t`)：
   - 只在当前 shell 启用代理
   - 不写入 shell 配置文件
   - 关闭 shell 后失效

2. **持久模式**（默认）：
   - 写入 shell 配置文件（`~/.zshrc` 或 `~/.bash-_profile`）
   - 新打开的 shell 自动启用代理
   - 持久化配置

**关键步骤**：
1. 使用 `SystemProxyReader::read()` 读取系统代理设置
2. 使用 `ProxyManager::enable(temporary)` 启用代理
3. 显示生成的代理命令
4. 复制命令到剪贴板
5. 提供使用说明（如何在当前 shell 启用）

### 2. 禁用代理命令 (`off`)

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

### 3. 检查代理状态命令 (`check`)

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

---

## 🔄 调用流程与数据流

### 开启代理流程

```
ProxyManager::enable(temporary)
  ↓
  1. SystemProxyReader::read()                    # 获取系统代理设置
  2. ProxyManager::is-_proxy-_configured()          # 检查代理是否已配置
  3. ProxyConfigGenerator::generate-_command()     # 生成代理命令
  4. ProxyConfigGenerator::generate-_env-_vars()    # 生成环境变量
  5. ShellConfigManager::set-_env-_vars()          # 保存到配置文件（如果非临时模式）
```

**模式说明**：
- **默认模式**（`temporary = false`）：写入 shell 配置文件，新开 shell 自动启用
- **临时模式**（`temporary = true`）：不写入配置文件，只在当前 shell 生效

### 关闭代理流程

```
ProxyManager::disable()
  ↓
  1. ProxyManager::collect-_current-_proxy()        # 收集当前代理设置（环境变量和配置文件）
  2. ProxyManager::remove-_from-_config-_file()      # 从配置文件移除
  3. ProxyManager::generate-_unset-_command()       # 生成 unset 命令
```

**行为说明**：
- 同时从 shell 配置文件和当前 shell 环境变量中移除代理设置
- 生成 `unset` 命令用于当前 shell 会话

### 检查代理流程

```
ProxyManager::check-_env-_proxy()
  ↓
  1. SystemProxyReader::read()                       # 获取系统代理设置
  2. ProxyManager::check-_env-_proxy()                 # 检查环境变量
  3. ShellConfigManager::load-_env-_vars()              # 加载配置文件
  4. ProxyManager::is-_proxy-_configured()             # 检查代理是否已正确配置
```

### 数据流

#### 代理管理数据流

```
macOS 系统代理设置
  ↓
SystemProxyReader::read()
  ↓
ProxyInfo (HashMap<ProxyType, ProxyConfig>)
  ↓
ProxyConfigGenerator::generate-_env-_vars()
  ↓
ShellConfigManager::set-_env-_vars() (持久化模式)
  ↓
Shell 配置文件 (~/.zshrc, ~/.bash-_profile)
```

#### 当前 Shell 会话数据流

```
ProxyInfo
  ↓
ProxyConfigGenerator::generate-_command()
  ↓
export http-_proxy=... https-_proxy=... all-_proxy=...
  ↓
用户执行 eval $(workflow proxy on)
  ↓
当前 Shell 环境变量
```

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

### 基本使用（Lib 层）

```rust
use workflow::ProxyManager;

// 开启代理（持久化模式）
let result = ProxyManager::enable(false)?;
if let Some(cmd) = result.proxy-_command {
    log-_message!("Run: eval $({})", cmd);
}

// 开启代理（临时模式）
let result = ProxyManager::enable(true)?;
if let Some(cmd) = result.proxy-_command {
    log-_message!("Run: {}", cmd);
}

// 关闭代理
let result = ProxyManager::disable()?;
if let Some(cmd) = result.unset-_command {
    log-_message!("Run: {}", cmd);
}

// 检查代理
let env-_proxy = ProxyManager::check-_env-_proxy();
let is-_configured = ProxyManager::is-_proxy-_configured(&proxy-_info);

// 手动启用代理（如果系统代理已启用）
// 注意：此函数已不再自动调用，需手动使用
ProxyManager::ensure-_proxy-_enabled()?;
```

---

## 📝 扩展性

### 添加新代理类型

1. 在 `ProxyType` 枚举中添加新类型
2. 实现 `env-_key()` 和 `url-_scheme()` 方法
3. 更新 `SystemProxyReader` 的映射表（如果需要）
4. 所有使用 `ProxyType::all()` 迭代器的代码会自动支持新类型

### 添加新功能

- 所有功能都通过 `ProxyManager` 提供统一的接口
- 新功能可以添加到 `ProxyManager` 或创建新的组件
- 保持单一职责原则，避免组件职责过重

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
  - 注意：Linux ARM64 和 musl 静态链接版本不支持剪贴板功能

---

## 📚 相关文档

- [主架构文档](./architecture.md)
- [Shell 检测与管理模块架构文档](./shell.md) - Shell 配置管理相关

---

## ✅ 总结

Proxy 模块采用清晰的分层架构设计：

1. **类型定义层**：`ProxyType`、`ProxyInfo`、`ProxyConfig` 提供类型安全的数据结构
2. **功能层**：`SystemProxyReader`、`ProxyConfigGenerator` 提供单一职责的功能组件
3. **协调层**：`ProxyManager` 协调其他组件，提供高级代理管理功能
4. **工具层**：`ShellConfigManager` 提供通用的 Shell 配置文件管理
5. **命令封装层**：提供 CLI 命令接口，处理用户交互和格式化输出

**设计优势**：
- ✅ **职责分离**：每个组件只负责单一功能，易于测试和维护
- ✅ **代码复用**：使用 `ProxyType` 迭代器统一处理，消除代码重复
- ✅ **易于扩展**：添加新代理类型只需扩展枚举
- ✅ **类型安全**：使用枚举和 HashMap 替代字符串硬编码
- ✅ **灵活配置**：支持临时模式和持久化模式
- ✅ **用户友好**：自动复制命令，提供清晰的使用说明

**重构成果**：
- ✅ 消除所有硬编码的环境变量名
- ✅ 消除字段重复（9 个字段 → 1 个 HashMap）
- ✅ 消除代码重复（提取公共逻辑）
- ✅ 提高可维护性和可扩展性

通过职责分离、迭代器模式和策略模式，实现了代码复用、易于维护和扩展的目标。命令层专注于用户交互和输出格式化，核心业务逻辑由 Lib 层提供，实现了清晰的职责分离。

---

**最后更新**: 2025-12-16

