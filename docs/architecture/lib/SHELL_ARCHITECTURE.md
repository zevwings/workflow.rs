# Shell 检测与管理模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 Shell 检测与管理模块架构，包括：
- Shell 类型检测（zsh、bash、fish、powershell、elvish）
- Shell 配置重新加载
- Shell 配置文件管理（环境变量、source 语句、配置块）

该模块为 Completion 和 Proxy 模块提供通用的 Shell 配置文件管理功能，支持多 shell 类型的差异化处理。

**模块统计：**
- 总代码行数：约 950 行
- 文件数量：4 个核心文件
- 主要组件：3 个（Detect, Reload, ShellConfigManager）
- 支持的 Shell：zsh, bash, fish, powershell, elvish

---

## 📁 模块结构

### 核心模块文件

```
src/lib/base/shell/
├── mod.rs                  # 模块声明和导出
├── detect.rs               # Shell 检测工具（Detect）
├── reload.rs               # Shell 配置重载工具（Reload）
└── config.rs               # Shell 配置管理器（ShellConfigManager）
```

### 依赖模块

- **`lib/base/settings/paths.rs`**：路径管理（`Paths::config_file()`）
- **`clap_complete::Shell`**：Shell 类型枚举
- **`duct`**：子进程执行（用于配置重载）

---

## 🏗️ 架构设计

### 组件职责分离

模块采用职责分离的设计模式，每个组件负责单一职责：

#### 1. Detect（结构体）

**职责**：Shell 类型检测

**功能**：
- 检测当前 shell 类型（从 `SHELL` 环境变量）
- 检测系统中已安装的 shell（从 `/etc/shells` 文件）
- 支持多种 shell 类型（zsh, bash, fish, powershell, elvish）

**关键方法**：
- `shell()` - 检测当前 shell 类型
- `installed_shells()` - 检测已安装的 shell 列表

#### 2. Reload（结构体）

**职责**：Shell 配置重新加载

**功能**：
- 在子 shell 中执行 `source` 命令来重新加载配置文件
- 验证配置文件是否有效
- 提供手动重载提示

**关键方法**：
- `shell(shell)` - 重新加载指定 shell 的配置

**注意**：在子进程中执行，不会影响当前 shell，但可以验证配置文件是否有效。

#### 3. ShellConfigManager（结构体）

**职责**：通用的 Shell 配置文件管理

**功能**：
- 环境变量管理（export 语句的添加、移除、读取）
- Source 语句管理（添加、移除、检查）
- 配置块管理（解析、构建、合并）
- 多 shell 支持（不同 shell 的配置文件路径和语法差异）

**关键方法**：
- **环境变量管理**：
  - `load_env_vars()` - 从配置块加载环境变量
  - `save_env_vars(env_vars)` - 保存环境变量到配置块
  - `set_env_vars(env_vars)` - 批量设置环境变量
  - `remove_env_vars(keys)` - 从文件中移除指定的 export 语句
- **Source 语句管理**：
  - `add_source(source_path, comment)` - 添加 source 语句（自动检测 shell）
  - `add_source_for_shell(shell, source_path, comment)` - 添加 source 语句（指定 shell）
  - `remove_source(source_path)` - 移除 source 语句（自动检测 shell）
  - `remove_source_for_shell(shell, source_path)` - 移除 source 语句（指定 shell）
  - `has_source(source_path)` - 检查 source 语句是否存在（自动检测 shell）
  - `has_source_for_shell(shell, source_path)` - 检查 source 语句是否存在（指定 shell）
- **工具方法**：
  - `get_config_path()` - 获取 shell 配置文件路径（自动检测 shell）

---

## 🔄 调用流程

### 整体架构流程

```
用户输入或其他模块调用
  ↓
shell 模块（Detect / Reload / ShellConfigManager）
  ↓
Paths::config_file() (获取配置文件路径)
  ↓
Shell 配置文件（~/.zshrc, ~/.bash_profile, 等）
```

### Shell 检测流程

```
Detect::shell()
  ↓
1. Shell::from_env()                    # 尝试从环境变量检测
  ↓ (失败)
2. std::env::var("SHELL")               # 读取 SHELL 环境变量
  ↓
3. Shell::from_shell_path()             # 从路径解析 shell 类型
  ↓
返回 Shell 类型或错误
```

