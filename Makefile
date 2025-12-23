# 主 Makefile
# 包含所有功能模块（按依赖顺序）

# 声明所有伪目标（统一管理）
.PHONY: help dev release clean install update uninstall test test-all lint fix setup bloat check-rustfmt check-clippy coverage coverage-open coverage-ci coverage-trend check-docs-links

# 设置默认目标
.DEFAULT_GOAL := help

# 包含功能模块（按依赖顺序）
include make/Makefile.build.mk       # 1. 构建和安装（命令: dev, release, clean, install, update, uninstall | 变量: BINARY_NAME）
include make/Makefile.lint.mk        # 2. 代码检查（命令: lint, fix, check-rustfmt, check-clippy）
include make/Makefile.test.mk        # 3. 测试（命令: test, test-all, coverage, coverage-open, coverage-ci, coverage-trend）
include make/Makefile.tools.mk       # 4. 工具安装（命令: setup）
include make/Makefile.analyze.mk     # 5. 分析工具（命令: bloat）

# 集成所有模块的 help 信息
help:
	@echo "可用的 Make 目标："
	@echo ""
	$(HELP_BUILD)
	$(HELP_LINT)
	$(HELP_TEST)
	$(HELP_TOOLS)
	$(HELP_ANALYZE)
