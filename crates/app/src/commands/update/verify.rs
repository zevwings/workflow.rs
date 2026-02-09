//! 验证模块
//!
//! 提供安装验证功能。路径从 pathService 获取，单文件验证。

use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

use prompt::{success, warning, Spinner};
use toolkit::{detect_shell, get_completion_files_for_shell, shell_to_string};

use super::types::VerificationResult;
use crate::registry::get_path_service;

/// 补全脚本对应的命令名（与 pathService 的 binary 一致，单命令）
const COMMAND_NAME: &str = "workflow";

/// 解压目录中 install 二进制文件名（平台相关）
fn install_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "install.exe"
    } else {
        "install"
    }
}

/// 二进制文件状态（内部使用）
struct BinaryStatus {
    name: String,
    path: String,
    exists: bool,
    executable: bool,
}

/// 检查文件是否可执行
#[cfg(unix)]
fn check_executable(path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(false);
    }

    let metadata = fs::metadata(path)
        .map_err(|e| format!("Failed to get metadata for {}: {}", path.display(), e))?;

    let permissions = metadata.permissions();
    let mode = permissions.mode();

    // 检查是否有执行权限（owner, group, or others）
    Ok((mode & 0o111) != 0)
}

#[cfg(windows)]
fn check_executable(path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(false);
    }

    // Windows 上通过扩展名判断可执行文件
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        Ok(ext_str == "exe"
            || ext_str == "bat"
            || ext_str == "cmd"
            || ext_str == "com"
            || ext_str == "ps1")
    } else {
        // 没有扩展名，假设可执行
        Ok(true)
    }
}

/// 验证单个二进制文件
fn verify_single_binary(
    path: &str,
    name: &str,
) -> Result<BinaryStatus, Box<dyn std::error::Error>> {
    let path_obj = Path::new(path);

    let exists = path_obj.exists();
    if !exists {
        return Ok(BinaryStatus {
            name: name.to_string(),
            path: path.to_string(),
            exists: false,
            executable: false,
        });
    }

    let executable = check_executable(path_obj)?;

    Ok(BinaryStatus {
        name: name.to_string(),
        path: path.to_string(),
        exists,
        executable,
    })
}

/// 验证二进制文件（从 pathService 获取单文件路径）
fn verify_binaries() -> Result<Vec<BinaryStatus>, Box<dyn std::error::Error>> {
    let path_service = get_path_service();
    let install_dir = path_service.get_binary_install_dir()?;
    let bin_name = path_service.get_binary_name()?;
    let path = install_dir.join(&bin_name);

    let spinner = Spinner::new("Verifying binaries...");
    let spinner_instance = spinner.start();

    let status = verify_single_binary(&path.to_string_lossy(), &bin_name)?;
    spinner_instance.stop();

    Ok(vec![status])
}

/// 验证补全脚本
fn verify_completions() -> Result<bool, Box<dyn std::error::Error>> {
    let shell = match detect_shell() {
        Ok(shell) => shell,
        Err(_) => {
            warning!("Unable to detect shell type, skipping completion script verification");
            return Ok(false);
        }
    };

    let path_service = get_path_service();
    let comp_dir = path_service.get_completion_dir()?;

    if !comp_dir.exists() {
        warning!(
            "Completion script directory does not exist: {}",
            comp_dir.display()
        );
        return Ok(false);
    }

    let commands: &[&str] = &[COMMAND_NAME];
    let shell_str = shell_to_string(&shell);
    let files = get_completion_files_for_shell(shell_str, commands).unwrap_or_default();

    let mut all_valid = true;

    let spinner = Spinner::new("Verifying completion scripts...");
    let spinner_instance = spinner.start();

    for file in &files {
        let path = comp_dir.join(file);

        if !path.exists() {
            warning!("Completion script does not exist: {}", path.display());
            all_valid = false;
            continue;
        }

        toolkit::log_debug!("Completion script verification passed: {}", path.display());
    }

    spinner_instance.stop();

    if all_valid {
        success!("Completion script verification passed");
    } else {
        warning!("Some completion script verifications failed");
    }

    Ok(all_valid)
}

/// 验证安装结果
pub fn verify_installation() -> Result<VerificationResult, Box<dyn std::error::Error>> {
    // 验证二进制文件
    let binaries = verify_binaries()?;

    let mut all_binaries_ok = true;
    for binary in &binaries {
        if !binary.exists {
            warning!("Binary file does not exist: {}", binary.path);
            all_binaries_ok = false;
        } else if !binary.executable {
            warning!("Binary file is not executable: {}", binary.path);
            all_binaries_ok = false;
        } else {
            success!(
                "{} verification passed (file exists and is executable)",
                binary.name
            );
        }
    }

    // 验证补全脚本
    let completions_installed = verify_completions()?;

    // 汇总结果
    let all_checks_passed = all_binaries_ok && completions_installed;

    if all_checks_passed {
        success!("All verifications passed!");
    } else {
        warning!("Some verifications failed, please check the above warning messages");
    }

    Ok(VerificationResult { all_checks_passed })
}

/// 运行安装程序
///
/// 在解压目录中运行 ./install 来安装二进制文件和补全脚本。
pub fn run_installer(extract_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let install_binary = extract_dir.join(install_binary_name());

    if !install_binary.exists() {
        return Err(format!(
            "Install binary does not exist: {}",
            install_binary.display()
        )
        .into());
    }

    // 设置执行权限（仅 Unix）
    #[cfg(unix)]
    {
        Command::new("chmod")
            .arg("+x")
            .arg(&install_binary)
            .status()
            .map_err(|e| format!("Failed to set executable permission for install: {}", e))?;
    }

    // 运行安装程序，捕获输出以避免与 spinner 冲突
    // stdin 继承以支持交互式输入（如 sudo 密码）
    // stdout/stderr 捕获，在 spinner 停止后显示
    let spinner = Spinner::new("Installing binaries and completion scripts...");
    let spinner_instance = spinner.start();

    let output = Command::new(&install_binary)
        .current_dir(extract_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run install: {}", e))?;

    spinner_instance.stop();

    // Spinner 停止后，显示子进程的输出
    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprint!("{}", stdout);
    }
    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprint!("{}", stderr);
    }

    if !output.status.success() {
        return Err("Installation failed".into());
    }

    success!("Binaries and completion scripts installation complete");
    Ok(())
}
