"""测试覆盖率检查实现

用法:
  从已有报告检查（不运行 tarpaulin）:
    python3 scripts/dev/py/testing/coverage/check.py --report coverage/tarpaulin-report.json --threshold 75
  运行 tarpaulin 并检查:
    python3 scripts/dev/py/testing/coverage/check.py --threshold 75
"""
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, Optional, Tuple

# 添加父目录到路径（coverage/ 在 testing/ 下，需到 py/ 才能找到 utils）
sys.path.insert(0, str(Path(__file__).parent.parent.parent))
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


def _normalize_files_data(files_raw: Any) -> Dict[str, Dict[str, int]]:
    """将 tarpaulin report['files'] 标准化为 {file_path: {covered, coverable}}。

    支持两种格式:
    - 旧格式 (dict): {"/path/to/file.rs": {"covered": 10, "coverable": 20}}
    - 新格式 (list): [{"path": ["/", "crates", "app", "src", "main.rs"], "covered": 10, "coverable": 20}]
    """
    result: Dict[str, Dict[str, int]] = {}
    if isinstance(files_raw, dict):
        # 旧格式: dict
        for file_path, file_data in files_raw.items():
            path_str = file_path if isinstance(file_path, str) else "/".join(file_path)
            result[path_str] = {
                "covered": file_data.get("covered", 0) if isinstance(file_data, dict) else 0,
                "coverable": file_data.get("coverable", 0) if isinstance(file_data, dict) else 0,
            }
    elif isinstance(files_raw, list):
        # 新格式: list
        for item in files_raw:
            if not isinstance(item, dict):
                continue
            path_parts = item.get("path", [])
            path_str = "/".join(str(p) for p in path_parts) if path_parts else ""
            if path_str:
                result[path_str] = {
                    "covered": item.get("covered", 0),
                    "coverable": item.get("coverable", 0),
                }
    return result


def _load_report_file(report_path: str) -> Tuple[float, Dict[str, Dict[str, int]]]:
    """从 tarpaulin JSON 报告文件加载整体覆盖率与 files 数据。

    支持格式: report['coverage'] 或 report['coverage_percent']，以及 report['files']。
    """
    path = Path(report_path)
    if not path.exists():
        log_error(f"覆盖率报告文件不存在: {report_path}")
        return 0.0, {}
    try:
        with open(path, 'r', encoding='utf-8') as f:
            data = json.load(f)
    except json.JSONDecodeError as e:
        log_error(f"无法解析 JSON 报告: {e}")
        return 0.0, {}
    overall = float(
        data.get('coverage', data.get('coverage_percent', 0.0))
    )
    files_raw = data.get('files', {})
    files_data = _normalize_files_data(files_raw)
    return overall, files_data


def _analyze_per_crate_coverage(files_data: Dict[str, Dict[str, int]]) -> Dict[str, float]:
    """根据 report['files'] 计算各 crate 的覆盖率。"""
    crate_stats: Dict[str, Dict[str, int]] = {}
    for file_path, file_data in files_data.items():
        parts = file_path.replace('\\', '/').split('/')
        # 支持相对路径 "crates/xxx/..." 和绝对路径 ".../crates/xxx/..."
        if 'crates' in parts:
            idx = parts.index('crates')
            if idx + 1 < len(parts):
                crate_name = parts[idx + 1]
                if crate_name not in crate_stats:
                    crate_stats[crate_name] = {'covered': 0, 'total': 0}
                covered = file_data.get('covered', 0)
                total = file_data.get('coverable', 0)
                crate_stats[crate_name]['covered'] += covered
                crate_stats[crate_name]['total'] += total
    result = {}
    for name, stats in crate_stats.items():
        if stats['total'] > 0:
            result[name] = (stats['covered'] / stats['total']) * 100
    return result


def _print_per_crate_report(per_crate: Dict[str, float], threshold: float) -> None:
    """打印各 crate 覆盖率（与文档中 check_coverage.py 输出一致）。"""
    if not per_crate:
        return
    log_break()
    log_info("各 Crate 覆盖率:")
    log_info("-" * 50)
    for crate_name, coverage in sorted(per_crate.items(), key=lambda x: -x[1]):
        if coverage >= 85:
            icon = "🟢"
        elif coverage >= (threshold or 75):
            icon = "🟡"
        elif coverage >= 60:
            icon = "🟠"
        else:
            icon = "🔴"
        log_info(f"  {icon} {crate_name:20s} {coverage:6.2f}%")


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
    coverage_dir = "coverage"
    coverage_value: float
    files_data: Dict[str, Any] = {}

    if args.report:
        # 从已有报告文件读取，无需运行 tarpaulin
        log_info(f"从报告文件读取: {args.report}")
        coverage_value, files_data = _load_report_file(args.report)
        if coverage_value == 0.0 and files_data:
            total_c = sum(f.get('covered', 0) for f in files_data.values())
            total_t = sum(f.get('coverable', 0) for f in files_data.values())
            coverage_value = (total_c / total_t * 100) if total_t else 0.0
        if coverage_value == 0.0 and not files_data:
            if args.ci:
                _output_ci_result(False, 0.0, threshold)
                return
            sys.exit(1)
    else:
        # 需要运行 tarpaulin
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

        log_info("生成覆盖率报告...")
        Path(coverage_dir).mkdir(parents=True, exist_ok=True)
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

        coverage_value = _parse_coverage_from_json(result.stdout)
        if coverage_value == 0.0:
            log_warning("无法从 stdout 解析覆盖率，尝试读取输出文件...")
        # 尝试从写入的 JSON 文件读取以获取 per-crate
        json_report = Path(coverage_dir) / "tarpaulin-report.json"
        if json_report.exists():
            _, files_data = _load_report_file(str(json_report))
        if coverage_value == 0.0:
            if files_data:
                total_c = sum(f.get('covered', 0) for f in files_data.values())
                total_t = sum(f.get('coverable', 0) for f in files_data.values())
                coverage_value = (total_c / total_t * 100) if total_t else 0.0
            if coverage_value == 0.0:
                log_warning("无法解析覆盖率数据，请检查报告")
                if args.ci:
                    _output_ci_result(False, 0.0, threshold)
                    return
                sys.exit(1)
    # 若尚未有 files_data 且未从报告加载，可再试一次默认路径
    if not files_data and not args.report:
        json_report = Path(coverage_dir) / "tarpaulin-report.json"
        if json_report.exists():
            _, files_data = _load_report_file(str(json_report))

    log_break()
    log_info("检查覆盖率阈值...")

    log_info(f"当前覆盖率: {coverage_value:.2f}%")
    log_info(f"目标覆盖率: {threshold:.2f}%")

    per_crate = _analyze_per_crate_coverage(files_data) if files_data else {}
    if per_crate:
        _print_per_crate_report(per_crate, threshold)

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
    if args.report:
        log_info(f"   报告文件: {args.report}")
    else:
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
    parser.add_argument('--report', help='Use existing tarpaulin JSON report file (skip running tarpaulin)')
    parser.add_argument('--ci', action='store_true', help='CI mode')
    parser.add_argument('--output', help='Output report file path')
    parser.add_argument('--check-type', help='Check type (default: 定期审查)')

    args = parser.parse_args()
    check(args)


if __name__ == '__main__':
    main()