### 配置块管理流程

#### 加载环境变量

```
ShellConfigManager::load_env_vars()
  ↓
1. get_config_path()                    # 获取配置文件路径（自动检测 shell）
  ↓
2. read_config_file()                   # 读取配置文件内容
  ↓
3. parse_config_block()                 # 解析配置块
  ├─ 查找配置块标记（# Workflow CLI Configuration - Start/End）
  ├─ 提取配置块内容
  └─ parse_shell_config_block()         # 解析 export KEY="VALUE" 格式
  ↓
返回环境变量 HashMap
```

#### 保存环境变量

```
ShellConfigManager::save_env_vars(env_vars)
  ↓
1. get_config_path()                    # 获取配置文件路径
  ↓
2. load_existing_config()               # 加载现有配置
  ├─ read_config_file()                 # 读取文件内容
  ├─ parse_config_block()               # 解析配置块
  └─ 返回 ExistingConfig { env_in_block, content_without_block }
  ↓
3. merge_env_vars()                     # 合并环境变量（新值覆盖旧值）
  ↓
4. build_config_content()               # 构建新内容
  ├─ build_config_block()               # 构建配置块
  │  ├─ 添加标记行
  │  ├─ 按字母顺序排序键
  │  ├─ 转义特殊字符（\, ", $, `）
  │  └─ 生成 export KEY="VALUE" 格式
  └─ 合并到 content_without_block
  ↓
5. write_config_file()                  # 写入配置文件
```

#### 移除环境变量

```
ShellConfigManager::remove_env_vars(keys)
  ↓
1. get_config_path()                    # 获取配置文件路径
  ↓
2. read_config_file()                   # 读取配置文件内容
  ↓
3. 过滤 export 语句
  ├─ 遍历所有行
  ├─ 检查是否匹配要删除的键（export KEY=）
  └─ 过滤掉匹配的行
  ↓
4. write_config_file()                  # 写入新内容
```

### Source 语句管理流程

#### 添加 Source 语句

```
ShellConfigManager::add_source_for_shell(shell, source_path, comment)
  ↓
1. Paths::config_file(shell)            # 获取指定 shell 的配置文件路径
  ↓
2. read_config_file()                   # 读取配置文件内容
  ↓
3. has_source_in_content_for_shell()    # 检查是否已存在
  ├─ 检查 source 关键字（PowerShell 使用 `.`，其他使用 `source`）
  ├─ 检查相对路径和绝对路径
  └─ 返回 true/false
  ↓ (如果不存在)
4. 构建新内容
  ├─ 添加注释（如果提供）
  ├─ 添加 source 语句（使用正确的关键字）
  └─ 添加空行
  ↓
5. write_config_file()                  # 写入配置文件
```

#### 移除 Source 语句

```
ShellConfigManager::remove_source_for_shell(shell, source_path)
  ↓
1. Paths::config_file(shell)            # 获取指定 shell 的配置文件路径
  ↓
2. read_config_file()                   # 读取配置文件内容
  ↓
3. remove_source_from_content_for_shell()  # 移除 source 语句
  ├─ 遍历所有行
  ├─ 检查配置块标记（# Workflow CLI）
  ├─ 检查 source 语句（支持不同关键字和路径格式）
  ├─ 跳过匹配的行和相关注释
  └─ 清理末尾空行
  ↓
4. write_config_file()                  # 写入新内容
```

### 配置重载流程

```
Reload::shell(shell)
  ↓
1. Paths::config_file(shell)           # 获取配置文件路径
  ↓
2. 构建 source 命令
  ├─ PowerShell: `. $config_file`
  └─ 其他: `source $config_file`
  ↓
3. cmd(shell, ["-c", source_cmd])      # 在子 shell 中执行
  ↓
4. 验证执行结果
  ├─ 成功：提示用户手动运行 source 命令
  └─ 失败：显示错误并提示手动运行
```

---

## 📊 数据流

### 配置块结构

配置块使用标记行来标识，格式如下：

```bash
# Workflow CLI Configuration - Start
# Generated by Workflow CLI - DO NOT edit manually
# These environment variables will be loaded when you start a new shell

export KEY1="value1"
export KEY2="value2"
...

