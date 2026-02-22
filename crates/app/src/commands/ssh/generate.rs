//! SSH 密钥生成命令

use std::path::PathBuf;

use prompt::{br, info, success, warning, PasswordFormField, SelectBuilder};

use crate::bootstrap::get_ssh_service;

/// SSH Generate 命令
pub struct SshGenerateCommand {
    output: Option<PathBuf>,
    algorithm: String,
    comment: Option<String>,
    force: bool,
    no_passphrase: bool,
}

impl SshGenerateCommand {
    pub fn new(
        output: Option<PathBuf>,
        algorithm: String,
        comment: Option<String>,
        force: bool,
        no_passphrase: bool,
    ) -> Self {
        Self {
            output,
            algorithm,
            comment,
            force,
            no_passphrase,
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let ssh = get_ssh_service();
        let algorithm = self.algorithm.clone();

        let output_path = self.output.clone().unwrap_or_else(|| ssh.default_key_path(&algorithm));

        info!("Generating {} SSH key...", algorithm.to_uppercase());
        info!("Output: {}", output_path.display());
        br!();

        if output_path.exists() && !self.force {
            return Err(format!(
                "Key already exists at {}. Use --force to overwrite.",
                output_path.display()
            )
            .into());
        }

        let passphrase = if self.no_passphrase {
            None
        } else {
            ask_passphrase()?
        };

        ssh.generate_key(
            &output_path,
            &algorithm,
            self.comment.as_deref(),
            passphrase.as_deref(),
            self.force,
        )?;

        br!();
        success!("SSH key generated successfully!");
        info!("Private key: {}", output_path.display());
        info!("Public key: {}.pub", output_path.display());
        br!();

        let add_to_agent = prompt::confirm!("Add the new key to ssh-agent?")
            .default(true)
            .result_title("Add to agent")
            .prompt()?;

        if add_to_agent {
            if ssh.is_agent_available() {
                ssh.add_key(&output_path, None)?;
                success!("Key added to ssh-agent.");
            } else {
                warning!("ssh-agent is not running. Start it with `eval $(ssh-agent)` and then run `workflow ssh add`.");
            }
        }

        Ok(())
    }
}

/// 交互式询问 passphrase
fn ask_passphrase() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let set_pass = prompt::confirm!("Set a passphrase for the key?")
        .default(false)
        .result_title("Set passphrase")
        .prompt()?;

    if !set_pass {
        return Ok(None);
    }

    let pass = PasswordFormField::new("passphrase", "Enter passphrase for the new key");
    let form = prompt::FormBuilder::new().add_password(pass);
    let result = form.run()?;
    let p = result.get_string("passphrase");
    Ok(if p.is_empty() { None } else { Some(p) })
}

/// 交互式生成密钥（从 stage 调用）
pub fn interactive_generate() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let ssh = get_ssh_service();

    let algorithms = vec![
        "Ed25519 (recommended)".to_string(),
        "RSA (4096-bit)".to_string(),
    ];

    let selected = SelectBuilder::new("Select key algorithm", algorithms)
        .default(0)
        .result_title("Algorithm")
        .prompt()?;

    let algorithm = if selected.contains("RSA") {
        "rsa"
    } else {
        "ed25519"
    };

    let output_path = ssh.default_key_path(algorithm);

    let force = if output_path.exists() {
        warning!("Key already exists at {}", output_path.display());
        prompt::confirm!("Overwrite existing key?")
            .default(false)
            .result_title("Overwrite")
            .prompt()?
    } else {
        false
    };

    if output_path.exists() && !force {
        info!("Keeping existing key.");
        return Ok(output_path);
    }

    let passphrase = ask_passphrase()?;

    ssh.generate_key(&output_path, algorithm, None, passphrase.as_deref(), force)?;

    br!();
    success!("SSH key generated successfully!");
    info!("Private key: {}", output_path.display());
    info!("Public key: {}.pub", output_path.display());

    Ok(output_path)
}
