# make/Makefile.deps.mk
# 依赖管理模块

# Help 信息
define HELP_DEPS
	@echo "依赖管理相关："
	@echo "  make audit             - 检查依赖中的安全漏洞（需要先运行 make setup）"
	@echo "  make outdated          - 检查依赖更新（需要先运行 make setup）"
	@echo ""
endef

# ============================================
# 工具检查
# ============================================

# 检查 cargo-audit 是否安装
check-cargo-audit:
	@if ! command -v cargo-audit >/dev/null 2>&1; then \
		echo "错误: cargo-audit 未安装"; \
		echo ""; \
		echo "请运行以下命令安装:"; \
		echo "  make setup"; \
		echo ""; \
		echo "或者手动安装:"; \
		echo "  cargo install cargo-audit"; \
		exit 1; \
	fi

# 检查 cargo-outdated 是否安装
check-cargo-outdated:
	@if ! command -v cargo-outdated >/dev/null 2>&1; then \
		echo "错误: cargo-outdated 未安装"; \
		echo ""; \
		echo "请运行以下命令安装:"; \
		echo "  make setup"; \
		echo ""; \
		echo "或者手动安装:"; \
		echo "  cargo install cargo-outdated"; \
		exit 1; \
	fi

# ============================================
# 依赖检查
# ============================================

# 检查依赖中的安全漏洞
audit: check-cargo-audit
	@echo "=========================================="
	@echo "检查依赖中的安全漏洞"
	@echo "=========================================="
	@echo ""
	@cargo audit || exit 1
	@echo ""
	@echo "✓ 安全检查完成"

# 检查依赖更新
outdated: check-cargo-outdated
	@echo "=========================================="
	@echo "检查依赖更新"
	@echo "=========================================="
	@echo ""
	@cargo outdated || exit 1
	@echo ""
	@echo "✓ 依赖更新检查完成"

