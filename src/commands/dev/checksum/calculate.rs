//! 文件哈希计算实现

use crate::{log_info, log_success};
use color_eyre::{eyre::WrapErr, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// 文件哈希计算命令
pub struct ChecksumCalculateCommand {
    file: String,
    output: Option<String>,
}

impl ChecksumCalculateCommand {
    /// 创建新的文件哈希计算命令
    pub fn new(file: String, output: Option<String>) -> Self {
        Self { file, output }
    }

    /// 计算文件哈希
    pub fn calculate(&self) -> Result<String> {
        let file_path = Path::new(&self.file);

        if !file_path.exists() {
            return Err(color_eyre::eyre::eyre!("File not found: {}", self.file));
        }

        // 读取文件内容
        let file_content =
            fs::read(file_path).wrap_err_with(|| format!("Failed to read file: {}", self.file))?;

        // 计算 SHA256 哈希
        let mut hasher = Sha256::new();
        hasher.update(&file_content);
        let hash = hasher.finalize();
        let hash_hex = format!("{:x}", hash);

        log_info!("📄 File: {}", self.file);
        log_info!("🔐 SHA256: {}", hash_hex);

        // 输出到文件或标准输出
        if let Some(ref output_path) = self.output {
            fs::write(output_path, &hash_hex)
                .wrap_err_with(|| format!("Failed to write hash to: {}", output_path))?;
            log_success!("Hash written to: {}", output_path);
        }

        Ok(hash_hex)
    }
}