# Workflow CLI Configuration - End
```

### 环境变量格式

- **格式**：`export KEY="VALUE"`
- **转义规则**：
  - `\` → `\\`
  - `"` → `\"`
  - `$` → `\$`
  - `` ` `` → ``\` ``
- **排序**：按字母顺序排序键

### Source 语句格式

不同 shell 使用不同的关键字：

- **zsh, bash, fish, elvish**：`source $HOME/.workflow/.completions`
- **PowerShell**：`. $HOME/.workflow/.completions`

### 配置文件路径

不同 shell 的配置文件路径：

- **zsh** → `~/.zshrc`
- **bash** → `~/.bash_profile`（如果不存在则使用 `~/.bashrc`）
- **fish** → `~/.config/fish/config.fish`
- **powershell** → `~/.config/powershell/Microsoft.PowerShell_profile.ps1`
- **elvish** → `~/.elvish/rc.elv`

---

## 🔗 与其他模块的集成

### Completion 模块集成

**使用场景**：
- 添加 completion 脚本的 source 语句到 shell 配置文件
- 移除 completion 脚本的 source 语句
- 检查 completion 是否已配置

**关键调用**：
```rust
// 添加 source 语句（指定 shell 类型）
ShellConfigManager::add_source_for_shell(
    shell,
    "$HOME/.workflow/.completions",
    Some("Workflow CLI completions"),
)?;

// 移除 source 语句
ShellConfigManager::remove_source_for_shell(shell, source_pattern)?;

// 检查 source 语句是否存在
ShellConfigManager::has_source_for_shell(shell, source_pattern)?;
```

**位置**：`src/lib/completion/completion.rs`

### Proxy 模块集成

**使用场景**：
- 保存代理环境变量到 shell 配置文件（持久化模式）
- 从 shell 配置文件加载代理环境变量
- 移除代理环境变量

**关键调用**：
```rust
// 保存环境变量到配置块
ShellConfigManager::set_env_vars(&env_vars)?;

// 从配置块加载环境变量
let env_vars = ShellConfigManager::load_env_vars()?;

// 移除环境变量
ShellConfigManager::remove_env_vars(&["http_proxy", "https_proxy", "all_proxy"])?;
```

**位置**：`src/lib/proxy/manager.rs`

### Rollback 模块集成

**使用场景**：
- 在回滚后重新加载 shell 配置

**关键调用**：
```rust
// 重新加载 shell 配置
Reload::shell(&shell)?;
```

**位置**：`src/lib/rollback/rollback.rs`

### 其他模块使用

- **Install/Update 命令**：使用 `Detect::shell()` 检测 shell 类型
- **Config Completion 命令**：使用 `Detect::shell()` 和 `ShellConfigManager` 管理 completion 配置

---

## 🎯 设计模式

### 1. 职责分离

每个组件负责单一职责：
- **Detect**：只负责 Shell 检测
- **Reload**：只负责配置重载
- **ShellConfigManager**：只负责配置文件管理

### 2. 多 Shell 支持策略

通过 `clap_complete::Shell` 枚举统一处理不同 shell：
- 使用 `Paths::config_file(shell)` 获取不同 shell 的配置文件路径
- 使用 `get_source_keyword(shell)` 获取不同 shell 的 source 关键字
- 配置文件路径和语法差异由 `Paths` 和 `ShellConfigManager` 统一处理

### 3. 配置块管理策略

使用标记行来标识配置块：
- **优点**：易于识别和管理，不会与用户自定义配置混淆
- **格式**：`# Workflow CLI Configuration - Start/End`
- **位置**：配置块始终放在文件末尾

### 4. 环境变量合并策略

- **合并规则**：新值覆盖旧值（`HashMap::extend()`）
- **排序规则**：按字母顺序排序键，便于阅读和维护
- **转义规则**：自动转义特殊字符，确保值正确解析

### 5. Source 语句去重策略

- **检查机制**：添加前检查是否已存在（支持相对路径和绝对路径）
- **格式兼容**：支持不同格式的 source 语句（`source`, `.`, 多个空格等）
- **注释处理**：自动处理相关注释块

---

## 🔍 错误处理

### Shell 检测失败

**场景**：
- `SHELL` 环境变量未设置
- Shell 类型不支持

**处理**：
```rust
Detect::shell() -> Result<Shell>
// 返回错误：Unsupported shell: {shell}
```

