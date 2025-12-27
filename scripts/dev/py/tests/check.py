"""测试覆盖率检查实现"""
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Optional

# 添加父目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_info, log_success, log_warning, log_error, log_break


def _check_tarpaulin_installed() -> bool:
    """检查 cargo-tarpaulin 是否安装"""
    try:
        result = subprocess.run(
            ['cargo', 'tarpaulin', '--version'],
            capture_output=True,
            text=True,
            check=False
        )
        return result.returncode == 0
    except FileNotFoundError:
        return False


def _get_tarpaulin_version() -> Optional[str]:
    """获取 cargo-tarpaulin 版本"""
    try:
        result = subprocess.run(
            ['cargo', 'tarpaulin', '--version'],
            capture_output=True,
            text=True,
            check=True
        )
        return result.stdout.strip()
    except Exception:
        return None


def _parse_coverage_from_json(json_output: bytes) -> float:
    """从 JSON 输出中解析覆盖率百分比"""
    output_str = json_output.decode('utf-8', errors='ignore')
    lines = output_str.strip().split('\n')

    # 尝试解析每一行，找到有效的 JSON
    for line in reversed(lines):
        line = line.strip()
        if not line:
            continue
        try:
            data = json.loads(line)
            if 'coverage_percent' in data:
                coverage = data['coverage_percent']
                if isinstance(coverage, (int, float)):
                    return float(coverage)
        except json.JSONDecodeError:
            continue

    # 如果单行解析失败，尝试解析整个输出
    try:
        data = json.loads(output_str)
        if 'coverage_percent' in data:
            coverage = data['coverage_percent']
            if isinstance(coverage, (int, float)):
                return float(coverage)
    except json.JSONDecodeError:
        pass

    return 0.0


def _output_ci_result(passed: bool, coverage: float, threshold: float) -> None:
    """输出 CI 模式结果到 GITHUB_OUTPUT"""
    output_file = os.environ.get('GITHUB_OUTPUT')
    if output_file:
        try:
            with open(output_file, 'a') as f:
                f.write(f'coverage_status={"pass" if passed else "fail"}\n')
                f.write(f'coverage_passed={str(passed).lower()}\n')
                f.write(f'coverage_value={coverage:.2f}\n')
                f.write(f'target_coverage={threshold:.2f}\n')
        except IOError as e:
            log_error(f"Failed to write to GITHUB_OUTPUT: {e}")


def _generate_report(passed: bool, coverage_value: float, threshold: float,
                     coverage_dir: str, output_path: Optional[str], check_type: str) -> None:
    """生成覆盖率检查报告"""
    from datetime import datetime

    log_info("生成覆盖率检查报告...")

    check_date = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    update_date = datetime.now().strftime("%Y-%m-%d")

    if passed:
        status_emoji = "✅"
        status_text = "通过"
        details = f"✅ 测试覆盖率已达到目标阈值（{threshold:.2f}%）。"
    else:
        gap = threshold - coverage_value
        status_emoji = "⚠️"
        status_text = "未达标"
        details = (
            f"⚠️  测试覆盖率未达到目标阈值。\n\n"
            f"**当前覆盖率**：{coverage_value:.2f}%\n"
            f"**目标覆盖率**：{threshold:.2f}%\n"
            f"**差距**：{gap:.2f}%\n\n"
            f"建议：\n"
            f"1. 检查未覆盖的代码模块\n"
            f"2. 为关键业务逻辑添加更多测试用例\n"
            f"3. 参考 `docs/requirements/QUICK_COVERAGE_MODULES.md` 了解快速提升覆盖率的模块"
        )

    report_content = f"""# 测试覆盖率检查报告

**检查日期**：{check_date}
**检查类型**：{check_type}

## 检查结果

### 覆盖率统计

- **当前覆盖率**：{coverage_value:.2f}%
- **目标覆盖率**：{threshold:.2f}%
- **检查状态**：{status_emoji} {status_text}

### 详细说明

{details}

## 报告文件

- HTML 报告已上传为 Artifact：`coverage-report`
- 查看详细覆盖率数据：下载 Artifact 中的 `{coverage_dir}/tarpaulin-report.html`

## 改进建议

1. 确保关键业务逻辑的测试覆盖率 > 90%
2. 定期审查未覆盖的代码模块
3. 为新功能添加相应的测试用例

参考文档：
- [测试覆盖率改进分析](docs/requirements/coverage-improvement.md)
- [快速覆盖率提升模块](docs/requirements/QUICK_COVERAGE_MODULES.md)
- [测试规范](docs/guidelines/testing.md)

---

**最后更新**: {update_date}
"""

    if output_path:
        output_file = Path(output_path)
        output_file.parent.mkdir(parents=True, exist_ok=True)
        output_file.write_text(report_content)
        log_success(f"报告已保存到: {output_path}")
    else:
        print(report_content)


