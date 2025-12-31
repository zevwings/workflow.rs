# 清理脚本 - 修复链接器错误
# 
# 用途：清理构建缓存并终止相关进程，解决 Windows 上 cargo 链接器无法打开目标文件的错误
# 使用方法：.\scripts\dev\clean.ps1

Write-Host "正在清理构建缓存..." -ForegroundColor Cyan

# 设置脚本根目录
$PSScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

# 步骤 1: 终止所有相关进程
Write-Host "`n[1/2] 检查并终止相关进程..." -ForegroundColor Yellow
$processes = Get-Process | Where-Object {
    $_.ProcessName -like "*cargo*" -or 
    $_.ProcessName -like "*rustc*" -or 
    $_.ProcessName -like "*module_test*"
}

if ($processes) {
    Write-Host "发现以下进程，正在终止..." -ForegroundColor Yellow
    $processes | ForEach-Object {
        Write-Host "  - $($_.ProcessName) (PID: $($_.Id))" -ForegroundColor Gray
        try {
            Stop-Process -Id $_.Id -Force -ErrorAction Stop
        } catch {
            Write-Host "    警告: 无法终止进程 $($_.ProcessName): $_" -ForegroundColor Red
        }
    }
    # 等待进程完全终止
    Start-Sleep -Seconds 2
    Write-Host "进程已终止" -ForegroundColor Green
} else {
    Write-Host "未发现相关进程" -ForegroundColor Green
}

# 步骤 2: 清理构建缓存
Write-Host "`n[2/2] 清理构建缓存..." -ForegroundColor Yellow
$cargoPath = Get-Command cargo -ErrorAction SilentlyContinue
if ($cargoPath) {
    try {
        Push-Location (Join-Path $PSScriptRoot "..\..")
        cargo clean
        if ($LASTEXITCODE -eq 0) {
            Write-Host "构建缓存已清理" -ForegroundColor Green
        } else {
            Write-Host "警告: cargo clean 返回非零退出码: $LASTEXITCODE" -ForegroundColor Red
        }
        Pop-Location
    } catch {
        Write-Host "警告: cargo clean 失败: $_" -ForegroundColor Red
        Pop-Location
    }
} else {
    Write-Host "错误: 未找到 cargo 命令，请确保 Rust 已正确安装" -ForegroundColor Red
    exit 1
}

Write-Host "`n清理完成！" -ForegroundColor Green

