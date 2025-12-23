# 配置管理命令模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的配置管理模块架构，包括：
- 交互式配置设置和配置查看功能
- 日志级别管理
- Shell Completion 管理

这些命令负责管理 Workflow CLI 的核心 TOML 配置文件，使用统一的 `ConfigManager` 进行配置更新。

**注意**：以下功能已独立到其他模块：
- **GitHub 账号管理** → `commands/github/`（详见 [GitHub 命令架构文档](./GITHUB_COMMAND_architecture.md)）
- **环境检查** → `commands/check/`（详见 [环境检查命令架构文档](./CHECK_COMMAND_architecture.md)）

---

## 📁 相关文件

### CLI 入口层

```
src/main.rs
```

### 命令封装层

```
src/commands/config/
├── setup.rs        # 初始化设置命令（~653 行）
├── show.rs         # 配置查看命令（~196 行）
├── log.rs          # 日志级别管理命令（~108 行）
├── validate.rs     # 配置验证命令（~480 行）
├── export.rs       # 配置导出命令（~224 行）
├── import.rs       # 配置导入命令（~513 行）
└── completion.rs   # Shell Completion 管理命令（~303 行）
```

### 依赖模块（简要说明）

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：
- **`lib/base/settings/`**：TOML 配置管理（`Settings`、`Paths`）
- **`lib/git/`**：Git 操作（`GitRepo`、`GitCommit`）
- **`lib/base/http/`**：HTTP 客户端（`HttpClient`）
- **`lib/jira/config.rs`**：配置管理器（`ConfigManager`）
- **`lib/git/config.rs`**：Git 配置管理（`GitConfig`）
- **`lib/base/util/`**：工具函数（`LogLevel`、`mask-_sensitive-_value()`）

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
  2. load-_existing-_config()                   # 转换为 CollectedConfig
  3. collect-_config()                          # 收集配置信息（交互式）
     ├─ 用户配置（EMAIL）
     ├─ Jira 配置（地址、Token）
     ├─ GitHub 配置（Token、分支前缀）
     ├─ 日志配置（文件夹、删除策略）
     ├─ 代理配置（是否禁用检查）
     ├─ LLM 配置（提供商、API Key）
     └─ Codeup 配置（项目 ID、CSRF Token、Cookie）
  4. save-_config()                             # 保存配置到 TOML 文件
     ├─ workflow.toml (主配置)
     └─ llm.toml (LLM 配置，如果存在)
  5. verify-_config()                            # 验证配置（可选）
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
   - 使用 `toml::to-_string-_pretty()` 序列化为 TOML 格式
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
  1. Paths::workflow-_config()                # 获取 workflow.toml 路径
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
   - 使用 `mask-_sensitive-_value()` 函数
   - 只显示前 4 个字符和后 4 个字符，中间用 `***` 替代

2. **配置验证**：
   - 使用 `Settings::verify()` 方法统一验证和显示配置
   - 自动处理空配置情况

---

## 3. 日志级别管理命令 (`log.rs`)

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
  1. LogLevel::get-_level()                   # 获取当前日志级别
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
   - 使用 `LogLevel::set-_level()` 在内存中设置日志级别
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

## 4. 配置验证命令 (`validate.rs`)

### 相关文件

```
src/commands/config/validate.rs
```

### 调用流程

```
src/main.rs::Commands::Config { subcommand: Validate }
  ↓
commands/config/validate.rs::ConfigValidateCommand::validate()
  ↓
  1. 确定配置文件路径（默认或指定路径）
  2. 读取并解析配置文件（支持 TOML、JSON、YAML）
  3. 执行配置验证（validate-_config）
     ├─ 验证 JIRA 配置（邮箱格式、URL 格式、API token）
     ├─ 验证 GitHub 配置（账号名称、API token）
     ├─ 验证日志配置（路径格式）
     └─ 验证 LLM 配置（URL 格式、provider 枚举值）
  4. 如果指定 --fix，尝试自动修复错误
  5. 显示验证结果（错误、警告）
  6. 如果指定 --strict，将警告视为错误
```

