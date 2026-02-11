# CI/CD 集成实施指南

> **目标**: 将测试覆盖率和性能测试集成到 CI/CD 流程中
> **优先级**: 🔴 P0 (高)
> **预计时间**: 1-2 天
> **依赖**: [测试覆盖率监控](./testing-01-coverage-monitoring.md), [系统化性能测试](./testing-02-performance-testing.md)

---

## 🎯 目标和范围

### 实施目标
1. ✅ 在 GitHub Actions 中添加覆盖率检查作业
2. ✅ 在 GitHub Actions 中添加性能基准作业
3. ✅ 配置 Codecov 集成
4. ✅ 创建预提交钩子 (pre-commit hook)
5. ✅ 更新 Makefile 添加 hooks 安装命令
6. ✅ 配置覆盖率徽章

### 产出物
```
workflow.rs/
├── .github/
│   └── workflows/
│       └── ci.yml                  # 更新 CI workflow
├── scripts/
│   └── git-hooks/
│       └── pre-commit              # 预提交钩子
├── make/
│   └── Makefile.tools.mk           # 更新工具命令
└── codecov.yml                     # Codecov 配置（可选）
```

---

## 📊 当前状态

### ✅ 已有基础
- ✅ `.github/workflows/ci.yml` 存在
- ✅ CI 中有基本的测试作业
- ✅ CI 中有代码质量检查（fmt, clippy）

### ❌ 缺失部分
- ❌ 没有覆盖率检查作业
- ❌ 没有性能基准作业
- ❌ 没有 Codecov 集成
- ❌ 没有预提交钩子
- ❌ 没有覆盖率徽章

---

## 📋 前置条件

### 系统要求
```bash
# 1. 确认覆盖率监控已实施
ls -la coverage.toml scripts/check_coverage.py

# 2. 确认性能测试已实施
ls -la benches/

# 3. 确认 GitHub Actions 可以访问
# 需要有仓库的 push 权限
```

### GitHub 配置
- 需要 GitHub 仓库的管理员权限
- 需要创建 GitHub Secrets（Codecov token）

---

## 🔨 详细实施步骤

### Step 1: 更新 GitHub Actions CI Workflow (45 分钟)

#### 1.1 备份现有 CI 配置

```bash
cp .github/workflows/ci.yml .github/workflows/ci.yml.backup
```

#### 1.2 添加覆盖率检查作业

在 `.github/workflows/ci.yml` 中添加覆盖率作业：

```yaml
# 在现有 jobs 后添加

  # 测试覆盖率检查
  coverage:
    name: 📊 Test Coverage
    runs-on: ubuntu-latest
    needs: check-skip-ci  # 如果你有 skip-ci 检查
    # if: needs.check-skip-ci.outputs.should_skip != 'true'  # 可选

    steps:
      - name: 📥 Checkout repository
        uses: actions/checkout@v4

      - name: 🔧 Setup Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: 💾 Cache Cargo dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-coverage-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-coverage-
            ${{ runner.os }}-cargo-

      - name: 📦 Install system dependencies
        run: |
          # 安装必要的系统依赖
          sudo apt-get update
          sudo apt-get install -y pkg-config libssl-dev
          # 如果有其他依赖，在这里添加

      - name: 📦 Install cargo-tarpaulin
        run: |
          # 使用缓存的 cargo-tarpaulin 或安装新的
          if ! command -v cargo-tarpaulin &> /dev/null; then
            cargo install cargo-tarpaulin
          fi

      - name: 📊 Generate coverage report
        run: |
          cargo tarpaulin \
            --skip-clean \
            --out Xml \
            --out Json \
            --out Lcov \
            --output-dir coverage \
            --exclude-files "src/bin/*" \
            --exclude-files "tests/*" \
            --exclude-files "benches/*" \
            --exclude-files "*/testing/*" \
            --exclude-files "*/mock/*" \
            --timeout 300 \
            --verbose

      - name: 📤 Upload coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: ./coverage/lcov.info
          fail_ci_if_error: true
          token: ${{ secrets.CODECOV_TOKEN }}
          flags: unittests
          name: codecov-umbrella

      - name: ✅ Check coverage threshold
        run: |
          python3 scripts/check_coverage.py coverage/tarpaulin-report.json 75

      - name: 📦 Upload coverage artifacts
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: coverage-report
          path: coverage/
          retention-days: 30
```