def check(args) -> None:
    """检查测试覆盖率"""
    log_break('=')
    log_info("测试覆盖率检查")
    log_break('=')
    log_break()

    threshold = args.threshold or 80.0

    # 检查 cargo-tarpaulin 是否安装
    if not _check_tarpaulin_installed():
        log_error("cargo-tarpaulin 未安装")
        log_error("   安装方法: cargo install cargo-tarpaulin")
        if args.ci:
            _output_ci_result(False, 0.0, threshold)
            return
        sys.exit(1)

    log_success("cargo-tarpaulin 已安装")
    version = _get_tarpaulin_version()
    if version:
        log_info(f"   {version}")
    log_break()

    # 生成覆盖率报告
    log_info("生成覆盖率报告...")
    coverage_dir = "coverage"
    Path(coverage_dir).mkdir(parents=True, exist_ok=True)

    # 生成 JSON 格式的覆盖率报告
    try:
        result = subprocess.run(
            ['cargo', 'tarpaulin', '--out', 'Json', '--output-dir', coverage_dir],
            capture_output=True,
            check=False
        )

        if result.returncode != 0:
            log_error("生成覆盖率报告失败")
            log_error(f"   错误: {result.stderr.decode('utf-8', errors='ignore')}")
            if args.ci:
                _output_ci_result(False, 0.0, threshold)
                return
            sys.exit(1)
    except Exception as e:
        log_error(f"Failed to run cargo tarpaulin: {e}")
        if args.ci:
            _output_ci_result(False, 0.0, threshold)
            return
        sys.exit(1)

    # 解析覆盖率数据
    log_break()
    log_info("检查覆盖率阈值...")

    coverage_value = _parse_coverage_from_json(result.stdout)

    if coverage_value == 0.0:
        log_warning("无法解析覆盖率数据，请检查报告")
        if args.ci:
            _output_ci_result(False, 0.0, threshold)
            return
        sys.exit(1)

    log_info(f"当前覆盖率: {coverage_value:.2f}%")
    log_info(f"目标覆盖率: {threshold:.2f}%")

    passed = coverage_value >= threshold

    if passed:
        log_success(f"覆盖率已达到目标阈值 ({threshold:.2f}%)")
    else:
        gap = threshold - coverage_value
        log_warning(f"覆盖率未达到目标阈值 ({threshold:.2f}%)")
        log_warning(f"   当前覆盖率: {coverage_value:.2f}%")
        log_warning(f"   目标覆盖率: {threshold:.2f}%")
        log_warning(f"   差距: {gap:.2f}%")

    log_break()
    log_success("覆盖率检查完成")
    log_info(f"   报告位置: {coverage_dir}/tarpaulin-report.html")
    log_break()

    # 如果需要生成报告
    if args.output:
        _generate_report(
            passed, coverage_value, threshold, coverage_dir,
            args.output, args.check_type or "定期审查"
        )

    # CI 模式：输出到 GITHUB_OUTPUT
    if args.ci:
        _output_ci_result(passed, coverage_value, threshold)
        return

    # 本地模式：如果未通过则退出
    if not passed:
        sys.exit(1)


def main():
    """CLI 入口（可以直接运行此脚本）"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='check',
        description='Check test coverage'
    )
    parser.add_argument('--threshold', type=float, help='Coverage threshold (default: 80.0)')
    parser.add_argument('--ci', action='store_true', help='CI mode')
    parser.add_argument('--output', help='Output report file path')
    parser.add_argument('--check-type', help='Check type (default: 定期审查)')

    args = parser.parse_args()
    check(args)


if __name__ == '__main__':
    main()

