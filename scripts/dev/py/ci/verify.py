"""CI 检查验证实现"""
import os
import sys
from pathlib import Path
from typing import Optional

# 添加父目录到路径，以便导入 utils
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_info, log_success, log_error


class JobResult:
    """Job 结果状态"""
    SUCCESS = "success"
    FAILURE = "failure"
    CANCELLED = "cancelled"
    SKIPPED = "skipped"
    UNKNOWN = "unknown"

    @staticmethod
    def from_str(s: str) -> str:
        """从字符串解析 Job 结果"""
        s_lower = s.lower()
        if s_lower == "success":
            return JobResult.SUCCESS
        elif s_lower == "failure":
            return JobResult.FAILURE
        elif s_lower == "cancelled":
            return JobResult.CANCELLED
        elif s_lower == "skipped":
            return JobResult.SKIPPED
        else:
            return JobResult.UNKNOWN


def _get_job_result(job: str) -> str:
    """获取 job 的结果

    在 GitHub Actions 中，job 结果通过 needs 上下文传递。
    这里我们尝试从环境变量读取（GitHub Actions 会自动设置）。
    """
    # 将 job 名称转换为环境变量格式（例如：check-lint -> CHECK_LINT_RESULT）
    env_var = f"{job.upper().replace('-', '_')}_RESULT"

    result_str = os.environ.get(env_var, '')
    if result_str:
        return JobResult.from_str(result_str)

    return JobResult.UNKNOWN


def verify(args) -> bool:
    """验证所有检查"""
    log_info("Checking job status:")

    # 优先级1: 如果 should_skip 为 true，说明应该跳过 CI
    should_skip = args.should_skip
    if should_skip is None:
        should_skip_env = os.environ.get('should_skip', '')
        if should_skip_env:
            should_skip = should_skip_env.lower() in ('true', '1', 'yes')
        else:
            should_skip = False

    if should_skip:
        log_success("CI should be skipped for version bump branch")
        return True

    # 解析 job 列表
    if args.jobs:
        jobs = [j.strip() for j in args.jobs.split(',')]
    else:
        jobs = ["check-lint", "tests", "doctests", "build"]

    # 检查各个 job 的状态
    all_passed = True
    all_skipped = True

    for job in jobs:
        result = _get_job_result(job)
        log_info(f"  {job}: {result}")

        if result == JobResult.SUCCESS:
            all_skipped = False
        elif result == JobResult.SKIPPED:
            # Skipped 也是允许的
            pass
        elif result in (JobResult.FAILURE, JobResult.CANCELLED):
            all_passed = False
            all_skipped = False
            log_error(f"{job} check failed: {result}")
        elif result == JobResult.UNKNOWN:
            # 如果 job 未运行，可能是被跳过了
            pass

    # 优先级2: 如果所有 job 都被跳过，说明应该跳过 CI
    if all_skipped:
        log_success("CI checks were skipped")
        return True

    # 检查是否有失败的 job
    if not all_passed:
        log_error("Some CI checks failed")
        sys.exit(1)

    log_success("All required checks passed or were skipped")
    return True


def main():
    """CLI 入口（可以直接运行此脚本）"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='verify',
        description='Verify CI job status'
    )
    parser.add_argument(
        '--jobs',
        help='Comma-separated list of job names (default: check-lint,tests,doctests,build)'
    )
    parser.add_argument(
        '--should-skip',
        type=lambda x: x.lower() in ('true', '1', 'yes'),
        help='Whether CI should be skipped'
    )

    args = parser.parse_args()
    verify(args)


if __name__ == '__main__':
    main()

