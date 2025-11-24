//! 更新命令
//! 提供从 GitHub Releases 更新 Workflow CLI 的功能

use crate::base::http::client::HttpClient;
use crate::base::http::{HttpMethod, HttpRetry, HttpRetryConfig, RequestConfig};
use crate::base::settings::paths::Paths;
use crate::base::shell::Detect;
use crate::base::util::{confirm, Checksum, Unzip};
use crate::rollback::RollbackManager;
use crate::{
    get_completion_files_for_shell, log_break, log_debug, log_error, log_info, log_success,
    log_warning,
};
use anyhow::{Context, Result};
use clap_complete::shells::Shell;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use serde_json::Value;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

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
    version: Option<String>,
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

/// 更新命令
#[allow(dead_code)]
pub struct UpdateCommand;

#[allow(dead_code)]
impl UpdateCommand {
    // ==================== 版本管理 ====================

    /// 获取当前安装的版本号
    ///
    /// 尝试多种方法获取当前版本：
    /// 1. 从环境变量 CARGO_PKG_VERSION（编译时注入）
    /// 2. 运行 `workflow --version` 命令获取版本
    /// 3. 从 Cargo.toml 读取（开发环境）
    fn get_current_version() -> Result<Option<String>> {
        // 方法 1: 尝试从环境变量获取（编译时注入）
        if let Ok(version) = std::env::var("CARGO_PKG_VERSION") {
            return Ok(Some(version));
        }

        // 方法 2: 尝试运行 workflow --version 命令
        if let Ok(output) = Command::new("workflow").arg("--version").output() {
            if output.status.success() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                // 解析版本号（格式可能是 "workflow 1.1.2" 或 "1.1.2"）
                if let Some(version) = version_str
                    .split_whitespace()
                    .last()
                    .and_then(|s| s.strip_prefix('v'))
                    .or_else(|| version_str.split_whitespace().last())
                {
                    return Ok(Some(version.to_string()));
                }
            }
        }

        // 方法 3: 尝试从 Cargo.toml 读取（开发环境）
        let cargo_toml_path = std::env::current_dir()
            .ok()
            .and_then(|dir| {
                // 尝试多个可能的路径
                let paths = [
                    dir.join("Cargo.toml"),
                    dir.join("../Cargo.toml"),
                    dir.join("../../Cargo.toml"),
                ];
                paths.iter().find(|p| p.exists()).cloned()
            })
            .or_else(|| {
                // 如果当前目录找不到，尝试从可执行文件位置推断
                std::env::current_exe()
                    .ok()
                    .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
                    .and_then(|mut path| {
                        // 向上查找 Cargo.toml
                        for _ in 0..5 {
                            let cargo_toml = path.join("Cargo.toml");
                            if cargo_toml.exists() {
                                return Some(cargo_toml);
                            }
                            path = path.parent()?.to_path_buf();
                        }
                        None
                    })
            });

        if let Some(cargo_toml) = cargo_toml_path {
            let content = fs::read_to_string(&cargo_toml)
                .with_context(|| format!("Failed to read Cargo.toml: {}", cargo_toml.display()))?;

            for line in content.lines() {
                if line.trim().starts_with("version =") {
                    if let Some(start) = line.find('"') {
                        if let Some(end) = line[start + 1..].find('"') {
                            let version = &line[start + 1..start + 1 + end];
                            return Ok(Some(version.to_string()));
                        }
                    }
                }
            }
        }

        // 如果都找不到，返回 None（允许继续更新流程）
        Ok(None)
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

    // ==================== 平台检测 ====================

    /// 第一步：检测当前平台
    ///
    /// 返回平台标识符，用于匹配 GitHub Releases 中的资源文件。
    fn detect_platform() -> Result<String> {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;

        match (os, arch) {
            ("macos", "x86_64") => Ok("macOS-Intel".to_string()),
            ("macos", "aarch64") => Ok("macOS-AppleSilicon".to_string()),
            _ => anyhow::bail!("Unsupported platform: {}-{}", os, arch),
        }
    }

    // ==================== 下载相关 ====================

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
                log_info!("Fetching latest version...");

                let url = "https://api.github.com/repos/zevwings/workflow.rs/releases/latest";
                let retry_config = HttpRetryConfig::new();

                let response = HttpRetry::retry(
                    || {
                        let client = HttpClient::global()?;
                        client
                            .get(url, RequestConfig::<Value, Value>::new())
                            .context("Failed to fetch latest release from GitHub")
                    },
                    &retry_config,
                    "Fetching latest version information",
                )?;

