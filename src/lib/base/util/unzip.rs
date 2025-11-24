//! 解压工具模块
//! 提供 tar.gz 文件解压功能

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use tar::Archive;

/// 解压工具
///
/// 提供文件解压功能。
pub struct Unzip;

impl Unzip {
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
    /// use workflow::base::util::unzip::Unzip;
    /// use std::path::Path;
    ///
    /// Unzip::extract_tar_gz(
    ///     Path::new("archive.tar.gz"),
    ///     Path::new("./output")
    /// )?;
    /// ```
    pub fn extract_tar_gz(tar_gz_path: &Path, output_dir: &Path) -> Result<()> {
        // 创建输出目录
        fs::create_dir_all(output_dir).context("Failed to create output directory")?;

        // 打开 tar.gz 文件
        let file = File::open(tar_gz_path)
            .with_context(|| format!("Failed to open file: {}", tar_gz_path.display()))?;

        // 创建 Gzip 解码器
        let decoder = GzDecoder::new(BufReader::new(file));
        let mut archive = Archive::new(decoder);

        // 解压到目标目录
        archive
            .unpack(output_dir)
            .context("Failed to extract tar.gz archive")?;

        Ok(())
    }
}