### 功能说明

1. **配置验证**：
   - 验证配置文件格式（TOML、JSON、YAML）
   - 检查必需字段是否存在
   - 验证字段类型和值的有效性
   - 验证 URL 格式、邮箱格式等

2. **自动修复**：
   - 使用 `--fix` 选项可以自动修复常见错误
   - 修复 URL 格式错误（自动添加 https://）
   - 修复其他可修复的配置问题

3. **严格模式**：
   - 使用 `--strict` 选项将所有警告视为错误
   - 确保配置完全符合规范

### 关键步骤说明

1. **配置解析**：
   - 支持 TOML、JSON、YAML 三种格式
   - 自动检测文件格式或根据扩展名判断

2. **验证项**：
   - **JIRA 配置**：邮箱格式、URL 格式、API token 非空
   - **GitHub 配置**：账号名称非空、API token 非空
   - **日志配置**：路径格式验证
   - **LLM 配置**：URL 格式、provider 枚举值验证

3. **错误报告**：
   - 显示详细的错误信息（字段名、错误原因）
   - 提供修复建议（如果可修复）
   - 区分错误和警告

### 使用示例

```bash
# 验证配置
workflow config validate

# 自动修复配置错误
workflow config validate --fix

# 严格模式（所有警告视为错误）
workflow config validate --strict

# 验证指定配置文件
workflow config validate /path/to/config.toml
```

---

## 5. 配置导出命令 (`export.rs`)

### 相关文件

```
src/commands/config/export.rs
```

### 调用流程

```
src/main.rs::Commands::Config { subcommand: Export }
  ↓
commands/config/export.rs::ConfigExportCommand::export()
  ↓
  1. 确定导出格式（toml、json、yaml，默认 toml）
  2. 加载当前配置（Settings::load()）
  3. 如果指定 --section，提取特定配置段
  4. 验证配置有效性（validate-_config）
  5. 如果指定 --no-secrets，过滤敏感信息
  6. 保存配置到指定文件
```

### 功能说明

1. **导出格式**：
   - 支持 TOML（默认）、JSON、YAML 三种格式
   - 通过 `--json`、`--yaml` 选项指定格式

2. **选择性导出**：
   - 使用 `--section` 选项可以只导出特定配置段（jira、github、log、llm）

3. **敏感信息过滤**：
   - 使用 `--no-secrets` 选项可以排除敏感信息
   - 自动过滤 API tokens、密钥等敏感字段

4. **配置验证**：
   - 导出前自动验证配置有效性
   - 如果验证失败，取消导出操作

### 关键步骤说明

1. **配置提取**：
   - 如果指定 `--section`，只提取对应配置段
   - 否则导出完整配置

2. **敏感信息过滤**：
   - 过滤 JIRA API token
   - 过滤 GitHub API tokens
   - 过滤 LLM key
   - 被过滤的字段显示为 `***FILTERED***`

3. **文件保存**：
   - 如果输出路径是目录，自动生成文件名
   - 文件名格式：`config.{section}.{format}` 或 `config.{format}`

### 使用示例

```bash
# 导出完整配置（TOML 格式）
workflow config export config.backup.toml

# 只导出 JIRA 配置
workflow config export config.backup.toml --section jira

# 导出配置并排除敏感信息
workflow config export config.backup.toml --no-secrets

# 导出为 JSON 格式
workflow config export config.backup.json --json

# 导出为 YAML 格式
workflow config export config.backup.yaml --yaml
```

---

## 6. 配置导入命令 (`import.rs`)

### 相关文件

```
src/commands/config/import.rs
```

### 调用流程

