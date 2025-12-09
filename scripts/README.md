# Workflow CLI 安装/卸载脚本

这个目录包含了 Workflow CLI 的安装和卸载脚本，支持通过 GitHub raw URL 一键安装。

## 安装脚本 (install.sh)

### 功能特性

- ✅ 自动检测操作系统和架构（macOS Intel/Apple Silicon, Linux x86_64/ARM64）
- ✅ 自动下载最新版本或指定版本
- ✅ SHA256 校验和验证确保文件完整性
- ✅ 自动安装二进制文件和 shell completion 脚本
- ✅ 错误处理和重试机制
- ✅ 临时文件自动清理

### 使用方法

#### 安装最新版本

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install.sh)"
```

#### 安装指定版本

```bash
VERSION=v1.4.8 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install.sh)"
```

### 安装流程

1. **平台检测**：自动检测操作系统（macOS/Linux）和架构（x86_64/ARM64）
2. **版本获取**：从 GitHub Releases API 获取最新版本，或使用指定的版本
3. **下载**：下载对应平台的二进制包（`.tar.gz`）
4. **验证**：下载并验证 SHA256 校验和
5. **解压**：解压二进制包到临时目录
6. **安装**：运行 `./install` 二进制文件进行安装
7. **清理**：自动清理临时文件

### 系统要求

- `curl` - 用于下载文件
- `tar` - 用于解压归档文件
- `sudo` - macOS/Linux 安装到系统目录时需要（脚本会自动提示）

## Windows 安装脚本 (install.ps1)

### 功能特性

- ✅ 自动检测 Windows 架构（x86_64/ARM64）
- ✅ 自动下载最新版本或指定版本
- ✅ SHA256 校验和验证确保文件完整性
- ✅ 自动安装二进制文件和 PowerShell completion 脚本
- ✅ 错误处理和重试机制
- ✅ 临时文件自动清理
- ✅ PATH 环境变量检查

### 使用方法

#### 安装最新版本

**PowerShell (推荐)**:
```powershell
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install.ps1" -OutFile install.ps1; .\install.ps1
```

**一行命令**:
```powershell
powershell -ExecutionPolicy Bypass -Command "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install.ps1' -OutFile install.ps1; .\install.ps1"
```

#### 安装指定版本

```powershell
$env:VERSION="v1.4.8"; powershell -ExecutionPolicy Bypass -Command "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install.ps1' -OutFile install.ps1; .\install.ps1"
```

### 安装流程

1. **平台检测**：自动检测 Windows 架构（x86_64/ARM64）
2. **版本获取**：从 GitHub Releases API 获取最新版本，或使用指定的版本
3. **下载**：下载对应平台的二进制包（`.zip`）
4. **验证**：下载并验证 SHA256 校验和
5. **解压**：解压二进制包到临时目录
6. **安装**：运行 `install.exe` 二进制文件进行安装
7. **清理**：自动清理临时文件

### 系统要求

- PowerShell 5.0 或更高版本
- 网络连接（用于下载）
- 管理员权限（可能需要，取决于安装目录权限）

## 卸载脚本 (uninstall.sh)

### 功能特性

- ✅ 自动检测已安装的 Workflow CLI
- ✅ 优先使用 `workflow uninstall` 命令（如果可用）
- ✅ 手动卸载作为备选方案
- ✅ 清理二进制文件、配置文件和 completion 脚本
- ✅ 交互式确认，避免误删

### 使用方法

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/uninstall.sh)"
```

### 卸载流程

1. **检测安装**：检查 `workflow` 命令是否在 PATH 中
2. **确认卸载**：提示用户确认是否卸载
3. **执行卸载**：
   - 优先使用 `workflow uninstall` 命令（如果可用）
   - 如果命令不可用，执行手动卸载
4. **清理文件**：
   - 删除二进制文件（`/usr/local/bin/workflow`, `/usr/local/bin/install`）
   - 可选删除配置文件（`~/.workflow/`）
   - 删除 completion 脚本
   - 从 shell 配置文件中移除 completion 配置

## Windows 卸载脚本 (uninstall.ps1)

### 功能特性

- ✅ 自动检测已安装的 Workflow CLI
- ✅ 优先使用 `workflow uninstall` 命令（如果可用）
- ✅ 手动卸载作为备选方案
- ✅ 清理二进制文件、配置文件和 completion 脚本
- ✅ 从 PATH 环境变量中移除安装目录
- ✅ 交互式确认，避免误删

### 使用方法

**PowerShell (推荐)**:
```powershell
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/uninstall.ps1" -OutFile uninstall.ps1; .\uninstall.ps1
```

**一行命令**:
```powershell
powershell -ExecutionPolicy Bypass -Command "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/uninstall.ps1' -OutFile uninstall.ps1; .\uninstall.ps1"
```

### 卸载流程

1. **检测安装**：检查 `workflow` 命令是否在 PATH 中
2. **确认卸载**：提示用户确认是否卸载
3. **执行卸载**：
   - 优先使用 `workflow uninstall` 命令（如果可用）
   - 如果命令不可用，执行手动卸载
4. **清理文件**：
   - 删除二进制文件（`%LOCALAPPDATA%\Programs\workflow\bin\workflow.exe`, `install.exe`）
   - 可选删除配置文件（`%APPDATA%\workflow\`）
   - 删除 completion 脚本
   - 从 PowerShell profile 中移除 completion 配置
   - 从 PATH 环境变量中移除安装目录

## 支持的平台

### macOS
- ✅ Intel (x86_64)
- ✅ Apple Silicon (ARM64)

### Linux
- ✅ x86_64 (glibc)
- ✅ ARM64 (aarch64)

### Windows
- ✅ x86_64 (Intel/AMD)
- ✅ ARM64

## 故障排除

### 安装失败

1. **网络问题**：检查网络连接，脚本会自动重试 3 次
2. **权限问题**：确保有 `sudo` 权限以安装到系统目录
3. **工具缺失**：确保已安装 `curl` 和 `tar`

### 卸载失败

1. **权限问题**：某些文件可能需要 `sudo` 权限才能删除
2. **手动清理**：如果自动卸载失败，可以手动删除：
   - 二进制文件：`/usr/local/bin/workflow`, `/usr/local/bin/install`
   - 配置文件：`~/.workflow/`
   - Shell 配置：从 `~/.zshrc` 或 `~/.bashrc` 中移除相关行

## 开发说明

### 脚本位置

- macOS/Linux 安装脚本：`scripts/install.sh`
- macOS/Linux 卸载脚本：`scripts/uninstall.sh`
- Windows 安装脚本：`scripts/install.ps1`
- Windows 卸载脚本：`scripts/uninstall.ps1`

### GitHub Raw URL

脚本通过 GitHub raw URL 访问：

**macOS/Linux**:
- 安装：`https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install.sh`
- 卸载：`https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/uninstall.sh`

**Windows**:
- 安装：`https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install.ps1`
- 卸载：`https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/uninstall.ps1`

### 版本管理

脚本会从 GitHub Releases 下载预编译的二进制包：
- 最新版本：通过 GitHub Releases API 获取
- 指定版本：通过 `VERSION` 环境变量指定

### 二进制包格式

- **macOS/Linux**: `workflow-{version}-{platform}.tar.gz`
  - 包含文件：`workflow`, `install`
  - 校验文件：`workflow-{version}-{platform}.tar.gz.sha256`

- **Windows**: `workflow-{version}-{platform}.zip`
  - 包含文件：`workflow.exe`, `install.exe`
  - 校验文件：`workflow-{version}-{platform}.zip.sha256`