#### 1.3 添加性能基准作业（可选）

在 `.github/workflows/ci.yml` 中添加性能基准作业：

```yaml
  # 性能基准测试（仅在 main 分支或特定标签）
  performance:
    name: ⚡ Performance Benchmarks
    runs-on: ubuntu-latest
    # 只在 main 分支或 release 标签运行
    if: github.ref == 'refs/heads/main' || startsWith(github.ref, 'refs/tags/v')

    steps:
      - name: 📥 Checkout repository
        uses: actions/checkout@v4

      - name: 🔧 Setup Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: 💾 Cache Cargo dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-bench-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-bench-
            ${{ runner.os }}-cargo-

      - name: 📦 Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y pkg-config libssl-dev

      - name: 🔨 Build release binary
        run: cargo build --release

      - name: ⚡ Run benchmarks
        run: |
          # 运行所有基准测试，保存基线
          cargo bench --no-fail-fast -- --save-baseline ci-${{ github.sha }}

      - name: 📤 Archive benchmark results
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-results-${{ github.sha }}
          path: target/criterion/
          retention-days: 90

      - name: 📊 Comment PR with benchmark results (可选)
        if: github.event_name == 'pull_request'
        run: |
          # 这里可以添加脚本，将性能结果评论到 PR
          echo "性能基准测试完成，结果已上传为 artifact"
```

#### ✅ **验证 Step 1**
```bash
# 验证 YAML 语法
yamllint .github/workflows/ci.yml

# 或使用在线工具验证
# https://www.yamllint.com/

# 查看差异
diff .github/workflows/ci.yml.backup .github/workflows/ci.yml
```

---

### Step 2: 配置 Codecov 集成 (30 分钟)

#### 2.1 注册 Codecov 账户

