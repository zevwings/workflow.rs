//! Log 验证结果类型

/// 日志配置信息
#[derive(Debug, Clone)]
pub struct LogConfigInfo {
    /// 日志输出文件夹名称
    pub output_folder_name: String,
    /// 日志下载基础目录
    pub download_base_dir: Option<String>,
    /// 日志级别
    pub level: Option<String>,
    /// 是否同时输出 tracing 日志到控制台
    pub enable_trace_console: bool,
}

/// 日志验证结果
#[derive(Debug, Clone)]
pub struct LogVerificationResult {
    /// 是否已配置
    pub configured: bool,
    /// 配置信息（如果已配置）
    pub config: Option<LogConfigInfo>,
}
