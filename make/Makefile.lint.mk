# make/Makefile.lint.mk
# 代码检查模块

# Help 信息
define HELP_LINT
	@echo "代码检查相关："
	@echo "  make lint             - 运行完整的代码检查（格式化 + Clippy + Check）"
	@echo "  make fix              - 自动修复代码问题（格式化 + Clippy + Cargo Fix）"
	@echo ""
endef

# ============================================
# 工具检查
# ============================================

# 检查 rustfmt 是否安装
check-rustfmt:
	@command -v cargo-fmt >/dev/null 2>&1 || rustup component list --toolchain stable | grep -q "rustfmt.*installed" || (echo "错误: rustfmt 未安装。运行 'make setup' 或 'rustup component add rustfmt'" && exit 1)

# 检查 clippy 是否安装
check-clippy:
	@command -v cargo-clippy >/dev/null 2>&1 || rustup component list --toolchain stable | grep -q "clippy.*installed" || (echo "错误: clippy 未安装。运行 'make setup' 或 'rustup component add clippy'" && exit 1)

# ============================================
# 代码检查
# ============================================

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
