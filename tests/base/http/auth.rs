//! Base HTTP Auth 模块测试
//!
//! 测试 Basic Authentication 的核心功能，包括 Authorization 结构体。

use pretty_assertions::assert_eq;
use workflow::base::http::Authorization;

// ==================== Authorization Creation Tests ====================

/// 测试Authorization使用有效凭据创建实例
///
/// ## 测试目的
/// 验证 `Authorization::new()` 方法能够使用有效的用户名和密码创建Authorization实例。
///
/// ## 测试场景
/// 1. 准备用户名和密码
/// 2. 创建Authorization实例
/// 3. 验证字段值正确
///
/// ## 预期结果
/// - 实例创建成功
/// - username和password字段值与输入一致
#[test]
fn test_authorization_new_with_valid_credentials_creates_instance() {
    // Arrange: 准备用户名和密码
    let username = "user@example.com";
    let password = "api_token";

    // Act: 创建 Authorization 实例
    let auth = Authorization::new(username, password);

    // Assert: 验证字段值正确
    assert_eq!(auth.username, username);
    assert_eq!(auth.password, password);
}

/// 测试Authorization使用String类型凭据创建实例
///
/// ## 测试目的
/// 验证 `Authorization::new()` 方法能够接受String类型的用户名和密码。
///
/// ## 测试场景
/// 1. 准备String类型的用户名和密码
/// 2. 创建Authorization实例
/// 3. 验证字段值正确
///
/// ## 预期结果
/// - 实例创建成功
/// - username和password字段值与输入一致
#[test]
fn test_authorization_new_with_string_credentials_creates_instance() {
    // Arrange: 准备 String 类型的用户名和密码
    let username = String::from("test@example.com");
    let password = String::from("secret_token");

    // Act: 创建 Authorization 实例
    let auth = Authorization::new(username.clone(), password.clone());

    // Assert: 验证字段值正确
    assert_eq!(auth.username, username);
    assert_eq!(auth.password, password);
}

/// 测试Authorization使用空凭据创建实例
///
/// ## 测试目的
/// 验证 `Authorization::new()` 方法能够接受空字符串作为用户名和密码。
///
/// ## 测试场景
/// 1. 准备空字符串作为用户名和密码
/// 2. 创建Authorization实例
/// 3. 验证字段值为空字符串
///
/// ## 预期结果
/// - 实例创建成功
/// - username和password字段值为空字符串
#[test]
fn test_authorization_new_with_empty_credentials_creates_instance() {
    // Arrange: 准备空字符串
    let username = "";
    let password = "";

    // Act: 创建 Authorization 实例
    let auth = Authorization::new(username, password);

    // Assert: 验证字段值为空字符串
    assert_eq!(auth.username, username);
    assert_eq!(auth.password, password);
}

// ==================== Authorization Debug Tests ====================

/// 测试Authorization Debug格式化
///
/// ## 测试目的
/// 验证 `Authorization` 结构体实现了 `Debug` trait，能够正确格式化输出。
///
/// ## 测试场景
/// 1. 创建Authorization实例
/// 2. 使用Debug格式化输出
/// 3. 验证输出包含用户名或密码
///
/// ## 预期结果
/// - Debug格式化成功
/// - 输出包含用户名或密码信息
#[test]
fn test_authorization_debug_with_valid_instance_returns_debug_string() {
    // Arrange: 准备 Authorization 实例
    let auth = Authorization::new("user@example.com", "token");

    // Act: 格式化 Debug 输出
    let debug_str = format!("{:?}", auth);

    // Assert: 验证 Debug 字符串包含用户名或密码
    assert!(debug_str.contains("user@example.com") || debug_str.contains("token"));
}

// ==================== Authorization Clone Tests ====================

/// 测试Authorization克隆功能
///
/// ## 测试目的
/// 验证 `Authorization` 结构体实现了 `Clone` trait，能够正确克隆实例。
///
/// ## 测试场景
/// 1. 创建原始Authorization实例
/// 2. 克隆实例
/// 3. 验证克隆的字段值与原始值相同
///
/// ## 预期结果
/// - 克隆成功
/// - 克隆的username和password与原始值相同
#[test]
fn test_authorization_clone_with_valid_instance_creates_clone() {
    // Arrange: 准备原始 Authorization 实例
    let original = Authorization::new("user@example.com", "token");

    // Act: 克隆实例
    let cloned = original.clone();

    // Assert: 验证克隆的字段值与原始值相同
    assert_eq!(original.username, cloned.username);
    assert_eq!(original.password, cloned.password);
}

// ==================== Authorization Field Access Tests ====================

/// 测试Authorization字段公开访问和修改
///
/// ## 测试目的
/// 验证 `Authorization` 结构体的字段是公开的，允许直接修改。
///
/// ## 测试场景
/// 1. 创建Authorization实例
/// 2. 直接修改username和password字段
/// 3. 验证字段值已更新
///
/// ## 预期结果
/// - 字段可以修改
/// - 修改后的值与预期一致
#[test]
fn test_authorization_fields_with_public_access_allows_modification() {
    // Arrange: 准备 Authorization 实例
    let mut auth = Authorization::new("user@example.com", "token");

    // Act: 修改字段值
    auth.username = "new@example.com".to_string();
    auth.password = "new_token".to_string();

    // Assert: 验证字段值已更新
    assert_eq!(auth.username, "new@example.com");
    assert_eq!(auth.password, "new_token");
}
