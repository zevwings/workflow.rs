# make/Makefile.analyze.mk
# 分析工具模块

# Help 信息
define HELP_ANALYZE
	@echo "分析工具相关："
	@echo "  make bloat            - 分析二进制文件大小（需要先运行 make setup）"
	@echo ""
endef

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

