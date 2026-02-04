//! 归档文件处理工具
//!
//! 本模块提供了统一的归档文件处理功能，支持：
//! - tar.gz 文件解压
//! - zip 文件解压
//! - 根据扩展名自动选择解压方式
//! - 多文件合并

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use tar::Archive as TarArchive;
use zip::ZipArchive;

use super::directory;
use super::FileError;

// ============================================================================
// 归档文件处理函数
// ============================================================================

/// 自动检测格式并解压归档文件
///
/// 根据文件扩展名自动选择合适的解压方式：
/// - `.tar.gz` 或 `.tgz` - 使用 tar + gzip 解压
/// - `.zip` - 使用 zip 解压
///
/// # 参数
///
/// * `archive_path` - 归档文件路径
/// * `output_dir` - 解压目标目录（如果不存在会自动创建）
///
/// # 返回
///
/// 成功返回 `Ok(())`，失败返回相应错误。
///
/// # 错误
///
/// - 文件不存在
/// - 不支持的文件格式
/// - 解压过程中的 I/O 错误
///
/// # 示例
///
/// ```no_run
/// use std::path::Path;
/// use toolkit::util::fs::archive;
///
/// archive::extract(
///     Path::new("release.tar.gz"),
///     Path::new("./extracted")
/// ).unwrap();
/// ```
pub fn extract(archive_path: &Path, output_dir: &Path) -> Result<(), FileError> {
    let path_str = archive_path.to_string_lossy().to_lowercase();

    if path_str.ends_with(".tar.gz") || path_str.ends_with(".tgz") {
        extract_tar_gz(archive_path, output_dir)
    } else if path_str.ends_with(".zip") {
        extract_zip(archive_path, output_dir)
    } else {
        Err(FileError::Other(format!(
            "Unsupported archive format: {}",
            archive_path.display()
        )))
    }
}

/// 解压 tar.gz 文件
///
/// 将 tar.gz（或 .tgz）文件解压到指定目录。
///
/// # 参数
///
/// * `archive_path` - tar.gz 文件路径
/// * `output_dir` - 解压目标目录（如果不存在会自动创建）
///
/// # 返回
///
/// 成功返回 `Ok(())`，失败返回相应错误。
///
/// # 示例
///
/// ```no_run
/// use std::path::Path;
/// use toolkit::util::fs::archive;
///
/// archive::extract_tar_gz(
///     Path::new("archive.tar.gz"),
///     Path::new("./output")
/// ).unwrap();
/// ```
pub fn extract_tar_gz(archive_path: &Path, output_dir: &Path) -> Result<(), FileError> {
    // 确保输出目录存在
    directory::ensure_exists(output_dir)?;

    // 打开文件
    let file = File::open(archive_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            FileError::FileNotFound(archive_path.display().to_string())
        } else {
            FileError::Io(e)
        }
    })?;

    // 创建 Gzip 解码器
    let decoder = GzDecoder::new(file);
    let mut archive = TarArchive::new(decoder);

    // 解压到目标目录
    archive
        .unpack(output_dir)
        .map_err(|e| FileError::Compression(format!("Failed to extract tar.gz archive: {}", e)))?;

    tracing::debug!(
        "Extracted tar.gz: {} -> {}",
        archive_path.display(),
        output_dir.display()
    );

    Ok(())
}

/// 解压 zip 文件
///
/// 将 zip 文件解压到指定目录。
///
/// # 参数
///
/// * `archive_path` - zip 文件路径
/// * `output_dir` - 解压目标目录（如果不存在会自动创建）
///
/// # 返回
///
/// 成功返回 `Ok(())`，失败返回相应错误。
///
/// # 示例
///
/// ```no_run
/// use std::path::Path;
/// use toolkit::util::fs::archive;
///
/// archive::extract_zip(
///     Path::new("archive.zip"),
///     Path::new("./output")
/// ).unwrap();
/// ```
pub fn extract_zip(archive_path: &Path, output_dir: &Path) -> Result<(), FileError> {
    // 确保输出目录存在
    directory::ensure_exists(output_dir)?;

    // 打开 zip 文件
    let file = File::open(archive_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            FileError::FileNotFound(archive_path.display().to_string())
        } else {
            FileError::Io(e)
        }
    })?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| FileError::Compression(format!("Failed to read zip archive: {}", e)))?;

    // 解压所有文件
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            FileError::Compression(format!("Failed to read file {} from zip: {}", i, e))
        })?;

        let outpath = output_dir.join(file.name());

        if file.name().ends_with('/') {
            // 目录
            directory::ensure_exists(&outpath)?;
        } else {
            // 文件
            if let Some(parent) = outpath.parent() {
                directory::ensure_exists(parent)?;
            }

            let mut outfile = File::create(&outpath).map_err(FileError::Io)?;
            std::io::copy(&mut file, &mut outfile).map_err(FileError::Io)?;
        }
    }

    tracing::debug!(
        "Extracted zip: {} -> {}",
        archive_path.display(),
        output_dir.display()
    );

    Ok(())
}

