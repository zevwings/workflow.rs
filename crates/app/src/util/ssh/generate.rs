use crate::{bootstrap::get_ssh_service, util::SshOperationError};
use prompt::{br, confirm, info, select, success, warning, PasswordFormField};
use std::path::PathBuf;

/// SSH 密钥生成选项
///
/// 各字段为 `None` 时使用交互式流程，为 `Some` 时直接使用给定值。
#[derive(Default)]
pub struct GenerateOptions {
    /// 算法，None 时交互选择
    pub algorithm: Option<String>,
    /// 输出路径，None 时使用默认路径
    pub output: Option<PathBuf>,
    /// 是否强制覆盖，None 时若已存在则交互确认
    pub force: Option<bool>,
    /// 是否跳过 passphrase 询问
    pub no_passphrase: bool,
    /// 密钥注释
    pub comment: Option<String>,
}

/// 生成 SSH 密钥
///
/// 根据 `opts` 决定交互或非交互：字段为 `None` 时走交互流程，为 `Some` 时使用给定值。
pub fn generate_ssh_key(opts: GenerateOptions) -> Result<PathBuf, SshOperationError> {
    let ssh = get_ssh_service();

    let algorithm = match &opts.algorithm {
        Some(alg) => alg.clone(),
        None => {
            let algorithms = vec![
                "Ed25519 (recommended)".to_string(),
                "RSA (4096-bit)".to_string(),
            ];
            let selected = select!("Select key algorithm", algorithms)
                .default(0)
                .result_title("Algorithm")
                .prompt()
                .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;
            if selected.contains("RSA") {
                "rsa".to_string()
            } else {
                "ed25519".to_string()
            }
        }
    };

    let output_path = opts.output.unwrap_or_else(|| ssh.default_key_path(&algorithm));

    if opts.algorithm.is_some() {
        info!("Generating {} SSH key...", algorithm.to_uppercase());
        info!("Output: {}", output_path.display());
        br!();
    }

    let force = match opts.force {
        Some(f) => {
            if output_path.exists() && !f {
                return Err(SshOperationError::OperationFailed(
                    format!(
                        "Key already exists at {}. Use --force to overwrite.",
                        output_path.display()
                    ),
                ));
            }
            f
        }
        None => {
            if output_path.exists() {
                warning!("Key already exists at {}", output_path.display());
                let overwrite = confirm!("Overwrite existing key?")
                    .default(false)
                    .result_title("Overwrite")
                    .prompt()
                    .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;
                if !overwrite {
                    info!("Keeping existing key.");
                    return Ok(output_path);
                }
                true
            } else {
                false
            }
        }
    };

    let passphrase = if opts.no_passphrase {
        None
    } else {
        ask_passphrase()?
    };

    ssh.generate_key(
        &output_path,
        &algorithm,
        opts.comment.as_deref(),
        passphrase.as_deref(),
        force,
    )
    .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;

    br!();
    success!("SSH key generated successfully!");
    info!("Private key: {}", output_path.display());
    info!("Public key: {}.pub", output_path.display());
    br!();

    Ok(output_path)
}

/// 交互式询问 passphrase
fn ask_passphrase() -> Result<Option<String>, SshOperationError> {
    let set_pass = confirm!("Set a passphrase for the key?")
        .default(false)
        .result_title("Set passphrase")
        .prompt()
        .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;

    if !set_pass {
        return Ok(None);
    }

    let pass = PasswordFormField::new("passphrase", "Enter passphrase for the new key");
    let form = prompt::FormBuilder::new().add_password(pass);
    let result = form.run().map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;
    let p = result.get_string("passphrase");
    Ok(if p.is_empty() { None } else { Some(p) })
}
