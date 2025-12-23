# 生命周期管理命令模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的生命周期管理模块架构，包括：
- 安装功能（二进制文件和 shell completion 脚本）
- 卸载功能（清理所有相关文件和配置）
- 更新功能（从 GitHub Releases 更新到新版本）
- 版本显示功能（显示当前版本信息）

这些命令负责管理 Workflow CLI 的完整生命周期，从初始安装到后续更新和卸载。

---

## 📁 Commands 层架构（命令封装）

> **架构说明**：本模块遵循项目的三层架构设计，详见 [architecture.md](./architecture.md#三层架构设计)

### CLI 入口层

```
src/bin/install.rs (独立可执行文件入口)
src/main.rs (卸载和更新命令入口)
```

### 命令封装层

```
src/commands/lifecycle/
├── install.rs      # 安装命令（144 行）
├── uninstall.rs    # 卸载命令（303 行）
├── update.rs       # 更新命令（924 行）
└── version.rs      # 版本显示命令（21 行）
```

### 依赖模块（简要说明）

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：
- **`lib/base/settings/paths.rs`**：路径管理（`Paths`）
- **`lib/base/shell/`**：Shell 检测和配置管理（`Detect`, `Reload`）
- **`lib/completion/`**：Completion 脚本生成和配置（`Completion`）
- **`lib/base/http/`**：HTTP 客户端（用于更新功能）
- **`lib/base/util/`**：工具函数（`Checksum`, `Unzip`, `confirm`）
- **`lib/rollback/`**：回滚管理器（`RollbackManager`）
- **`lib/proxy/`**：代理管理器（`ProxyManager`）

---

## 🔄 集成关系

Lifecycle 命令模块是 Workflow CLI 的生命周期管理命令，提供安装、卸载、更新等功能。该命令通过以下方式与 Lib 层集成：

1. **Settings 模块集成**：使用 `Paths` 获取安装路径、配置文件路径等
2. **Shell 模块集成**：使用 `Detect` 检测 Shell 类型，使用 `Reload` 重新加载 Shell 配置
3. **Completion 模块集成**：使用 `Completion` 生成和配置补全脚本
4. **HTTP 模块集成**：使用 `HttpClient` 下载更新文件
5. **工具模块集成**：使用 `Checksum` 验证文件完整性，使用 `Unzip` 解压文件，使用 `confirm` 进行用户确认
6. **Rollback 模块集成**：使用 `RollbackManager` 在更新失败时回滚
7. **Proxy 模块集成**：使用 `ProxyManager` 管理代理

### 主要集成场景

- **安装**：下载二进制文件，生成补全脚本，配置 Shell
- **更新**：下载新版本，验证完整性，更新文件，失败时回滚
- **卸载**：删除文件，清理配置

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
bin/install.rs 或 main.rs (CLI 入口，参数解析)
  ↓
commands/lifecycle/* (命令封装层)
  ├── install.rs (安装命令)
  ├── uninstall.rs (卸载命令)
  ├── update.rs (更新命令)
  └── version.rs (版本显示命令)
  ↓
lib/* (通过 API 调用，具体实现见相关模块文档)
```

---

## 1. 安装命令 (`install.rs`)

### 相关文件

```
src/commands/lifecycle/install.rs
src/bin/install.rs (独立可执行文件入口)
```

### 调用流程

```
bin/install.rs::main()
  ↓
commands/lifecycle/install.rs::InstallCommand::{install-_binaries|install-_completions}()
  ↓
  1. install-_binaries()                       # 安装二进制文件（可选）
     └─ 复制 workflow 到 /usr/local/bin
  2. install-_completions()                    # 安装 shell completion（可选）
     ├─ Detect::shell()                      # 检测 shell 类型
     ├─ Completion::generate-_all-_completions() # 生成 completion 脚本
     └─ Completion::configure-_shell-_config()  # 配置 shell 配置文件
```

### 功能说明

安装命令提供两个独立的功能，可以通过 `bin/install.rs` 的参数选择：

1. **安装二进制文件** (`install-_binaries()`)：
   - 在当前可执行文件所在目录查找 `workflow` 二进制文件
   - 将它复制到 `/usr/local/bin`
   - 使用 `sudo` 复制文件并设置执行权限
   - 显示安装进度和结果

2. **安装 Shell Completion** (`install-_completions()`)：
   - 自动检测当前 shell 类型（zsh, bash, fish, powershell, elvish）
   - 只生成当前检测到的 shell 类型的 completion 脚本
   - 为 `workflow` 命令及其所有子命令生成 completion 脚本
   - 自动在 shell 配置文件中添加 completion 加载代码
   - 提供重新加载 shell 配置的提示

### 关键步骤说明

1. **二进制文件安装**：
   - 从当前可执行文件所在目录查找二进制文件
   - 使用 `sudo cp` 复制到 `/usr/local/bin`
   - 使用 `sudo chmod +x` 设置执行权限
   - 跳过不存在的二进制文件（给出警告）

2. **Completion 脚本生成**：
   - 使用 `Completion::generate-_all-_completions()` 生成脚本
   - 只生成当前 shell 类型的脚本（简化安装流程）
   - 根据 shell 类型生成对应的文件（如 zsh 的 `_workflow`）

3. **Shell 配置管理**：
   - 使用 `Completion::configure-_shell-_config()` 配置 shell 配置文件
   - 在 `~/.zshrc` 或 `~/.bashrc` 等文件中添加 completion 加载代码
   - 根据检测到的 shell 类型提供相应的重新加载命令提示

---

## 2. 卸载命令 (`uninstall.rs`)

### 相关文件

```
src/commands/lifecycle/uninstall.rs
```

### 调用流程

```
main.rs::Commands::Uninstall
  ↓
commands/lifecycle/uninstall.rs::UninstallCommand::run()
  ↓
  1. 显示卸载信息（确认提示）
  2. 第一步确认：是否删除二进制文件和 completion 脚本
  3. 第二步确认：是否删除 TOML 配置文件
  4. remove-_binaries()                      # 删除二进制文件
     └─ 直接删除或使用 sudo 删除
  5. Completion::remove-_completion-_files()  # 删除 completion 脚本
  6. Completion::remove-_completion-_config-_file() # 删除 completion 配置文件
  7. Completion::remove-_all-_completion-_configs() # 移除所有 shell 的 completion 配置
  8. remove-_config-_files()                  # 删除 TOML 配置文件（如果确认）
  9. remove-_proxy-_settings()                # 移除代理设置
     └─ ProxyManager::disable()
  10. Reload::shell()                        # 重新加载 shell 配置
```

### 功能说明

1. **两步确认机制**：
   - 第一步：确认是否删除二进制文件和 completion 脚本
   - 第二步：确认是否删除 TOML 配置文件（可选）

2. **二进制文件删除**：
   - 删除 `workflow`、`install` 二进制文件
   - 自动处理需要 sudo 权限的文件
   - 显示将要删除的文件列表

3. **Completion 清理**：
   - 删除所有 shell 类型的 completion 脚本文件
   - 删除 completion 配置文件
   - 从所有 shell 配置文件中移除 completion 加载代码

4. **配置清理**：
   - 删除 TOML 配置文件（`workflow.toml`、`jira-users.toml`）
   - 可选：用户可以选择保留配置文件

5. **代理设置清理**：
   - 从 shell 环境变量中移除代理设置
   - 使用 `ProxyManager::disable()` 方法

### 关键步骤说明

1. **二进制文件删除**：
   - 检查文件是否存在
   - 尝试直接删除，如果失败则使用 `sudo rm`
   - 提供清晰的错误提示和手动删除建议

2. **Completion 清理**：
   - 删除所有 shell 类型的 completion 文件（不依赖当前 shell）
   - 删除 completion 配置文件（`.completions`）
   - 从所有 shell 配置文件中移除 completion 配置块

3. **配置清理**：
   - 删除 `workflow.toml` 和 `jira-users.toml`
   - 支持部分清理（只删除二进制和 completion，保留配置）

4. **Shell 配置重新加载**：
   - 尝试自动重新加载 shell 配置
   - 如果失败，提供手动重新加载的命令提示

---

## 3. 更新命令 (`update.rs`)

### 相关文件

```
src/commands/lifecycle/update.rs
```

### 调用流程

```
main.rs::Commands::Update { version }
  ↓
commands/lifecycle/update.rs::UpdateCommand::update(version)
  ↓
  1. get-_current-_version()                   # 获取当前版本
  2. detect-_platform()                       # 检测平台（macOS-Intel/macOS-AppleSilicon）
  3. get-_version(version)                     # 获取目标版本（指定或最新）
  4. compare-_versions()                      # 比较版本
  5. confirm()                                # 用户确认
  6. RollbackManager::create-_backup()        # 创建备份
  7. build-_download-_url()                     # 构建下载 URL
  8. download-_file()                         # 下载 tar.gz 文件
  9. Checksum::verify()                      # 验证文件完整性（SHA256）
  10. extract-_archive()                      # 解压文件
  11. install()                              # 运行 ./install 安装
  12. verify-_installation()                 # 验证安装结果
  13. RollbackManager::rollback()            # 如果失败，回滚（可选）
```

### 功能说明

更新命令提供从 GitHub Releases 更新 Workflow CLI 的完整功能，包括版本管理、下载、验证、安装和回滚：

1. **版本管理**：
   - 获取当前安装的版本号（多种方法：环境变量、命令执行、Cargo.toml）
   - 从 GitHub API 获取最新版本号
   - 版本比较（UpToDate, NeedsUpdate, Downgrade）
   - 支持指定版本号更新

2. **平台检测**：
   - 自动检测当前平台（macOS-Intel 或 macOS-AppleSilicon）
   - 根据平台匹配对应的 Release 资源文件

3. **下载流程**：
   - 从 GitHub Releases 下载 tar.gz 文件
   - 显示下载进度条（使用 `indicatif`）
   - 支持重试机制（使用 `HttpRetry`）
   - 显示文件大小和下载速度

4. **验证机制**：
   - 下载校验和文件（SHA256）
   - 验证下载文件的完整性
   - 确保文件未被篡改

5. **安装流程**：
   - 解压 tar.gz 文件到临时目录
   - 运行解压目录中的 `./install` 脚本
   - 自动安装二进制文件和 completion 脚本

6. **验证功能**：
   - 验证二进制文件状态（存在、可执行、版本正确、可用）
   - 验证 completion 脚本安装
   - 验证安装结果

7. **回滚机制**：
   - 更新前自动创建备份（使用 `RollbackManager`）
   - 更新失败时自动回滚到之前版本
   - 更新成功后清理备份和临时文件

### 关键步骤说明

1. **版本获取**：
   - 优先从环境变量 `CARGO_PKG_VERSION` 获取（编译时注入）
   - 其次运行 `workflow --version` 命令获取
   - 最后从 `Cargo.toml` 读取（开发环境）
   - 如果都找不到，允许继续更新流程

2. **版本比较**：
   - 支持语义化版本号比较（major.minor.patch）
   - 如果当前版本已是最新，直接返回
   - 如果目标版本更低，提示降级操作

3. **下载和验证**：
   - 使用流式下载，支持大文件
   - 显示实时下载进度
   - 下载完成后验证 SHA256 校验和
   - 确保文件完整性

4. **安装过程**：
   - 解压到临时目录
   - 运行解压目录中的 `./install` 脚本
   - 自动安装二进制文件和 completion 脚本

5. **回滚机制**：
   - 更新前创建备份（二进制文件和 completion 脚本）
   - 如果更新失败，自动回滚到备份版本
   - 如果回滚失败，提供手动恢复指导

6. **验证流程**：
   - 验证 `workflow` 二进制文件
   - 检查文件存在、可执行、版本正确、可用
   - 验证 completion 脚本安装
   - 如果验证失败，认为更新失败并触发回滚

### 错误处理

- **版本获取失败**：允许继续更新流程，但无法比较版本
- **下载失败**：自动重试，如果仍然失败则回滚
- **校验和验证失败**：认为文件损坏，回滚
- **安装失败**：自动回滚到之前版本
- **验证失败**：认为更新失败，回滚
- **回滚失败**：提供详细的错误信息和手动恢复指导

### 数据流

#### 安装数据流

```
Shell 检测
  ↓
Completion 生成
  ↓
文件系统操作（创建文件）
  ↓
Shell 配置更新
```

#### 卸载数据流

```
用户确认
  ↓
二进制文件删除
  ↓
Completion 清理
  ↓
配置清理（可选）
  ↓
代理设置清理
  ↓
Shell 配置重新加载
```

#### 更新数据流

```
版本检查
  ↓
用户确认
  ↓
创建备份
  ↓
下载 Release
  ↓
验证校验和
  ↓
解压文件
  ↓
安装（运行 ./install）
  ↓
验证安装结果
  ↓
成功：清理备份和临时文件
失败：回滚到备份版本
```

---

## 4. 版本显示命令 (`version.rs`)

### 相关文件

```
src/commands/lifecycle/version.rs
```

### 调用流程

```
main.rs::Commands::Version
  ↓
commands/lifecycle/version.rs::VersionCommand::show()
  ↓
  1. 从编译时嵌入的版本号获取（使用 env! 宏）
  2. 显示版本信息
```

### 功能说明

版本显示命令提供显示当前 Workflow CLI 版本信息的功能：

1. **版本号获取**：
   - 从编译时嵌入的版本号获取（使用 `env!("CARGO_PKG_VERSION")` 宏）
   - 注意：`env!` 宏在编译时展开，所以这个值在运行时总是可用的

2. **版本显示**：
   - 使用 `log-_success!` 宏显示版本信息
   - 格式：`workflow v{version}`

### 关键步骤说明

1. **版本号获取**：
   - 使用 `env!("CARGO_PKG_VERSION")` 宏在编译时获取版本号
   - 这个值来自 `Cargo.toml` 中的 `version` 字段
   - 在运行时总是可用，无需额外处理

2. **版本显示**：
   - 使用 `log-_success!` 宏显示版本信息
   - 提供清晰的版本输出格式

### 使用场景

- **版本查询**：用户可以通过 `workflow --version` 或 `workflow version` 查看当前版本
- **更新检查**：在更新命令中用于比较当前版本和目标版本

---

## 🏗️ 架构设计

### 设计模式

### 1. 命令模式

每个命令都是一个独立的结构体，实现统一的方法接口：
- `InstallCommand` - 安装命令
- `UninstallCommand` - 卸载命令
- `UpdateCommand` - 更新命令

### 2. 工具函数模式

将复杂的操作封装到 `lib/` 中的工具函数，命令层只负责调用和交互：
- `Completion` - Completion 管理
- `RollbackManager` - 回滚管理
- `Checksum` - 校验和验证
- `Unzip` - 文件解压

### 3. 回滚模式（更新功能）

更新功能使用回滚模式确保系统一致性：
- 更新前创建备份
- 更新失败时自动回滚
- 更新成功后清理备份

### 4. 重试模式（更新功能）

下载和网络请求使用重试模式提高可靠性：
- 使用 `HttpRetry` 实现自动重试
- 提供重试配置和错误提示

---

## 🔍 错误处理

### 分层错误处理

1. **CLI 层**：参数验证错误
2. **命令层**：用户交互错误、业务逻辑错误
3. **工具层**：文件操作错误、配置读写错误

#### 容错机制

- **文件操作失败**：提供清晰的错误提示和手动操作建议
- **权限不足**：自动使用 sudo 尝试删除或安装
- **配置清理失败**：提供手动清理步骤
- **更新下载失败**：自动重试，如果仍然失败则回滚
- **更新验证失败**：认为更新失败，自动回滚到之前版本
- **回滚失败**：提供详细的错误信息和手动恢复指导

---

## 📝 扩展性

### 添加新的 Shell 支持

1. 在 `lib/completion/` 中添加新的 shell 类型支持（参考相关模块文档）
2. `Completion::generate-_all-_completions()` 会自动支持新的 shell 类型

### 添加新的二进制文件

1. 在 `lib/base/settings/paths.rs` 的 `binary-_paths()` 方法中添加新的二进制路径（参考相关模块文档）
2. 在 `update.rs` 的 `verify-_binaries()` 方法中添加新的二进制文件验证
3. 在 `install.rs` 的 `install-_binaries()` 方法中添加新的二进制文件安装逻辑

**注意**：当前架构中，所有功能都通过 `workflow` 主命令及其子命令提供，不再需要独立的二进制文件。

### 添加新的更新验证项

1. 在 `update.rs` 的 `verify-_installation()` 方法中添加新的验证步骤
2. 更新 `VerificationResult` 结构体以包含新的验证结果

---

## 4. GitHub Actions 发布流程

### 概述

项目使用 GitHub Actions 自动构建和发布。发布流程包括：
- 自动创建版本 tag
- 构建多平台二进制文件
- 创建 GitHub Release
- 自动更新 Homebrew Formula

### 相关文件

```
.github/workflows/release.yml  # GitHub Actions 工作流定义
homebrew/Formula.template       # Homebrew Formula 模板
```

### 发布流程说明

1. **触发条件**：
   - 推送到 `master` 分支时自动创建 tag
   - 创建版本 tag（如 `v1.0.0`）时触发发布
   - 手动触发（workflow-_dispatch）

2. **自动版本管理机制**：

   项目使用基于 **Conventional Commits** 规范的自动版本管理机制，根据 commit messages 自动确定版本更新类型：

   - **Major 版本更新** (`1.0.0` → `2.0.0`)：
     - 检测到 `BREAKING CHANGE` 或 `BREAKING:` 关键词
     - 检测到 commit message 中包含 `!`（如 `feat!: add new API`）

   - **Minor 版本更新** (`1.0.0` → `1.1.0`)：
     - 检测到 `feat:` 或 `feature:` 开头的 commit message
     - 例如：`feat: add new feature` 或 `feature: implement new functionality`

   - **Patch 版本更新** (`1.0.0` → `1.0.1`)：
     - 其他所有情况（bug fix、docs、refactor 等）
     - 例如：`fix: bug fix`、`docs: update documentation`、`refactor: code cleanup`

   **工作流程**：
   1. 当代码推送到 `master` 分支时，workflow 会：
      - 分析从最新 tag 到当前 commit 的所有 commit messages
      - 根据 commit messages 确定版本更新类型（major/minor/patch）
      - 如果 tag 已存在但指向不同 commit，自动递增版本号
      - 如果 tag 不存在，检查 `Cargo.toml` 中的版本是否需要更新

   2. **版本更新优先级**：
      - 如果检测到 `BREAKING CHANGE` → 递增 major 版本
      - 如果检测到 `feat:` → 递增 minor 版本
      - 其他情况 → 递增 patch 版本

   3. **示例场景**：
      - 当前版本：`1.0.9`，最新 tag：`v1.0.9`
      - 提交了 `feat: add new feature` commit
      - 推送到 master 后，自动创建 tag `v1.1.0`（minor 版本更新）
      - 如果 `Cargo.toml` 中版本是 `1.0.9`，会自动更新为 `1.1.0`

3. **Token 配置**：
   - 需要在仓库 Secrets 中配置 `HOMEBREW_TAP_TOKEN`
   - Token 需要 `repo` scope
   - Token 所属账号需要有访问 `homebrew-workflow` 仓库的权限
   - Workflow 会自动验证 token 的有效性和权限

4. **验证机制**：
   - 检查 token 是否存在
   - 验证 token 是否有效（通过 GitHub API）
   - 验证 token 是否有访问目标仓库的权限
   - 提供详细的错误信息和解决建议

5. **发布步骤**：
   - 分析 commit messages 确定版本更新类型
   - 创建或更新版本 tag（如果从 master 分支触发）
   - 自动更新 `Cargo.toml` 和 `Cargo.lock` 中的版本号（如果需要）
   - 构建二进制文件（多平台）
   - 创建 GitHub Release
   - 更新 Homebrew Formula（使用 `HOMEBREW_TAP_TOKEN` checkout 和更新）

### 配置 HOMEBREW_TAP_TOKEN

详细配置步骤请参考主 README.md 中的"发布"章节。

---

## 📚 相关文档

- [主架构文档](./architecture.md)
- [配置管理命令模块架构文档](./config.md)
- [Completion 模块架构文档](../architecture/completion.md) - Completion 管理相关
- [回滚模块架构文档](../architecture/rollback.md) - 回滚机制相关
- [HTTP 模块架构文档](../architecture/http.md) - HTTP 客户端相关
- [主 README.md](../README.md) - 包含发布流程和 HOMEBREW_TAP_TOKEN 配置说明

---

## 📋 使用示例

### Install 命令

```bash
# 安装 workflow CLI
./install
```

### Uninstall 命令

```bash
# 卸载 workflow CLI
workflow uninstall

# 完全卸载（包括配置文件）
workflow uninstall --purge
```

### Update 命令

```bash
# 更新到最新版本
workflow update

# 跳过确认直接更新
workflow update --yes
```

### Version 命令

```bash
# 显示当前版本
workflow --version

# 或使用 version 子命令
workflow version
```

---

## ✅ 总结

Lifecycle 命令层采用清晰的生命周期管理设计：

1. **安装**：自动检测 shell，生成 completion，配置环境
2. **卸载**：完整清理所有文件和配置
3. **更新**：自动备份、下载、验证、安装，失败自动回滚
4. **版本显示**：显示当前版本信息

**设计优势**：
- ✅ **安全性**：更新前自动备份，失败自动回滚
- ✅ **完整性**：完整的安装和卸载流程
- ✅ **可靠性**：校验和验证，确保文件完整性
- ✅ **用户友好**：清晰的进度提示和错误信息
- ✅ **版本管理**：简单的版本查询功能

---

**最后更新**: 2025-12-16
