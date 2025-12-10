//! 更新命令
//! 提供从 GitHub Releases 更新 Workflow CLI 的功能

use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, UNIX_EPOCH};

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::HeaderMap;
use serde::Deserialize;
use serde_json::Value;

use crate::base::dialog::ConfirmDialog;
use crate::base::http::client::HttpClient;
use crate::base::http::{
    response::HttpResponse, HttpMethod, HttpRetry, HttpRetryConfig, RequestConfig,
};
use crate::base::indicator::Spinner;
use crate::base::settings::paths::Paths;
use crate::base::settings::Settings;
use crate::base::shell::Detect;
use crate::base::util::{detect_release_platform, format_size, Checksum, Unzip};
use crate::rollback::RollbackManager;
use crate::{
    get_completion_files_for_shell, log_break, log_debug, log_error, log_info, log_success,
    log_warning,
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// GitHub Release 信息
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubRelease {
    tag_name: String,
    #[allow(dead_code)]
    assets: Vec<ReleaseAsset>,
}

/// Release 资源文件
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ReleaseAsset {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    browser_download_url: String,
}

/// 版本比较结果
enum VersionComparison {
    /// 当前版本已是最新
    UpToDate,
    /// 需要更新
    NeedsUpdate,
    /// 当前版本更新（降级）
    Downgrade,
}

/// 二进制文件状态
#[derive(Debug)]
struct BinaryStatus {
    name: String,
    path: String,
    exists: bool,
    executable: bool,
    #[allow(dead_code)]
    version: Option<String>,
    #[allow(dead_code)]
    working: bool,
}

/// 验证结果
#[derive(Debug)]
struct VerificationResult {
    #[allow(dead_code)]
    binaries: Vec<BinaryStatus>,
    #[allow(dead_code)]
    completions_installed: bool,
    all_checks_passed: bool,
}

/// 临时目录管理器
struct TempDirManager {
    temp_dir: PathBuf,
    extract_dir: PathBuf,
    archive_path: PathBuf,
}

impl TempDirManager {
    fn new(version: &str, platform: &str) -> Result<Self> {
        let temp_dir = env::temp_dir().join(format!("workflow-update-{}", version));

        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).context("Failed to remove existing temp directory")?;
        }

        fs::create_dir_all(&temp_dir).context("Failed to create temp directory")?;

        let archive_name = format!("workflow-{}-{}.tar.gz", version, platform);
        let archive_path = temp_dir.join(&archive_name);
        let extract_dir = temp_dir.join("extracted");

        Ok(Self {
            temp_dir,
            extract_dir,
            archive_path,
        })
    }
}

/// 更新命令
#[allow(dead_code)]
pub struct UpdateCommand;

#[allow(dead_code)]
impl UpdateCommand {
    // ==================== 版本管理 ====================

    /// 获取当前安装的版本号
    ///
    /// 从编译时嵌入的版本号获取（使用 env! 宏）。
    /// 注意：env!("CARGO_PKG_VERSION") 在编译时总是有值，所以总是可用。
    fn get_current_version() -> Result<Option<String>> {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        Ok(Some(VERSION.to_string()))
    }

    /// 比较两个版本号
    ///
    /// 返回版本比较结果。
    fn compare_versions(current: &str, target: &str) -> VersionComparison {
        let current_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();
        let target_parts: Vec<u32> = target.split('.').filter_map(|s| s.parse().ok()).collect();

        // 补齐到相同长度
        let max_len = current_parts.len().max(target_parts.len());
        let mut current_parts_padded = current_parts.clone();
        let mut target_parts_padded = target_parts.clone();
        current_parts_padded.resize(max_len, 0);
        target_parts_padded.resize(max_len, 0);

        // 逐级比较
        for (c, t) in current_parts_padded.iter().zip(target_parts_padded.iter()) {
            if c < t {
                return VersionComparison::NeedsUpdate;
            } else if c > t {
                return VersionComparison::Downgrade;
            }
        }

        VersionComparison::UpToDate
    }

    // ==================== 下载相关 ====================

    /// 获取速率限制重置时间
    fn get_rate_limit_reset_time(headers: &reqwest::header::HeaderMap) -> String {
        headers
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .map(|ts| {
                let reset = UNIX_EPOCH + Duration::from_secs(ts as u64);
                format!("Rate limit will reset at: {:?}", reset)
            })
            .unwrap_or_else(|| "Rate limit exceeded".to_string())
    }

