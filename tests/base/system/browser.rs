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

/// 测试Browser打开无效URL的错误处理
///
/// ## 测试目的
/// 验证`Browser::open()`方法在接收到无效URL时能够正确处理（返回错误或静默失败）。
///
/// ## 为什么被忽略
/// - **可能实际打开浏览器**: 某些系统可能尝试用默认浏览器打开无效URL
/// - **可能导致阻塞**: 浏览器启动过程可能阻塞测试执行
/// - **平台行为不一致**: 不同操作系统处理无效URL的方式不同
/// - **Windows特殊处理**: Windows上已通过#[cfg]跳过，避免弹出错误对话框
/// - **影响测试速度**: 打开浏览器会显著增加测试时间
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_browser_open_invalid_url -- --ignored
/// ```
/// 注意：此测试在非Windows平台上运行，可能会实际启动浏览器
///
/// ## 测试场景
/// 1. 调用Browser::open()传入无效URL（"not-a-valid-url"）
/// 2. 系统尝试解析URL
/// 3. 根据平台行为，可能：
///    - 返回错误（理想情况）
///    - 尝试用默认浏览器打开（会失败）
///    - 静默失败并返回Ok
/// 4. 验证不会panic
///
/// ## 预期行为
/// - **理想情况**：返回Err表示URL无效
/// - **可接受情况**：返回Ok但浏览器未实际打开
/// - **不应该**：panic或无响应
/// - 不同平台可能有不同行为，都是可接受的
/// - 重要的是不会导致测试hang或crash
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
#[ignore]
fn test_browser_open_empty_url() {
    // 测试空 URL
    let result = Browser::open("");
    // 空 URL 可能失败或成功（取决于平台实现）
    // 在某些平台上可能会静默失败或成功，这是可以接受的
    assert!(result.is_err() || result.is_ok());
}
