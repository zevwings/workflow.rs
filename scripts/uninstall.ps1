# Workflow CLI 卸载脚本 (Windows PowerShell)
#
# 使用方法:
#   Invoke-WebRequest -Uri "https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/uninstall.ps1" -OutFile uninstall.ps1; .\uninstall.ps1
#
# 或一行命令:
#   powershell -ExecutionPolicy Bypass -Command "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/uninstall.ps1' -OutFile uninstall.ps1; .\uninstall.ps1"

[CmdletBinding()]
param()

# 错误处理
$ErrorActionPreference = "Stop"

# 日志函数
function Write-Info {
    param([string]$Message)
    Write-Host "ℹ $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "✓ $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "⚠ $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "✗ $Message" -ForegroundColor Red
}

# 检测 workflow 命令是否存在
function Test-WorkflowInstalled {
    try {
        $null = Get-Command workflow -ErrorAction Stop
        return $true
    }
    catch {
        return $false
    }
}

# 使用 workflow uninstall 命令卸载
function Uninstall-WithCommand {
    Write-Info "Using 'workflow uninstall' command..."

    try {
        & workflow uninstall
        if ($LASTEXITCODE -eq 0 -or $LASTEXITCODE -eq $null) {
            Write-Success "Uninstallation completed successfully!"
            return $true
        }
        else {
            Write-Warning "Uninstallation command failed, trying manual removal..."
            return $false
        }
    }
    catch {
        Write-Warning "Uninstallation command failed, trying manual removal..."
        return $false
    }
}

# 手动卸载（当 workflow 命令不可用时）
function Remove-Manually {
    Write-Info "Performing manual uninstallation..."

    $removedCount = 0

    # 安装目录
    $installDir = Join-Path $env:LOCALAPPDATA "Programs\workflow\bin"

    # 删除二进制文件
    $binaries = @("workflow.exe", "install.exe")
    foreach ($binary in $binaries) {
        $binaryPath = Join-Path $installDir $binary
        if (Test-Path $binaryPath) {
            Write-Info "Removing $binaryPath..."
            try {
                Remove-Item -Path $binaryPath -Force -ErrorAction Stop
                Write-Success "Removed $binaryPath"
                $removedCount++
            }
            catch {
                Write-Warning "Failed to remove $binaryPath : $_"
                Write-Warning "You may need to run this script as Administrator"
            }
        }
    }

    # 删除安装目录（如果为空）
    if (Test-Path $installDir) {
        try {
            $items = Get-ChildItem -Path $installDir -ErrorAction SilentlyContinue
            if ($null -eq $items -or $items.Count -eq 0) {
                Remove-Item -Path $installDir -Force -ErrorAction SilentlyContinue
                Write-Info "Removed empty installation directory: $installDir"
            }
        }
        catch {
            Write-Warning "Could not remove installation directory: $_"
        }
    }

    # 删除配置文件（可选）
    $configDir = Join-Path $env:APPDATA "workflow"
    if (Test-Path $configDir) {
        Write-Info "Configuration directory found: $configDir"
        Write-Host ""
        $response = Read-Host "Do you want to remove configuration files? (y/N)"
        if ($response -match '^[Yy]$') {
            try {
                Remove-Item -Path $configDir -Recurse -Force -ErrorAction Stop
                Write-Success "Removed configuration directory: $configDir"
            }
            catch {
                Write-Warning "Failed to remove configuration directory: $_"
            }
        }
        else {
            Write-Info "Keeping configuration files"
        }
    }

    # 删除 completion 脚本
    $completionDir = Join-Path $configDir "completions"
    if (Test-Path $completionDir) {
        Write-Info "Removing completion scripts..."
        try {
            Remove-Item -Path $completionDir -Recurse -Force -ErrorAction Stop
            Write-Success "Removed completion scripts"
        }
        catch {
            Write-Warning "Failed to remove completion scripts: $_"
        }
    }

    # 从 PowerShell profile 中移除 completion 配置
    $profiles = @(
        $PROFILE.CurrentUserAllHosts,
        $PROFILE.CurrentUserCurrentHost,
        $PROFILE.AllUsersAllHosts,
        $PROFILE.AllUsersCurrentHost
    )

    foreach ($profilePath in $profiles) {
        if (Test-Path $profilePath) {
            $content = Get-Content $profilePath -Raw -ErrorAction SilentlyContinue
            if ($content -and $content -match '\.workflow.*completions') {
                Write-Info "Removing completion config from $profilePath..."
                try {
                    $newContent = $content -replace '(?m)^.*\.workflow.*completions.*$', ''
                    Set-Content -Path $profilePath -Value $newContent -Force -ErrorAction Stop
                    Write-Success "Removed completion config from $profilePath"
                }
                catch {
                    Write-Warning "Failed to remove completion config from $profilePath : $_"
                }
            }
        }
    }

    # 从 PATH 环境变量中移除安装目录
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -like "*$installDir*") {
        Write-Info "Removing installation directory from PATH..."
        $newPath = ($currentPath -split ';' | Where-Object { $_ -ne $installDir }) -join ';'
        try {
            [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
            Write-Success "Removed installation directory from PATH"
            Write-Warning "You may need to restart your terminal for PATH changes to take effect"
        }
        catch {
            Write-Warning "Failed to remove installation directory from PATH: $_"
            Write-Info "You may need to manually remove it from your PATH environment variable"
        }
    }

    if ($removedCount -gt 0) {
        Write-Success "Uninstallation completed! ($removedCount binary file(s) removed)"
    }
    else {
        Write-Warning "No binary files found to remove"
    }
}

# 主卸载函数
function Uninstall-Workflow {
    Write-Host ""
    Write-Info "=========================================="
    Write-Info "Workflow CLI Uninstallation"
    Write-Info "=========================================="
    Write-Host ""

    # 检查是否已安装
    if (-not (Test-WorkflowInstalled)) {
        Write-Warning "Workflow CLI is not installed or not in PATH"
        Write-Info "Checking for manual installation..."

        # 尝试手动卸载
        Remove-Manually
        return
    }

    # 显示已安装的版本
    try {
        $installedVersion = & workflow --version 2>$null
        if ($installedVersion) {
            Write-Info "Found installed version: $installedVersion"
        }
    }
    catch {
        Write-Info "Found installed version: unknown"
    }
    Write-Host ""

    # 确认卸载
    $response = Read-Host "Are you sure you want to uninstall Workflow CLI? (y/N)"
    if ($response -notmatch '^[Yy]$') {
        Write-Info "Uninstallation cancelled"
        return
    }

    Write-Host ""

    # 尝试使用 workflow uninstall 命令
    if (-not (Uninstall-WithCommand)) {
        # 如果命令失败，尝试手动卸载
        Write-Info "Falling back to manual uninstallation..."
        Remove-Manually
    }

    Write-Host ""
    Write-Success "Uninstallation process completed!"
    Write-Info "You may need to restart your terminal for changes to take effect."
}

# 运行主函数
try {
    Uninstall-Workflow
}
catch {
    Write-Error "Uninstallation failed: $_"
    exit 1
}