                if response.status < 200 || response.status >= 300 {
                    anyhow::bail!("Failed to fetch latest version: HTTP {}", response.status);
                }

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
        format!(
            "https://github.com/zevwings/workflow.rs/releases/download/v{}/workflow-{}-{}.tar.gz",
            version, version, platform
        )
    }

    /// 格式化文件大小
    ///
    /// 将字节数格式化为人类可读的格式（B, KB, MB, GB）。
    fn format_size(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
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

        HttpRetry::retry(
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
                    log_info!("File size: {}", Self::format_size(size));
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
                    let bytes_read = response
                        .read(&mut buffer)
                        .context("Failed to read response data")?;

                    if bytes_read == 0 {
                        break;
                    }

                    file.write_all(&buffer[..bytes_read])
                        .context("Failed to write to file")?;

                    downloaded_bytes += bytes_read as u64;
                    pb.set_position(downloaded_bytes);
                }

                pb.finish_with_message("Download complete");
                Ok(())
            },
            &retry_config,
            "Downloading update package",
        )
    }

    /// 第五步：解压文件
    ///
    /// 解压 tar.gz 文件到指定目录。
    fn extract_archive(tar_gz_path: &Path, output_dir: &Path) -> Result<()> {
        log_info!("Extracting update package...");
        log_debug!("Extracting: {}", tar_gz_path.display());
        log_debug!("Extracting to: {}", output_dir.display());

        Unzip::extract_tar_gz(tar_gz_path, output_dir)?;

        log_success!("  Extraction complete");
        Ok(())
    }

    // ==================== 解压和安装 ====================

    /// 第六步：使用 ./install 安装二进制文件和补全脚本
    ///
    /// 在解压目录中运行 ./install 来安装二进制文件到 /usr/local/bin 和补全脚本。
    /// 默认行为是安装全部（二进制文件 + completions）。
    fn install(extract_dir: &Path) -> Result<()> {
        log_info!("Installing binaries and completion scripts...");

        let install_binary = extract_dir.join("install");

        if !install_binary.exists() {
            anyhow::bail!(
                "Install binary does not exist: {}",
                install_binary.display()
            );
        }

        // 设置执行权限
        Command::new("chmod")
            .arg("+x")
            .arg(&install_binary)
            .status()
            .context("Failed to set executable permission for install")?;

        // 运行 ./install 安装二进制文件和补全脚本（默认安装全部）
        let status = Command::new(&install_binary)
            .current_dir(extract_dir)
            .status()
            .context("Failed to run install")?;

        if !status.success() {
            anyhow::bail!("Installation failed");
        }

        log_success!("  Binaries and completion scripts installation complete");
        Ok(())
    }

    // ==================== 验证相关 ====================

    // --- 基础验证工具方法 ---

    /// 检查文件是否可执行
    ///
    /// 检查文件是否存在且具有执行权限。
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

    /// 获取二进制文件的版本号
    ///
    /// 运行 `binary_name --version` 命令并解析版本号。
    fn get_binary_version(binary_name: &str) -> Result<Option<String>> {
        let output = Command::new(binary_name).arg("--version").output();

        match output {
            Ok(result) if result.status.success() => {
                let version_str = String::from_utf8_lossy(&result.stdout);
                // 解析版本号（格式可能是 "workflow 1.1.2" 或 "1.1.2"）
                let version = version_str
                    .split_whitespace()
                    .last()
                    .and_then(|s| s.strip_prefix('v'))
                    .or_else(|| version_str.split_whitespace().last())
                    .map(|s| s.to_string());
                Ok(version)
            }
            _ => Ok(None),
        }
    }

    /// 测试二进制文件是否可用
    ///
    /// 运行 `binary_name --help` 命令测试二进制文件是否正常工作。
    fn test_binary_works(binary_name: &str) -> Result<bool> {
        let output = Command::new(binary_name).arg("--help").output();

        match output {
            Ok(result) => Ok(result.status.success()),
            Err(_) => Ok(false),
        }
    }

    // --- 高级验证方法 ---

    /// 验证单个二进制文件
    ///
    /// 检查二进制文件是否存在、可执行、版本正确且可用。
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

        // 3. 检查版本号
        let version = Self::get_binary_version(name)?;

        // 4. 测试命令是否可用
        let working = Self::test_binary_works(name)?;

        Ok(BinaryStatus {
            name: name.to_string(),
            path: path.to_string(),
            exists,
            executable,
            version,
            working,
        })
    }

    /// 验证所有二进制文件
    ///
    /// 验证 workflow 二进制文件。
    fn verify_binaries(target_version: &str) -> Result<Vec<BinaryStatus>> {
        log_info!("Verifying binaries...");

        let binaries = ["workflow"];
        let mut results = Vec::new();

        for binary in &binaries {
            let path = format!("/usr/local/bin/{}", binary);
            let status = Self::verify_single_binary(&path, binary, target_version)?;
            results.push(status);
        }

        Ok(results)
    }

    /// 验证补全脚本
    ///
    /// 检查补全脚本文件是否存在、是否可读，并验证文件内容基本格式。
    fn verify_completions() -> Result<bool> {
        log_info!("Verifying completion scripts...");

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
        let commands = ["workflow"];
        let shell_type_str = shell.to_string();
        let files = get_completion_files_for_shell(&shell_type_str, &commands).unwrap_or_default();

        let mut all_valid = true;

        for file in &files {
            let path = completion_dir.join(file);

            // 1. 检查文件是否存在
            if !path.exists() {
                log_warning!("Completion script does not exist: {}", path.display());
                all_valid = false;
                continue;
            }

            // 2. 检查文件是否可读
            if let Err(e) = fs::metadata(&path) {
                log_warning!(
                    "Unable to read completion script metadata: {} ({})",
                    path.display(),
                    e
                );
                all_valid = false;
                continue;
            }

            // 3. 检查文件大小（应该大于 0）
            if let Ok(metadata) = fs::metadata(&path) {
                if metadata.len() == 0 {
                    log_warning!("Completion script file is empty: {}", path.display());
                    all_valid = false;
                    continue;
                }
            }

            // 4. 尝试读取文件内容，验证基本格式
            if let Ok(content) = fs::read_to_string(&path) {
                // 对于 zsh，检查是否包含基本的补全函数定义
                if shell == Shell::Zsh {
                    if !content.contains("#compdef") && !content.contains("compdef") {
                        log_warning!(
                            "Completion script format may be incorrect: {} (missing compdef)",
                            path.display()
                        );
                        // 不标记为失败，因为可能是其他格式的补全脚本
                    }
                } else {
                    // 对于 bash，检查是否包含基本的补全函数定义
                    if !content.contains("complete") && !content.contains("_") {
                        log_warning!(
                            "Completion script format may be incorrect: {} (missing complete command)",
                            path.display()
                        );
                        // 不标记为失败，因为可能是其他格式的补全脚本
                    }
                }
            } else {
                log_warning!(
                    "Unable to read completion script content: {}",
                    path.display()
                );
                all_valid = false;
                continue;
            }

            log_debug!("Completion script verification passed: {}", path.display());
        }

        if all_valid {
            log_success!("  Completion script verification passed");
        } else {
            log_warning!("Some completion script verifications failed");
        }

        Ok(all_valid)
    }

    /// 验证安装结果
    ///
    /// 验证更新后的安装是否成功，包括二进制文件和补全脚本。
    fn verify_installation(target_version: &str) -> Result<VerificationResult> {
        log_info!("Verifying installation...");
        log_break!();

        // 验证二进制文件
        let binaries = Self::verify_binaries(target_version)?;

        let mut all_binaries_ok = true;
        for binary in &binaries {
            if !binary.exists {
                log_warning!("Binary file does not exist: {}", binary.path);
                all_binaries_ok = false;
            } else if !binary.executable {
                log_warning!("Binary file is not executable: {}", binary.path);
                all_binaries_ok = false;
            } else if let Some(ref version) = binary.version {
                if version != target_version {
                    log_warning!(
                        "Binary file version mismatch: {} (expected: {}, actual: {})",
                        binary.name,
                        target_version,
                        version
                    );
                    all_binaries_ok = false;
                } else {
                    log_success!("  {} v{} verification passed", binary.name, version);
                }
            } else {
                log_warning!("Unable to get version number for {}", binary.name);
                all_binaries_ok = false;
            }

            if !binary.working {
                log_warning!("Binary file cannot work properly: {}", binary.name);
                all_binaries_ok = false;
            }
        }

        log_break!();

        // 验证补全脚本
        let completions_installed = Self::verify_completions()?;
        if completions_installed {
            log_success!("  Completion script verification passed");
        } else {
            log_warning!("Completion script verification failed");
        }

        log_break!();

        // 汇总结果
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

    // ==================== 主流程 ====================

    /// 执行完整的更新操作
    ///
    /// 按照以下步骤更新 Workflow CLI：
    /// 1. 检测平台
    /// 2. 获取当前版本和目标版本
    /// 3. 比较版本并提示用户
    /// 4. 获取用户确认
    /// 5. 构建下载 URL
    /// 6. 下载 tar.gz 文件
    /// 7. 验证文件完整性
    /// 8. 解压文件
    /// 9. 使用 ./install 安装二进制文件和补全脚本（默认安装全部）
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
        let platform = Self::detect_platform()?;
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

        if !confirm(&confirm_message, true, Some("Update cancelled"))? {
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

        // 第五步：构建下载 URL
        let download_url = Self::build_download_url(&target_version, &platform);
        log_info!("Download URL: {}", download_url);
        log_break!();

        // 创建临时目录
        let temp_dir = std::env::temp_dir().join(format!("workflow-update-{}", target_version));
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).context("Failed to remove existing temp directory")?;
        }
        fs::create_dir_all(&temp_dir).context("Failed to create temp directory")?;

        let archive_name = format!("workflow-{}-{}.tar.gz", target_version, platform);
        let tar_gz_path = temp_dir.join(&archive_name);
        let extract_dir = temp_dir.join("extracted");

        // 执行更新操作，如果失败则回滚
        let update_result = (|| -> Result<()> {
            // 第六步：下载文件
            Self::download_file(&download_url, &tar_gz_path)?;
            log_break!();

            // 第七步：验证文件完整性
            let checksum_url = Checksum::build_url(&download_url);

            // 下载校验和文件（使用 http 模块）
            let http_client = HttpClient::global()?;
            let retry_config = HttpRetryConfig::new();

            let checksum_content = HttpRetry::retry(
                || {
                    let config = RequestConfig::<Value, Value>::new();
                    let response = http_client.get(&checksum_url, config)?;
                    response.as_text()
                },
                &retry_config,
                "Downloading checksum file",
            )
            .context("Failed to download checksum file")?;

            // 解析哈希值（使用 checksum 模块）
            let expected_hash = Checksum::parse_hash_from_content(&checksum_content)
                .context("Failed to parse checksum file")?;

            // 验证文件（使用 checksum 模块）
            Checksum::verify(&tar_gz_path, &expected_hash)?;
            log_break!();

            // 第八步：解压文件
            Self::extract_archive(&tar_gz_path, &extract_dir)?;
            log_break!();

            // 第九步：使用 ./install 安装二进制文件和补全脚本（默认安装全部）
            Self::install(&extract_dir)?;
            log_break!();

            // 第十步：验证安装结果
            let verification_result = Self::verify_installation(&target_version)?;
            log_break!();

            // 如果验证失败，认为更新失败
            if !verification_result.all_checks_passed {
                anyhow::bail!("Installation verification failed, some checks did not pass");
            }

            Ok(())
        })();

        // 处理更新结果
        match update_result {
            Ok(()) => {
                // 更新成功，清理临时文件和备份
                log_debug!("Cleaning up temporary files...");
                if let Err(e) = fs::remove_dir_all(&temp_dir) {
                    log_warning!("Failed to clean up temporary files: {}", e);
                }

                // 清理备份
                if let Some(ref backup) = backup_info {
                    if let Err(e) = RollbackManager::cleanup_backup(backup) {
                        log_warning!("Failed to clean up backup: {}", e);
                    }
                }

                log_success!("Workflow CLI update complete! All verifications passed.");
                Ok(())
            }
            Err(e) => {
                // 更新失败，执行回滚
                log_error!("Update failed: {}", e);
                log_break!();

                if let Some(ref backup) = backup_info {
                    match RollbackManager::rollback(backup) {
                        Ok(()) => {
                            log_break!();
                            // 回滚成功后清理备份
                            if let Err(cleanup_err) = RollbackManager::cleanup_backup(backup) {
                                log_warning!("Failed to clean up backup: {}", cleanup_err);
                            }
                        }
                        Err(rollback_err) => {
                            log_error!("Rollback failed: {}", rollback_err);
                            log_error!("  System may be in an inconsistent state");
                            log_error!("  Please manually check and restore files");
                            log_error!("  Backup location: {}", backup.backup_dir.display());
                        }
                    }
                } else {
                    log_error!("Unable to rollback: no available backup");
                    log_error!("  Please manually check and restore files");
                }

                // 清理临时文件
                log_debug!("Cleaning up temporary files...");
                if let Err(cleanup_err) = fs::remove_dir_all(&temp_dir) {
                    log_warning!("Failed to clean up temporary files: {}", cleanup_err);
                }

                Err(e.context("Update failed"))
            }
        }
    }
}
