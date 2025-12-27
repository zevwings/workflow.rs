# Python Dev 工具使用检查清单

## ✅ 使用前检查

### 1. Python 版本检查

```bash
python3 --version
# 应该显示 Python 3.13.x 或更高版本
```

如果版本不符合要求：
- **macOS**: `brew install python@3.13`
- **Linux**: 使用系统包管理器或 pyenv

### 2. 验证工具可用性

```bash
# 测试主入口
python3 scripts/dev/dev.py --help

# 测试 CI 命令
python3 scripts/dev/dev.py ci check-skip --branch "test"

# 测试 Checksum 命令
python3 scripts/dev/dev.py checksum calculate --file "Cargo.toml"
```

### 3. Git 配置检查（Git 操作需要）

```bash
# 检查 Git 配置
git config user.name
git config user.email

# 如果未设置，运行：
git config user.name "Your Name"
git config user.email "your.email@example.com"
```

### 4. GitHub Token 设置（PR 操作需要）

```bash
# 检查环境变量
echo $GITHUB_TOKEN
# 或
echo $GITHUB_PAT

# 如果未设置，运行：
export GITHUB_TOKEN="your_token_here"
# 或添加到 ~/.zshrc / ~/.bashrc
```

## 🚀 开始使用

### 本地使用

1. **CI 检查**
   ```bash
   python3 scripts/dev/dev.py ci check-skip --branch "bump-version-1.7.0" --ci
   ```

2. **版本生成**
   ```bash
   python3 scripts/dev/dev.py version generate --master --update
   ```

3. **创建 Tag**
   ```bash
   python3 scripts/dev/dev.py tag create --tag "v1.7.0"
   ```

### CI/CD 集成

1. **更新 GitHub Actions workflow**

   在 `.github/workflows/ci.yml` 中：

   ```yaml
   - name: 🐍 Setup Python 3.13
     uses: actions/setup-python@v5
     with:
       python-version: '3.13'

   - name: 🔍 Check if version bump branch
     run: |
       python3 scripts/dev/dev.py ci check-skip \
         --branch "${{ github.head_ref || github.ref_name }}" \
         --pr-creator "${{ github.event.pull_request.user.login }}" \
         --expected-user "${{ env.WORKFLOW_USER_NAME }}" \
         --ci
   ```

2. **移除 dev 二进制构建步骤**（可选）

   如果不再需要 Rust dev 工具，可以移除：
   - `build-dev-tool` job
   - dev binary artifact 下载步骤

## 📋 命令速查表

| 命令 | 用途 | 示例 |
|------|------|------|
| `ci check-skip` | 检查是否跳过 CI | `--branch "test" --ci` |
| `ci verify` | 验证 CI 状态 | `--jobs "check-lint,tests"` |
| `checksum calculate` | 计算文件哈希 | `--file "file.txt"` |
| `version generate` | 生成版本号 | `--master --update --ci` |
| `tag create` | 创建 Git 标签 | `--tag "v1.7.0" --ci` |
| `tag cleanup` | 清理 Alpha 标签 | `--merge-commit "abc" --version "1.7.0"` |
| `pr create` | 创建 PR | `--version "1.7.0" --ci` |
| `pr merge` | 合并 PR | `--pr-number 123 --ci` |
| `homebrew update` | 更新 Formula | `--version "1.7.0" --tag "v1.7.0"` |

## 🔍 故障排除

### 问题：Python 版本错误

```
❌ Error: Python 3.13+ required, but found 3.9.0
```

**解决**：
- 安装 Python 3.13+
- 或使用 `python3.13` 命令（如果已安装）

### 问题：GitHub API 错误

```
GitHub API error (401): Bad credentials
```

**解决**：
- 设置 `GITHUB_TOKEN` 或 `GITHUB_PAT` 环境变量
- 确保 token 有足够的权限

### 问题：Git 操作失败

```
Git command failed: git checkout -b test
```

**解决**：
- 检查 Git 配置（user.name, user.email）
- 确保在 Git 仓库中运行
- 检查 Git 权限

## 📚 相关文档

- [快速开始指南](./QUICK_START.md)
- [完整使用文档](./PYTHON_DEV_TOOL.md)
- [CI 集成指南](./CI_USAGE.md)
- [迁移状态](./MIGRATION_STATUS.md)

## ✨ 下一步

1. ✅ 完成检查清单
2. ✅ 测试本地命令
3. ✅ 更新 CI workflow（如需要）
4. ✅ 开始使用！

