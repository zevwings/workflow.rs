//! SSH 服务实现
//!
//! 调用系统 ssh-keygen、ssh-add 命令。

use std::path::{Path, PathBuf};
use std::process::Command;

use domain::{SshError, SshKeyInfo, SshService};

/// SSH 服务实现
pub struct SshServiceImpl;

impl SshServiceImpl {
    pub fn new() -> Self {
        Self
    }
}

impl SshService for SshServiceImpl {
    fn is_agent_available(&self) -> bool {
        std::env::var("SSH_AUTH_SOCK").is_ok()
            && Command::new("ssh-add")
                .arg("-l")
                .output()
                .map(|o| {
                    // exit code 0 = has keys, 1 = no keys but agent running
                    // exit code 2 = agent not running
                    o.status.code().unwrap_or(2) != 2
                })
                .unwrap_or(false)
    }

    fn list_loaded_keys(&self) -> Result<Vec<SshKeyInfo>, SshError> {
        if !self.is_agent_available() {
            return Err(SshError::AgentUnavailable);
        }

        let output = Command::new("ssh-add")
            .arg("-l")
            .output()
            .map_err(|e| SshError::CommandFailed(format!("Failed to run ssh-add: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // exit code 1 means agent is running but no keys
        if output.status.code() == Some(1) || stdout.contains("no identities") {
            return Ok(vec![]);
        }

        if !output.status.success() {
            return Err(SshError::AgentUnavailable);
        }

        Ok(stdout.lines().filter_map(SshKeyInfo::parse_ssh_add_line).collect())
    }

    fn scan_keys(&self) -> Vec<PathBuf> {
        let ssh_dir = match dirs::home_dir() {
            Some(home) => home.join(".ssh"),
            None => return vec![],
        };

        let candidates = [
            "id_ed25519",
            "id_ed25519_sk",
            "id_rsa",
            "id_ecdsa",
            "id_ecdsa_sk",
            "id_dsa",
        ];

        candidates
            .iter()
            .map(|name| ssh_dir.join(name))
            .filter(|path| path.exists())
            .collect()
    }

    fn default_key_path(&self, algorithm: &str) -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        let filename = match algorithm {
            "rsa" => "id_rsa",
            _ => "id_ed25519",
        };
        home.join(".ssh").join(filename)
    }

    fn generate_key(
        &self,
        output_path: &Path,
        algorithm: &str,
        comment: Option<&str>,
        passphrase: Option<&str>,
        force: bool,
    ) -> Result<(), SshError> {
        if output_path.exists() && !force {
            return Err(SshError::KeyAlreadyExists(
                output_path.display().to_string(),
            ));
        }

        // 确保 ~/.ssh 目录存在
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    SshError::GenerationFailed(format!("Failed to create .ssh directory: {}", e))
                })?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))
                        .map_err(|e| {
                            SshError::GenerationFailed(format!(
                                "Failed to set directory permissions: {}",
                                e
                            ))
                        })?;
                }
            }
        }

        let mut cmd = Command::new("ssh-keygen");
        cmd.arg("-t").arg(algorithm);

        if algorithm == "rsa" {
            cmd.arg("-b").arg("4096");
        }

        cmd.arg("-f").arg(output_path);

        if let Some(c) = comment {
            cmd.arg("-C").arg(c);
        }

        let pass = passphrase.unwrap_or("");
        cmd.arg("-N").arg(pass);

        let output = cmd
            .output()
            .map_err(|e| SshError::CommandFailed(format!("Failed to run ssh-keygen: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SshError::GenerationFailed(stderr.to_string()));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(output_path, std::fs::Permissions::from_mode(0o600)).map_err(
                |e| SshError::GenerationFailed(format!("Failed to set key permissions: {}", e)),
            )?;
        }

        Ok(())
    }

    fn add_key(&self, key_path: &Path, lifetime: Option<u64>) -> Result<(), SshError> {
        if !self.is_agent_available() {
            return Err(SshError::AgentUnavailable);
        }

        if !key_path.exists() {
            return Err(SshError::KeyNotFound(key_path.display().to_string()));
        }

        let mut cmd = Command::new("ssh-add");

        if let Some(seconds) = lifetime {
            cmd.arg("-t").arg(seconds.to_string());
        }

        cmd.arg(key_path);

        let output = cmd
            .output()
            .map_err(|e| SshError::CommandFailed(format!("Failed to run ssh-add: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SshError::AddFailed(stderr.to_string()));
        }

        Ok(())
    }

    fn find_key_path_by_fingerprint(&self, fingerprint: &str) -> Option<PathBuf> {
        let ssh_dir = dirs::home_dir()?.join(".ssh");

        let entries = std::fs::read_dir(&ssh_dir).ok()?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("pub") {
                continue;
            }

            // ssh-keygen -lf <pubkey> 输出: "256 SHA256:xxx comment (ED25519)"
            let output = Command::new("ssh-keygen").arg("-lf").arg(&path).output().ok()?;

            if !output.status.success() {
                continue;
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains(fingerprint) {
                // .pub → 私钥路径（去掉 .pub 后缀）
                let private_key = path.with_extension("");
                if private_key.exists() {
                    return Some(private_key);
                }
            }
        }

        None
    }

    fn remove_key_by_path(&self, key_path: &Path) -> Result<(), SshError> {
        if !self.is_agent_available() {
            return Err(SshError::AgentUnavailable);
        }

        let output = Command::new("ssh-add")
            .arg("-d")
            .arg(key_path)
            .output()
            .map_err(|e| SshError::CommandFailed(format!("Failed to run ssh-add -d: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SshError::RemoveFailed(stderr.to_string()));
        }

        Ok(())
    }

    fn remove_all_keys(&self) -> Result<(), SshError> {
        if !self.is_agent_available() {
            return Err(SshError::AgentUnavailable);
        }

        let output = Command::new("ssh-add")
            .arg("-D")
            .output()
            .map_err(|e| SshError::CommandFailed(format!("Failed to run ssh-add -D: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SshError::RemoveFailed(stderr.to_string()));
        }

        Ok(())
    }
}
