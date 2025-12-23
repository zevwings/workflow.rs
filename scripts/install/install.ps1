# Workflow CLI 安装脚本 (Windows PowerShell)
#
# 使用方法:
#   Invoke-WebRequest -Uri "https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install/install.ps1" -OutFile install.ps1; .\install.ps1
#
# 或一行命令:
#   powershell -ExecutionPolicy Bypass -Command "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install/install.ps1' -OutFile install.ps1; .\install.ps1"
#
# 指定版本:
#   $env:VERSION="v1.4.8"; powershell -ExecutionPolicy Bypass -Command "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install/install.ps1' -OutFile install.ps1; .\install.ps1"

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

# 检测平台
function Get-Platform {
    Write-Info "Detecting platform..."

    $arch = $env:PROCESSOR_ARCHITECTURE
    if ($arch -eq "AMD64") {
        $arch = "x86_64"
    } elseif ($arch -eq "ARM64") {
        $arch = "ARM64"
    } else {
        throw "Unsupported architecture: $arch"
    }

    $platform = "Windows-$arch"
    Write-Success "Detected platform: $platform"
    return $platform
}

# 检查 PowerShell 版本
function Test-PowerShellVersion {
    Write-Info "Checking PowerShell version..."

    if ($PSVersionTable.PSVersion.Major -lt 5) {
        throw "PowerShell 5.0 or later is required. Current version: $($PSVersionTable.PSVersion)"
    }

    Write-Success "PowerShell version: $($PSVersionTable.PSVersion)"
}

# 获取最新版本
function Get-LatestVersion {
    Write-Info "Fetching latest version..."

    try {
        $apiUrl = "https://api.github.com/repos/zevwings/workflow.rs/releases/latest"
        $response = Invoke-RestMethod -Uri $apiUrl -Method Get -ErrorAction Stop
        $version = $response.tag_name

        if ([string]::IsNullOrWhiteSpace($version)) {
            throw "Failed to fetch latest version from GitHub"
        }

        # 移除 'v' 前缀（如果有）
        $version = $version -replace '^v', ''

        Write-Success "Latest version: $version"
        return $version
    }
    catch {
        throw "Failed to fetch latest version: $_"
    }
}

# 获取版本号
function Get-Version {
    if ($env:VERSION) {
        # 移除 'v' 前缀（如果有）
        $version = $env:VERSION -replace '^v', ''
        Write-Success "Using specified version: $version"
        return $version
    }
    else {
        return Get-LatestVersion
    }
}

# 下载文件（带重试）
function Get-FileWithRetry {
    param(
        [string]$Url,
        [string]$OutputPath,
        [int]$MaxRetries = 3
    )

    $retryCount = 0
    while ($retryCount -lt $MaxRetries) {
        try {
            Write-Info "Downloading from: $Url"
            Invoke-WebRequest -Uri $Url -OutFile $OutputPath -ErrorAction Stop
            return $true
        }
        catch {
            $retryCount++
            if ($retryCount -lt $MaxRetries) {
                Write-Warning "Download failed, retrying... ($retryCount/$MaxRetries)"
                Start-Sleep -Seconds 2
            }
            else {
                Write-Error "Failed to download after $MaxRetries attempts: $_"
                return $false
            }
        }
    }
    return $false
}

# 验证 SHA256
function Test-FileHash {
    param(
        [string]$FilePath,
        [string]$ExpectedHash
    )

    Write-Info "Verifying SHA256 checksum..."

    try {
        $fileHash = Get-FileHash -Path $FilePath -Algorithm SHA256
        $calculatedHash = $fileHash.Hash.ToLower()
        $expectedHash = $ExpectedHash.ToLower().Trim()

        if ($calculatedHash -ne $expectedHash) {
            Write-Error "SHA256 verification failed!"
            Write-Error "Expected: $expectedHash"
            Write-Error "Got:      $calculatedHash"
            return $false
        }

        Write-Success "SHA256 verification passed"
        return $true
    }
    catch {
        Write-Error "Failed to calculate SHA256: $_"
        return $false
    }
}

# 主安装函数
function Install-Workflow {
    Write-Host ""
    Write-Info "=========================================="
    Write-Info "Workflow CLI Installation"
    Write-Info "=========================================="
    Write-Host ""

    # 检查 PowerShell 版本
    Test-PowerShellVersion

    # 检测平台
    $platform = Get-Platform

    # 获取版本
    $version = Get-Version
    $tag = "v$version"

    # 创建临时目录
    $tempDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
    Write-Info "Using temporary directory: $tempDir"

    try {
        # 构建下载 URL
        $archiveName = "workflow-${version}-${platform}.zip"
        $downloadUrl = "https://github.com/zevwings/workflow.rs/releases/download/${tag}/${archiveName}"
        $sha256Url = "${downloadUrl}.sha256"

        # 下载二进制包
        $archivePath = Join-Path $tempDir $archiveName
        if (-not (Get-FileWithRetry -Url $downloadUrl -OutputPath $archivePath)) {
            throw "Failed to download binary package"
        }
        Write-Success "Downloaded binary package"

        # 下载 SHA256 文件
        $sha256Path = Join-Path $tempDir "${archiveName}.sha256"
        $sha256Downloaded = $false
        if (Get-FileWithRetry -Url $sha256Url -OutputPath $sha256Path) {
            $sha256Downloaded = $true
            $expectedHash = (Get-Content $sha256Path -Raw).Trim()

            if ($expectedHash) {
                # 验证 SHA256
                if (-not (Test-FileHash -FilePath $archivePath -ExpectedHash $expectedHash)) {
                    throw "SHA256 verification failed. The downloaded file may be corrupted."
                }
            }
            else {
                Write-Warning "Could not read SHA256 from file, skipping verification"
            }
        }
        else {
            Write-Warning "Failed to download SHA256 file, skipping verification"
        }

        # 解压
        Write-Info "Extracting archive..."
        $extractDir = Join-Path $tempDir "extract"
        New-Item -ItemType Directory -Path $extractDir -Force | Out-Null

        try {
            Expand-Archive -Path $archivePath -DestinationPath $extractDir -Force
        }
        catch {
            throw "Failed to extract archive: $_"
        }
        Write-Success "Extracted archive"

        # 检查 install.exe 是否存在
        $installBinary = Join-Path $extractDir "install.exe"
        if (-not (Test-Path $installBinary)) {
            throw "install.exe not found in archive"
        }

        # 运行安装
        Write-Info "Running installation..."
        Write-Host ""

        try {
            & $installBinary
            if ($LASTEXITCODE -eq 0 -or $LASTEXITCODE -eq $null) {
                Write-Host ""
                Write-Success "Installation completed successfully!"
                Write-Info "You can now use 'workflow' command."
                Write-Info "Run 'workflow --help' to get started."

                # 检查 PATH
                $installDir = Join-Path $env:LOCALAPPDATA "Programs\workflow\bin"
                $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
                if ($currentPath -notlike "*$installDir*") {
                    Write-Warning "The installation directory may not be in your PATH."
                    Write-Info "Installation directory: $installDir"
                    Write-Info "You may need to add it to your PATH environment variable."
                }
            }
            else {
                throw "Installation failed with exit code: $LASTEXITCODE"
            }
        }
        catch {
            throw "Failed to run installation: $_"
        }
    }
    finally {
        # 清理临时文件
        Write-Info "Cleaning up temporary files..."
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# 运行主函数
try {
    Install-Workflow
}
catch {
    Write-Error "Installation failed: $_"
    exit 1
}
