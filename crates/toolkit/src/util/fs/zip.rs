//! ZIP 文件处理工具
//!
//! 提供通用的 ZIP 文件操作功能，不包含任何业务逻辑。

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use zip::ZipArchive;

use super::{directory::DirectoryWalker, FsError};

/// ZIP 文件处理工具
///
/// 提供通用的 ZIP 文件操作，包括解压和文件合并。
pub struct ZipUtil;

impl ZipUtil {
    /// 解压 ZIP 文件到指定目录
    ///
    /// # 参数
    ///
    /// * `zip_path` - ZIP 文件路径
    /// * `output_dir` - 输出目录路径
    ///
    /// # 返回
    ///
    /// 如果解压成功，返回 `Ok(())`；否则返回错误。
    pub fn extract(&self, zip_path: &Path, output_dir: &Path) -> Result<(), FsError> {
        let file = File::open(zip_path).map_err(FsError::Io)?;

        let mut archive = ZipArchive::new(file)
            .map_err(|e| FsError::Compression(format!("Failed to read zip archive: {}", e)))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                FsError::Compression(format!("Failed to read file {} from zip: {}", i, e))
            })?;

            let outpath = output_dir.join(file.name());

            if file.name().ends_with('/') {
                // 目录
                DirectoryWalker::new(&outpath).ensure_exists()?;
            } else {
                // 文件
                if let Some(parent) = outpath.parent() {
                    DirectoryWalker::new(parent).ensure_exists()?;
                }

                let mut outfile = File::create(&outpath).map_err(FsError::Io)?;

                std::io::copy(&mut file, &mut outfile).map_err(FsError::Io)?;
            }
        }

        Ok(())
    }

    /// 合并多个文件为一个文件
    ///
    /// 将基础文件和分片文件按顺序合并为一个文件。
    ///
    /// # 参数
    ///
    /// * `base_file` - 基础文件路径
    /// * `split_files` - 分片文件路径列表（按顺序）
    /// * `output_file` - 输出文件路径
    ///
    /// # 返回
    ///
    /// 如果合并成功，返回输出文件路径；否则返回错误。
    pub fn merge_files(
        &self,
        base_file: &Path,
        split_files: &[PathBuf],
        output_file: &Path,
    ) -> Result<PathBuf, FsError> {
        if !base_file.exists() {
            return Err(FsError::FileNotFound(format!(
                "Base file not found: {:?}",
                base_file
            )));
        }

        let mut output = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(output_file)
            .map_err(FsError::Io)?;

        // 写入基础文件
        let mut input = File::open(base_file).map_err(FsError::Io)?;
        std::io::copy(&mut input, &mut output).map_err(FsError::Io)?;

        // 写入所有分片文件
        for split_file in split_files {
            let mut input = File::open(split_file).map_err(FsError::Io)?;
            std::io::copy(&mut input, &mut output).map_err(FsError::Io)?;
        }

        output.flush().map_err(FsError::Io)?;

        // 验证文件大小
        let expected_size: u64 = std::fs::metadata(base_file)?.len()
            + split_files
                .iter()
                .map(|f| std::fs::metadata(f).map(|m| m.len()).unwrap_or(0))
                .sum::<u64>();

        let actual_size = std::fs::metadata(output_file)?.len();

        if actual_size != expected_size {
            tracing::warn!(
                "Merged file size mismatch (expected: {}, actual: {})",
                expected_size,
                actual_size
            );
        }

        Ok(output_file.to_path_buf())
    }
}
