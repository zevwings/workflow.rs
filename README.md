# Workflow - Rust CLI 工具

![GitHub Release](https://img.shields.io/github/v/release/zevwings/workflow.rs)
![License](https://img.shields.io/badge/license-MIT-green)
![CI](https://github.com/zevwings/workflow.rs/workflows/CI/badge.svg)
![Rust Version](https://img.shields.io/badge/rust-1.82+-orange)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)

工作流自动化工具的 Rust 实现版本。

## 🌐 跨平台支持

Workflow CLI 完全支持以下平台：
- **macOS** (Intel 和 Apple Silicon)
- **Linux** (x86_64, ARM64, 包括静态链接版本)
- **Windows** (x86_64, ARM64)

### 平台特定说明

#### macOS / Linux
- 二进制文件安装到 `/usr/local/bin`
- 配置文件存储在 `~/.workflow/config/`
- 补全脚本存储在 `~/.workflow/completions/`
- 安装/卸载可能需要 `sudo` 权限
- **剪贴板功能限制**：
  - Linux ARM64 和 musl 静态链接版本不支持剪贴板功能（XCB 库依赖问题）
  - 其他平台（macOS、Linux x86_64、Windows）完全支持剪贴板功能

#### Windows
- 二进制文件安装到 `%LOCALAPPDATA%\Programs\workflow\bin`
- 配置文件存储在 `%APPDATA%\workflow\config\`
- 补全脚本存储在 `%APPDATA%\workflow\completions\`
- 支持 PowerShell (PowerShell Core 和 Windows PowerShell)
- 安装/卸载可能需要管理员权限

## 🚀 快速开始

### 安装

#### 方式一：使用 Homebrew（推荐）

```bash
# 添加 tap（如果已创建）
brew tap zevwings/workflow

# 安装
brew install workflow
```

> **注意**：
> - 需要先在 GitHub 上创建 `homebrew-workflow` tap 仓库，并将 `Formula/workflow.rb` 文件推送到该仓库。
> - 如果使用 GitHub Actions 自动发布，需要配置 `HOMEBREW_TAP_TOKEN` secret（见下方说明）。

#### 方式二：使用安装脚本（推荐，macOS/Linux）

使用一键安装脚本自动下载并安装最新版本：

```bash
# 安装最新版本
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install/install.sh)"

# 安装指定版本（版本号见 GitHub Release，如 v0.0.1）
VERSION=v0.0.1 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install/install.sh)"
```

**功能特性**：
- ✅ 自动检测操作系统和架构（macOS Intel/Apple Silicon, Linux x86_64/ARM64）
- ✅ 自动下载最新版本或指定版本
- ✅ SHA256 校验和验证确保文件完整性
- ✅ 自动安装二进制文件和 shell completion 脚本
- ✅ 错误处理和重试机制
- ✅ 临时文件自动清理

**安装流程**：
1. **平台检测**：自动检测操作系统（macOS/Linux）和架构（x86_64/ARM64）
2. **版本获取**：从 GitHub Releases API 获取最新版本，或使用指定的版本
3. **下载**：下载对应平台的二进制包（`.tar.gz`）
4. **验证**：下载并验证 SHA256 校验和
5. **解压**：解压二进制包到临时目录
6. **安装**：运行 `./install` 二进制文件进行安装
7. **清理**：自动清理临时文件

**系统要求**：
- `curl` - 用于下载文件
- `tar` - 用于解压归档文件
- `sudo` - macOS/Linux 安装到系统目录时需要（脚本会自动提示）

**卸载**：

```bash
# 使用卸载脚本（会先尝试 workflow uninstall，不可用时回退到手动删除）
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install/uninstall.sh)"
```

已安装的 CLI 支持 `workflow uninstall` 时，卸载脚本会调用该命令；否则回退到手动删除二进制与 `~/.workflow`。也可按脚本提示手动删除。

卸载脚本功能：
- ✅ 自动检测已安装的 Workflow CLI
- ✅ 若支持则尝试 `workflow uninstall`，否则执行手动卸载
- ✅ 清理二进制文件、配置文件和 completion 脚本
- ✅ 交互式确认，避免误删

#### 方式三：使用安装脚本（Windows）

使用 PowerShell 安装脚本自动下载并安装最新版本：

**PowerShell (推荐)**:
```powershell
# 安装最新版本
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install.ps1" -OutFile install.ps1; .\install.ps1

# 或一行命令
powershell -ExecutionPolicy Bypass -Command "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install.ps1' -OutFile install.ps1; .\install.ps1"
```

**安装指定版本**:
```powershell
$env:VERSION="v1.6.4"; powershell -ExecutionPolicy Bypass -Command "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install.ps1' -OutFile install.ps1; .\install.ps1"
```

**功能特性**：
- ✅ 自动检测 Windows 架构（x86_64/ARM64）
- ✅ 自动下载最新版本或指定版本
- ✅ SHA256 校验和验证确保文件完整性
- ✅ 自动安装二进制文件和 PowerShell completion 脚本
- ✅ 错误处理和重试机制
- ✅ 临时文件自动清理
- ✅ PATH 环境变量检查

**安装流程**：
1. **平台检测**：自动检测 Windows 架构（x86_64/ARM64）
2. **版本获取**：从 GitHub Releases API 获取最新版本，或使用指定的版本
3. **下载**：下载对应平台的二进制包（`.zip`）
4. **验证**：下载并验证 SHA256 校验和
5. **解压**：解压二进制包到临时目录
6. **安装**：运行 `install.exe` 二进制文件进行安装
7. **清理**：自动清理临时文件

**系统要求**：
- PowerShell 5.0 或更高版本
- 网络连接（用于下载）
- 管理员权限（可能需要，取决于安装目录权限）

**卸载**：

```powershell
# 使用卸载脚本（PowerShell 推荐）
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/uninstall.ps1" -OutFile uninstall.ps1; .\uninstall.ps1

# 或一行命令
powershell -ExecutionPolicy Bypass -Command "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/uninstall.ps1' -OutFile uninstall.ps1; .\uninstall.ps1"

# 或使用已安装的命令
workflow uninstall
```

卸载脚本功能：
- ✅ 自动检测已安装的 Workflow CLI
- ✅ 若支持则尝试 `workflow uninstall`，否则执行手动卸载
- ✅ 手动卸载作为备选方案
- ✅ 清理二进制文件、配置文件和 completion 脚本
- ✅ 从 PATH 环境变量中移除安装目录
- ✅ 交互式确认，避免误删

卸载流程：
1. **检测安装**：检查 `workflow` 命令是否在 PATH 中
2. **确认卸载**：提示用户确认是否卸载
3. **执行卸载**：
   - 若支持则尝试 `workflow uninstall`，否则执行手动卸载
4. **清理文件**：
   - 删除二进制文件（`%LOCALAPPDATA%\Programs\workflow\bin\workflow.exe`、`install.exe`）
   - 可选删除配置文件（`%APPDATA%\workflow\`）
   - 删除 completion 脚本
   - 从 PowerShell profile 中移除 completion 配置
   - 从 PATH 环境变量中移除安装目录

#### 方式四：使用 Makefile（仅 macOS/Linux）

使用 Makefile 安装所有二进制文件到系统：

```bash
make install
```

这会安装以下命令到 `/usr/local/bin`（macOS/Linux）或 `%LOCALAPPDATA%\Programs\workflow\bin`（Windows）：
- `workflow` - 主命令（包含所有子命令：pr, log, jira 等）

**重要提示**：
- macOS/Linux：安装后如果命令无法识别，请重新加载 shell：`hash -r` 或重启终端
- Windows：确保安装目录已添加到 PATH 环境变量中

### 安装/卸载故障排除

#### 安装失败

1. **网络问题**：检查网络连接，脚本会自动重试 3 次
2. **权限问题**：
   - macOS/Linux：确保有 `sudo` 权限以安装到系统目录
   - Windows：确保有管理员权限（取决于安装目录权限）
3. **工具缺失**：
   - macOS/Linux：确保已安装 `curl` 和 `tar`
   - Windows：确保 PowerShell 版本为 5.0 或更高

#### 卸载失败

1. **权限问题**：某些文件可能需要管理员权限才能删除
2. **手动清理**：如果自动卸载失败，可以手动删除：
   - **macOS/Linux**：
     - 二进制文件：`/usr/local/bin/workflow`, `/usr/local/bin/install`
     - 配置文件：`~/.workflow/`
     - Shell 配置：从 `~/.zshrc` 或 `~/.bashrc` 中移除相关行
   - **Windows**：
     - 二进制文件：`%LOCALAPPDATA%\Programs\workflow\bin\workflow.exe`, `install.exe`
     - 配置文件：`%APPDATA%\workflow\`
     - PowerShell 配置：从 PowerShell profile 中移除相关行
     - PATH 环境变量：从 PATH 中移除 `%LOCALAPPDATA%\Programs\workflow\bin`

### 编译项目

```bash
cargo build --release
# 或使用 Makefile
make release
```

### 运行测试

```bash
cargo test
```

### 运行 CLI

```bash
cargo run -- --help
```

## 配置

在首次使用之前，需要配置必要的设置。推荐使用交互式设置命令：

```bash
workflow setup
```

这将引导你完成所有配置项的设置，并自动保存到 TOML 配置文件（`~/.workflow/config/workflow.toml`）。

### 配置文件位置

- **macOS/Linux**：`~/.workflow/config/workflow.toml`
- **Windows**：`%APPDATA%\workflow\config\workflow.toml`

配置文件包含用户、Jira、GitHub、日志、代理、Codeup、LLM/AI 等配置。

### 必填配置

以下配置项是**必须**设置的：

| 配置项 | 说明 | 示例 |
|-------|------|------|
| `user.email` | 用户邮箱地址 | `user@example.com` |
| `jira.api_token` | Jira API Token | 从 Jira 设置中获取 |
| `jira.service_address` | Jira 服务地址 | `https://your-company.atlassian.net` |
| `github.api_token` | GitHub API Token（用于 PR 操作） | 从 GitHub 设置中获取 |

### 可选配置

以下配置项是**可选**的，根据你的使用场景选择配置：

#### GitHub 配置

| 配置项 | 说明 | 默认值 |
|-------|------|--------|
| `github.accounts` | GitHub 账号列表 | - |
| `github.current` | 当前激活的账号名称 | - |

#### 日志配置

| 配置项 | 说明 | 默认值 |
|-------|------|--------|
| `log.output_folder_name` | 日志输出文件夹名称 | `logs` |
| `log.download_base_dir` | 下载基础目录 | `~/Documents/Workflow` |

#### LLM/AI 配置

| 配置项 | 说明 | 默认值 |
|-------|------|--------|
| `llm.provider` | LLM 提供者（`openai`/`deepseek`/`proxy`） | `openai` |
| `llm.key` | LLM API Key（所有提供者通用） | - |
| `llm.url` | LLM 服务 URL（仅 `proxy` 提供者需要） | - |
| `llm.model` | LLM 模型名称（可选，`openai` 默认 `gpt-4.0`，`deepseek` 默认 `deepseek-chat`，`proxy` 必填） | - |
| `llm.response_format` | 响应格式路径（用于从响应中提取内容，空字符串表示使用默认的 OpenAI 格式） | 空（不保存到配置文件） |

#### Codeup 配置

| 配置项 | 说明 | 默认值 |
|-------|------|--------|
| `codeup.project_id` | Codeup 项目 ID | - |
| `codeup.csrf_token` | Codeup CSRF Token | - |
| `codeup.cookie` | Codeup Cookie | - |

### 查看配置

查看当前所有配置：

```bash
workflow config
```

### 手动配置

如果不想使用交互式设置，也可以手动编辑 TOML 配置文件：

**macOS/Linux**：
```bash
# 编辑主配置文件
vim ~/.workflow/config/workflow.toml
```

**Windows**：
```powershell
# 编辑主配置文件（使用 PowerShell）
notepad $env:APPDATA\workflow\config\workflow.toml
```

配置文件示例：

```toml
# ~/.workflow/config/workflow.toml
[user]
email = "user@example.com"

[jira]
api_token = "your-jira-token"
service_address = "https://your-company.atlassian.net"

[github]
api_token = "your-github-token"

[log]
output_folder_name = "logs"
download_base_dir = "~/Documents/Workflow"

[llm]
provider = "openai"
key = "your-llm-api-key"
# model = "gpt-4.0"  # 可选，openai 默认 gpt-4.0
# response_format = ""  # 可选，空字符串表示使用默认的 OpenAI 格式，不保存到配置文件

# 如果使用 proxy 提供者，需要配置 url：
# [llm]
# provider = "proxy"
# url = "https://your-proxy-url"
# key = "your-proxy-key"
# model = "your-model-name"  # proxy 提供者必填
```

## 📋 命令清单

### 检查与版本
```bash
workflow check                     # 运行环境检查（Git 状态和网络连接）
workflow version                   # 显示 Workflow CLI 版本
```

> **注意**：pre-commit 检查已集成到 Git 提交流程中。当执行 `git commit` 时，如果工程中存在 pre-commit hooks（`.git/hooks/pre-commit` 或 `.pre-commit-config.yaml`），系统会自动执行 pre-commit 检查。

### 配置与生命周期
```bash
workflow setup                     # 初始化或更新配置（交互式设置）
workflow update [--target <VERSION>] [--force] [--github-token <TOKEN>]  # 更新 Workflow CLI 到最新或指定版本
workflow uninstall [--force] [--keep-config]   # 卸载 Workflow CLI（可选保留配置文件）
```

### GitHub 账号管理
```bash
workflow github check              # 检查 GitHub 账号配置（列出/当前账号等）
workflow github setup              # 设置 GitHub 账号（添加/切换/更新/删除，交互式）
```

### 日志级别管理
```bash
workflow log setup                 # 设置日志级别（交互式选择：none/error/warn/info/debug）
workflow log check                 # 检查当前日志级别（显示当前、默认和配置文件中的级别）
```

### LLM 配置管理
```bash
workflow llm check                 # 显示当前 LLM 配置（提供者、API Key（已掩码）、模型、语言设置）
workflow llm setup                 # 设置 LLM 配置（交互式配置提供者、代理 URL、API Key、模型等）
```

### Shell Completion 管理
```bash
workflow completion generate [-s <shell>] [-o <output>]  # 生成并配置 completion 脚本
workflow completion check          # 检查 completion 状态
workflow completion remove [--all] # 移除 completion 配置（--all 移除所有 shell）
```

### 仓库与分支管理
```bash
workflow repo setup                 # 交互式配置项目级设置（分支前缀、模板等）
workflow repo check                 # 验证仓库配置（项目/用户配置、模板等）

# 创建新分支
workflow branch create [JIRA_ID] [--from-default] [--dry-run]  # 创建新分支（可选 JIRA ticket、从默认分支、预览）

# 切换分支
workflow branch switch [BRANCH_NAME] # 切换到指定分支（不提供则交互式选择）

# 重命名分支
workflow branch rename             # 交互式重命名分支

# 清理本地分支
workflow branch clean [--dry-run]  # 清理已合并分支（保留 main/master、develop、当前分支与忽略列表）

# 管理分支忽略列表
workflow branch ignore add <BRANCH_NAME>    # 添加分支到忽略列表
workflow branch ignore remove [BRANCH_NAME] # 从忽略列表移除分支（不提供则交互式多选）
workflow branch ignore list                # 列出当前仓库的忽略分支

# 删除分支
workflow branch remove [BRANCH_NAME] [--local-only] [--remote-only] [--dry-run] [--force]
```

### Tag 管理
```bash
workflow tag create <TAG_NAME> [-t <target>] [-m <message>] [--local] [--force]  # 创建 tag（可选推送到远程）
workflow tag remove [TAG_NAME] [--local] [--remote] [-p <pattern>] [--dry-run] [--force]  # 删除 tag
```

### Stash 管理
```bash
workflow stash push                # 保存当前更改到 stash
workflow stash pop                 # 应用并删除最新 stash
workflow stash apply               # 应用 stash（保留条目）
workflow stash drop                # 删除 stash 条目
workflow stash list                # 列出所有 stash
```

### 别名管理
```bash
workflow alias list                # 列出所有已定义的别名
workflow alias add [NAME] [COMMAND] [--force]   # 添加别名（不提供参数则交互式）
workflow alias remove [NAME]       # 移除别名（不提供则交互式选择）
```

> **注意**：别名功能允许您为常用命令创建简短别名。例如，创建别名 `ci` 映射到 `pr create` 后，可以直接使用 `workflow ci` 来创建 PR。别名会在命令解析前自动展开。

### 安装命令
```bash
install                            # 安装 Workflow CLI 到系统（默认安装二进制文件 + shell completions）
install --binaries                 # 只安装二进制文件到 /usr/local/bin
install --completions              # 只安装 shell completion 脚本
```

> **注意**：`install` 命令是一个独立的可执行文件，用于将编译好的二进制文件安装到系统。如果同时指定 `--binaries` 和 `--completions`，或不指定任何选项，则安装全部内容。


### PR 操作
```bash
workflow pr create [JIRA_ID] [--dry-run]      # 创建 PR（可选 Jira ticket，AI 生成标题）
workflow pr list [--state <open|closed|all>] [--limit N]  # 列出 PR
workflow pr comment <PR_ID> [COMMENT]         # 添加评论（不提供 COMMENT 则交互式输入）
workflow pr update [PR_ID] [-m <message>]     # 提交本地更改并推送到 PR（使用 PR 标题或指定 message）
workflow pr merge <PR_ID> [--force]           # 合并 PR
workflow pr close <PR_ID>                     # 关闭 PR
workflow pr approve <PR_ID>                    # 批准 PR
workflow pr summarize [PR_ID]                 # 使用 LLM 总结 PR
```

### Jira 操作
```bash
workflow jira check                           # 检查 Jira 配置
workflow jira setup                           # 设置 Jira 配置（交互式）
workflow jira info [PROJ-123] [--json] [--markdown]   # 显示 ticket 信息
workflow jira attachments [PROJ-123]          # 下载所有附件
workflow jira clean [PROJ-123] [--all]        # 清理附件/日志目录（不指定 ID 则交互式）
workflow jira transition <PROJ-123>            # 过渡 Jira 状态
workflow jira assign <PROJ-123>                # 分配 ticket 给当前用户
```

> **注意**：Codeup 仓库的 PR 查看和合并功能正在开发中，GitHub 仓库已完整支持。详细说明请查看 [架构设计文档](./docs/guidelines/architecture.md)。

## 🚀 发布

### GitHub Actions 自动发布

项目使用 GitHub Actions 自动构建和发布。当推送到 `master` 分支或创建版本 tag 时，会自动触发发布流程。

#### 配置 HOMEBREW_TAP_TOKEN

为了自动更新 Homebrew Formula，需要在 GitHub 仓库中配置 `HOMEBREW_TAP_TOKEN` secret。

**配置步骤：**

1. **创建 Personal Access Token (PAT)**：
   - 访问：https://github.com/settings/tokens
   - 点击 "Generate new token" → 选择 "Generate new token (classic)"
   - 配置 Token：
     - Note（描述）：例如 "Homebrew Tap Token for workflow.rs"
     - Expiration（过期时间）：根据需要选择（建议至少 90 天或更长）
     - Select scopes：勾选 `repo`（Full control of private repositories）
   - 点击 "Generate token"
   - 复制生成的 token（只显示一次，请保存）

2. **在仓库中设置 Secret**：
   - 进入仓库设置页面：`Settings` → `Secrets and variables` → `Actions`
   - 点击 "New repository secret"
   - Name：输入 `HOMEBREW_TAP_TOKEN`
   - Secret：粘贴第一步复制的 token
   - 点击 "Add secret"

**重要提示：**
- Token 必须包含 `repo` scope
- Token 所属账号需要有访问 `homebrew-workflow` 仓库的权限
- 如果 `homebrew-workflow` 是私有仓库，确保 token 有访问权限
- Workflow 会自动验证 token 的有效性和权限

**验证配置：**

运行 GitHub Actions 时，workflow 会自动验证：
- Token 是否存在
- Token 是否有效（通过 GitHub API `/user` 端点）
- Token 是否有访问目标仓库的权限（通过 GitHub API `/repos/zevwings/homebrew-workflow` 端点）

如果验证失败，workflow 会提供详细的错误信息和解决建议。

### 发布流程

1. **自动创建 Tag**：当代码合并到 `master` 分支时，自动根据 `Cargo.toml` 中的版本号创建 tag
2. **构建二进制**：为多个平台构建 release 二进制文件
3. **创建 Release**：在 GitHub 上创建 Release，并上传构建产物
4. **更新 Homebrew Formula**：自动更新 `homebrew-workflow` 仓库中的 Formula 文件

## 🔧 开发

### 开发环境设置

首次开发前，请先安装所需的开发工具：

```bash
make setup
```

这会自动安装：
- `rustfmt` - 代码格式化工具
- `clippy` - 代码检查工具
- `rust-analyzer` - 语言服务器（从源码构建）

> **注意**：如果您的平台没有预编译的 rust-analyzer 二进制文件，`make setup` 会自动从源码构建安装。这可能需要几分钟时间。

### 添加依赖

```bash
cargo add <package-name>
```

### 代码格式化

```bash
cargo fmt
```

### Lint 检查

```bash
cargo clippy
# 或使用 Makefile 进行完整检查
make lint
```

### 开发规范

详细的开发规范请参考 [开发规范](./docs/guidelines/development.md)，包括：
- 代码风格规范（格式化、Clippy、命名约定）
- 错误处理规范
- 文档规范
- 提交规范
- 检查流程（pre-commit、review）

## 📚 文档

完整的文档索引请查看 [docs/README.md](./docs/README.md)。

主要参考文档：
- [架构设计](./docs/guidelines/architecture.md) - 项目整体架构设计
- [开发规范](./docs/guidelines/development.md) - 代码风格、错误处理、命名、模块组织、文档规范、提交规范、检查流程
- [测试规范](./docs/guidelines/testing.md) - 测试组织、编写、命令参考
- [迁移文档](./docs/migration/README.md) - 版本迁移指南

**API 文档**：运行 `cargo doc --open` 查看完整的 API 文档。


## 🏗️ 架构总览

v2 采用 Cargo workspace 多 crate 结构：

| Crate | 职责 |
|-------|------|
| **app** | CLI 入口（`bin/workflow.rs`、`bin/install.rs`）、`cli/` 参数与子命令、`commands/` 命令实现、`workflows/` 工作流编排 |
| **domain** | 领域模型与接口（config、git、jira、github、llm、pr、path、template 等） |
| **storage** | 存储与 Git 实现（实现 domain 中的仓储） |
| **services** | 应用服务（pull_request、completion、path、alias 等） |
| **toolkit** | 通用能力（http、logger、paths、template、util、rollback、shell、terminal 等） |
| **prompt** | 交互与输出（dialog、form、output、style） |
| **registry** | 依赖注入（供 app 解析服务） |

数据流向：**用户输入 → app (Cli) → commands/\* → domain / storage / services → 执行操作**

更完整的层次与模块说明见 [架构设计](./docs/guidelines/architecture.md)。

## 📝 贡献

请参考以下文档了解更多信息：
- [docs/README.md](./docs/README.md) - 完整文档索引
- [架构设计](./docs/guidelines/architecture.md) - 了解架构设计和核心模块详情

---
