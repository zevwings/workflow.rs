# make/Makefile.tools.mk
# 工具安装模块

# Help 信息
define HELP_TOOLS
	@echo "工具安装相关："
	@echo "  make setup            - 安装所需的开发工具（rustfmt, clippy, rust-analyzer, cargo-bloat）"
	@echo ""
endef

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