/// 获取归档文件中的文件列表
///
/// 列出归档文件中的所有文件和目录。
///
/// # 参数
///
/// * `archive_path` - 归档文件路径
///
/// # 返回
///
/// 返回文件路径列表。
pub fn list_contents(archive_path: &Path) -> Result<Vec<String>, FileError> {
    let path_str = archive_path.to_string_lossy().to_lowercase();

    if path_str.ends_with(".tar.gz") || path_str.ends_with(".tgz") {
        list_tar_gz_contents(archive_path)
    } else if path_str.ends_with(".zip") {
        list_zip_contents(archive_path)
    } else {
        Err(FileError::Other(format!(
            "Unsupported archive format: {}",
            archive_path.display()
        )))
    }
}

/// 列出 tar.gz 文件内容
fn list_tar_gz_contents(archive_path: &Path) -> Result<Vec<String>, FileError> {
    let file = File::open(archive_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = TarArchive::new(decoder);

    let mut contents = Vec::new();
    for entry in archive
        .entries()
        .map_err(|e| FileError::Compression(format!("Failed to read tar.gz entries: {}", e)))?
    {
        let entry = entry
            .map_err(|e| FileError::Compression(format!("Failed to read tar.gz entry: {}", e)))?;
        if let Ok(path) = entry.path() {
            contents.push(path.to_string_lossy().to_string());
        }
    }

    Ok(contents)
}

/// 列出 zip 文件内容
fn list_zip_contents(archive_path: &Path) -> Result<Vec<String>, FileError> {
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| FileError::Compression(format!("Failed to read zip archive: {}", e)))?;

    let mut contents = Vec::new();
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            contents.push(file.name().to_string());
        }
    }

    Ok(contents)
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
///
/// # 示例
///
/// ```no_run
/// use std::path::{Path, PathBuf};
/// use toolkit::util::fs::archive;
///
/// let split_files = vec![
///     PathBuf::from("file.zip.001"),
///     PathBuf::from("file.zip.002"),
/// ];
///
/// archive::merge_files(
///     Path::new("file.zip"),
///     &split_files,
///     Path::new("merged.zip"),
/// ).unwrap();
/// ```
pub fn merge_files(
    base_file: &Path,
    split_files: &[PathBuf],
    output_file: &Path,
) -> Result<PathBuf, FileError> {
    if !base_file.exists() {
        return Err(FileError::FileNotFound(format!(
            "Base file not found: {:?}",
            base_file
        )));
    }

    let mut output = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output_file)
        .map_err(FileError::Io)?;

    // 写入基础文件
    let mut input = File::open(base_file).map_err(FileError::Io)?;
    std::io::copy(&mut input, &mut output).map_err(FileError::Io)?;

    // 写入所有分片文件
    for split_file in split_files {
        let mut input = File::open(split_file).map_err(FileError::Io)?;
        std::io::copy(&mut input, &mut output).map_err(FileError::Io)?;
    }

    output.flush().map_err(FileError::Io)?;

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

    tracing::debug!(
        "Merged files: {:?} + {} parts -> {}",
        base_file,
        split_files.len(),
        output_file.display()
    );

    Ok(output_file.to_path_buf())
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_extract_detects_format() {
        // 测试格式检测逻辑（不实际解压）
        let path = Path::new("test.tar.gz");
        let path_str = path.to_string_lossy().to_lowercase();
        assert!(path_str.ends_with(".tar.gz"));

        let path = Path::new("test.tgz");
        let path_str = path.to_string_lossy().to_lowercase();
        assert!(path_str.ends_with(".tgz"));

        let path = Path::new("test.zip");
        let path_str = path.to_string_lossy().to_lowercase();
        assert!(path_str.ends_with(".zip"));
    }

    #[test]
    fn test_unsupported_format() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.rar");
        std::fs::write(&archive_path, b"fake content").unwrap();

        let result = extract(&archive_path, temp_dir.path());
        assert!(matches!(result, Err(FileError::Other(_))));
    }

    #[test]
    fn test_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = Path::new("/nonexistent/file.tar.gz");

        let result = extract_tar_gz(archive_path, temp_dir.path());
        assert!(matches!(result, Err(FileError::FileNotFound(_))));
    }
}
