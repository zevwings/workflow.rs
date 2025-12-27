# 依赖安装脚本分析

## 脚本用途

### `install-basic.sh`
- **主要用途**: 安装 XCB 开发库（用于 Rust clipboard 功能）
- **Python 用途**:
  - `python3` 包：用于 `xcb-proto` 的代码生成（构建 Rust 项目时需要）
  - 不是用于运行 dev 工具

### `install-build.sh`
- **主要用途**: 安装构建依赖（用于编译 Rust 项目）
- **Python 用途**:
  - `python3-pip`: pip 包管理器
  - `python3-xcbgen`: xcbgen Python 模块（xcb-proto 需要）

## 关键问题

### 问题：使用 `actions/setup-python@v5` 设置 Python 3.13 后，还需要 `install-basic.sh` 中的 `python3` 吗？

**答案：取决于用途**

#### 情况 1: 仅运行 dev 工具（不构建 Rust 项目）

✅ **不需要** `install-basic.sh` 中的 `python3`

```yaml
# 只需要设置 Python 3.13
- name: 🐍 Setup Python 3.13
  uses: actions/setup-python@v5
  with:
    python-version: '3.13'

# 直接使用 dev 工具
- name: 🔍 Check CI skip
  run: python3 scripts/dev/dev.py ci check-skip --ci
```

**原因**:
- dev 工具只使用 Python 标准库，不需要系统包
- `actions/setup-python` 安装的 Python 3.13 已经足够

#### 情况 2: 需要构建 Rust 项目（需要 xcb-proto）

⚠️ **仍然需要** Python，但有两种方案：

**方案 A: 使用系统 Python（当前方案）**
```yaml
# 安装系统 Python（用于 xcb-proto）
- name: 📦 Install system dependencies
  run: bash scripts/dev/dependencies/install-basic.sh

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

## 建议

### 对于 dev 工具使用场景

**可以移除 `install-basic.sh` 中的 `python3`**，因为：
1. dev 工具只需要 Python 3.13（通过 `actions/setup-python` 提供）
2. dev 工具使用标准库，不需要系统 Python 包

### 对于构建 Rust 项目场景

**建议采用方案 B**：
1. 使用 `actions/setup-python@v5` 设置 Python 3.13
2. 通过 `pip install xcbgen` 安装 xcbgen（而不是系统包）
3. 只安装 XCB 开发库（移除 `python3` 包）

### 修改建议

#### 选项 1: 修改 `install-basic.sh`（条件安装 Python）

可以修改 `install-basic.sh`，使其在检测到已安装 Python 3.13+ 时跳过 Python 安装：

```bash
# 检查是否已有 Python 3.13+
if command -v python3 >/dev/null 2>&1 && python3 -c "import sys; exit(0 if sys.version_info >= (3, 13) else 1)" 2>/dev/null; then
    echo "✅ Python 3.13+ already available, skipping python3 installation"
else
    sudo apt-get install -y python3
fi
```

#### 选项 2: 创建 `install-xcb-only.sh`（推荐）

创建一个新的脚本，只安装 XCB 库，不安装 Python：

```bash
#!/usr/bin/env bash
# 只安装 XCB 开发库（不包含 Python）
# 用于配合 actions/setup-python 使用

sudo apt-get update
sudo apt-get install -y \
    libxcb1-dev \
    libxcb-render0-dev \
    # ... 其他 XCB 库
    xcb-proto
```

然后在需要构建的项目中：
```yaml
- name: 🐍 Setup Python 3.13
  uses: actions/setup-python@v5
  with:
    python-version: '3.13'

- name: 📦 Install xcbgen
  run: pip install xcbgen

- name: 📦 Install XCB libraries only
  run: bash scripts/dev/dependencies/install-xcb-only.sh
```

#### 选项 3: 保持现状（最简单）

如果大部分 CI job 都需要构建 Rust 项目，可以保持现状：
- 使用 `install-basic.sh` 安装系统 Python（用于 xcb-proto）
- 使用 `actions/setup-python` 设置 Python 3.13（用于 dev 工具）
- 两者可以共存，`actions/setup-python` 的 Python 优先级更高

## 总结

| 场景 | 需要 `install-basic.sh` 的 `python3`? | 说明 |
|------|--------------------------------------|------|
| 仅运行 dev 工具 | ❌ 不需要 | 使用 `actions/setup-python` 即可 |
| 构建 Rust 项目 | ⚠️ 可选 | 可以使用系统 Python 或 Python 3.13 + pip |

