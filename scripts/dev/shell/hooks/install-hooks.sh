#!/bin/bash
# Git Hooks 安装脚本（开发工具）
#
# 安装 Git pre-commit hook 到项目的 .git/hooks/ 目录
#
# 使用方法:
#   ./scripts/dev/shell/hooks/install-hooks.sh
#   或
#   make install-hooks

set -euo pipefail

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1" >&2
}

# 获取脚本所在目录（项目根目录）
# 脚本位于 scripts/dev/shell/hooks/，需要向上 4 级到达项目根目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../../" && pwd)"

# 检查是否在 Git 仓库中
if [ ! -d "$PROJECT_ROOT/.git" ]; then
    log_error "错误: 当前目录不是 Git 仓库"
    exit 1
fi

# Git hooks 目录
GIT_HOOKS_DIR="$PROJECT_ROOT/.git/hooks"
PRE_COMMIT_HOOK="$GIT_HOOKS_DIR/pre-commit"

log_info "安装 Git pre-commit hook..."
echo ""

# 检查是否在 Git 仓库中
if [ ! -d "$PROJECT_ROOT/.git" ]; then
    log_error "错误: 当前目录不是 Git 仓库"
    exit 1
fi

# 确保 hooks 目录存在
mkdir -p "$GIT_HOOKS_DIR"

# 检查 pre-commit hook 是否已存在
if [ -f "$PRE_COMMIT_HOOK" ]; then
    # 检查是否是我们的 hook（通过检查文件内容中的标识）
    if grep -q "Git pre-commit hook for Workflow" "$PRE_COMMIT_HOOK" 2>/dev/null; then
        log_info "检测到已存在的 Workflow pre-commit hook，将更新..."
    else
        # 备份现有的 hook
        BACKUP_FILE="${PRE_COMMIT_HOOK}.backup.$(date +%Y%m%d_%H%M%S)"
        log_warning "检测到已存在的 pre-commit hook，正在备份到: $BACKUP_FILE"
        cp "$PRE_COMMIT_HOOK" "$BACKUP_FILE"
        log_info "备份完成，将继续安装新的 hook"
    fi
else
    log_info "pre-commit hook 不存在，将创建新的 hook..."
fi

# 生成 pre-commit hook 内容
log_info "生成 pre-commit hook 内容..."
cat > "$PRE_COMMIT_HOOK" << 'HOOK_EOF'
#!/bin/bash
#
# Git pre-commit hook for Workflow
# 在提交前运行代码质量检查和自动修复
#
# 检查项：
# 1. 代码格式化（自动修复）
# 2. Clippy 警告检查（尝试自动修复）
# 3. 编译检查

set -e

# 获取项目根目录
PROJECT_ROOT="$(git rev-parse --show-toplevel)"
cd "$PROJECT_ROOT"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo ""
echo -e "${BLUE}🚀 运行 pre-commit 检查和修复...${NC}"
echo ""

# 1. 代码格式化（自动修复）
echo -e "${BLUE}🎨 [1/4] 自动格式化代码...${NC}"
# 检查是否有格式问题
if ! cargo fmt --check --all > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠ 检测到格式问题，正在自动修复...${NC}"
    # 自动格式化
    cargo fmt --all
    # 将格式化后的文件添加到暂存区
    git add -u
    echo -e "${GREEN}✓ 代码已自动格式化并添加到暂存区${NC}"
else
    echo -e "${GREEN}✓ 代码格式正确${NC}"
fi

# 2. Clippy 警告检查（尝试自动修复）
echo -e "${BLUE}🔍 [2/4] 运行 Clippy 检查...${NC}"
# 保存当前暂存区状态
STAGED_FILES_BEFORE="$(git diff --cached --name-only | sort)"
# 尝试自动修复（静默运行，失败不影响后续检查）
cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged > /dev/null 2>&1 || true
# 检查是否有文件被修改
STAGED_FILES_AFTER="$(git diff --cached --name-only | sort)"
UNSTAGED_FILES="$(git diff --name-only | sort)"
if [ "$STAGED_FILES_BEFORE" != "$STAGED_FILES_AFTER" ] || [ -n "$UNSTAGED_FILES" ]; then
    echo -e "${YELLOW}⚠ Clippy 自动修复了一些问题，正在添加到暂存区...${NC}"
    git add -u
    echo -e "${GREEN}✓ Clippy 自动修复完成${NC}"
fi

# 再次检查 Clippy 警告（包括无法自动修复的）
if ! cargo clippy --all-targets --all-features -- -D warnings > /dev/null 2>&1; then
    echo -e "${RED}✗ Clippy 检查失败（存在无法自动修复的警告）${NC}"
    echo -e "${YELLOW}提示: 运行 'cargo clippy --all-targets --all-features' 查看详细警告信息${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Clippy 检查通过${NC}"

# 3. 编译检查
echo -e "${BLUE}🔨 [3/4] 检查代码编译...${NC}"
if ! cargo check --all-targets > /dev/null 2>&1; then
    echo -e "${RED}✗ 编译检查失败${NC}"
    echo -e "${YELLOW}提示: 运行 'cargo check --all-targets' 查看详细编译错误${NC}"
    exit 1
fi
echo -e "${GREEN}✓ 编译检查通过${NC}"

# 4. 最终验证
echo -e "${BLUE}✅ [4/4] 最终验证...${NC}"
# 再次检查格式（确保修复后格式正确）
if ! cargo fmt --check --all > /dev/null 2>&1; then
    echo -e "${RED}✗ 格式验证失败（这不应该发生）${NC}"
    exit 1
fi
echo -e "${GREEN}✓ 所有检查通过${NC}"

echo ""
echo -e "${GREEN}✅ 所有 pre-commit 检查通过！代码已自动修复并准备提交。${NC}"
echo ""
HOOK_EOF

# 设置执行权限
log_info "设置 pre-commit hook 执行权限..."
chmod +x "$PRE_COMMIT_HOOK"

log_success "Git pre-commit hook 安装完成！"
echo ""
log_info "Hook 位置: $PRE_COMMIT_HOOK"
log_info "现在每次 git commit 前都会自动运行代码质量检查和修复"
echo ""

