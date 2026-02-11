//! 归档文件处理工具
//!
//! 本模块提供了统一的归档文件处理功能，支持：
//! - tar.gz 文件解压
//! - zip 文件解压
//! - 根据扩展名自动选择解压方式
//! - 多文件合并

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use flate2::read::GzDecoder;
use tar::Archive;
use zip::ZipArchive;

use super::{directory, FileError};

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
/// use toolkit::archive;
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
/// use toolkit::archive;
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
    let mut archive = Archive::new(decoder);

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
/// use toolkit::archive;
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
    let mut archive = Archive::new(decoder);

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

    let mut contents = Vec::with_capacity(archive.len());
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
/// use toolkit::archive;
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
    use std::io::Write;

    use flate2::{write::GzEncoder, Compression};
    use tar::{Builder, Header};
    use tempfile::TempDir;
    use zip::{write::FileOptions, ZipWriter};

    use super::*;

    // ========================================================================
    // 辅助函数：创建测试归档文件
    // ========================================================================

    /// 创建一个测试用的 tar.gz 文件
    fn create_test_tar_gz(dir: &Path) -> PathBuf {
        let archive_path = dir.join("test.tar.gz");
        let file = File::create(&archive_path).unwrap();
        let encoder = GzEncoder::new(file, Compression::default());
        let mut builder = Builder::new(encoder);

        // 添加一个测试文件
        let test_content = b"Hello from tar.gz!";
        let mut header = Header::new_gnu();
        header.set_path("test_file.txt").unwrap();
        header.set_size(test_content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, &test_content[..]).unwrap();

        // 添加一个子目录和文件
        let mut header2 = Header::new_gnu();
        header2.set_path("subdir/nested.txt").unwrap();
        header2.set_size(12);
        header2.set_mode(0o644);
        header2.set_cksum();
        builder.append(&header2, &b"nested file!"[..]).unwrap();

        builder.finish().unwrap();
        archive_path
    }

    /// 创建一个测试用的 zip 文件
    fn create_test_zip(dir: &Path) -> PathBuf {
        let archive_path = dir.join("test.zip");
        let file = File::create(&archive_path).unwrap();
        let mut zip = ZipWriter::new(file);

        let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        // 添加一个测试文件
        zip.start_file("test_file.txt", options).unwrap();
        zip.write_all(b"Hello from zip!").unwrap();

        // 添加一个子目录和文件
        zip.add_directory("subdir/", options).unwrap();
        zip.start_file("subdir/nested.txt", options).unwrap();
        zip.write_all(b"nested zip!").unwrap();

        zip.finish().unwrap();
        archive_path
    }

    // ========================================================================
    // 格式检测测试
    // ========================================================================

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

    // ========================================================================
    // tar.gz 解压测试
    // ========================================================================

    #[test]
    fn test_extract_tar_gz_success() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = create_test_tar_gz(temp_dir.path());
        let output_dir = temp_dir.path().join("output");

        let result = extract_tar_gz(&archive_path, &output_dir);
        assert!(result.is_ok());

        // 验证文件被正确解压
        let extracted_file = output_dir.join("test_file.txt");
        assert!(extracted_file.exists());

        let content = std::fs::read_to_string(&extracted_file).unwrap();
        assert_eq!(content, "Hello from tar.gz!");

        // 验证嵌套文件
        let nested_file = output_dir.join("subdir/nested.txt");
        assert!(nested_file.exists());
    }

    #[test]
    fn test_extract_tar_gz_creates_output_dir() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = create_test_tar_gz(temp_dir.path());
        let output_dir = temp_dir.path().join("new_dir/sub_dir");

        // 输出目录不存在
        assert!(!output_dir.exists());

        let result = extract_tar_gz(&archive_path, &output_dir);
        assert!(result.is_ok());

        // 验证目录被创建
        assert!(output_dir.exists());
    }

    // ========================================================================
    // zip 解压测试
    // ========================================================================

    #[test]
    fn test_extract_zip_success() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = create_test_zip(temp_dir.path());
        let output_dir = temp_dir.path().join("output");

        let result = extract_zip(&archive_path, &output_dir);
        assert!(result.is_ok());

        // 验证文件被正确解压
        let extracted_file = output_dir.join("test_file.txt");
        assert!(extracted_file.exists());

        let content = std::fs::read_to_string(&extracted_file).unwrap();
        assert_eq!(content, "Hello from zip!");

        // 验证嵌套文件
        let nested_file = output_dir.join("subdir/nested.txt");
        assert!(nested_file.exists());
    }

    #[test]
    fn test_extract_zip_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("nonexistent.zip");

        let result = extract_zip(&archive_path, temp_dir.path());
        assert!(matches!(result, Err(FileError::FileNotFound(_))));
    }

    // ========================================================================
    // extract 自动检测测试
    // ========================================================================

    #[test]
    fn test_extract_auto_tar_gz() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = create_test_tar_gz(temp_dir.path());
        let output_dir = temp_dir.path().join("auto_output");

        let result = extract(&archive_path, &output_dir);
        assert!(result.is_ok());
        assert!(output_dir.join("test_file.txt").exists());
    }

    #[test]
    fn test_extract_auto_zip() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = create_test_zip(temp_dir.path());
        let output_dir = temp_dir.path().join("auto_output");

        let result = extract(&archive_path, &output_dir);
        assert!(result.is_ok());
        assert!(output_dir.join("test_file.txt").exists());
    }

    // ========================================================================
    // list_contents 测试
    // ========================================================================

    #[test]
    fn test_list_tar_gz_contents() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = create_test_tar_gz(temp_dir.path());

        let result = list_contents(&archive_path);
        assert!(result.is_ok());

        let contents = result.unwrap();
        assert!(contents.iter().any(|s| s.contains("test_file.txt")));
        assert!(contents.iter().any(|s| s.contains("nested.txt")));
    }

    #[test]
    fn test_list_zip_contents() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = create_test_zip(temp_dir.path());

        let result = list_contents(&archive_path);
        assert!(result.is_ok());

        let contents = result.unwrap();
        assert!(contents.iter().any(|s| s.contains("test_file.txt")));
        assert!(contents.iter().any(|s| s.contains("nested.txt")));
    }

    #[test]
    fn test_list_contents_unsupported() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.7z");
        std::fs::write(&archive_path, b"fake").unwrap();

        let result = list_contents(&archive_path);
        assert!(matches!(result, Err(FileError::Other(_))));
    }

    // ========================================================================
    // merge_files 测试
    // ========================================================================

    #[test]
    fn test_merge_files_success() {
        let temp_dir = TempDir::new().unwrap();

        // 创建基础文件
        let base_file = temp_dir.path().join("base.bin");
        std::fs::write(&base_file, b"BASE").unwrap();

        // 创建分片文件
        let split1 = temp_dir.path().join("split.001");
        let split2 = temp_dir.path().join("split.002");
        std::fs::write(&split1, b"PART1").unwrap();
        std::fs::write(&split2, b"PART2").unwrap();

        let output_file = temp_dir.path().join("merged.bin");
        let result = merge_files(&base_file, &[split1, split2], &output_file);
        assert!(result.is_ok());

        let merged_content = std::fs::read(&output_file).unwrap();
        assert_eq!(merged_content, b"BASEPART1PART2");
    }

    #[test]
    fn test_merge_files_base_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let base_file = temp_dir.path().join("nonexistent.bin");
        let output_file = temp_dir.path().join("merged.bin");

        let result = merge_files(&base_file, &[], &output_file);
        assert!(matches!(result, Err(FileError::FileNotFound(_))));
    }

    #[test]
    fn test_merge_files_empty_splits() {
        let temp_dir = TempDir::new().unwrap();

        // 只有基础文件，没有分片
        let base_file = temp_dir.path().join("base.bin");
        std::fs::write(&base_file, b"ONLY_BASE").unwrap();

        let output_file = temp_dir.path().join("merged.bin");
        let result = merge_files(&base_file, &[], &output_file);
        assert!(result.is_ok());

        let merged_content = std::fs::read(&output_file).unwrap();
        assert_eq!(merged_content, b"ONLY_BASE");
    }

    #[test]
    fn test_merge_files_large_content() {
        let temp_dir = TempDir::new().unwrap();

        // 创建较大的测试数据
        let base_content: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        let split_content: Vec<u8> = (0..500).map(|i| ((i + 100) % 256) as u8).collect();

        let base_file = temp_dir.path().join("base.bin");
        let split_file = temp_dir.path().join("split.001");
        std::fs::write(&base_file, &base_content).unwrap();
        std::fs::write(&split_file, &split_content).unwrap();

        let output_file = temp_dir.path().join("merged.bin");
        let result = merge_files(&base_file, &[split_file], &output_file);
        assert!(result.is_ok());

        let merged_content = std::fs::read(&output_file).unwrap();
        assert_eq!(merged_content.len(), 1500);
    }
}
