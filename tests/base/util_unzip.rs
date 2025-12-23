//! Base Util Unzip 模块测试
//!
//! 测试解压工具的核心功能，包括 tar.gz 和 zip 文件解压。

use pretty_assertions::assert_eq;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use workflow::base::util::unzip::Unzip;

// 辅助函数：创建测试用的 tar.gz 文件
fn create_test_tar_gz(temp_dir: &TempDir) -> PathBuf {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::Builder;

    let tar_gz_path = temp_dir.path().join("test.tar.gz");

    // 创建临时文件用于打包
    let file1_path = temp_dir.path().join("file1.txt");
    fs::write(&file1_path, "content1").unwrap();

    let file2_path = temp_dir.path().join("file2.txt");
    fs::write(&file2_path, "content2").unwrap();

    let subdir = temp_dir.path().join("subdir");
    fs::create_dir_all(&subdir).unwrap();
    let file3_path = subdir.join("file3.txt");
    fs::write(&file3_path, "content3").unwrap();

    // 创建 tar.gz 文件
    let tar_gz_file = fs::File::create(&tar_gz_path).unwrap();
    let enc = GzEncoder::new(tar_gz_file, Compression::default());
    let mut tar = Builder::new(enc);

    tar.append_path_with_name(&file1_path, "file1.txt").unwrap();
    tar.append_path_with_name(&file2_path, "file2.txt").unwrap();
    tar.append_path_with_name(&file3_path, "subdir/file3.txt").unwrap();

    tar.finish().unwrap();

    tar_gz_path
}

// 辅助函数：创建测试用的 zip 文件
fn create_test_zip(temp_dir: &TempDir) -> PathBuf {
    use std::io::Write;
    use zip::write::{FileOptions, ZipWriter};
    use zip::CompressionMethod;

    let zip_path = temp_dir.path().join("test.zip");
    let zip_file = fs::File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(zip_file);

    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // 添加文件
    zip.start_file("file1.txt", options).unwrap();
    zip.write_all(b"content1").unwrap();

    zip.start_file("file2.txt", options).unwrap();
    zip.write_all(b"content2").unwrap();

    // 添加目录
    zip.add_directory("subdir/", options).unwrap();

    // 添加子目录中的文件
    zip.start_file("subdir/file3.txt", options).unwrap();
    zip.write_all(b"content3").unwrap();

    zip.finish().unwrap();

    zip_path
}

#[test]
fn test_unzip_extract_tar_gz() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let tar_gz_path = create_test_tar_gz(&temp_dir);
    let output_dir = temp_dir.path().join("output");

    Unzip::extract_tar_gz(&tar_gz_path, &output_dir)?;

    // 验证文件已解压
    assert!(output_dir.join("file1.txt").exists());
    assert!(output_dir.join("file2.txt").exists());
    assert!(output_dir.join("subdir/file3.txt").exists());

    // 验证文件内容
    assert_eq!(
        fs::read_to_string(output_dir.join("file1.txt"))?,
        "content1"
    );
    assert_eq!(
        fs::read_to_string(output_dir.join("file2.txt"))?,
        "content2"
    );
    assert_eq!(
        fs::read_to_string(output_dir.join("subdir/file3.txt"))?,
        "content3"
    );

    Ok(())
}

#[test]
fn test_unzip_extract_tar_gz_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_path = temp_dir.path().join("nonexistent.tar.gz");
    let output_dir = temp_dir.path().join("output");

    let result = Unzip::extract_tar_gz(&nonexistent_path, &output_dir);
    assert!(result.is_err());
}

#[test]
fn test_unzip_extract_tar_gz_invalid_format() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let invalid_file = temp_dir.path().join("invalid.tar.gz");
    fs::write(&invalid_file, "not a valid tar.gz file")?;

    let output_dir = temp_dir.path().join("output");
    let result = Unzip::extract_tar_gz(&invalid_file, &output_dir);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_unzip_extract_tar_gz_output_dir_created() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let tar_gz_path = create_test_tar_gz(&temp_dir);
    let output_dir = temp_dir.path().join("new/output/dir");

    // 输出目录不存在，应该自动创建
    assert!(!output_dir.exists());

    Unzip::extract_tar_gz(&tar_gz_path, &output_dir)?;

    // 验证目录已创建
    assert!(output_dir.exists());
    assert!(output_dir.is_dir());

    Ok(())
}

