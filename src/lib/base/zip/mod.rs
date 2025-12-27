//! 解压工具模块
//!
//! 提供 tar.gz 和 zip 文件解压功能。

pub mod tar;

mod zip_impl;

use std::path::Path;

use color_eyre::Result;

/// 解压工具
///
/// 提供文件解压功能。
/// 这是一个向后兼容的包装结构体，内部调用各个解压模块的函数。
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
    /// use workflow::base::zip::Unzip;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// Unzip::extract_tar_gz(
    ///     Path::new("archive.tar.gz"),
    ///     Path::new("./output")
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn extract_tar_gz(tar_gz_path: &Path, output_dir: &Path) -> Result<()> {
        tar::extract_tar_gz(tar_gz_path, output_dir)
    }

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
    /// Unzip::extract_zip(
    ///     Path::new("archive.zip"),
    ///     Path::new("./output")
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn extract_zip(zip_path: &Path, output_dir: &Path) -> Result<()> {
        zip_impl::extract_zip(zip_path, output_dir)
    }
}
