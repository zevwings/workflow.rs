# Python Dev 工具

Python 版本的开发工具集合，用于替代 Rust 二进制版本，实现零编译时间、快速启动。

## 🚀 快速开始

### 前置要求

- **Python 3.13+** （必需）
- Git（用于 Git 操作）
- GitHub Token（用于 PR 和 GitHub API 操作，可选）

### 基本使用

```bash
# 查看所有可用命令
python3 scripts/dev/py/dev.py --help

# 统一入口调用（推荐）
python3 scripts/dev/py/dev.py ci check-skip --branch "test" --ci

# 直接调用单个命令（也支持）
python3 scripts/dev/py/ci/check_skip.py --branch "test" --ci
```

### 使用前检查

```bash
# 1. 检查 Python 版本（应 >= 3.13）
python3 --version

# 2. 验证工具可用性
python3 scripts/dev/py/dev.py --help

# 3. Git 配置（Git 操作需要）
git config user.name "Your Name"
git config user.email "your.email@example.com"

# 4. GitHub Token（PR 操作需要）
export GITHUB_TOKEN="your_token_here"
```

## 📚 命令列表

### CI 命令

| 命令 | 说明 |
|------|------|
| `ci check-skip` | 检查 CI 是否应该跳过（版本更新分支检查） |
| `ci verify` | 验证 CI job 状态 |

### Checksum 命令

| 命令 | 说明 |
|------|------|
| `checksum calculate` | 计算文件哈希值 |

### Version 命令

| 命令 | 说明 |
|------|------|
| `version generate` | 生成版本号 |

### Tag 命令

| 命令 | 说明 |
|------|------|
| `tag create` | 创建 Git 标签 |
| `tag cleanup` | 清理 Alpha 标签 |

### PR 命令

| 命令 | 说明 |
|------|------|
| `pr create` | 创建 Pull Request |
| `pr merge` | 合并 Pull Request |

### Homebrew 命令

| 命令 | 说明 |
|------|------|
| `homebrew update` | 更新 Homebrew Formula |

### Tests 命令

| 命令 | 说明 |
|------|------|
| `tests check` | 检查测试覆盖率 |
| `tests metrics` | 收集测试指标 |
| `tests report` | 生成测试报告 |
| `tests trends` | 分析测试趋势 |

### Performance 命令

| 命令 | 说明 |
|------|------|
| `performance analyze` | 分析性能数据 |

### Docs 命令

| 命令 | 说明 |
|------|------|
| `docs check integrity` | 检查文档完整性 |
| `docs check links` | 检查文档链接 |
| `docs report generate` | 生成文档报告 |

## 💡 使用示例

### CI Check Skip

```bash
# 基本用法（非 CI 模式）
python3 scripts/dev/py/dev.py ci check-skip --branch "feature/testing"

# CI 模式（输出到 GITHUB_OUTPUT）
python3 scripts/dev/py/dev.py ci check-skip \
    --branch "bump-version-1.6.10" \
    --pr-creator "workflow-bot" \
    --expected-user "workflow-bot" \
    --ci
```

### Version Generate

```bash
# Master 分支
python3 scripts/dev/py/dev.py version generate --master --update --ci

# 非 Master 分支（预发布版本）
python3 scripts/dev/py/dev.py version generate --update
```

### PR Create

```bash
python3 scripts/dev/py/dev.py pr create \
  --version "1.7.0" \
  --branch "bump-version-1.7.0" \
  --base "master" \
  --ci
```

### Tag Create

```bash
python3 scripts/dev/py/dev.py tag create \
  --tag "v1.7.0" \
  --commit "abc123" \
  --ci
```

### Homebrew Update

```bash
python3 scripts/dev/py/dev.py homebrew update \
    --version "1.7.0" \
    --tag "v1.7.0" \
    --sha256 "abc123..." \
    --commit \
    --push
```

## 🔧 CI/CD 集成

### GitHub Actions 设置

#### 1. 设置 Python 3.13

```yaml
- name: 🐍 Setup Python 3.13
  uses: actions/setup-python@v5
  with:
    python-version: '3.13'
```

#### 2. 替换 dev 二进制调用

**之前（Rust 二进制）：**
```yaml
- name: 🔨 Build dev binary
  run: cargo build --bin dev --release

- name: 🔍 Check if version bump branch
  run: ./target/release/dev ci check-skip ...
```

**现在（Python 脚本）：**
```yaml
- name: 🐍 Setup Python 3.13
  uses: actions/setup-python@v5
  with:
    python-version: '3.13'

- name: 🔍 Check if version bump branch
  run: |
    python3 scripts/dev/py/dev.py ci check-skip \
      --branch "${{ github.head_ref || github.ref_name }}" \
      --pr-creator "${{ github.event.pull_request.user.login }}" \
      --expected-user "${{ env.WORKFLOW_USER_NAME }}" \
      --ci
```

#### 3. 设置 GitHub Token（PR 操作需要）

