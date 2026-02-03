//! 验证模块
//!
//! 提供安装验证功能。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use color_eyre::{eyre::WrapErr, Result};
use prompt::{success, warning, Spinner};
use toolkit::{detect_shell, get_completion_files_for_shell, shell_to_string, Paths};

use super::types::{BinaryStatus, VerificationResult};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// 检查文件是否可执行
#[cfg(unix)]
fn check_executable(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }

    let metadata =
        fs::metadata(path).wrap_err_with(|| format!("Failed to get metadata for: {}", path.display()))?;

    let permissions = metadata.permissions();
    let mode = permissions.mode();

    // 检查是否有执行权限（owner, group, or others）
    Ok((mode & 0o111) != 0)
}

#[cfg(windows)]
fn check_executable(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }

    // Windows 上通过扩展名判断可执行文件
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        Ok(ext_str == "exe" || ext_str == "bat" || ext_str == "cmd" || ext_str == "com" || ext_str == "ps1")
    } else {
        // 没有扩展名，假设可执行
        Ok(true)
    }
}

/// 验证单个二进制文件
fn verify_single_binary(path: &str, name: &str) -> Result<BinaryStatus> {
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

/// 验证所有二进制文件
fn verify_binaries() -> Result<Vec<BinaryStatus>> {
    let install_dir = Paths::binary_install_dir();
    let install_path = PathBuf::from(&install_dir);
    let binaries = Paths::command_names();
    let mut results = Vec::new();

    let spinner = Spinner::new("Verifying binaries...");
    let spinner_instance = spinner.start();

    for binary in binaries {
        let binary_name = Paths::binary_name(binary);
        let path = install_path.join(&binary_name);
        let status = verify_single_binary(&path.to_string_lossy(), &binary_name)?;
        results.push(status);
    }

    spinner_instance.stop();

    Ok(results)
}

/// 验证补全脚本
fn verify_completions() -> Result<bool> {
    let shell = match detect_shell() {
        Ok(shell) => shell,
        Err(_) => {
            warning!("Unable to detect shell type, skipping completion script verification");
            return Ok(false);
        }
    };

    let completion_dir = Paths::completion_dir()?;

    if !completion_dir.exists() {
        warning!(
            "Completion script directory does not exist: {}",
            completion_dir.display()
        );
        return Ok(false);
    }

    let commands = Paths::command_names();
    let shell_str = shell_to_string(&shell);
    let files = get_completion_files_for_shell(shell_str, commands).unwrap_or_default();

    let mut all_valid = true;

    let spinner = Spinner::new("Verifying completion scripts...");
    let spinner_instance = spinner.start();

    for file in &files {
        let path = completion_dir.join(file);

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
pub fn verify_installation() -> Result<VerificationResult> {
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

    Ok(VerificationResult {
        binaries,
        completions_installed,
        all_checks_passed,
    })
}

/// 运行安装程序
///
/// 在解压目录中运行 ./install 来安装二进制文件和补全脚本。
pub fn run_installer(extract_dir: &Path) -> Result<()> {
    let install_binary = extract_dir.join(Paths::binary_name("install"));

    if !install_binary.exists() {
        color_eyre::eyre::bail!(
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
            .wrap_err("Failed to set executable permission for install")?;
    }

    // 运行安装程序
    let spinner = Spinner::new("Installing binaries and completion scripts...");
    let spinner_instance = spinner.start();

    let status = Command::new(&install_binary)
        .current_dir(extract_dir)
        .status()
        .wrap_err("Failed to run install")?;

    spinner_instance.stop();

    if !status.success() {
        color_eyre::eyre::bail!("Installation failed");
    }

    success!("Binaries and completion scripts installation complete");
    Ok(())
}
