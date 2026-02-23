/// 获取当前安装的版本号
///
/// 从编译时嵌入的版本号获取。
pub fn get_current_version() -> Option<String> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    Some(VERSION.to_string())
}