```yaml
env:
  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### 优势

- ✅ **无需构建**: 节省约 3 分钟构建时间
- ✅ **无需缓存**: 不需要缓存二进制文件
- ✅ **快速启动**: Python 脚本直接运行
- ✅ **版本检查**: 脚本会自动检查 Python 版本

### 与构建依赖的关系

#### 仅使用 dev 工具

如果只是运行 dev 工具（不构建 Rust 项目），**不需要** `install-basic.sh`：

```yaml
- name: 🐍 Setup Python 3.13
  uses: actions/setup-python@v5
  with:
    python-version: '3.13'

- name: 🔍 Check CI skip
  run: python3 scripts/dev/py/dev.py ci check-skip --ci
```

#### 需要构建 Rust 项目

如果需要构建 Rust 项目（需要 xcb-proto），有两种方案：

**方案 A: 使用系统 Python（当前方案）**
```yaml
- name: 📦 Install system dependencies
  run: bash scripts/dev/shell/dependencies/install-basic.sh
```

**方案 B: 使用 Python 3.13 + pip（推荐）**
```yaml
- name: 🐍 Setup Python 3.13
  uses: actions/setup-python@v5
  with:
    python-version: '3.13'

- name: 📦 Install xcbgen
  run: pip install xcbgen

- name: 📦 Install XCB libraries (without python3)
  run: |
    sudo apt-get update
    sudo apt-get install -y libxcb1-dev ...
```

详见 [shell/dependencies/ANALYSIS.md](../shell/dependencies/ANALYSIS.md)

## 🔑 环境变量

### 必需的环境变量

- **`GITHUB_TOKEN`** 或 **`GITHUB_PAT`**: GitHub API 操作（PR create/merge）
- **`GITHUB_OUTPUT`**: CI 模式输出（由 GitHub Actions 自动设置）

### 可选的环境变量

- **`GITHUB_REPOSITORY`**: 仓库名称（格式：owner/repo），如果不设置会从 Git remote 自动检测
- **`GITHUB_REPOSITORY_OWNER`**: 仓库所有者
- **`GITHUB_HEAD_REF`**: PR 分支名
- **`GITHUB_REF_NAME`**: 当前分支名
- **`GITHUB_EVENT_NAME`**: GitHub 事件类型
- **`GITHUB_PR_CREATOR`**: PR 创建者
- **`WORKFLOW_USER_NAME`**: 期望的用户名

## ⚠️ 注意事项

1. **Python 版本**: 必须使用 Python 3.13+，脚本会自动检查
2. **GitHub Token**: PR 操作需要有效的 GitHub token
3. **Git 配置**: Git 操作需要正确的 Git 配置（user.name, user.email）
4. **CI 模式**: 使用 `--ci` 标志时，输出会写入 `GITHUB_OUTPUT`

## 🐛 故障排除

### Python 版本不匹配

```bash
# 错误信息
❌ Error: Python 3.13+ required, but found 3.9.0

# 解决方案
# macOS: 使用 Homebrew 安装 Python 3.13
brew install python@3.13

# Linux: 使用系统包管理器或 pyenv
```

### GitHub API 错误

```bash
# 错误信息
GitHub API error (401): Bad credentials

# 解决方案
export GITHUB_TOKEN="your_token_here"
# 或
export GITHUB_PAT="your_token_here"
```

### Git 操作失败

```bash
# 确保 Git 已配置
git config user.name "Your Name"
git config user.email "your.email@example.com"
```

## ✨ 特性

- ✅ **零依赖**: 完全使用 Python 标准库
- ✅ **快速启动**: 无需编译，直接运行
- ✅ **Python 3.13+**: 使用最新 Python 特性
- ✅ **双重调用**: 支持统一入口和直接运行
- ✅ **颜色输出**: 支持 ANSI 颜色码（GitHub Actions 自动支持）
- ✅ **错误处理**: 完善的错误处理和退出码
- ✅ **版本检查**: 自动检查 Python 版本（要求 3.13+）

### 与 Rust 版本的对比

| 特性 | Rust 版本 | Python 版本 |
|------|----------|------------|
| 启动时间 | ~3 分钟（编译） | 0 秒（直接运行） |
| 依赖 | Rust 工具链 | Python 3.13+ |
| 维护性 | 需要编译 | 直接修改 |
| 性能 | 编译后快速 | 运行时快速 |

## 🔧 开发指南

### 添加新命令

1. 在对应的模块目录创建新的 Python 文件（如 `new_module/command.py`）
2. 在 `dev.py` 中添加命令解析和路由
3. 实现命令处理函数

### 日志使用

```python
from utils.logger import log_info, log_error, log_success, log_warning

log_info("信息消息")
log_success("成功消息")
log_warning("警告消息")
log_error("错误消息")
```

### Git 操作

```python
from utils.git import run_git_command, get_current_branch

result = run_git_command(['status'], check=True)
branch = get_current_branch()
```

### GitHub API

```python
from utils.github import GitHubClient

client = GitHubClient()
pr = client.get_pull_request(owner='owner', repo='repo', pr_number=123)
```
