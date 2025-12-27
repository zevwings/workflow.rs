# Python Dev 工具快速开始指南

## 🚀 快速开始

### 前置要求

- **Python 3.13+** （必需）
- Git（用于 Git 操作）
- GitHub Token（用于 PR 和 GitHub API 操作，可选）

### 验证安装

```bash
# 检查 Python 版本
python3 --version  # 应该 >= 3.13

# 测试 dev 工具
python3 scripts/dev/dev.py --help
```

## 📝 基本使用

### 统一入口（推荐）

```bash
# 查看所有可用命令
python3 scripts/dev/dev.py --help

# CI 命令
python3 scripts/dev/dev.py ci check-skip --branch "bump-version-1.7.0" --ci
python3 scripts/dev/dev.py ci verify --jobs "check-lint,tests"

# Checksum 命令
python3 scripts/dev/dev.py checksum calculate --file "Cargo.toml" --output "hash.txt"

# Version 命令
python3 scripts/dev/dev.py version generate --master --update --ci

# Tag 命令
python3 scripts/dev/dev.py tag create --tag "v1.7.0" --ci
python3 scripts/dev/dev.py tag cleanup --merge-commit "abc123" --version "1.7.0" --ci

# PR 命令
python3 scripts/dev/dev.py pr create --version "1.7.0" --ci
python3 scripts/dev/dev.py pr merge --pr-number 123 --ci

# Homebrew 命令
python3 scripts/dev/dev.py homebrew update \
    --version "1.7.0" \
    --tag "v1.7.0" \
    --sha256 "abc123..." \
    --commit \
    --push
```

### 直接调用（也支持）

```bash
# 直接运行单个命令脚本
python3 scripts/dev/ci/check_skip.py --branch "test" --ci
python3 scripts/dev/checksum/calculate.py --file "Cargo.toml"
python3 scripts/dev/version/generate.py --master --update
```

## 🔧 CI/CD 集成

### GitHub Actions 设置

#### 1. 设置 Python 3.13

在 workflow 文件开头添加：

```yaml
- name: 🐍 Setup Python 3.13
  uses: actions/setup-python@v5
  with:
    python-version: '3.13'
```

#### 2. 替换 dev 二进制调用

**之前（Rust 二进制）：**
```yaml
- name: 🔍 Check if version bump branch
  run: |
    ./target/release/dev ci check-skip \
      --branch "${{ github.head_ref || github.ref_name }}" \
      --pr-creator "${{ github.event.pull_request.user.login }}" \
      --expected-user "${{ env.WORKFLOW_USER_NAME }}" \
      --ci
```

**现在（Python 脚本）：**
```yaml
- name: 🔍 Check if version bump branch
  run: |
    python3 scripts/dev/dev.py ci check-skip \
      --branch "${{ github.head_ref || github.ref_name }}" \
      --pr-creator "${{ github.event.pull_request.user.login }}" \
      --expected-user "${{ env.WORKFLOW_USER_NAME }}" \
      --ci
```

#### 3. 设置 GitHub Token（PR 操作需要）

```yaml
env:
  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  # 或使用自定义 token
  # GITHUB_TOKEN: ${{ secrets.WORKFLOW_PAT }}
```

## 🔑 环境变量

### 必需的环境变量

- **`GITHUB_TOKEN`** 或 **`GITHUB_PAT`**: GitHub API 操作（PR create/merge）
- **`GITHUB_OUTPUT`**: CI 模式输出（由 GitHub Actions 自动设置）

### 可选的环境变量

- **`GITHUB_REPOSITORY`**: 仓库名称（格式：owner/repo），如果不设置会从 Git remote 自动检测
- **`GITHUB_REPOSITORY_OWNER`**: 仓库所有者

## 📋 常见使用场景

### 场景 1: 检查 CI 是否应该跳过

```bash
python3 scripts/dev/dev.py ci check-skip \
  --branch "bump-version-1.7.0" \
  --pr-creator "github-actions[bot]" \
  --expected-user "github-actions[bot]" \
  --ci
```

### 场景 2: 生成版本号

```bash
# Master 分支
python3 scripts/dev/dev.py version generate --master --update --ci

# 非 Master 分支（预发布版本）
python3 scripts/dev/dev.py version generate --update
```

### 场景 3: 创建版本更新 PR

```bash
python3 scripts/dev/dev.py pr create \
  --version "1.7.0" \
  --branch "bump-version-1.7.0" \
  --base "master" \
  --ci
```

### 场景 4: 合并 PR

```bash
python3 scripts/dev/dev.py pr merge \
  --pr-number 123 \
  --max-wait 300 \
  --ci
```

### 场景 5: 创建并推送 Tag

```bash
python3 scripts/dev/dev.py tag create \
  --tag "v1.7.0" \
  --commit "abc123" \
  --ci
```

## ⚠️ 注意事项

1. **Python 版本**: 必须使用 Python 3.13+，脚本会自动检查
2. **GitHub Token**: PR 操作需要有效的 GitHub token
3. **Git 配置**: Git 操作需要正确的 Git 配置（user.name, user.email）
4. **CI 模式**: 使用 `--ci` 标志时，输出会写入 `GITHUB_OUTPUT`

## 🐛 故障排除

### 问题 1: Python 版本不匹配

```bash
# 错误信息
❌ Error: Python 3.13+ required, but found 3.9.0

# 解决方案
# macOS: 使用 Homebrew 安装 Python 3.13
brew install python@3.13

# Linux: 使用系统包管理器或 pyenv
```

### 问题 2: GitHub API 错误

```bash
# 错误信息
GitHub API error (401): Bad credentials

# 解决方案
export GITHUB_TOKEN="your_token_here"
# 或
export GITHUB_PAT="your_token_here"
```

### 问题 3: Git 操作失败

```bash
# 确保 Git 已配置
git config user.name "Your Name"
git config user.email "your.email@example.com"
```

## 📚 更多文档

- [完整使用文档](./PYTHON_DEV_TOOL.md)
- [CI 集成指南](./CI_USAGE.md)
- [架构设计](./ARCHITECTURE.md)
- [迁移状态](./MIGRATION_STATUS.md)

