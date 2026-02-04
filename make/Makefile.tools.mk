# make/Makefile.tools.mk
# 工具安装模块

# Help 信息
define HELP_TOOLS
	@echo "工具安装相关："
	@echo "  make setup            - 安装所需的开发工具（rustfmt, clippy, rust-analyzer, cargo-bloat, cargo-audit, cargo-outdated, cargo-tarpaulin, lychee）"
	@echo "  make install-hooks    - 安装 Git hooks（pre-commit）"
	@echo ""
endef

# ============================================
# 工具安装相关目标
# ============================================

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
	@echo ""
	@echo "安装 cargo-audit..."
	@if command -v cargo-audit >/dev/null 2>&1; then \
		echo "cargo-audit 已安装"; \
	else \
		cargo install cargo-audit --locked 2>/dev/null || echo "⚠ cargo-audit 安装失败，可稍后手动运行: cargo install cargo-audit"; \
	fi
	@echo ""
	@echo "安装 cargo-outdated..."
	@if command -v cargo-outdated >/dev/null 2>&1; then \
		echo "cargo-outdated 已安装"; \
	else \
		cargo install cargo-outdated --locked 2>/dev/null || echo "⚠ cargo-outdated 安装失败，可稍后手动运行: cargo install cargo-outdated"; \
	fi
	@echo ""
	@echo "安装 cargo-tarpaulin..."
	@if command -v cargo-tarpaulin >/dev/null 2>&1; then \
		echo "cargo-tarpaulin 已安装"; \
	else \
		cargo install cargo-tarpaulin --locked 2>/dev/null || echo "⚠ cargo-tarpaulin 安装失败，可稍后手动运行: cargo install cargo-tarpaulin"; \
	fi
	@echo ""
	@echo "安装 lychee..."
	@if command -v lychee >/dev/null 2>&1; then \
		echo "lychee 已安装"; \
	else \
		cargo install lychee --locked 2>/dev/null || echo "⚠ lychee 安装失败，可稍后手动运行: cargo install lychee"; \
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
	@echo ""
	@echo "=========================================="
	@echo "安装 Git hooks"
	@echo "=========================================="
	@$(MAKE) install-hooks

# 安装 Git hooks
# 自动检测操作系统并使用相应的安装脚本
# - Windows (MINGW/MSYS/CYGWIN): 优先使用 PowerShell，fallback 到 bash
# - macOS/Linux: 使用 bash
install-hooks:
	@echo "安装 Git hooks..."
	@if [ -f "scripts/dev/shell/hooks/install-hooks.sh" ]; then \
		UNAME_S=$$(uname -s 2>/dev/null || echo ""); \
		if echo "$$UNAME_S" | grep -qiE "(MINGW|MSYS|CYGWIN)"; then \
			echo "检测到 Windows 环境，尝试使用 PowerShell..."; \
			if command -v powershell >/dev/null 2>&1 && [ -f "scripts/dev/shell/hooks/install-hooks.ps1" ]; then \
				powershell -ExecutionPolicy Bypass -File scripts/dev/shell/hooks/install-hooks.ps1 || \
				(echo "⚠ PowerShell 失败，使用 bash 脚本..."; bash scripts/dev/shell/hooks/install-hooks.sh); \
			elif command -v pwsh >/dev/null 2>&1 && [ -f "scripts/dev/shell/hooks/install-hooks.ps1" ]; then \
				pwsh -ExecutionPolicy Bypass -File scripts/dev/shell/hooks/install-hooks.ps1 || \
				(echo "⚠ PowerShell Core 失败，使用 bash 脚本..."; bash scripts/dev/shell/hooks/install-hooks.sh); \
			else \
				bash scripts/dev/shell/hooks/install-hooks.sh; \
			fi \
		else \
			bash scripts/dev/shell/hooks/install-hooks.sh; \
		fi \
	else \
		echo "⚠ 警告: scripts/dev/shell/hooks/install-hooks.sh 文件不存在，跳过安装"; \
	fi
