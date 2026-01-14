# 安装脚本

用于安装和卸载 Workflow CLI 的脚本。

## 脚本列表

| 脚本 | 说明 | 平台 |
|------|------|------|
| `install.sh` | 安装 Workflow CLI | Linux/macOS |
| `install.ps1` | 安装 Workflow CLI | Windows |
| `uninstall.sh` | 卸载 Workflow CLI | Linux/macOS |
| `uninstall.ps1` | 卸载 Workflow CLI | Windows |

## 使用方法

### 安装

**Linux/macOS**:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install/install.sh)"
```

**指定版本**:

```bash
VERSION=v1.4.8 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install/install.sh)"
```

**Windows**:

```powershell
irm https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install/install.ps1 | iex
```

### 卸载

**Linux/macOS**:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install/uninstall.sh)"
```

**Windows**:

```powershell
irm https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install/uninstall.ps1 | iex
```

