.PHONY: help build release clean install test lint fix setup uninstall tag dev bloat

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
	@echo "  make build              - 构建 debug 版本"
	@echo "  make release            - 构建 release 版本（打包）"
	@echo "  make clean              - 清理构建产物"
	@echo "  make install            - 一次性安装全部（构建 + 安装二进制 + 安装 completion）"
	@echo "  make update             - 更新 Workflow CLI（重新构建 + 更新二进制 + 更新 completion）"
	@echo "  make test               - 运行测试"
	@echo "  make test-all           - 运行所有测试（包括被忽略的）"
	@echo "  make lint               - 运行完整的代码检查（格式化 + Clippy + Check）"
	@echo "  make fix                - 自动修复代码问题（格式化 + Clippy + Cargo Fix）"
	@echo "  make setup              - 安装所需的开发工具（rustfmt, clippy, rust-analyzer, cargo-bloat）"
	@echo "  make uninstall          - 卸载二进制文件和 shell completion 脚本"
	@echo "  make tag VERSION=v1.0.0 - 创建 git tag 并推送到远程仓库"
	@echo "  make bloat              - 分析二进制文件大小（需要先运行 make setup）"

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
	@echo "二进制文件:"
	@ls -lh target/release/{workflow,install} 2>/dev/null || ls -lh target/release/$(BINARY_NAME)

# 清理构建产物
clean:
	@echo "清理构建产物..."
	cargo clean
	@echo "清理完成"

# 运行测试（包括单元测试、集成测试和文档测试）
test:
	@echo "运行测试..."
	cargo test

# 运行所有测试（包括被忽略的）
test-all:
	@echo "运行所有测试（包括被忽略的）..."
	cargo test -- --include-ignored

# 安装所需的开发工具
setup:
	@echo "安装开发工具..."
	@rustup component add rustfmt 2>/dev/null || echo "rustfmt 已安装或安装失败"
	@rustup component add clippy 2>/dev/null || echo "clippy 已安装或安装失败"
	@echo ""
	@echo "安装 cargo-bloat..."
	@if command -v cargo-bloat >/dev/null 2>&1; then \
		echo "cargo-bloat 已安装"; \
	else \
		cargo install cargo-bloat --locked 2>/dev/null || echo "⚠ cargo-bloat 安装失败，可稍后手动运行: cargo install cargo-bloat"; \
	fi
	@echo "开发工具安装完成"
	@echo ""
	@echo "=========================================="
	@echo "安装 rust-analyzer (从源码构建)"
	@echo "=========================================="
	@echo ""
	@echo "注意: 如果您的平台没有预编译的 rust-analyzer 二进制文件，"
	@echo "      需要从源码构建安装。"
	@echo ""
	@echo "正在克隆 rust-analyzer 仓库..."
	@cd /tmp && \
	if [ -d "rust-analyzer" ]; then \
		echo "检测到已存在的 rust-analyzer 目录，正在更新..."; \
		cd rust-analyzer && \
		git fetch origin main 2>/dev/null || git fetch origin master 2>/dev/null || git pull; \
		git reset --hard origin/main 2>/dev/null || git reset --hard origin/master 2>/dev/null || true; \
	else \
		echo "克隆 rust-analyzer 仓库..."; \
		git clone --depth 1 https://github.com/rust-lang/rust-analyzer.git && \
		cd rust-analyzer; \
	fi
	@echo "正在构建并安装 rust-analyzer..."
	@echo "这可能需要几分钟时间，请耐心等待..."
	@cd /tmp/rust-analyzer && \
	XTASK_CMD="cargo run --package xtask --bin xtask" && \
	if command -v cargo-xtask >/dev/null 2>&1 || cargo xtask --version >/dev/null 2>&1; then \
		XTASK_CMD="cargo xtask"; \
	fi && \
	if $$XTASK_CMD install --server; then \
		echo ""; \
		echo "✅ rust-analyzer 安装完成！"; \
		echo "安装位置: $$(which rust-analyzer)"; \
		rust-analyzer --version; \
	else \
		echo ""; \
		echo "❌ rust-analyzer 安装失败"; \
		echo "请检查错误信息，或手动运行以下命令:"; \
		echo "  cd /tmp/rust-analyzer && cargo run --package xtask --bin xtask install --server"; \
		exit 1; \
	fi

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

