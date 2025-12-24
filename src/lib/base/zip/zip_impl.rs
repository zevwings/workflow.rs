//! Zip 解压功能
//!
//! 提供 zip 文件解压功能。

use color_eyre::{eyre::WrapErr, Result};
use std::fs::File;
use std::path::Path;
use zip::ZipArchive;

use crate::base::fs::directory::DirectoryWalker;

/// 解压 zip 文件
///
/// 将 zip 文件解压到指定目录。
///
/// # 参数
///
/// * `zip_path` - zip 文件路径
/// * `output_dir` - 解压目标目录
///
/// # 返回
///
/// 如果解压成功，返回 `Ok(())`，否则返回相应的错误信息。
///
/// # 示例
///
/// ```no_run
/// use workflow::base::zip::Unzip;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// workflow::base::zip::Unzip::extract_zip(
///     Path::new("archive.zip"),
///     Path::new("./output")
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn extract_zip(zip_path: &Path, output_dir: &Path) -> Result<()> {
    // 创建输出目录
    DirectoryWalker::new(output_dir).ensure_exists()?;

    // 打开 zip 文件
    // 注意：ZipArchive::new() 需要 File 类型，不能使用 FileReader::open() 返回的 BufReader<File>
    let file = File::open(zip_path)
        .wrap_err_with(|| format!("Failed to open zip file: {}", zip_path.display()))?;

    let mut archive = ZipArchive::new(file).wrap_err("Failed to read zip archive")?;

    // 解压所有文件
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .wrap_err_with(|| format!("Failed to read file {} from zip", i))?;

        let outpath = output_dir.join(file.name());

        if file.name().ends_with('/') {
            // 目录
            DirectoryWalker::new(&outpath).ensure_exists()?;
        } else {
            // 文件
            if let Some(parent) = outpath.parent() {
                DirectoryWalker::new(parent).ensure_exists()?;
            }

            let mut outfile = File::create(&outpath)
                .wrap_err_with(|| format!("Failed to create file: {}", outpath.display()))?;

            std::io::copy(&mut file, &mut outfile)
                .wrap_err_with(|| format!("Failed to extract file: {}", outpath.display()))?;
        }
    }

    Ok(())
}