1. 访问 [Codecov.io](https://codecov.io/)
2. 使用 GitHub 账户登录
3. 授权 Codecov 访问你的仓库

#### 2.2 获取 Codecov Token

1. 在 Codecov 中找到你的仓库
2. 进入 Settings > General
3. 复制 **Upload Token**

#### 2.3 添加 GitHub Secret

1. 进入 GitHub 仓库的 Settings
2. 点击 Secrets and variables > Actions
3. 点击 "New repository secret"
4. Name: `CODECOV_TOKEN`
5. Value: 粘贴上一步复制的 token
6. 点击 "Add secret"

#### 2.4 创建 `codecov.yml` 配置（可选）

在项目根目录创建 `codecov.yml`：

```yaml
# Codecov 配置
# 文档: https://docs.codecov.com/docs/codecov-yaml

# 覆盖率目标
coverage:
  status:
    project:
      default:
        target: 80%              # 整体目标 80%
        threshold: 5%            # 允许下降 5%
        if_ci_failed: error      # CI 失败时的行为

    patch:
      default:
        target: 75%              # 新增代码目标 75%
        threshold: 10%           # 允许下降 10%

# 评论设置
comment:
  layout: "header, diff, flags, components, footer"
  behavior: default
  require_changes: false
  require_base: false
  require_head: true

# 忽略文件
ignore:
  - "tests/**"
  - "benches/**"
  - "examples/**"
  - "**/testing/**"
  - "**/mock/**"
  - "target/**"

# 标记（用于区分不同类型的测试）
flags:
  unittests:
    paths:
      - src/
    carryforward: true
```

#### 2.5 添加覆盖率徽章到 README

在 `README.md` 顶部添加徽章：

```markdown
# Workflow

[![CI](https://github.com/YOUR_USERNAME/workflow.rs/actions/workflows/ci.yml/badge.svg)](https://github.com/YOUR_USERNAME/workflow.rs/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/YOUR_USERNAME/workflow.rs/branch/main/graph/badge.svg)](https://codecov.io/gh/YOUR_USERNAME/workflow.rs)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

<!-- 其他内容 -->
```

记得替换 `YOUR_USERNAME` 为实际的 GitHub 用户名或组织名。

#### ✅ **验证 Step 2**
```bash
# 验证 codecov.yml 语法
# 可以使用 Codecov 的在线验证工具
# https://codecov.io/validate

# 验证 GitHub Secret 已添加
# 在 GitHub 仓库页面检查 Settings > Secrets

# 提交更改并触发 CI
git add .github/workflows/ci.yml codecov.yml README.md
git commit -m "ci: add coverage reporting and Codecov integration"
git push

# 检查 CI 运行情况
# https://github.com/YOUR_USERNAME/workflow.rs/actions
```

---

### Step 3: 创建预提交钩子 (30 分钟)

#### 3.1 创建 hooks 目录

```bash
mkdir -p scripts/git-hooks
```

#### 3.2 创建 `scripts/git-hooks/pre-commit`

这个钩子会在每次提交前运行检查：

```bash
#!/bin/bash
#
# Git Pre-commit Hook
#
# 此钩子在 git commit 前运行，执行代码质量检查
#
# 安装方法:
#   make install-hooks
#
# 跳过方法:
#   git commit --no-verify
#

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查是否在 Git 仓库中
if ! git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    echo -e "${RED}❌ 错误: 不在 Git 仓库中${NC}"
    exit 1
fi

echo -e "${BLUE}🔍 Running pre-commit checks...${NC}"
echo ""

# 变量: 是否启用覆盖率检查（默认关闭，因为较慢）
CHECK_COVERAGE="${CHECK_COVERAGE:-0}"

# 获取暂存的文件
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(rs|toml)$' || true)

if [ -z "$STAGED_FILES" ]; then
    echo -e "${YELLOW}ℹ️  没有 Rust 文件被暂存，跳过检查${NC}"
    exit 0
fi

echo -e "${BLUE}检查 $(echo "$STAGED_FILES" | wc -l) 个文件...${NC}"
echo ""

# 步骤计数器
STEP=1
TOTAL_STEPS=4
if [ "$CHECK_COVERAGE" = "1" ]; then
    TOTAL_STEPS=5
fi

#------------------------------------------------------------------------------
# 1. 代码格式检查
#------------------------------------------------------------------------------
echo -e "${BLUE}[$STEP/$TOTAL_STEPS] ✨ Checking code formatting...${NC}"
if ! cargo fmt --check --quiet; then
    echo -e "${RED}❌ 代码格式检查失败${NC}"
    echo -e "${YELLOW}💡 修复方法: cargo fmt${NC}"
    exit 1
fi
echo -e "${GREEN}✅ 代码格式检查通过${NC}"
echo ""
STEP=$((STEP + 1))

#------------------------------------------------------------------------------
# 2. Clippy 检查
#------------------------------------------------------------------------------
echo -e "${BLUE}[$STEP/$TOTAL_STEPS] 🔍 Running Clippy...${NC}"
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo -e "${RED}❌ Clippy 检查失败${NC}"
    echo -e "${YELLOW}💡 请修复上述警告${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Clippy 检查通过${NC}"
echo ""
STEP=$((STEP + 1))

#------------------------------------------------------------------------------
# 3. 运行测试
#------------------------------------------------------------------------------
echo -e "${BLUE}[$STEP/$TOTAL_STEPS] 🧪 Running tests...${NC}"
if ! cargo test --quiet; then
    echo -e "${RED}❌ 测试失败${NC}"
    echo -e "${YELLOW}💡 请修复失败的测试${NC}"
    exit 1
fi
echo -e "${GREEN}✅ 测试通过${NC}"
echo ""
STEP=$((STEP + 1))

#------------------------------------------------------------------------------
# 4. 构建检查
#------------------------------------------------------------------------------
echo -e "${BLUE}[$STEP/$TOTAL_STEPS] 🔨 Building project...${NC}"
if ! cargo build --quiet; then
    echo -e "${RED}❌ 构建失败${NC}"
    exit 1
fi
echo -e "${GREEN}✅ 构建成功${NC}"
echo ""
STEP=$((STEP + 1))

#------------------------------------------------------------------------------
# 5. 覆盖率检查（可选）
#------------------------------------------------------------------------------
if [ "$CHECK_COVERAGE" = "1" ]; then
    echo -e "${BLUE}[$STEP/$TOTAL_STEPS] 📊 Checking coverage...${NC}"
    if ! make coverage-check > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  覆盖率检查未通过（不阻塞提交）${NC}"
        echo -e "${YELLOW}💡 运行 'make coverage' 查看详细报告${NC}"
        # 不退出，允许提交
    else
        echo -e "${GREEN}✅ 覆盖率检查通过${NC}"
    fi
    echo ""
fi

#------------------------------------------------------------------------------
# 完成
#------------------------------------------------------------------------------
echo -e "${GREEN}✅ All pre-commit checks passed!${NC}"
echo ""
echo -e "${BLUE}💡 提示:${NC}"
echo -e "  - 跳过检查: ${YELLOW}git commit --no-verify${NC}"
echo -e "  - 启用覆盖率检查: ${YELLOW}CHECK_COVERAGE=1 git commit${NC}"
echo ""

exit 0
```

#### 3.3 设置钩子权限

```bash
chmod +x scripts/git-hooks/pre-commit
```

#### 3.4 创建简化版钩子（可选）

如果完整版钩子太慢，可以创建一个简化版 `scripts/git-hooks/pre-commit-fast`:

```bash
#!/bin/bash
#
# Git Pre-commit Hook (Fast Version)
#
# 只运行快速检查
#

set -e

echo "🔍 Running fast pre-commit checks..."

# 只检查格式和 Clippy
cargo fmt --check
cargo clippy -- -D warnings

echo "✅ Fast checks passed!"
```

#### ✅ **验证 Step 3**
```bash
# 验证钩子文件存在
ls -la scripts/git-hooks/pre-commit

# 验证权限
test -x scripts/git-hooks/pre-commit && echo "✅ 权限正确" || echo "❌ 需要执行权限"

# 测试钩子（不实际提交）
bash scripts/git-hooks/pre-commit
```

---

### Step 4: 更新 Makefile 添加 hooks 安装 (15 分钟)

#### 4.1 备份 Makefile

```bash
cp make/Makefile.tools.mk make/Makefile.tools.mk.backup
```

#### 4.2 更新 `make/Makefile.tools.mk`

在文件末尾添加 Git hooks 相关命令：

```makefile
#------------------------------------------------------------------------------
# Git Hooks 管理
#------------------------------------------------------------------------------

# 安装 Git hooks
install-hooks:
	@echo "📦 Installing Git hooks..."
	@if [ ! -d .git ]; then \
		echo "❌ 错误: 不在 Git 仓库中"; \
		exit 1; \
	fi
	@# 安装 pre-commit hook
	@if [ -f scripts/git-hooks/pre-commit ]; then \
		chmod +x scripts/git-hooks/pre-commit; \
		ln -sf ../../scripts/git-hooks/pre-commit .git/hooks/pre-commit; \
		echo "✅ Pre-commit hook 已安装"; \
	else \
		echo "❌ 错误: scripts/git-hooks/pre-commit 不存在"; \
		exit 1; \
	fi
	@echo ""
	@echo "💡 提示:"
	@echo "  - 跳过钩子: git commit --no-verify"
	@echo "  - 卸载钩子: make uninstall-hooks"
	@echo "  - 启用覆盖率检查: CHECK_COVERAGE=1 git commit"
	@echo ""

# 安装快速版钩子
install-hooks-fast:
	@echo "📦 Installing fast Git hooks..."
	@if [ ! -d .git ]; then \
		echo "❌ 错误: 不在 Git 仓库中"; \
		exit 1; \
	fi
	@if [ -f scripts/git-hooks/pre-commit-fast ]; then \
		chmod +x scripts/git-hooks/pre-commit-fast; \
		ln -sf ../../scripts/git-hooks/pre-commit-fast .git/hooks/pre-commit; \
		echo "✅ Fast pre-commit hook 已安装"; \
	else \
		echo "⚠️  scripts/git-hooks/pre-commit-fast 不存在，使用标准版本"; \
		$(MAKE) install-hooks; \
	fi
	@echo ""

# 卸载 Git hooks
uninstall-hooks:
	@echo "🗑️  Uninstalling Git hooks..."
	@if [ -L .git/hooks/pre-commit ]; then \
		rm .git/hooks/pre-commit; \
		echo "✅ Pre-commit hook 已卸载"; \
	else \
		echo "ℹ️  Pre-commit hook 未安装"; \
	fi
	@echo ""

# 测试 Git hooks
test-hooks:
	@echo "🧪 Testing Git hooks..."
	@if [ -f .git/hooks/pre-commit ]; then \
		bash .git/hooks/pre-commit; \
	else \
		echo "❌ 错误: Pre-commit hook 未安装"; \
		echo "运行: make install-hooks"; \
		exit 1; \
	fi

# 查看 hooks 状态
hooks-status:
	@echo "📊 Git Hooks 状态:"
	@echo ""
	@if [ -L .git/hooks/pre-commit ]; then \
		echo "✅ Pre-commit hook: 已安装"; \
		echo "   位置: $$(readlink .git/hooks/pre-commit)"; \
	else \
		echo "❌ Pre-commit hook: 未安装"; \
		echo "   运行: make install-hooks"; \
	fi
	@echo ""

# 更新 setup 命令（如果存在）
setup: install-tools install-hooks
	@echo "✅ 开发环境设置完成"
	@echo ""
	@echo "📚 下一步:"
	@echo "  1. 运行测试: make test"
	@echo "  2. 生成覆盖率: make coverage"
	@echo "  3. 运行基准测试: make bench"
	@echo ""

.PHONY: install-hooks install-hooks-fast uninstall-hooks test-hooks hooks-status
```

#### ✅ **验证 Step 4**
```bash
# 验证 Makefile 语法
make -n install-hooks

# 测试 hooks 状态命令
make hooks-status

# 测试安装命令（实际安装）
make install-hooks

# 验证安装成功
ls -la .git/hooks/pre-commit
```

---

### Step 5: 测试完整 CI/CD 流程 (30 分钟)

#### 5.1 本地测试

```bash
# 1. 测试预提交钩子
make test-hooks

# 2. 测试覆盖率生成
make coverage-check

# 3. 测试性能基准
make bench

# 4. 提交测试
git add .
git commit -m "test: CI/CD integration"
# 应该触发预提交钩子
```

#### 5.2 CI 测试

```bash
# 1. 推送到远程
git push origin feature-branch

# 2. 创建 Pull Request
# 在 GitHub 网页上创建 PR

# 3. 检查 CI 运行
# - 进入 GitHub Actions 页面
# - 查看 coverage 作业是否运行
# - 查看 Codecov 报告是否上传成功

# 4. 检查 Codecov
# - 访问 Codecov.io 查看覆盖率报告
# - 检查 PR 上的 Codecov 评论
```

#### 5.3 验证覆盖率徽章

```bash
# 1. 查看 README 中的徽章是否显示
# 2. 点击徽章链接，应该跳转到 Codecov 页面
```

#### ✅ **验证 Step 5**
```bash
# 验证清单
# [ ] 预提交钩子正常工作
# [ ] CI 中的 coverage 作业成功运行
# [ ] Codecov 报告成功上传
# [ ] Codecov 评论出现在 PR 中
# [ ] 覆盖率徽章正常显示
# [ ] 性能基准作业运行（如果启用）
```

---

## ✅ 验证和测试

### 完整验证流程

```bash
# 1. 本地 hooks 验证
make install-hooks
make test-hooks

# 2. 创建测试提交
echo "# Test" >> test.md
git add test.md
git commit -m "test: pre-commit hook"

# 3. 检查 CI 配置
yamllint .github/workflows/ci.yml

# 4. 推送并创建 PR
git push origin feature-branch

# 5. 在 GitHub 上查看 CI 运行结果

# 6. 检查 Codecov 报告

# 7. 清理测试
git reset HEAD~1
rm test.md
```

### 预期结果

✅ **成功标准**:
- [ ] 预提交钩子可以正常运行
- [ ] CI 中的 coverage 作业成功运行
- [ ] 覆盖率报告上传到 Codecov
- [ ] 覆盖率徽章显示正确的百分比
- [ ] 性能基准测试正常运行（如果启用）
- [ ] 所有检查通过后 PR 可以合并

---

## 📝 最佳实践

### 1. 预提交钩子使用建议

```bash
# 日常开发：使用标准钩子
make install-hooks

# 快速迭代：临时跳过
git commit --no-verify -m "wip: work in progress"

# 重要提交：启用覆盖率检查
CHECK_COVERAGE=1 git commit -m "feat: add new feature"

# 如果钩子太慢：使用快速版
make install-hooks-fast
```

### 2. CI 触发策略

```yaml
# 建议的触发策略

# 所有 PR 和 push 到 main：运行测试和覆盖率
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

# 只在 main 分支或 release 标签：运行性能基准
if: github.ref == 'refs/heads/main' || startsWith(github.ref, 'refs/tags/v')
```

### 3. Codecov 配置建议

```yaml
# 推荐的 Codecov 设置

coverage:
  status:
    project:
      default:
        target: 80%       # 不要设置太高，80% 是平衡点
        threshold: 5%     # 允许适度波动
    patch:
      default:
        target: 75%       # 新代码可以略低
```

### 4. CI 性能优化

```yaml
# 优化 CI 运行时间

# 1. 使用缓存
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/bin/
      ~/.cargo/registry/
      target/
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

# 2. 并行运行作业
# 测试、覆盖率、基准测试可以并行

# 3. 按需运行
# 性能基准只在 main 分支运行
```

---

## ⚠️ 注意事项

### 1. 预提交钩子性能
- **完整检查需要时间**: 格式 + Clippy + 测试可能需要 1-2 分钟
- **解决方案**:
  - 使用快速版钩子 (`install-hooks-fast`)
  - 临时跳过 (`--no-verify`)
  - 在后台运行测试

### 2. CI 配额限制
- **GitHub Actions 免费额度**: 公开仓库无限，私有仓库每月 2000 分钟
- **Codecov 免费额度**: 公开仓库无限，私有仓库有限制
- **解决方案**:
  - 只在关键分支运行性能基准
  - 使用 `if` 条件控制作业执行

### 3. Codecov Token 安全
- **不要提交 token 到代码**: 使用 GitHub Secrets
- **定期轮换 token**: 如果泄露，立即重新生成
- **最小权限**: 只给 Codecov 必要的权限

### 4. 测试失败处理
```bash
# 如果预提交钩子失败

# 1. 查看详细错误
git commit -m "test" --verbose

# 2. 修复问题后重新提交
# 不要使用 --no-verify 跳过

# 3. 如果是钩子本身的问题
make uninstall-hooks
# 修复钩子脚本
make install-hooks
```

---

## 🔗 相关资源

### 内部文档
- [测试覆盖率监控实施指南](./testing-01-coverage-monitoring.md)
- [系统化性能测试实施指南](./testing-02-performance-testing.md)
- [测试文档编写指南](./testing-04-documentation.md)

### 外部资源
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Codecov 文档](https://docs.codecov.com/)
- [Git Hooks 文档](https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks)
- [Cargo-tarpaulin 文档](https://github.com/xd009642/tarpaulin)

---

## 📋 检查清单

实施完成后，确认以下项目：

### GitHub Actions
- [ ] `.github/workflows/ci.yml` 添加了 coverage 作业
- [ ] `.github/workflows/ci.yml` 添加了 performance 作业（可选）
- [ ] YAML 语法正确，可以被 GitHub 解析
- [ ] CI 作业可以成功运行

### Codecov
- [ ] Codecov 账户已注册
- [ ] `CODECOV_TOKEN` 已添加到 GitHub Secrets
- [ ] `codecov.yml` 配置文件已创建
- [ ] 覆盖率报告可以成功上传
- [ ] Codecov 徽章已添加到 README
- [ ] Codecov 徽章显示正确

### Git Hooks
- [ ] `scripts/git-hooks/pre-commit` 已创建
- [ ] 钩子脚本有执行权限
- [ ] `make install-hooks` 命令可用
- [ ] `make uninstall-hooks` 命令可用
- [ ] `make test-hooks` 命令可用
- [ ] 预提交钩子正常工作
- [ ] 可以使用 `--no-verify` 跳过钩子

### Makefile
- [ ] `make/Makefile.tools.mk` 添加了 hooks 命令
- [ ] `make setup` 包含了 `install-hooks`
- [ ] 所有 make 命令都可以正常运行

### 集成测试
- [ ] 创建测试 PR，所有 CI 检查通过
- [ ] Codecov 报告出现在 PR 中
- [ ] 覆盖率徽章更新
- [ ] 性能基准测试运行（如果启用）

---

## 🎯 成功指标

### 量化指标
- [ ] CI 通过率 ≥ 95%
- [ ] 覆盖率报告上传成功率 100%
- [ ] 预提交钩子使用率 ≥ 80%
- [ ] CI 平均运行时间 < 10 分钟

### 质量指标
- [ ] 团队成员熟悉 CI/CD 流程
- [ ] 覆盖率趋势可追踪
- [ ] 性能回归可及时发现
- [ ] 代码质量问题在提交前被捕获

---

**文档版本**: 1.0
**创建日期**: 2025-02-11
**最后更新**: 2025-02-11
**下一步**: [测试文档编写指南](./testing-04-documentation.md)
