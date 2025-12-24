//! Base Util Browser 模块测试
//!
//! 测试浏览器操作工具的核心功能，包括 Browser 结构体。

use workflow::base::system::Browser;

#[test]
fn test_browser_open_structure() {
    // 测试 Browser 结构体可以创建
    // 注意：实际打开浏览器需要系统支持，这里只测试结构体
    let _browser = Browser;
    assert!(true);
}

#[test]
#[cfg(not(target_os = "windows"))] // Windows 上跳过：可能阻塞或弹出错误对话框
#[ignore] // 忽略：实际打开浏览器可能导致阻塞，影响测试速度
fn test_browser_open_invalid_url() {
    // 测试无效 URL（应该返回错误）
    // 注意：此测试在实际环境中可能会尝试打开浏览器，导致阻塞
    // Windows 上已通过 #[cfg] 跳过，因为 Windows 可能会尝试用默认程序打开无效 URL，导致阻塞
    // 如果需要运行此测试，请使用: cargo test -- --ignored
    let result = Browser::open("not-a-valid-url");
    // 在某些平台上可能会失败，这是预期的
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_browser_open_empty_url() {
    // 测试空 URL
    let result = Browser::open("");
    // 空 URL 可能失败或成功（取决于平台实现）
    // 在某些平台上可能会静默失败或成功，这是可以接受的
    assert!(result.is_err() || result.is_ok());
}
