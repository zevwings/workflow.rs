.PHONY: help build release clean install test lint fix setup uninstall dev bloat check-rustfmt check-clippy audit outdated check-cargo-audit check-cargo-outdated

# 默认目标
.DEFAULT_GOAL := help

# 包含功能模块（按依赖顺序）
include make/Makefile.build.mk       # 1. 构建和安装
include make/Makefile.lint.mk        # 2. 代码检查
include make/Makefile.test.mk        # 3. 测试
include make/Makefile.tools.mk       # 4. 工具安装
include make/Makefile.analyze.mk     # 5. 分析工具
include make/Makefile.deps.mk        # 6. 依赖管理

# 显示帮助信息
help:
	@echo "可用的 Make 目标："
	@echo ""
	$(HELP_BUILD)
	$(HELP_LINT)
	$(HELP_TEST)
	$(HELP_TOOLS)
	$(HELP_ANALYZE)
	$(HELP_DEPS)