### 配置文件读写失败

**场景**：
- 配置文件不存在（首次使用）
- 权限不足
- 磁盘空间不足

**处理**：
- **读取**：配置文件不存在时返回空内容（`Ok(String::new())`）
- **写入**：使用 `anyhow::Context` 提供详细的错误信息

### 配置块解析失败

**场景**：
- 配置块格式错误
- 标记行不匹配

**处理**：
- **解析失败**：返回空 HashMap（`unwrap_or_default()`）
- **标记不匹配**：忽略配置块，保留原有内容

### 配置重载失败

**场景**：
- 配置文件语法错误
- Shell 命令执行失败

**处理**：
- 显示错误信息
- 提示用户手动运行 `source` 命令

---

## 📝 扩展性

### 添加新的 Shell 支持

1. **确认 `clap_complete::Shell` 支持**：
   - 检查 `clap_complete` 是否已支持该 shell
   - 如果不支持，需要扩展 `clap_complete::Shell` 枚举

2. **添加配置文件路径**：
   - 在 `lib/base/settings/paths.rs` 的 `Paths::config_file()` 方法中添加新 shell 的路径映射

3. **添加 Source 关键字**：
   - 在 `ShellConfigManager::get_source_keyword()` 方法中添加新 shell 的关键字映射

4. **测试**：
   - 测试 Shell 检测
   - 测试配置文件读写
   - 测试 Source 语句管理
   - 测试配置重载

### 添加新的配置管理功能

1. **在 `ShellConfigManager` 中添加新方法**：
   - 遵循现有的命名规范
   - 提供自动检测 shell 和指定 shell 两个版本（如 `xxx()` 和 `xxx_for_shell()`）

2. **更新配置块格式**（如需要）：
   - 确保新功能与现有配置块格式兼容
   - 更新 `parse_config_block()` 和 `build_config_block()` 方法

3. **测试**：
   - 测试新功能在不同 shell 下的行为
   - 测试与现有功能的兼容性

### 优化配置块解析性能

**当前实现**：
- 使用字符串查找和切片操作
- 逐行解析环境变量

**可能的优化**：
- 使用正则表达式解析（如果性能成为瓶颈）
- 缓存解析结果（如果配置文件不经常变化）

---

## 🔧 实现细节

### 配置块解析算法

1. **查找标记行**：
   ```rust
   let start_pos = content.find("# Workflow CLI Configuration - Start");
   let end_pos = content[start_pos..].find("# Workflow CLI Configuration - End");
   ```

2. **提取配置块内容**：
   ```rust
   let block_content = &content[start_pos + start_marker.len()..start_pos + end_pos];
   ```

3. **解析环境变量**：
   - 逐行解析 `export KEY="VALUE"` 格式
   - 处理引号和转义字符
   - 反转义特殊字符

### Source 语句检测算法

1. **检查相对路径**：
   ```rust
   if content.contains(source_path) {
       return Ok(true);
   }
   ```

2. **检查绝对路径**（如果 source_path 包含 `$HOME`）：
   ```rust
   let home = std::env::var("HOME")?;
   let abs_path = source_path.replace("$HOME", &home);
   if content.contains(&abs_path) {
       return Ok(true);
   }
   ```

3. **检查不同格式**：
   - 支持 `source` 和 `.` 关键字
   - 支持多个空格
   - 支持相对路径和绝对路径

### 环境变量转义规则

**转义字符映射**：
- `\` → `\\`（反斜杠）
- `"` → `\"`（双引号）
- `$` → `\$`（美元符号，防止变量展开）
- `` ` `` → ``\` ``（反引号，防止命令替换）

**反转义**：解析时执行相反的操作。

---

## 📚 相关文档

- [Settings 模块架构文档](./SETTINGS_ARCHITECTURE.md) - 配置文件路径管理（`Paths::config_file()`）
- [Completion 架构文档](./COMPLETION_ARCHITECTURE.md) - Completion 模块如何使用 ShellConfigManager
- [Proxy 架构文档](./PROXY_ARCHITECTURE.md) - Proxy 模块如何使用 ShellConfigManager
- [主架构文档](../ARCHITECTURE.md)

---

## 🔄 更新记录

- **2024-12** - 创建 Shell 检测与管理模块架构文档

