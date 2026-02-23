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
	@echo "  make coverage-detailed - 生成详细覆盖率报告"
	@echo "  make coverage-check    - 检查覆盖率是否达标（≥75%）"
	@echo "  make coverage-ci      - CI环境覆盖率检查"
	@echo "  make coverage-trend   - 分析覆盖率历史趋势"
	@echo "  make coverage-open    - 打开覆盖率报告"
	@echo "  make coverage-clean   - 清理覆盖率数据"
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

# ============================================
# 覆盖率相关目标
# ============================================

# 公共 tarpaulin 排除参数
TARPAULIN_EXCLUDE_BASE := \
	--exclude-files "src/bin/*" \
	--exclude-files "tests/*" \
	--exclude-files "benches/*" \
	--exclude-files "src/*/mod.rs"
TARPAULIN_EXCLUDE_EXT := \
	$(TARPAULIN_EXCLUDE_BASE) \
	--exclude-files "*/testing/*" \
	--exclude-files "*/mock/*"

# 检查 cargo-tarpaulin 是否安装
check-tarpaulin:
	@if ! command -v cargo-tarpaulin >/dev/null 2>&1; then \
		echo "错误: cargo-tarpaulin 未安装"; \
		echo ""; \
		echo "请运行以下命令安装:"; \
		echo "  make setup"; \
		echo ""; \
		echo "或者手动安装:"; \
		echo "  cargo install cargo-tarpaulin"; \
		exit 1; \
	fi

# 生成覆盖率报告（HTML格式）
coverage: check-tarpaulin
	@echo "生成覆盖率报告..."
	cargo tarpaulin --skip-clean --out Html --out Json --output-dir coverage \
		$(TARPAULIN_EXCLUDE_BASE)
	@echo "覆盖率报告已生成到 coverage/ 目录"

# 打开覆盖率报告
coverage-open:
	@if [ -f "coverage/tarpaulin-report.html" ]; then \
		open coverage/tarpaulin-report.html 2>/dev/null || xdg-open coverage/tarpaulin-report.html 2>/dev/null || echo "请手动打开 coverage/tarpaulin-report.html"; \
	elif [ -f "coverage/index.html" ]; then \
		open coverage/index.html 2>/dev/null || xdg-open coverage/index.html 2>/dev/null || echo "请手动打开 coverage/index.html"; \
	else \
		echo "错误: 覆盖率报告不存在，请先运行 make coverage"; \
		exit 1; \
	fi

# CI环境覆盖率检查（输出Lcov格式，适合CI/CD集成）
coverage-ci: check-tarpaulin
	@echo "运行CI覆盖率检查..."
	cargo tarpaulin --skip-clean --out Lcov --output-dir coverage \
		$(TARPAULIN_EXCLUDE_BASE)
	@echo "CI覆盖率报告已生成到 coverage/ 目录"

# 覆盖率报告（详细）
coverage-detailed: check-tarpaulin
	@echo "🔍 生成详细覆盖率报告..."
	@mkdir -p coverage
	cargo tarpaulin \
		--skip-clean \
		--out Html \
		--out Json \
		--out Lcov \
		--output-dir coverage \
		$(TARPAULIN_EXCLUDE_EXT) \
		--timeout 300 \
		--verbose
	@echo "✅ 报告已生成到 coverage/ 目录"

# 覆盖率阈值检查
coverage-check: check-tarpaulin
	@echo "📊 检查覆盖率是否达标..."
	@mkdir -p coverage
	@cargo tarpaulin \
		--skip-clean \
		--out Json \
		--output-dir coverage \
		$(TARPAULIN_EXCLUDE_EXT) \
		--timeout 300 \
		> /dev/null 2>&1
	@python3 scripts/dev/py/testing/coverage/check.py --report coverage/tarpaulin-report.json --threshold 75
	@echo ""

# 覆盖率趋势分析
coverage-trend: check-tarpaulin
	@echo "📈 生成覆盖率趋势报告..."
	@mkdir -p coverage/history
	@cargo tarpaulin \
		--skip-clean \
		--out Json \
		--output-dir coverage \
		$(TARPAULIN_EXCLUDE_EXT) \
		--timeout 300 \
		> /dev/null 2>&1 || (echo "❌ 覆盖率采集失败（可能是测试失败）。请先运行 make test 确保所有测试通过"; exit 1)
	@cp coverage/tarpaulin-report.json coverage/history/$$(date +%Y%m%d_%H%M%S).json
	@python3 scripts/dev/py/testing/coverage/trends.py coverage/history/
	@echo ""

# 清理覆盖率数据
coverage-clean:
	@echo "🧹 清理覆盖率数据..."
	@rm -rf coverage/
	@echo "✅ 覆盖率数据已清理"

.PHONY: coverage coverage-detailed coverage-check coverage-ci coverage-trend coverage-open coverage-clean
