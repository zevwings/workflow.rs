# make/Makefile.test.mk
# 测试模块

# Help 信息
define HELP_TEST
	@echo "测试相关："
	@echo "  make test             - 运行测试"
	@echo "  make test-all         - 运行所有测试（包括被忽略的）"
	@echo ""
	@echo "覆盖率相关："
	@echo "  make coverage         - 生成覆盖率报告（HTML）"
	@echo "  make coverage-open    - 打开覆盖率报告"
	@echo "  make coverage-ci      - CI环境覆盖率检查"
	@echo "  make coverage-trend   - 查看覆盖率趋势"
	@echo ""
endef

# 运行测试（包括单元测试、集成测试和文档测试）
test:
	@echo "运行测试..."
	cargo test

# 运行所有测试（包括被忽略的）
test-all:
	@echo "运行所有测试（包括被忽略的）..."
	cargo test -- --include-ignored

# 在 release 模式下运行测试（覆盖 release 模式下的代码）
test-release:
	@echo "在 release 模式下运行测试..."
	cargo test --release

# 生成覆盖率报告（HTML格式）
coverage:
	@echo "生成覆盖率报告..."
	@if ! command -v cargo-tarpaulin >/dev/null 2>&1; then \
		echo "错误: cargo-tarpaulin 未安装"; \
		echo "请运行: cargo install cargo-tarpaulin"; \
		exit 1; \
	fi
	cargo tarpaulin --skip-clean --out Html --out Json --output-dir coverage \
		--exclude-files "src/bin/*" \
		--exclude-files "tests/*" \
		--exclude-files "benches/*" \
		--exclude-files "src/*/mod.rs"
	@echo "覆盖率报告已生成到 coverage/ 目录"
	@echo ""
	@echo "注意: 要覆盖 release 模式下的代码（如 logger/log_level.rs），请运行:"
	@echo "  cargo test --release"
	@echo "  或"
	@echo "  make test-release"

# 打开覆盖率报告
coverage-open:
	@if [ -f "coverage/tarpaulin-report.html" ]; then \
		open coverage/tarpaulin-report.html; \
	elif [ -f "coverage/index.html" ]; then \
		open coverage/index.html; \
	else \
		echo "错误: 覆盖率报告不存在，请先运行 make coverage"; \
		exit 1; \
	fi

# CI环境覆盖率检查（输出Lcov格式，适合CI/CD集成）
coverage-ci:
	@echo "运行CI覆盖率检查..."
	@if ! command -v cargo-tarpaulin >/dev/null 2>&1; then \
		echo "错误: cargo-tarpaulin 未安装"; \
		echo "请运行: cargo install cargo-tarpaulin"; \
		exit 1; \
	fi
	cargo tarpaulin --skip-clean --out Lcov --output-dir coverage \
		--exclude-files "src/bin/*" \
		--exclude-files "tests/*" \
		--exclude-files "benches/*" \
		--exclude-files "src/*/mod.rs"
	@echo "CI覆盖率报告已生成到 coverage/ 目录"

# 查看覆盖率趋势（需要历史数据支持）
coverage-trend:
	@echo "查看覆盖率趋势..."
	@if ! command -v cargo-tarpaulin >/dev/null 2>&1; then \
		echo "错误: cargo-tarpaulin 未安装"; \
		echo "请运行: cargo install cargo-tarpaulin"; \
		exit 1; \
	fi
	@if [ ! -d "coverage" ]; then \
		echo "错误: coverage 目录不存在，请先运行 make coverage"; \
		exit 1; \
	fi
	cargo tarpaulin --skip-clean --out Html --out Json --output-dir coverage \
		--exclude-files "src/bin/*" \
		--exclude-files "tests/*" \
		--exclude-files "benches/*" \
		--exclude-files "src/*/mod.rs"
	@echo "覆盖率报告已更新，运行 make coverage-open 查看"

