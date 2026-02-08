//! 更新命令类型定义

use std::env;
use std::fs;
use std::path::PathBuf;

use toolkit::directory;

// ============================================================================
// 版本比较
// ============================================================================

/// 版本比较结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionComparison {
    /// 当前版本已是最新
    UpToDate,
    /// 需要更新
    NeedsUpdate,
    /// 当前版本更新（降级）
    Downgrade,
}

// ============================================================================
// 验证结果
// ============================================================================

/// 验证结果
#[derive(Debug)]
pub struct VerificationResult {
    /// 所有检查是否通过
    pub all_checks_passed: bool,
}

// ============================================================================
// 临时目录管理
// ============================================================================

/// 临时目录管理器
///
/// 管理更新过程中的临时文件和目录。
pub struct TempDirManager {
    /// 临时目录路径
    pub temp_dir: PathBuf,
    /// 解压目录路径
    pub extract_dir: PathBuf,
    /// 归档文件路径
    pub archive_path: PathBuf,
}

impl TempDirManager {
    /// 创建新的临时目录管理器
    ///
    /// # 参数
    ///
    /// * `version` - 目标版本号
    /// * `platform` - 平台标识
    pub fn new(
        version: impl AsRef<str>,
        platform: impl AsRef<str>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let version = version.as_ref();
        let platform = platform.as_ref();
        let temp_dir = env::temp_dir().join(format!("workflow-update-{}", version));

        // 如果临时目录已存在，先删除
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)
                .map_err(|e| format!("Failed to remove existing temp directory: {}", e))?;
        }

        // 创建临时目录
        directory::ensure_exists(&temp_dir)?;

        // 根据平台确定归档文件扩展名
        let extension = if platform.starts_with("Windows") {
            "zip"
        } else {
            "tar.gz"
        };

        let archive_name = format!("workflow-{}-{}.{}", version, platform, extension);
        let archive_path = temp_dir.join(&archive_name);
        let extract_dir = temp_dir.join("extracted");

        Ok(Self {
            temp_dir,
            extract_dir,
            archive_path,
        })
    }

    /// 清理临时目录
    pub fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.temp_dir.exists() {
            fs::remove_dir_all(&self.temp_dir)
                .map_err(|e| format!("Failed to clean up temp directory: {}", e))?;
        }
        Ok(())
    }
}

impl Drop for TempDirManager {
    fn drop(&mut self) {
        // 尝试清理，忽略错误
        let _ = self.cleanup();
    }
}

// ============================================================================
// 常量定义
// ============================================================================

/// GitHub API 基础 URL
pub const GITHUB_API_BASE: &str = "https://api.github.com";

/// GitHub 下载基础 URL
pub const GITHUB_DOWNLOAD_BASE: &str = "https://github.com";

/// 仓库所有者
pub const REPO_OWNER: &str = "zevwings";

/// 仓库名称
pub const REPO_NAME: &str = "workflow.rs";
