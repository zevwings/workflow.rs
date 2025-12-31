# 修复链接器错误 LNK1104 的脚本
# 
# 用途：解决 Windows 上 cargo 链接器无法打开目标文件的错误
# 使用方法：.\scripts\dev\fix_linker_error.ps1

Write-Host "正在修复链接器错误 LNK1104..." -ForegroundColor Cyan

# 步骤 1: 终止所有相关进程
Write-Host "`n[1/4] 检查并终止相关进程..." -ForegroundColor Yellow
$processes = Get-Process | Where-Object {
    $_.ProcessName -like "*cargo*" -or 
    $_.ProcessName -like "*rustc*" -or 
    $_.ProcessName -like "*module_test*" -or
    $_.ProcessName -like "*workflow*"
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
} else {
    Write-Host "未发现相关进程" -ForegroundColor Green
}

# 步骤 2: 检查并解锁目标文件
Write-Host "`n[2/4] 检查目标文件锁定状态..." -ForegroundColor Yellow
$targetDir = Join-Path $PSScriptRoot "..\..\target"
if (Test-Path $targetDir) {
    $exeFiles = Get-ChildItem -Path $targetDir -Recurse -Filter "*.exe" -ErrorAction SilentlyContinue
    foreach ($file in $exeFiles) {
        try {
            $stream = [System.IO.File]::Open($file.FullName, 'Open', 'ReadWrite', 'None')
            $stream.Close()
        } catch {
            Write-Host "  警告: 文件可能被锁定: $($file.FullName)" -ForegroundColor Yellow
            Write-Host "    尝试删除..." -ForegroundColor Gray
            try {
                Remove-Item $file.FullName -Force -ErrorAction Stop
                Write-Host "    已删除" -ForegroundColor Green
            } catch {
                Write-Host "    无法删除，可能需要手动处理" -ForegroundColor Red
            }
        }
    }
} else {
    Write-Host "target 目录不存在，跳过" -ForegroundColor Gray
}

# 步骤 3: 清理构建缓存
Write-Host "`n[3/4] 清理构建缓存..." -ForegroundColor Yellow
$cargoPath = Get-Command cargo -ErrorAction SilentlyContinue
if ($cargoPath) {
    try {
        Push-Location (Join-Path $PSScriptRoot "..\..")
        cargo clean
        Write-Host "构建缓存已清理" -ForegroundColor Green
        Pop-Location
    } catch {
        Write-Host "警告: cargo clean 失败: $_" -ForegroundColor Red
        Pop-Location
    }
} else {
    Write-Host "警告: 未找到 cargo 命令" -ForegroundColor Red
}

# 步骤 4: 验证修复
Write-Host "`n[4/4] 验证修复..." -ForegroundColor Yellow
$targetDir = Join-Path $PSScriptRoot "..\..\target"
if (Test-Path $targetDir) {
    $canWrite = $false
    try {
        $testFile = Join-Path $targetDir "test_write.tmp"
        "test" | Out-File $testFile -ErrorAction Stop
        Remove-Item $testFile -ErrorAction Stop
        $canWrite = $true
    } catch {
        Write-Host "警告: 无法写入 target 目录，可能存在权限问题" -ForegroundColor Red
    }
    
    if ($canWrite) {
        Write-Host "target 目录可写" -ForegroundColor Green
    }
} else {
    Write-Host "target 目录不存在（正常，将在下次构建时创建）" -ForegroundColor Gray
}

Write-Host "`n修复完成！" -ForegroundColor Green
Write-Host "`n建议的下一步操作：" -ForegroundColor Cyan
Write-Host "  1. 如果问题仍然存在，检查防病毒软件是否锁定了文件" -ForegroundColor White
Write-Host "  2. 将 target 目录添加到防病毒软件的排除列表" -ForegroundColor White
Write-Host "  3. 尝试使用单线程编译: `$env:CARGO_BUILD_JOBS='1'; cargo test ..." -ForegroundColor White
Write-Host "  4. 重新运行测试: cargo test --test module_test test_resolve_target_branch" -ForegroundColor White

