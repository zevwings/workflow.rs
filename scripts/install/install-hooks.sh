#!/bin/bash
# Git Hooks 安装脚本
#
# 安装 Git pre-commit hook 到项目的 .git/hooks/ 目录
#
# 使用方法:
#   ./scripts/install/install-hooks.sh
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
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

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

# 检查 pre-commit hook 是否存在
if [ ! -f "$PRE_COMMIT_HOOK" ]; then
    log_error "错误: 找不到 pre-commit hook 文件: $PRE_COMMIT_HOOK"
    log_info "请确保 .git/hooks/pre-commit 文件存在"
    exit 1
fi

# 设置执行权限
log_info "设置 pre-commit hook 执行权限..."
chmod +x "$PRE_COMMIT_HOOK"

log_success "Git pre-commit hook 安装完成！"
echo ""
log_info "Hook 位置: $PRE_COMMIT_HOOK"
log_info "现在每次 git commit 前都会自动运行代码质量检查"
echo ""

