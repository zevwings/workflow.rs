# make/Makefile.test.mk
# 测试模块

# Help 信息
define HELP_TEST
	@echo "测试相关："
	@echo "  make test             - 运行测试"
	@echo "  make test-all         - 运行所有测试（包括被忽略的）"
	@echo ""
endef

# ============================================
# 测试相关目标
# ============================================

# 运行测试（包括单元测试、集成测试和文档测试）
test:
	@echo "运行测试..."
	cargo test

# 运行所有测试（包括被忽略的）
test-all:
	@echo "运行所有测试（包括被忽略的）..."
	cargo test -- --include-ignored