#[test]
fn test_unzip_extract_zip() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let zip_path = create_test_zip(&temp_dir);
    let output_dir = temp_dir.path().join("output");

    Unzip::extract_zip(&zip_path, &output_dir)?;

    // 验证文件已解压
    assert!(output_dir.join("file1.txt").exists());
    assert!(output_dir.join("file2.txt").exists());
    assert!(output_dir.join("subdir/file3.txt").exists());

    // 验证文件内容
    assert_eq!(
        fs::read_to_string(output_dir.join("file1.txt"))?,
        "content1"
    );
    assert_eq!(
        fs::read_to_string(output_dir.join("file2.txt"))?,
        "content2"
    );
    assert_eq!(
        fs::read_to_string(output_dir.join("subdir/file3.txt"))?,
        "content3"
    );

    Ok(())
}

#[test]
fn test_unzip_extract_zip_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_path = temp_dir.path().join("nonexistent.zip");
    let output_dir = temp_dir.path().join("output");

    let result = Unzip::extract_zip(&nonexistent_path, &output_dir);
    assert!(result.is_err());
}

#[test]
fn test_unzip_extract_zip_invalid_format() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let invalid_file = temp_dir.path().join("invalid.zip");
    fs::write(&invalid_file, "not a valid zip file")?;

    let output_dir = temp_dir.path().join("output");
    let result = Unzip::extract_zip(&invalid_file, &output_dir);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_unzip_extract_zip_output_dir_created() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let zip_path = create_test_zip(&temp_dir);
    let output_dir = temp_dir.path().join("new/output/dir");

    // 输出目录不存在，应该自动创建
    assert!(!output_dir.exists());

    Unzip::extract_zip(&zip_path, &output_dir)?;

    // 验证目录已创建
    assert!(output_dir.exists());
    assert!(output_dir.is_dir());

    Ok(())
}

#[test]
fn test_unzip_extract_zip_with_directories() -> color_eyre::Result<()> {
    use std::io::Write;
    use zip::write::{FileOptions, ZipWriter};
    use zip::CompressionMethod;

    let temp_dir = TempDir::new()?;
    let zip_path = temp_dir.path().join("test.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);

    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // 添加嵌套目录结构
    zip.add_directory("level1/", options)?;
    zip.add_directory("level1/level2/", options)?;
    zip.start_file("level1/level2/file.txt", options)?;
    zip.write_all(b"nested content")?;

    zip.finish()?;

    let output_dir = temp_dir.path().join("output");
    Unzip::extract_zip(&zip_path, &output_dir)?;

    // 验证嵌套目录结构已创建
    assert!(output_dir.join("level1/level2/file.txt").exists());
    assert_eq!(
        fs::read_to_string(output_dir.join("level1/level2/file.txt"))?,
        "nested content"
    );

    Ok(())
}

#[test]
fn test_unzip_extract_tar_gz_single_file() -> color_eyre::Result<()> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::Builder;

    let temp_dir = TempDir::new()?;
    let tar_gz_path = temp_dir.path().join("single.tar.gz");

    // 创建一个只包含一个文件的 tar.gz 归档
    let tar_gz_file = fs::File::create(&tar_gz_path)?;
    let enc = GzEncoder::new(tar_gz_file, Compression::default());
    let mut tar = Builder::new(enc);

    // 添加一个文件条目
    let file_path = temp_dir.path().join("single.txt");
    fs::write(&file_path, "single file content")?;
    tar.append_path_with_name(&file_path, "single.txt")?;
    drop(tar); // 确保 tar builder 被 drop，刷新所有数据

    let output_dir = temp_dir.path().join("output");
    Unzip::extract_tar_gz(&tar_gz_path, &output_dir)?;

    // 验证文件已解压
    assert!(output_dir.exists());
    assert!(output_dir.join("single.txt").exists());
    assert_eq!(
        fs::read_to_string(output_dir.join("single.txt"))?,
        "single file content"
    );

    Ok(())
}

#[test]
fn test_unzip_extract_zip_empty_archive() -> color_eyre::Result<()> {
    use zip::write::ZipWriter;

    let temp_dir = TempDir::new()?;
    let zip_path = temp_dir.path().join("empty.zip");

    // 创建空的 zip 文件
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    zip.finish()?;

    let output_dir = temp_dir.path().join("output");
    let result = Unzip::extract_zip(&zip_path, &output_dir);
    // 空归档应该成功解压（只是没有文件）
    assert!(result.is_ok());
    assert!(output_dir.exists());

    Ok(())
}
