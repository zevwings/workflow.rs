//! Tar.gz 解压功能
//!
//! 提供 tar.gz 文件解压功能。

use color_eyre::{eyre::WrapErr, Result};
use flate2::read::GzDecoder;
use std::path::Path;
use tar::Archive;

use crate::base::fs::directory::DirectoryWalker;
use crate::base::fs::file::FileReader;

/// 解压 tar.gz 文件
///
/// 将 tar.gz 文件解压到指定目录。
///
/// # 参数
///
/// * `tar_gz_path` - tar.gz 文件路径
/// * `output_dir` - 解压目标目录
///
/// # 返回
///
/// 如果解压成功，返回 `Ok(())`，否则返回相应的错误信息。
///
/// # 示例
///
/// ```no_run
/// use workflow::base::zip::tar::extract_tar_gz;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// extract_tar_gz(
///     Path::new("archive.tar.gz"),
///     Path::new("./output")
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn extract_tar_gz(tar_gz_path: &Path, output_dir: &Path) -> Result<()> {
    // 创建输出目录
    DirectoryWalker::new(output_dir).ensure_exists()?;

    // 打开 tar.gz 文件并创建 BufReader
    let reader = FileReader::new(tar_gz_path).open()?;

    // 创建 Gzip 解码器
    let decoder = GzDecoder::new(reader);
    let mut archive = Archive::new(decoder);

    // 解压到目标目录
    archive.unpack(output_dir).wrap_err("Failed to extract tar.gz archive")?;

    Ok(())
}
