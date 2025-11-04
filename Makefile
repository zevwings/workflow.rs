.PHONY: help build release clean install test lint setup

# 项目名称
BINARY_NAME = workflow

# 默认目标
.DEFAULT_GOAL := help

# 检查 rustfmt 是否安装
check-rustfmt:
	@command -v cargo-fmt >/dev/null 2>&1 || rustup component list --toolchain stable | grep -q "rustfmt.*installed" || (echo "错误: rustfmt 未安装。运行 'make setup' 或 'rustup component add rustfmt'" && exit 1)

# 检查 clippy 是否安装
check-clippy:
	@command -v cargo-clippy >/dev/null 2>&1 || rustup component list --toolchain stable | grep -q "clippy.*installed" || (echo "错误: clippy 未安装。运行 'make setup' 或 'rustup component add clippy'" && exit 1)

# 显示帮助信息
help:
	@echo "可用的 Make 目标："
	@echo "  make build      - 构建 debug 版本"
	@echo "  make release    - 构建 release 版本（打包）"
	@echo "  make clean      - 清理构建产物"
	@echo "  make install    - 安装到系统 (默认: /usr/local/bin)"
	@echo "  make test       - 运行测试"
	@echo "  make lint       - 运行完整的代码检查（格式化 + Clippy + Check）"
	@echo "  make setup      - 安装所需的开发工具（rustfmt, clippy）"

# 构建 debug 版本
dev:
	@echo "构建 debug 版本..."
	cargo build
	@echo "构建完成: target/debug/$(BINARY_NAME)"

# 构建 release 版本（打包）
release:
	@echo "构建 release 版本..."
	cargo build --release
	@echo "打包完成: target/release/$(BINARY_NAME)"
	@ls -lh target/release/$(BINARY_NAME)

# 清理构建产物
clean:
	@echo "清理构建产物..."
	cargo clean
	@echo "清理完成"

# 安装到系统（默认 /usr/local/bin）
install: release
	@echo "安装 $(BINARY_NAME) 到 /usr/local/bin..."
	sudo cp target/release/$(BINARY_NAME) /usr/local/bin/
	@echo "安装完成"

# 运行测试
test:
	@echo "运行测试..."
	cargo test

# 安装所需的开发工具
setup:
	@echo "安装开发工具..."
	@rustup component add rustfmt 2>/dev/null || echo "rustfmt 已安装或安装失败"
	@rustup component add clippy 2>/dev/null || echo "clippy 已安装或安装失败"
	@echo "开发工具安装完成"

# 运行完整的代码检查（格式化检查 + Clippy + Check）
lint: check-rustfmt check-clippy
	@echo "运行完整的代码检查..."
	@echo ""
	@echo "1/3 检查代码格式..."
	@cargo fmt --check || (echo "✗ 代码格式不正确，运行 'cargo fmt' 自动修复" && exit 1)
	@echo "✓ 代码格式正确"
	@echo ""
	@echo "2/3 运行 Clippy 检查..."
	@cargo clippy -- -D warnings
	@echo "✓ Clippy 检查通过"
	@echo ""
	@echo "3/3 运行 cargo check..."
	@cargo check
	@echo "✓ Check 通过"
	@echo ""
	@echo "✓ 所有检查通过！"

