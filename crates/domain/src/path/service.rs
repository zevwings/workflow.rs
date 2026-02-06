use std::path::PathBuf;

// use crate::path::entity::Dir;
use crate::path::error::PathError;

pub trait PathService: Send + Sync {
    // /// 获取基础目录
    // fn get_base_dir(&self) -> Result<Dir, PathError>;

    /// 获取配置目录
    fn get_workflow_config_filepath(&self) -> Result<PathBuf, PathError>;

    /// 获取配置目录
    fn get_jira_config_filepath(&self) -> Result<PathBuf, PathError>;

    /// 获取二进制安装目录
    fn get_binary_install_dir(&self) -> Result<PathBuf, PathError>;

    /// 获取二进制文件名
    fn get_binary_name(&self) -> Result<String, PathError>;
}