```
src/main.rs::Commands::Config { subcommand: Import }
  ↓
commands/config/import.rs::ConfigImportCommand::import()
  ↓
  1. 读取并解析输入文件（支持 TOML、JSON、YAML）
  2. 如果指定 --section，提取特定配置段
  3. 验证导入的配置有效性
  4. 如果指定 --dry-run，只预览变更
  5. 创建当前配置的备份（ImportTransaction）
  6. 根据模式执行导入（合并或覆盖）
  7. 验证最终配置
  8. 如果验证失败，自动回滚到备份
  9. 保存配置并提交事务
```

### 功能说明

1. **导入模式**：
   - **合并模式**（默认）：保留现有配置，只更新导入的部分
   - **覆盖模式**（`--overwrite`）：完全替换现有配置

2. **选择性导入**：
   - 使用 `--section` 选项可以只导入特定配置段

3. **试运行模式**：
   - 使用 `--dry-run` 选项可以预览变更而不实际导入

4. **自动备份和回滚**：
   - 导入前自动创建备份文件
   - 如果导入后验证失败，自动回滚到备份
   - 备份文件命名：`config.backup.{timestamp}.toml`

5. **配置验证**：
   - 导入前验证输入配置
   - 导入后验证最终配置
   - 验证失败时自动回滚

### 关键步骤说明

1. **配置解析**：
   - 支持 TOML、JSON、YAML 三种格式
   - 自动检测文件格式

2. **事务管理**：
   - 使用 `ImportTransaction` 管理导入过程
   - 创建备份、回滚、提交事务

3. **配置合并**：
   - **合并模式**：深度合并配置，保留现有值
   - **覆盖模式**：完全替换配置
   - **段导入**：只更新指定配置段

4. **错误处理**：
   - 验证失败时自动回滚
   - 保存失败时自动回滚
   - 提供详细的错误信息

### 使用示例

```bash
# 导入配置（合并模式）
workflow config import config.backup.toml

# 导入配置（覆盖模式）
workflow config import config.backup.toml --overwrite

# 只导入 JIRA 配置
workflow config import config.backup.toml --section jira

# 试运行（预览变更）
workflow config import config.backup.toml --dry-run

# 从 JSON 文件导入
workflow config import config.backup.json

# 从 YAML 文件导入
workflow config import config.backup.yaml
```

---

## 7. Shell Completion 管理命令 (`completion.rs`)

### 相关文件

```
src/commands/config/completion.rs
```

### 功能说明

Shell Completion 管理命令提供 Shell 补全脚本的生成和管理功能，支持多种 Shell 类型（zsh, bash, fish, powershell, elvish）。

### 数据流

#### 配置管理数据流

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

#### 配置文件结构

```
~/.workflow/
└── config/
    ├── workflow.toml      # 主配置文件
    ├── llm.toml           # LLM 配置（可选）
    ├── jira-status.toml   # Jira 状态配置
    └── jira-users.toml    # Jira 用户缓存
```

**注意**：分支配置已迁移到项目级配置（`.workflow/config.toml`），由 `repo` 命令模块管理，详见相关模块文档。

---

## 🏗️ 架构设计

### 设计模式

### 1. 命令模式

每个命令都是一个独立的结构体，实现统一的方法接口。

### 2. 配置管理模式

使用 `Settings` 结构体和 `ConfigPaths` 统一管理 TOML 配置文件的读写：
- **单例模式**：使用 `OnceLock` 实现配置的单次加载和缓存
- **路径管理**：使用 `ConfigPaths` 统一管理所有配置文件路径
- **默认值**：使用 `#[serde(default)]` 和 `Default` trait 提供默认值
- **分离存储**：主配置和 LLM 配置分别存储在不同的文件中

### 错误处理

#### 分层错误处理

1. **CLI 层**：参数验证错误
2. **命令层**：用户交互错误、业务逻辑错误
3. **工具层**：文件操作错误、配置读写错误

#### 容错机制

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
3. 在 `setup.rs` 的 `collect-_config()` 方法中添加配置收集逻辑
4. 在 `setup.rs` 的 `save-_config()` 方法中添加保存逻辑
5. 在 `show.rs` 中，`Settings::verify()` 会自动显示新配置项

### 添加新的配置管理子命令

