# Shell 脚本工具

本目录包含用于开发和维护项目的 Shell（Bash）脚本。

## 📋 目录

- [脚本分类](#脚本分类)
- [依赖安装脚本](#依赖安装脚本)
- [使用说明](#使用说明)
- [注意事项](#注意事项)

## 📁 脚本分类

### 🔧 环境配置相关

#### 依赖安装脚本

| 脚本 | 说明 | 状态 |
|------|------|------|
| `dependencies/install-basic.sh` | 安装 Linux 基本系统依赖（XCB 开发库等） | ✅ |
| `dependencies/install-build.sh` | 安装 Linux 构建依赖（基本依赖 + 构建工具） | ✅ |

## 🔧 依赖安装脚本

### install-basic.sh

安装 Linux 基本系统依赖，包括 XCB 开发库和 Python3。主要用于测试、运行和 CI/CD 环境。

**功能**:
- 安装 XCB 开发库（clipboard 依赖）
- 安装 Python3（xcb-proto 代码生成需要）
- 基本验证

**前置要求**:
- Linux 系统（脚本会自动检查）
- sudo 权限

**使用方法**:

```bash
./scripts/dev/shell/dependencies/install-basic.sh
```

**说明**:
- 脚本会自动检测操作系统，非 Linux 系统会安全退出
- 包含错误处理和验证步骤
- 在 CI/CD 中使用时，建议配合 `if: runner.os == 'Linux'` 条件

**安装的依赖**:
- `python3` - Python 3 解释器
- `libxcb1-dev` - XCB 核心库开发文件
- `libxcb-render0-dev` - XCB Render 扩展开发文件
- `libxcb-shape0-dev` - XCB Shape 扩展开发文件
- `libxcb-xfixes0-dev` - XCB XFixes 扩展开发文件
- `xcb-proto` - XCB 协议描述文件
- `libxcb-keysyms1-dev` - XCB Keysyms 扩展开发文件
- `libxcb-image0-dev` - XCB Image 扩展开发文件
- `libxcb-util-dev` - XCB 工具库开发文件
- 以及其他 XCB 相关开发库

### install-build.sh

安装 Linux 构建依赖，包含基本依赖 + 构建工具（python3-pip, python3-xcbgen, pkg-config）。用于编译 Linux x86_64 平台的二进制文件。

**功能**:
- 调用 `install-basic.sh` 安装基本依赖
- 安装构建工具（python3-pip, python3-xcbgen, pkg-config）
- 验证构建依赖（xcbgen 模块、pkg-config）

**前置要求**:
- Linux 系统（脚本会自动检查）
- sudo 权限

**使用方法**:

```bash
./scripts/dev/shell/dependencies/install-build.sh
```

**说明**:
- 会自动调用基本依赖安装脚本
- 包含完整的构建依赖验证
- 主要用于 Release workflow 的构建阶段

**额外安装的依赖**:
- `python3-pip` - Python 包管理器
- `python3-xcbgen` - xcbgen Python 模块（xcb-proto 需要）
- `pkg-config` - 包配置工具

## 📖 使用说明

### CI/CD 集成

#### GitHub Actions 示例

```yaml
# 安装基本依赖（用于测试和运行）
- name: 📦 Install basic dependencies
  if: runner.os == 'Linux'
  run: bash scripts/dev/shell/dependencies/install-basic.sh

# 安装构建依赖（用于编译）
- name: 📦 Install build dependencies
  if: runner.os == 'Linux'
  run: bash scripts/dev/shell/dependencies/install-build.sh
```

#### 与 Python Dev 工具配合使用

**场景 1: 仅运行 dev 工具（不构建 Rust 项目）**

✅ **不需要** `install-basic.sh` 中的 `python3`

```yaml
# 只需要设置 Python 3.13
- name: 🐍 Setup Python 3.13
  uses: actions/setup-python@v5
  with:
    python-version: '3.13'

# 直接使用 dev 工具
- name: 🔍 Check CI skip
  run: python3 scripts/dev/py/dev.py ci check-skip --ci
```

**场景 2: 需要构建 Rust 项目（需要 xcb-proto）**

⚠️ **仍然需要** Python，但有两种方案：

**方案 A: 使用系统 Python（当前方案）**

```yaml
# 安装系统 Python（用于 xcb-proto）
- name: 📦 Install system dependencies
  if: runner.os == 'Linux'
  run: bash scripts/dev/shell/dependencies/install-basic.sh

# 构建 Rust 项目
- name: 🔨 Build
  run: cargo build
```

**方案 B: 使用 Python 3.13 + pip 安装 xcbgen（推荐）**

```yaml
# 设置 Python 3.13
- name: 🐍 Setup Python 3.13
  uses: actions/setup-python@v5
  with:
    python-version: '3.13'

# 安装 xcbgen（用于 xcb-proto）
- name: 📦 Install xcbgen
  run: pip install xcbgen

# 安装 XCB 开发库（不需要 python3 包）
- name: 📦 Install XCB libraries
  if: runner.os == 'Linux'
  run: |
    sudo apt-get update
    sudo apt-get install -y \
      libxcb1-dev \
      libxcb-render0-dev \
      # ... 其他 XCB 库
      xcb-proto

# 构建 Rust 项目
- name: 🔨 Build
  run: cargo build
```

### 本地开发

在本地 Linux 环境中，可以直接运行脚本安装依赖：

```bash
# 安装基本依赖
./scripts/dev/shell/dependencies/install-basic.sh

# 安装构建依赖
./scripts/dev/shell/dependencies/install-build.sh
```

## ⚠️ 注意事项

1. **操作系统限制**: 脚本设计用于 Linux 系统，在其他系统上会安全退出
2. **权限要求**: 脚本需要 sudo 权限来安装系统包
3. **Python 版本**:
   - `install-basic.sh` 安装系统 Python3（用于 xcb-proto）
   - 如果使用 `actions/setup-python` 设置 Python 3.13，两者可以共存
   - `actions/setup-python` 的 Python 优先级更高
4. **依赖关系**: `install-build.sh` 会自动调用 `install-basic.sh`，无需单独运行
5. **错误处理**: 脚本使用 `set -euo pipefail` 确保错误时退出

## 📚 相关文档

- [依赖分析文档](./dependencies/ANALYSIS.md) - 详细的依赖分析和使用建议
- [主 README](../README.md) - 开发工具脚本总览
- [Python Dev 工具](../py/README.md) - Python 开发工具文档

## 🔄 迁移状态

这些 Shell 脚本属于**保持现状**的脚本，原因：

- ✅ **简单直接**: 脚本逻辑简单，适合 Shell 实现
- ✅ **系统级操作**: 涉及系统包管理，Shell 脚本更合适
- ✅ **CI/CD 专用**: 主要用于 CI/CD 环境，不需要复杂功能
- ✅ **稳定可靠**: 脚本已经过验证，稳定可靠

如果未来需要更复杂的功能，可以考虑迁移到 Python，但目前保持 Shell 实现即可。

