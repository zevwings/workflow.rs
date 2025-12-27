# CI 中使用 Python Dev 工具

## Python 版本要求

Dev 工具要求 **Python 3.13+**。

## GitHub Actions 配置

在 GitHub Actions workflow 中，需要先设置 Python 3.13，然后才能使用 dev 工具。

### 示例配置

```yaml
steps:
  - name: 📥 Checkout repository
    uses: actions/checkout@v4

  - name: 🐍 Setup Python 3.13
    uses: actions/setup-python@v5
    with:
      python-version: '3.13'

  - name: 🔍 Check if version bump branch
    id: check
    env:
      GITHUB_HEAD_REF: ${{ github.head_ref }}
      GITHUB_REF_NAME: ${{ github.ref_name }}
      GITHUB_EVENT_NAME: ${{ github.event_name }}
      GITHUB_PR_CREATOR: ${{ github.event.pull_request.user.login }}
      WORKFLOW_USER_NAME: ${{ env.WORKFLOW_USER_NAME }}
    run: |
      python3 scripts/dev/dev.py ci check-skip \
        --branch "${{ github.head_ref || github.ref_name }}" \
        --pr-creator "${{ github.event.pull_request.user.login }}" \
        --expected-user "${{ env.WORKFLOW_USER_NAME }}" \
        --ci
```

### 替换现有的 Rust 版本

将原来的：
```yaml
- name: 🔨 Build dev binary
  run: cargo build --bin dev --release

- name: 📥 Download dev binary artifact
  uses: actions/download-artifact@v4
  with:
    name: dev-binary
    path: target/release/

- name: 🔍 Check if version bump branch
  run: ./target/release/dev ci check-skip ...
```

替换为：
```yaml
- name: 🐍 Setup Python 3.13
  uses: actions/setup-python@v5
  with:
    python-version: '3.13'

- name: 🔍 Check if version bump branch
  run: python3 scripts/dev/dev.py ci check-skip ...
```

## 优势

- ✅ **无需构建**: 节省约 3 分钟构建时间
- ✅ **无需缓存**: 不需要缓存二进制文件
- ✅ **快速启动**: Python 脚本直接运行
- ✅ **版本检查**: 脚本会自动检查 Python 版本

## 注意事项

1. **Python 版本**: 必须使用 Python 3.13+，脚本会自动检查
2. **无需安装依赖**: 脚本使用标准库，无需 `pip install`
3. **跨平台**: Python 3.13 在所有 GitHub Actions runner 上都可用
4. **不需要系统 Python**: 使用 `actions/setup-python@v5` 设置 Python 3.13 后，**不需要**运行 `install-basic.sh` 中的 `python3` 安装步骤

## 与构建依赖的关系

### 仅使用 dev 工具

如果只是运行 dev 工具（不构建 Rust 项目），**不需要** `install-basic.sh`：

```yaml
- name: 🐍 Setup Python 3.13
  uses: actions/setup-python@v5
  with:
    python-version: '3.13'

- name: 🔍 Check CI skip
  run: python3 scripts/dev/dev.py ci check-skip --ci
```

### 需要构建 Rust 项目

如果需要构建 Rust 项目（需要 xcb-proto），有两种方案：

**方案 A: 使用系统 Python（当前方案）**
```yaml
- name: 📦 Install system dependencies
  run: bash scripts/dev/dependencies/install-basic.sh  # 包含 python3
```

**方案 B: 使用 Python 3.13 + pip（推荐）**
```yaml
- name: 🐍 Setup Python 3.13
  uses: actions/setup-python@v5
  with:
    python-version: '3.13'

- name: 📦 Install xcbgen
  run: pip install xcbgen

- name: 📦 Install XCB libraries (without python3)
  run: |
    sudo apt-get update
    sudo apt-get install -y \
      libxcb1-dev \
      # ... 其他 XCB 库（但不包括 python3）
```

详见 [dependencies/ANALYSIS.md](./dependencies/ANALYSIS.md)