    /// 处理 GitHub API 错误响应
    fn handle_github_api_error(response: &HttpResponse) -> Result<()> {
        let status = response.status;

        if (200..300).contains(&status) {
            return Ok(());
        }

        let error_msg = match status {
            403 => {
                let rate_limit_remaining = response
                    .headers
                    .get("x-ratelimit-remaining")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u32>().ok());

                if rate_limit_remaining == Some(0) {
                    let reset_time = Self::get_rate_limit_reset_time(&response.headers);
                    format!(
                        "Failed to fetch latest version: HTTP 403 (Rate limit exceeded)\n\
                        {}\n\
                        Tip: Configure a GitHub token to increase rate limit from 60/hour to 5000/hour.\n\
                        Run 'workflow setup' to configure your GitHub token.",
                        reset_time
                    )
                } else {
                    "Failed to fetch latest version: HTTP 403 (Forbidden)\n\
                    This may be due to repository access restrictions, network issues, or GitHub API restrictions.\n\
                    Tip: Configure a GitHub token to improve reliability.\n\
                    Run 'workflow setup' to configure your GitHub token.".to_string()
                }
            }
            404 => "Failed to fetch latest version: HTTP 404 (Not Found)\n\
                The repository or release may not exist, or you may not have access to it."
                .to_string(),
            429 => {
                let reset_time = Self::get_rate_limit_reset_time(&response.headers);
                format!(
                    "Failed to fetch latest version: HTTP 429 (Too Many Requests)\n\
                    {}\n\
                    Tip: Configure a GitHub token to increase rate limit from 60/hour to 5000/hour.\n\
                    Run 'workflow setup' to configure your GitHub token.",
                    reset_time
                )
            }
            _ => {
                format!(
                    "Failed to fetch latest version: HTTP {}\n\
                    Please check your network connection and try again.",
                    status
                )
            }
        };

