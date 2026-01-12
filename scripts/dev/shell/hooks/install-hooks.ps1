# Git Hooks 安装脚本 (PowerShell) - 开发工具
#
# 安装 Git pre-commit hook 到项目的 .git/hooks/ 目录
#
# 使用方法:
#   .\scripts\dev\shell\hooks\install-hooks.ps1
#   或
#   make install-hooks

$ErrorActionPreference = "Stop"

# 获取脚本所在目录（项目根目录）
# 脚本位于 scripts/dev/shell/hooks/，需要向上 4 级到达项目根目录
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Resolve-Path (Join-Path $ScriptDir "..\..\..\..")

# 检查是否在 Git 仓库中
$GitDir = Join-Path $ProjectRoot ".git"
if (-not (Test-Path $GitDir)) {
    Write-Host "错误: 当前目录不是 Git 仓库" -ForegroundColor Red
    exit 1
}

# Git hooks 目录
$GitHooksDir = Join-Path $ProjectRoot ".git\hooks"
$PreCommitHook = Join-Path $GitHooksDir "pre-commit"

Write-Host "安装 Git pre-commit hook..." -ForegroundColor Blue
Write-Host ""

# 检查是否在 Git 仓库中
$GitDir = Join-Path $ProjectRoot ".git"
if (-not (Test-Path $GitDir)) {
    Write-Host "错误: 当前目录不是 Git 仓库" -ForegroundColor Red
    exit 1
}

# 确保 hooks 目录存在
if (-not (Test-Path $GitHooksDir)) {
    New-Item -ItemType Directory -Path $GitHooksDir -Force | Out-Null
}

# 检查 pre-commit hook 是否已存在
if (Test-Path $PreCommitHook) {
    # 检查是否是我们的 hook（通过检查文件内容中的标识）
    $HookContent = Get-Content $PreCommitHook -Raw -ErrorAction SilentlyContinue
    if ($HookContent -match "Git pre-commit hook for Workflow") {
        Write-Host "检测到已存在的 Workflow pre-commit hook，将更新..." -ForegroundColor Blue
    } else {
        # 备份现有的 hook
        $Timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
        $BackupFile = "${PreCommitHook}.backup.${Timestamp}"
        Write-Host "检测到已存在的 pre-commit hook，正在备份到: $BackupFile" -ForegroundColor Yellow
        Copy-Item $PreCommitHook $BackupFile -Force
        Write-Host "备份完成，将继续安装新的 hook" -ForegroundColor Blue
    }
} else {
    Write-Host "pre-commit hook 不存在，将创建新的 hook..." -ForegroundColor Blue
}

# 生成 pre-commit hook 内容
Write-Host "生成 pre-commit hook 内容..." -ForegroundColor Blue

$HookContent = @'
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
'@

# 写入 hook 文件（使用 UTF-8 编码，无 BOM）
[System.IO.File]::WriteAllText($PreCommitHook, $HookContent, [System.Text.UTF8Encoding]::new($false))

# 设置执行权限（在 Windows 上，Git 会处理执行权限）
Write-Host "设置 pre-commit hook 执行权限..." -ForegroundColor Blue

Write-Host "Git pre-commit hook 安装完成！" -ForegroundColor Green
Write-Host ""
Write-Host "Hook 位置: $PreCommitHook" -ForegroundColor Blue
Write-Host "现在每次 git commit 前都会自动运行代码质量检查和修复" -ForegroundColor Blue
Write-Host ""

