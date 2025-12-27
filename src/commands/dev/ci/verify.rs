//! CI 检查验证实现

use crate::{log_error, log_info, log_success};
use color_eyre::Result;
use std::env;

/// CI 检查验证命令
pub struct CiVerifyCommand {
    jobs: Vec<String>,
    should_skip: Option<bool>,
}

impl CiVerifyCommand {
    /// 创建新的 CI 检查验证命令
    pub fn new(jobs: Option<String>, should_skip: Option<bool>) -> Self {
        let jobs_list = if let Some(ref jobs_str) = jobs {
            jobs_str.split(',').map(|s| s.trim().to_string()).collect()
        } else {
            vec![
                "check-lint".to_string(),
                "tests".to_string(),
                "doctests".to_string(),
                "build".to_string(),
            ]
        };

        Self {
            jobs: jobs_list,
            should_skip,
        }
    }

    /// 验证所有检查
    pub fn verify(&self) -> Result<bool> {
        log_info!("📊 Checking job status:");

        // 优先级1: 如果 should_skip 为 true，说明应该跳过 CI
        let should_skip = self.should_skip.unwrap_or_else(|| {
            env::var("should_skip")
                .ok()
                .and_then(|v| v.parse::<bool>().ok())
                .unwrap_or(false)
        });

        if should_skip {
            log_success!("CI should be skipped for version bump branch");
            return Ok(true);
        }

        // 检查各个 job 的状态
        let mut all_passed = true;
        let mut all_skipped = true;

        for job in &self.jobs {
            let result = self.get_job_result(job);
            log_info!("  {}: {:?}", job, result);

            match result {
                JobResult::Success => {
                    all_skipped = false;
                }
                JobResult::Skipped => {
                    // Skipped 也是允许的
                }
                JobResult::Failure | JobResult::Cancelled => {
                    all_passed = false;
                    all_skipped = false;
                    log_error!("{} check failed: {:?}", job, result);
                }
                JobResult::Unknown => {
                    // 如果 job 未运行，可能是被跳过了
                }
            }
        }

        // 优先级2: 如果所有 job 都被跳过，说明应该跳过 CI
        if all_skipped {
            log_success!("CI checks were skipped");
            return Ok(true);
        }

        // 检查是否有失败的 job
        if !all_passed {
            log_error!("Some CI checks failed");
            return Err(color_eyre::eyre::eyre!("CI checks failed"));
        }

        log_success!("All required checks passed or were skipped");
        Ok(true)
    }

    /// 获取 job 的结果
    fn get_job_result(&self, job: &str) -> JobResult {
        // 在 GitHub Actions 中，job 结果通过 needs 上下文传递
        // 这里我们尝试从环境变量读取（GitHub Actions 会自动设置）
        let env_var = format!("{}_RESULT", job.to_uppercase().replace('-', "_"));

        if let Ok(result_str) = env::var(&env_var) {
            return JobResult::from_str(&result_str);
        }

        // 尝试从 GITHUB_OUTPUT 读取（如果之前有输出）
        // 注意：在实际 GitHub Actions 中，needs 上下文会自动提供这些值
        // 这里我们模拟读取过程
        JobResult::Unknown
    }
}

/// Job 结果状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JobResult {
    Success,
    Failure,
    Cancelled,
    Skipped,
    Unknown,
}

impl JobResult {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "success" => JobResult::Success,
            "failure" => JobResult::Failure,
            "cancelled" => JobResult::Cancelled,
            "skipped" => JobResult::Skipped,
            _ => JobResult::Unknown,
        }
    }
}