        anyhow::bail!("{}", error_msg)
    }

    /// 第二步：获取版本号
    ///
    /// 如果指定了版本，使用指定版本；否则从 GitHub API 获取最新版本。
    fn get_version(version: Option<String>) -> Result<String> {
        match version {
            Some(v) => {
                log_info!("Using specified version: v{}", v);
                Ok(v)
            }
            None => {
                let url = "https://api.github.com/repos/zevwings/workflow.rs/releases/latest";
                let retry_config = HttpRetryConfig::new();

                let retry_result = Spinner::with("Fetching latest version...", || {
                    HttpRetry::retry(
                        || {
                            // GitHub API 要求必须包含 User-Agent 头
                            let mut headers = HeaderMap::new();
                            headers.insert(
                                "User-Agent",
                                "workflow-cli"
                                    .parse()
                                    .context("Failed to parse User-Agent header")?,
                            );

                            // 添加 Accept 头（GitHub API 推荐）
                            headers.insert(
                                "Accept",
                                "application/vnd.github+json"
                                    .parse()
                                    .context("Failed to parse Accept header")?,
                            );

                            // 可选地使用 GitHub token（如果用户已配置）
                            // 使用 token 可以提高速率限制（从 60/小时 提升到 5000/小时）
                            let settings = Settings::load();
                            if let Some(token) = settings.github.get_current_token() {
                                headers.insert(
                                    "Authorization",
                                    format!("Bearer {}", token)
                                        .parse()
                                        .context("Failed to parse Authorization header")?,
                                );
                                log_debug!("Using GitHub token for API request");
                            }

                            let client = HttpClient::global()?;
                            let config = RequestConfig::<Value, Value>::new().headers(&headers);
                            client
                                .get(url, config)
                                .context("Failed to fetch latest release from GitHub")
                        },
                        &retry_config,
                        "Fetching latest version information",
                    )
                })?;

                let response = retry_result.result;
                if !retry_result.succeeded_on_first_attempt {
                    log_success!(
                        "Fetching latest version information succeeded after {} retry attempts",
                        retry_result.retry_count
                    );
                }

                // 检查响应状态码并处理错误
                Self::handle_github_api_error(&response)?;

                let release: GitHubRelease = response.as_json()?;
                let version = release.tag_name.trim_start_matches('v').to_string();

                log_success!("  Latest version: v{}", version);
                Ok(version)
            }
        }
    }

    /// 第三步：构建下载 URL
    ///
    /// 根据平台和版本号拼接下载链接。
    fn build_download_url(version: &str, platform: &str) -> String {
        let extension = if platform.starts_with("Windows") {
            "zip"
        } else {
            "tar.gz"
        };
        format!(
            "https://github.com/zevwings/workflow.rs/releases/download/v{}/workflow-{}-{}.{}",
            version, version, platform, extension
        )
    }

    /// 第四步：下载文件
    ///
    /// 从指定 URL 下载文件到临时目录，显示下载进度。
    /// 支持重试机制，如果下载失败会自动重试。
    fn download_file(url: &str, output_path: &Path) -> Result<()> {
        log_info!("Downloading update package...");
        log_debug!("Download URL: {}", url);
        log_debug!("Saving to: {}", output_path.display());

        let retry_config = HttpRetryConfig::new();

        let retry_result = HttpRetry::retry(
            || {
                // 如果文件已存在且不完整，先删除它
                if output_path.exists() {
                    if let Err(e) = fs::remove_file(output_path) {
                        log_debug!("Failed to delete incomplete file: {}", e);
                    }
                }

                // 使用 get_stream 方法流式下载二进制文件
                let http_client = HttpClient::global()?;
                let mut response = http_client
                    .stream(HttpMethod::Get, url, RequestConfig::<Value, Value>::new())
                    .context("Failed to send HTTP request")?;

                if !response.status().is_success() {
                    anyhow::bail!("Download failed: HTTP {}", response.status());
                }

                // 获取文件总大小（如果可用）
                let total_size = response
                    .headers()
                    .get("content-length")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok());

                // 创建进度条
                let pb = if let Some(size) = total_size {
                    log_info!("File size: {}", format_size(size));
                    let pb = ProgressBar::new(size);
                    pb.set_style(
                        ProgressStyle::default_bar()
                            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                            .unwrap()
                            .progress_chars("#>-"),
                    );
                    pb
                } else {
                    let pb = ProgressBar::new_spinner();
                    pb.set_style(
                        ProgressStyle::default_spinner()
                            .template("{spinner:.green} [{elapsed_precise}] {bytes} downloaded...")
                            .unwrap(),
                    );
                    pb
                };

                let mut file = File::create(output_path)
                    .with_context(|| format!("Failed to create file: {}", output_path.display()))?;

                let mut buffer = vec![0u8; 8192];
                let mut downloaded_bytes = 0u64;

                loop {
                    let bytes_read =
                        response.read(&mut buffer).context("Failed to read response data")?;

                    if bytes_read == 0 {
                        break;
                    }

                    file.write_all(&buffer[..bytes_read]).context("Failed to write to file")?;

                    downloaded_bytes += bytes_read as u64;
                    pb.set_position(downloaded_bytes);
                }

                pb.finish_with_message("Download complete");
                Ok(())
            },
            &retry_config,
            "Downloading update package",
        )?;

        if !retry_result.succeeded_on_first_attempt {
            log_success!(
                "Downloading update package succeeded after {} retry attempts",
                retry_result.retry_count
            );
        }

        Ok(())
    }

    /// 第五步：解压文件
    ///
    /// 解压 tar.gz 或 zip 文件到指定目录。
    /// 在 macOS 上，解压后立即移除所有二进制文件的隔离属性，
    /// 确保安装时不会遇到 Gatekeeper 阻止。
    fn extract_archive(archive_path: &Path, output_dir: &Path) -> Result<()> {
        log_debug!("Extracting: {}", archive_path.display());
        log_debug!("Extracting to: {}", output_dir.display());

        // 根据文件扩展名选择解压方法
        let extension = archive_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        Spinner::with("Extracting update package...", || -> Result<()> {
            if extension == "zip" {
                Unzip::extract_zip(archive_path, output_dir)?;
            } else {
                // 默认使用 tar.gz 解压
                Unzip::extract_tar_gz(archive_path, output_dir)?;
            }
            Ok(())
        })?;

        log_success!("  Extraction complete");

        Ok(())
    }

    // ==================== 解压和安装 ====================

    /// 第六步：使用 ./install 安装二进制文件和补全脚本
    ///
    /// 在解压目录中运行 ./install 来安装二进制文件到系统目录和补全脚本。
    /// 默认行为是安装全部（二进制文件 + completions）。
    fn install(extract_dir: &Path) -> Result<()> {
        let install_binary = extract_dir.join(Paths::binary_name("install"));

        if !install_binary.exists() {
            anyhow::bail!(
                "Install binary does not exist: {}",
                install_binary.display()
            );
        }

        // 设置执行权限（仅 Unix）
        #[cfg(unix)]
        {
            Command::new("chmod")
                .arg("+x")
                .arg(&install_binary)
                .status()
                .context("Failed to set executable permission for install")?;
        }

        // 运行 ./install 安装二进制文件和补全脚本（默认安装全部）
        Spinner::with(
            "Installing binaries and completion scripts...",
            || -> Result<()> {
                let status = Command::new(&install_binary)
                    .current_dir(extract_dir)
                    .status()
                    .context("Failed to run install")?;

                if !status.success() {
                    anyhow::bail!("Installation failed");
                }
                Ok(())
            },
        )?;

        log_success!("  Binaries and completion scripts installation complete");
        Ok(())
    }

    // ==================== 验证相关 ====================

    // --- 基础验证工具方法 ---

    /// 检查文件是否可执行
    ///
    /// 检查文件是否存在且具有执行权限。
    #[cfg(unix)]
    fn check_executable(path: &Path) -> Result<bool> {
        if !path.exists() {
            return Ok(false);
        }

        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to get metadata for: {}", path.display()))?;

        let permissions = metadata.permissions();
        let mode = permissions.mode();

        // 检查是否有执行权限（owner, group, or others）
        Ok((mode & 0o111) != 0)
    }

    /// 检查文件是否可执行（Windows 版本）
    ///
    /// 在 Windows 上，通过文件扩展名判断是否可执行。
    #[cfg(windows)]
    fn check_executable(path: &Path) -> Result<bool> {
        if !path.exists() {
            return Ok(false);
        }

        // Windows 上通过扩展名判断可执行文件
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            // .exe, .bat, .cmd, .com, .ps1 等是可执行的
            Ok(ext_str == "exe"
                || ext_str == "bat"
                || ext_str == "cmd"
                || ext_str == "com"
                || ext_str == "ps1")
        } else {
            // 没有扩展名，可能是脚本文件，检查是否有执行权限（通过文件属性）
            // 在 Windows 上，我们假设文件存在就是可执行的（简化处理）
            Ok(true)
        }
    }

    // --- 高级验证方法 ---

    /// 验证单个二进制文件
    ///
    /// 只检查二进制文件是否存在和是否有执行权限，不执行任何命令。
    fn verify_single_binary(
        path: &str,
        name: &str,
        _expected_version: &str,
    ) -> Result<BinaryStatus> {
        let path_obj = Path::new(path);

        // 1. 检查文件是否存在
        let exists = path_obj.exists();
        if !exists {
            return Ok(BinaryStatus {
                name: name.to_string(),
                path: path.to_string(),
                exists: false,
                executable: false,
                version: None,
                working: false,
            });
        }

        // 2. 检查文件是否可执行
        let executable = Self::check_executable(path_obj)?;

        // 不再检查版本号和执行能力，避免 Gatekeeper 问题
        Ok(BinaryStatus {
            name: name.to_string(),
            path: path.to_string(),
            exists,
            executable,
            version: None,
            working: false,
        })
    }

    /// 验证所有二进制文件
    ///
    /// 验证 workflow 二进制文件。
    fn verify_binaries(target_version: &str) -> Result<Vec<BinaryStatus>> {
        let install_dir = Paths::binary_install_dir();
        let install_path = PathBuf::from(&install_dir);
        let binaries = Paths::command_names();
        let mut results = Vec::new();

        Spinner::with("Verifying binaries...", || -> Result<()> {
            for binary in binaries {
                let binary_name = Paths::binary_name(binary);
                let path = install_path.join(&binary_name);
                let status = Self::verify_single_binary(
                    &path.to_string_lossy(),
                    &binary_name,
                    target_version,
                )?;
                results.push(status);
            }
            Ok(())
        })?;

        Ok(results)
    }

    /// 验证补全脚本
    ///
    /// 只检查补全脚本文件是否存在，不验证文件内容。
    fn verify_completions() -> Result<bool> {
        let shell = match Detect::shell() {
            Ok(shell) => shell,
            Err(_) => {
                log_warning!(
                    "Unable to detect shell type, skipping completion script verification"
                );
                return Ok(false);
            }
        };

        let completion_dir = Paths::completion_dir()?;

        // 检查补全脚本目录是否存在
        if !completion_dir.exists() {
            log_warning!(
                "Completion script directory does not exist: {}",
                completion_dir.display()
            );
            return Ok(false);
        }

        // 检查补全脚本文件是否存在（根据 shell 类型）
        let commands = Paths::command_names();
        let shell_type_str = shell.to_string();
        let files = get_completion_files_for_shell(&shell_type_str, commands).unwrap_or_default();

        let mut all_valid = true;

        Spinner::with("Verifying completion scripts...", || -> Result<()> {
            for file in &files {
                let path = completion_dir.join(file);

                // 只检查文件是否存在
                if !path.exists() {
                    log_warning!("Completion script does not exist: {}", path.display());
                    all_valid = false;
                    continue;
                }

                log_debug!("Completion script verification passed: {}", path.display());
            }
            Ok(())
        })?;

        if all_valid {
            log_success!("  Completion script verification passed");
        } else {
            log_warning!("Some completion script verifications failed");
        }

        Ok(all_valid)
    }

    /// 验证安装结果
    ///
    /// 只验证文件是否存在和是否有执行权限，不执行任何命令验证。
    fn verify_installation(_target_version: &str) -> Result<VerificationResult> {
        log_break!();

        // 验证二进制文件（只检查存在性和执行权限）
        let binaries = Spinner::with("Verifying installation...", || {
            Self::verify_binaries(_target_version)
        })?;

        let mut all_binaries_ok = true;
        for binary in &binaries {
            if !binary.exists {
                log_warning!("Binary file does not exist: {}", binary.path);
                all_binaries_ok = false;
            } else if !binary.executable {
                log_warning!("Binary file is not executable: {}", binary.path);
                all_binaries_ok = false;
            } else {
                log_success!(
                    "  {} verification passed (file exists and is executable)",
                    binary.name
                );
            }
        }

        log_break!();

        // 验证补全脚本（只检查文件存在）
        let completions_installed = Self::verify_completions()?;
        if completions_installed {
            log_success!("  Completion script verification passed");
        } else {
            log_warning!("Completion script verification failed");
        }

        log_break!();

        // 汇总结果
        // 注意：即使 Gatekeeper 阻止执行，只要文件存在且可执行，就认为安装成功
        // 用户需要手动在系统设置中允许执行
        let all_checks_passed = all_binaries_ok && completions_installed;

        if all_checks_passed {
            log_success!("All verifications passed!");
        } else {
            log_warning!("Some verifications failed, please check the above warning messages");
        }

        Ok(VerificationResult {
            binaries,
            completions_installed,
            all_checks_passed,
        })
    }

    // ==================== 临时目录管理 ====================

    /// 清理更新过程中的临时资源
    fn cleanup_update_resources(
        temp_dir: &Path,
        backup_info: Option<&crate::rollback::BackupInfo>,
    ) {
        // 清理临时文件
        if let Err(e) = fs::remove_dir_all(temp_dir) {
            log_warning!("Failed to clean up temporary files: {}", e);
        }

        // 清理备份
        if let Some(backup) = backup_info {
            if let Err(e) = RollbackManager::cleanup_backup(backup) {
                log_warning!("Failed to clean up backup: {}", e);
            }
        }
    }

    // ==================== 主流程 ====================

    /// 执行完整的更新操作
    ///
    /// 按照以下步骤更新 Workflow CLI：
    /// 1. 检测平台
    /// 2. 获取目标版本号
    /// 3. 比较版本并获取用户确认
    /// 4. 创建备份
    /// 5. 准备临时目录和构建下载 URL
    /// 6. 下载文件
    /// 7. 验证文件完整性
    /// 8. 解压文件
    /// 9. 使用 ./install 安装二进制文件和补全脚本
    /// 10. 验证安装结果
    pub fn update(version: Option<String>) -> Result<()> {
        log_info!("Starting Workflow CLI update...");
        log_break!();

        // 获取当前版本
        let current_version = Self::get_current_version()?;
        if let Some(ref current) = current_version {
            log_info!("Current version: v{}", current);
        } else {
            log_warning!("Unable to detect current version, will continue update process");
        }
        log_break!();

        // 第一步：检测平台
        let platform = detect_release_platform()?;
        log_info!("Detected platform: {}", platform);
        log_break!();

        // 第二步：获取目标版本号
        let target_version = Self::get_version(version)?;

        // 比较版本
        if let Some(ref current) = current_version {
            match Self::compare_versions(current, &target_version) {
                VersionComparison::UpToDate => {
                    log_success!("Already at latest version (v{}), no update needed", current);
                    return Ok(());
                }
                VersionComparison::NeedsUpdate => {
                    log_info!("New version found: v{} -> v{}", current, target_version);
                }
                VersionComparison::Downgrade => {
                    log_warning!(
                        "Target version (v{}) is lower than current version (v{})",
                        target_version,
                        current
                    );
                    log_warning!("  This will perform a downgrade operation");
                }
            }
        } else {
            log_info!("Target version: v{}", target_version);
        }
        log_break!();

        // 第三步：获取用户确认
        let confirm_message = if let Some(ref current) = current_version {
            format!(
                "Are you sure you want to update Workflow CLI?\n  Current version: v{}\n  Target version: v{}",
                current, target_version
            )
        } else {
            format!(
                "Are you sure you want to update Workflow CLI to v{}?",
                target_version
            )
        };

        if !ConfirmDialog::new(&confirm_message)
            .with_default(true)
            .with_cancel_message("Update cancelled")
            .prompt()?
        {
            return Ok(());
        }
        log_break!();

        // 第四步：创建备份（在更新前备份当前版本）
        let backup_info = match RollbackManager::create_backup() {
            Ok(backup) => {
                log_break!();
                Some(backup)
            }
            Err(e) => {
                log_warning!("Failed to create backup: {}", e);
                log_warning!("  Will continue update, but cannot rollback on failure");
                log_warning!("  If update fails, manual recovery may be required");
                log_break!();
                None
            }
        };

        // 第五步：准备临时目录和构建下载 URL
        let temp_manager = TempDirManager::new(&target_version, &platform)?;
        let download_url = Self::build_download_url(&target_version, &platform);
        log_info!("Download URL: {}", download_url);
        log_break!();

        // 执行更新操作（可回滚）
        let update_result = (|| -> Result<()> {
            // 第六步：下载文件
            Self::download_file(&download_url, &temp_manager.archive_path)?;
            log_break!();

            // 第七步：验证文件完整性
            let checksum_url = Checksum::build_url(&download_url);

            // 下载校验和文件（使用 http 模块）
            let http_client = HttpClient::global()?;
            let retry_config = HttpRetryConfig::new();

            // 尝试下载校验和文件，如果不存在（404）则跳过验证
            match HttpRetry::retry(
                || {
                    let config = RequestConfig::<Value, Value>::new();
                    let response = http_client.get(&checksum_url, config)?;
                    // 使用 ensure_success_with 统一处理 404 错误
                    let response = response.ensure_success_with(|r| {
                        if r.status == 404 {
                            anyhow::anyhow!("Checksum file not found (404)")
                        } else {
                            anyhow::anyhow!(
                                "HTTP request failed with status {}: {}",
                                r.status,
                                r.status_text
                            )
                        }
                    })?;
                    response.as_text()
                },
                &retry_config,
                "Downloading checksum file",
            ) {
                Ok(retry_result) => {
                    let checksum_content = retry_result.result;
                    if !retry_result.succeeded_on_first_attempt {
                        log_success!(
                            "Downloading checksum file succeeded after {} retry attempts",
                            retry_result.retry_count
                        );
                    }
                    // 解析哈希值（使用 checksum 模块）
                    let expected_hash = Checksum::parse_hash_from_content(&checksum_content)
                        .context("Failed to parse checksum file")?;

                    // 验证文件（使用 checksum 模块）
                    Checksum::verify(&temp_manager.archive_path, &expected_hash)?;
                }
                Err(e) => {
                    // 如果是 404 错误，跳过验证但给出警告
                    if e.to_string().contains("404") || e.to_string().contains("not found") {
                        log_warning!("Checksum file not found, skipping integrity verification");
                        log_warning!("  Checksum URL: {}", checksum_url);
                        log_warning!(
                            "  This may indicate the release does not include checksum files"
                        );
                        log_warning!("  Proceeding with update without verification...");

                        // 仍然计算并显示文件的 SHA256，供用户参考
                        if let Ok(actual_hash) =
                            Checksum::calculate_file_sha256(&temp_manager.archive_path)
                        {
                            log_info!("Downloaded file SHA256: {}", actual_hash);
                        }
                    } else {
                        // 其他错误，仍然返回错误
                        return Err(e.context("Failed to download checksum file"));
                    }
                }
            }
            log_break!();

            // 第八步：解压文件
            Self::extract_archive(&temp_manager.archive_path, &temp_manager.extract_dir)?;
            log_break!();

            // 第九步：使用 ./install 安装二进制文件和补全脚本（默认安装全部）
            // 注意：隔离属性已在解压时移除，安装后的文件不应该有隔离属性
            Self::install(&temp_manager.extract_dir)?;
            log_break!();

            // 第十步：验证安装结果（只检查文件存在和执行权限）
            let verification_result = Self::verify_installation(&target_version)?;
            log_break!();

            // 如果验证失败，说明文件不存在或没有执行权限，这是真正的安装失败
            if !verification_result.all_checks_passed {
                anyhow::bail!("Installation verification failed, some checks did not pass");
            }

            Ok(())
        })();

        // 处理更新结果
        match update_result {
            Ok(()) => {
                // 更新成功，清理临时文件和备份
                Self::cleanup_update_resources(
                    &temp_manager.temp_dir,
                    backup_info.as_ref().map(|b| &b.backup_info),
                );
                log_success!("Workflow CLI update complete! All verifications passed.");
                Ok(())
            }
            Err(e) => {
                // 更新失败，执行回滚
                log_error!("Update failed: {}", e);
                log_break!();

                if let Some(ref backup) = backup_info {
                    log_warning!("Update failed, rolling back to previous version...");
                    log_break!();

                    match RollbackManager::rollback(&backup.backup_info) {
                        Ok(rollback_result) => {
                            // 显示恢复的二进制文件
                            if !rollback_result.restored_binaries.is_empty() {
                                log_info!("Restoring binary files...");
                                for binary in &rollback_result.restored_binaries {
                                    log_info!("  Restored: {}", binary);
                                }
                            }

                            // 显示失败的二进制文件
                            if !rollback_result.failed_binaries.is_empty() {
                                log_warning!("Failed to restore some binary files:");
                                for (binary, error) in &rollback_result.failed_binaries {
                                    log_warning!("  {}: {}", binary, error);
                                }
                            }

                            // 显示恢复的补全脚本
                            if !rollback_result.restored_completions.is_empty() {
                                log_info!("Restoring completion scripts...");
                                for completion in &rollback_result.restored_completions {
                                    log_info!("  Restored: {}", completion);
                                }
                            }

                            // 显示失败的补全脚本
                            if !rollback_result.failed_completions.is_empty() {
                                log_warning!("Failed to restore some completion scripts:");
                                for (completion, error) in &rollback_result.failed_completions {
                                    log_warning!("  {}: {}", completion, error);
                                }
                            }

                            // 处理 shell 重新加载
                            if let Some(reload_success) = rollback_result.shell_reload_success {
                                if reload_success {
                                    log_info!(
                                        "Note: Configuration has been reloaded in subprocess"
                                    );
                                    if let Some(ref config_file) = rollback_result.shell_config_file
                                    {
                                        log_info!(
                                            "  If completion is not working, please run manually: source {}",
                                            config_file.display()
                                        );
                                    }
                                } else {
                                    log_warning!("Failed to reload shell configuration");
                                    if let Some(ref config_file) = rollback_result.shell_config_file
                                    {
                                        log_info!(
                                            "Please run manually: source {}",
                                            config_file.display()
                                        );
                                    }
                                }
                            } else {
                                log_info!(
                                    "Please manually reload shell config file to enable completion"
                                );
                            }

                            log_success!("Rollback completed");
                            log_break!();

                            // 回滚成功后清理备份
                            if let Err(cleanup_err) =
                                RollbackManager::cleanup_backup(&backup.backup_info)
                            {
                                log_warning!("Failed to clean up backup: {}", cleanup_err);
                            }
                        }
                        Err(rollback_err) => {
                            log_error!("Rollback failed: {}", rollback_err);
                            log_error!("  System may be in an inconsistent state");
                            log_error!("  Please manually check and restore files");
                            log_error!(
                                "  Backup location: {}",
                                backup.backup_info.backup_dir.display()
                            );
                        }
                    }
                } else {
                    log_error!("Unable to rollback: no available backup");
                    log_error!("  Please manually check and restore files");
                }

                // 清理临时资源
                Self::cleanup_update_resources(
                    &temp_manager.temp_dir,
                    backup_info.as_ref().map(|b| &b.backup_info),
                );
                Err(e.context("Update failed"))
            }
        }
    }
}