# 自动修复代码问题（格式化 + Clippy + Cargo Fix）
fix: check-rustfmt check-clippy
	@echo "自动修复代码问题..."
	@echo ""
	@echo "1/3 修复代码格式..."
	@cargo fmt
	@echo "✓ 代码格式已修复"
	@echo ""
	@echo "2/3 运行 Clippy 自动修复..."
	@cargo clippy --fix --allow-dirty --allow-staged || echo "⚠ Clippy 无法自动修复所有问题，请手动检查"
	@echo "✓ Clippy 修复完成"
	@echo ""
	@echo "3/3 运行 Cargo Fix 修复编译警告..."
	@cargo fix --allow-dirty --allow-staged || echo "⚠ Cargo Fix 无法修复所有问题"
	@echo "✓ Cargo Fix 完成"
	@echo ""
	@echo "✓ 所有修复完成！"

# 安装到系统（默认 /usr/local/bin，一次性安装全部）
# 直接调用 cargo build --release 确保总是重新构建并安装最新代码
# 清理增量编译缓存以确保使用最新代码
install:
	@echo "构建 release 版本（确保使用最新代码）..."
	@echo "清理增量编译缓存以确保重新编译..."
	@rm -rf target/release/incremental/*/workflow-* 2>/dev/null || true
	@CARGO_INCREMENTAL=0 cargo build --release || cargo build --release
	@echo ""
	@echo "安装 Workflow CLI (二进制文件 + shell completion)..."
	@./target/release/install

# 更新 Workflow CLI（重新构建 + 更新二进制 + 更新 completion）
# 使用 workflow update 命令
update:
	@echo "更新 Workflow CLI..."
	@cargo build --release --bin workflow
	@./target/release/workflow update

# 卸载二进制文件和 shell completion 脚本（一次性清理全部）
uninstall:
	@cargo run --bin workflow -- uninstall

# 创建 git tag 并推送到远程仓库
# 用法: make tag VERSION=v1.0.0
# 如需覆盖已存在的 tag，使用: make tag VERSION=v1.0.0 FORCE=1
tag:
	@if [ -z "$(VERSION)" ]; then \
		echo "错误: 请指定版本号，例如: make tag VERSION=v1.0.0"; \
		exit 1; \
	fi
	@echo "创建 tag: $(VERSION)..."
	@if git rev-parse "$(VERSION)" >/dev/null 2>&1; then \
		if [ "$(FORCE)" = "1" ]; then \
			echo "警告: tag $(VERSION) 已存在，强制覆盖..."; \
			git tag -d $(VERSION) 2>/dev/null || true; \
			git push origin :refs/tags/$(VERSION) 2>/dev/null || true; \
		else \
			echo "错误: tag $(VERSION) 已存在"; \
			echo "如需覆盖，请使用: make tag VERSION=$(VERSION) FORCE=1"; \
			exit 1; \
		fi; \
	fi
	@git tag $(VERSION)
	@echo "✓ Tag 创建成功: $(VERSION)"
	@echo "推送 tag 到远程仓库..."
	@git push origin $(VERSION)
	@echo "✓ Tag 推送成功: $(VERSION)"

# 分析二进制文件大小（需要先安装 cargo-bloat）
# 使用: make bloat [ARGS="--crates --limit 10"]
bloat:
	@if ! command -v cargo-bloat >/dev/null 2>&1; then \
		echo "错误: cargo-bloat 未安装"; \
		echo ""; \
		echo "请运行以下命令安装:"; \
		echo "  make setup"; \
		echo ""; \
		echo "或者手动安装:"; \
		echo "  cargo install cargo-bloat"; \
		exit 1; \
	fi
	@echo "分析二进制文件大小..."
	@cargo bloat --release $(ARGS) || cargo bloat --release

