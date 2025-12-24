# Git Hooks 安装脚本 (PowerShell)
#
# 安装 Git pre-commit hook 到项目的 .git/hooks/ 目录
#
# 使用方法:
#   .\scripts\install\install-hooks.ps1
#   或
#   make install-hooks

$ErrorActionPreference = "Stop"

# 获取脚本所在目录（项目根目录）
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Resolve-Path (Join-Path $ScriptDir "..\..")

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

# 检查 pre-commit hook 是否存在
if (-not (Test-Path $PreCommitHook)) {
    Write-Host "错误: 找不到 pre-commit hook 文件: $PreCommitHook" -ForegroundColor Red
    Write-Host "请确保 .git/hooks/pre-commit 文件存在" -ForegroundColor Yellow
    exit 1
}

# 设置执行权限
Write-Host "设置 pre-commit hook 执行权限..." -ForegroundColor Blue

Write-Host "Git pre-commit hook 安装完成！" -ForegroundColor Green
Write-Host ""
Write-Host "Hook 位置: $PreCommitHook" -ForegroundColor Blue
Write-Host "现在每次 git commit 前都会自动运行代码质量检查" -ForegroundColor Blue
Write-Host ""