1. 在 `src/commands/config/` 中创建新的命令文件（如 `xxx.rs`）
2. 实现命令结构体和方法（参考 `github.rs` 或 `log.rs`）
3. 在 `src/commands/config/mod.rs` 中声明模块
4. 在 `src/main.rs` 中添加命令枚举和分发逻辑
5. 如果需要共享逻辑，在 `helpers.rs` 中添加辅助函数
6. 使用 `ConfigManager::<Settings>::update()` 更新配置（推荐）

### 添加新的环境检查项

环境检查功能已独立到 `commands/check/` 模块，详见 [环境检查命令架构文档](./CHECK_COMMAND_architecture.md)。

如需添加新的检查项：
1. 在 `commands/check/check.rs` 的 `run-_all()` 方法中添加新的检查步骤
2. 调用相应的工具函数或库函数
3. 提供清晰的错误提示和解决建议
4. 更新检查步骤的编号和日志输出

---

## 📚 相关文档

- [主架构文档](../architecture.md)
- [生命周期管理命令模块架构文档](./LIFECYCLE_COMMAND_architecture.md)
- [GitHub 命令架构文档](./GITHUB_COMMAND_architecture.md) - GitHub 账号管理相关
- [环境检查命令架构文档](./CHECK_COMMAND_architecture.md) - 环境检查相关
- [Git 模块架构文档](../lib/git.md) - Git 操作相关
- [HTTP 模块架构文档](../lib/http.md) - HTTP 客户端相关
- [Jira 模块架构文档](../lib/jira.md) - ConfigManager 使用说明

---

## 📋 使用示例

### Setup 命令

```bash
# 初始化配置
workflow config setup
```

### Show 命令

```bash
# 查看配置
workflow config show
```

### 日志级别管理

```bash
# 设置日志级别
workflow config log set

# 查看当前日志级别
workflow config log check
```

### 配置验证

```bash
# 验证配置
workflow config validate

# 自动修复配置错误
workflow config validate --fix

# 严格模式
workflow config validate --strict
```

### 配置导出

```bash
# 导出完整配置
workflow config export config.backup.toml

# 只导出 JIRA 配置
workflow config export config.backup.toml --section jira

# 导出并排除敏感信息
workflow config export config.backup.toml --no-secrets

# 导出为 JSON 格式
workflow config export config.backup.json --json
```

### 配置导入

```bash
# 导入配置（合并模式）
workflow config import config.backup.toml

# 导入配置（覆盖模式）
workflow config import config.backup.toml --overwrite

# 只导入 JIRA 配置
workflow config import config.backup.toml --section jira

# 试运行（预览变更）
workflow config import config.backup.toml --dry-run
```

### Shell Completion 管理

```bash
# 生成补全脚本
workflow completion generate

# 检查补全状态
workflow completion check

# 移除补全配置
workflow completion remove
```

---

## ✅ 总结

Config 命令层采用清晰的配置管理设计：

1. **交互式配置**：通过 `setup` 命令交互式收集配置
2. **配置查看**：通过 `show` 命令查看和验证配置
3. **配置验证**：通过 `validate` 命令验证配置完整性和有效性
4. **配置导出**：通过 `export` 命令导出配置用于备份和迁移
5. **配置导入**：通过 `import` 命令导入配置（支持合并和覆盖模式）
6. **日志管理**：通过 `log` 命令管理日志级别
7. **Completion 管理**：通过 `completion` 命令管理 Shell 补全

**设计优势**：
- ✅ **易用性**：交互式配置，用户友好
- ✅ **完整性**：配置验证、导入/导出功能完善
- ✅ **安全性**：支持敏感信息过滤、自动备份和回滚
- ✅ **灵活性**：支持多种格式（TOML、JSON、YAML）、选择性导入/导出
- ✅ **模块化**：与其他命令模块（GitHub、Check、Branch、Proxy）职责分离
- ✅ **可扩展性**：易于添加新的配置项

---

**最后更新**: 2025-12-16
