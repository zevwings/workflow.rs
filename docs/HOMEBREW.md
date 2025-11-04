# Homebrew 安装指南

本项目支持通过 Homebrew 安装。有两种方式：

## 方式一：从源码构建（当前推荐）

如果项目还没有发布 GitHub Releases，可以使用从源码构建的方式。

### 1. 创建 Homebrew Tap

首先，在 GitHub 上创建一个新的仓库用于存放 Homebrew formula，命名为 `homebrew-workflow`：

```bash
# 在 GitHub 上创建新仓库：https://github.com/zevwings/homebrew-workflow
```

### 2. 设置 Formula 文件

将 `Formula/workflow.rb` 文件推送到 tap 仓库：

```bash
# 克隆 tap 仓库
git clone git@github.com:zevwings/homebrew-workflow.git
cd homebrew-workflow

# 复制 formula 文件
cp /path/to/workflow/Formula/workflow.rb Formula/workflow.rb

# 提交并推送
git add Formula/workflow.rb
git commit -m "Add workflow formula"
git push origin main
```

### 3. 安装

用户可以通过以下命令安装：

```bash
brew tap zevwings/workflow
brew install workflow
```

## 方式二：使用 GitHub Releases（推荐用于生产环境）

如果项目已经发布了 GitHub Releases，可以使用预编译的二进制文件，这样安装更快。

### 1. 创建 GitHub Release

首先需要为每个平台构建并上传二进制文件：

```bash
# 构建所有平台的二进制文件
# macOS Intel
cargo build --release --target x86_64-apple-darwin
# macOS ARM
cargo build --release --target aarch64-apple-darwin
# Linux x86_64
cargo build --release --target x86_64-unknown-linux-gnu
# Linux ARM
cargo build --release --target aarch64-unknown-linux-gnu

# 创建压缩包
tar czf workflow-0.1.0-x86_64-apple-darwin.tar.gz -C target/x86_64-apple-darwin/release workflow pr qk
tar czf workflow-0.1.0-aarch64-apple-darwin.tar.gz -C target/aarch64-apple-darwin/release workflow pr qk
# ... 其他平台

# 在 GitHub 上创建 Release 并上传这些文件
```

### 2. 更新 Formula 文件

修改 `Formula/workflow.rb`，取消注释预编译二进制相关的部分，并填写正确的 SHA256：

```bash
# 获取 SHA256
shasum -a 256 workflow-0.1.0-x86_64-apple-darwin.tar.gz
```

### 3. 安装

```bash
brew tap zevwings/workflow
brew install workflow
```

## 方式三：本地安装（开发测试）

如果只是想本地测试，可以直接从本地路径安装：

```bash
brew install --build-from-source /path/to/workflow/Formula/workflow.rb
```

或者使用 `brew` 的 `--HEAD` 选项从 Git 仓库安装最新版本：

```bash
brew install --HEAD https://raw.githubusercontent.com/zevwings/workflow.rs/master/Formula/workflow.rb
```

## 验证安装

安装完成后，验证是否安装成功：

```bash
workflow --help
pr --help
qk --help
```

## 更新 Formula

当项目有新版本时，需要更新 Formula 文件中的版本号和 SHA256（如果使用预编译二进制），然后推送到 tap 仓库。

## 注意事项

1. **Tap 仓库命名**：Homebrew tap 仓库必须遵循 `homebrew-<name>` 的命名规范
2. **Formula 文件位置**：在 tap 仓库中，formula 文件应该放在 `Formula/` 目录下
3. **版本号**：确保 Formula 中的版本号与 `Cargo.toml` 中的版本号一致
4. **SHA256**：如果使用预编译二进制，必须提供正确的 SHA256 校验和
5. **二进制文件**：确保所有三个二进制文件（`workflow`、`pr`、`qk`）都被正确安装

## 参考资源

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Homebrew Tap 创建指南](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
- [Rust 项目 Homebrew Formula 示例](https://github.com/Homebrew/homebrew-core/blob/master/Formula/ripgrep.rb)

