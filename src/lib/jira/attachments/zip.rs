//! ZIP 处理相关功能

use crate::Logger;
use anyhow::{Context, Result};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::constants::*;

/// ZIP 处理器
///
/// 提供 ZIP 文件的合并和解压功能。
pub struct ZipProcessor;

impl ZipProcessor {
    /// 合并分片 zip 文件
    pub fn merge_split_zips(&self, download_dir: &Path) -> Result<PathBuf> {
        let log_zip = download_dir.join(LOG_ZIP_FILENAME);
        if !log_zip.exists() {
            anyhow::bail!("{} not found in {:?}", LOG_ZIP_FILENAME, download_dir);
        }

        // 查找所有分片文件（log.z01, log.z02, ...）
        let mut split_files: Vec<PathBuf> = WalkDir::new(download_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                if let Some(name) = e.file_name().to_str() {
                    name.starts_with(LOG_ZIP_SPLIT_PREFIX)
                        && name.len() == 8
                        && name[6..].parse::<u8>().is_ok()
                } else {
                    false
                }
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        // 按文件名排序
        split_files.sort();

        if split_files.is_empty() {
            // 没有分片文件，直接复制 log.zip 为 merged.zip
            let merged_zip = download_dir.join(MERGED_ZIP_FILENAME);
            std::fs::copy(&log_zip, &merged_zip).with_context(|| {
                format!(
                    "Failed to copy {} to {}",
                    LOG_ZIP_FILENAME, MERGED_ZIP_FILENAME
                )
            })?;
            return Ok(merged_zip);
        }

        // 合并文件
        let merged_zip = download_dir.join(MERGED_ZIP_FILENAME);
        let mut output = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&merged_zip)
            .with_context(|| format!("Failed to create {}", MERGED_ZIP_FILENAME))?;

        // 写入 log.zip
        let mut input =
            File::open(&log_zip).with_context(|| format!("Failed to open {}", LOG_ZIP_FILENAME))?;
        std::io::copy(&mut input, &mut output)
            .with_context(|| format!("Failed to copy {}", LOG_ZIP_FILENAME))?;

        // 写入所有分片文件
        for split_file in &split_files {
            let mut input = File::open(split_file)
                .with_context(|| format!("Failed to open {:?}", split_file))?;
            std::io::copy(&mut input, &mut output)
                .with_context(|| format!("Failed to copy {:?}", split_file))?;
        }

        output
            .flush()
            .with_context(|| format!("Failed to flush {}", MERGED_ZIP_FILENAME))?;

        // 验证文件大小
        let expected_size: u64 = std::fs::metadata(&log_zip)?.len()
            + split_files
                .iter()
                .map(|f| std::fs::metadata(f).map(|m| m.len()).unwrap_or(0))
                .sum::<u64>();

        let actual_size = std::fs::metadata(&merged_zip)?.len();

        if actual_size != expected_size {
            Logger::print_warning(format!(
                "Merged file size mismatch (expected: {}, actual: {})",
                expected_size, actual_size
            ));
        }

        Ok(merged_zip)
    }

    /// 解压 zip 文件
    pub fn extract_zip(&self, zip_path: &Path, output_dir: &Path) -> Result<()> {
        let file = File::open(zip_path)
            .with_context(|| format!("Failed to open zip file: {:?}", zip_path))?;

        let mut archive = zip::ZipArchive::new(file)
            .with_context(|| format!("Failed to read zip archive: {:?}", zip_path))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .with_context(|| format!("Failed to read file {} from zip", i))?;

            let outpath = output_dir.join(file.name());

            if file.name().ends_with('/') {
                // 目录
                std::fs::create_dir_all(&outpath)
                    .with_context(|| format!("Failed to create directory: {:?}", outpath))?;
            } else {
                // 文件
                if let Some(parent) = outpath.parent() {
                    std::fs::create_dir_all(parent).with_context(|| {
                        format!("Failed to create parent directory: {:?}", parent)
                    })?;
                }

                let mut outfile = File::create(&outpath)
                    .with_context(|| format!("Failed to create file: {:?}", outpath))?;

                std::io::copy(&mut file, &mut outfile)
                    .with_context(|| format!("Failed to extract file: {:?}", outpath))?;
            }
        }

        Ok(())
    }
}
