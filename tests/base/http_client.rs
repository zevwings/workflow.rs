//! Base/HTTP/Client 模块测试
//!
//! 测试 HTTP 客户端的功能，包括：
//! - HttpClient::global 单例
//! - 配置应用
//! - post_multipart 的 retry 警告

use serial_test::serial;
use workflow::http::{Authorization, HttpClientConfig};
use workflow::http::{HttpClient, HttpRetryConfig, MultipartRequestConfig, RequestConfig};

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== HttpClient::global 单例测试 ====================

    #[test]
    #[serial]
    fn test_http_client_global_singleton() {
        // 测试全局单例：多次调用应该返回同一个实例
        let client1 = HttpClient::global().expect("Should create global client");
        let client2 = HttpClient::global().expect("Should return same global client");

        // 验证是同一个实例（通过地址比较）
        // 注意：由于 HttpClient 不实现 Eq，我们通过其他方式验证
        // 实际上，OnceLock 保证返回同一个引用
        assert!(std::ptr::eq(client1, client2));
    }

    #[test]
    #[serial]
    fn test_http_client_global_multiple_calls() {
        // 测试多次调用 global() 的一致性
        let clients: Vec<&HttpClient> = (0..5)
            .map(|_| HttpClient::global().expect("Should create global client"))
            .collect();

        // 所有客户端应该是同一个实例
        let first = clients[0];
        for client in clients.iter().skip(1) {
            assert!(std::ptr::eq(first, *client));
        }
    }

    // ==================== HttpClient 配置测试 ====================

    #[test]
    fn test_http_client_config_default() {
        let config = HttpClientConfig::default();

        assert_eq!(config.pool_max_idle_per_host, 100);
        assert_eq!(config.keep_alive_timeout.as_secs(), 90);
        assert_eq!(config.connect_timeout.as_secs(), 10);
        assert_eq!(config.timeout.as_secs(), 30);
        assert!(config.user_agent.contains("workflow"));
        assert!(config.tls_verify);
        assert_eq!(config.max_request_body_size, 10 * 1024 * 1024);
        assert_eq!(config.max_response_body_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_http_client_config_new() {
        let config = HttpClientConfig::new();
        let default_config = HttpClientConfig::default();

        assert_eq!(
            config.pool_max_idle_per_host,
            default_config.pool_max_idle_per_host
        );
        assert_eq!(config.keep_alive_timeout, default_config.keep_alive_timeout);
        assert_eq!(config.connect_timeout, default_config.connect_timeout);
        assert_eq!(config.timeout, default_config.timeout);
        assert_eq!(config.user_agent, default_config.user_agent);
        assert_eq!(config.tls_verify, default_config.tls_verify);
    }

    #[test]
    fn test_http_client_config_builder() {
        use std::time::Duration;

        let config = HttpClientConfig::new()
            .pool_max_idle_per_host(50)
            .keep_alive_timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(20))
            .user_agent("custom-agent")
            .tls_verify(false)
            .max_request_body_size(5 * 1024 * 1024)
            .max_response_body_size(50 * 1024 * 1024);

        assert_eq!(config.pool_max_idle_per_host, 50);
        assert_eq!(config.keep_alive_timeout.as_secs(), 60);
        assert_eq!(config.connect_timeout.as_secs(), 5);
        assert_eq!(config.timeout.as_secs(), 20);
        assert_eq!(config.user_agent, "custom-agent");
        assert!(!config.tls_verify);
        assert_eq!(config.max_request_body_size, 5 * 1024 * 1024);
        assert_eq!(config.max_response_body_size, 50 * 1024 * 1024);
    }

    // ==================== RequestConfig 应用测试 ====================

    #[test]
    fn test_request_config_with_auth() {
        // 测试 RequestConfig 的认证配置
        let auth = Authorization::bearer("token");
        let config = RequestConfig::new().auth(auth);

        assert!(config.auth.is_some());
    }

    #[test]
    fn test_request_config_with_query() {
        // 测试 RequestConfig 的查询参数配置
        let query = serde_json::json!({"page": "1"});
        let config = RequestConfig::new().query(&query);

        assert!(config.query.is_some());
    }

    #[test]
    fn test_request_config_with_timeout() {
        // 测试 RequestConfig 的超时配置
        use std::time::Duration;
        let timeout = Duration::from_secs(60);
        let config = RequestConfig::new().timeout(timeout);

        assert_eq!(config.timeout, Some(timeout));
    }

    #[test]
    fn test_request_config_with_retry() {
        // 测试 RequestConfig 的重试配置
        let retry_config = HttpRetryConfig::new();
        let config = RequestConfig::new().retry(retry_config);

        assert!(config.retry_config.is_some());
    }

    // ==================== MultipartRequestConfig Retry 警告测试 ====================

    #[test]
    fn test_multipart_request_config_retry_ignored() {
        // 测试 MultipartRequestConfig 的 retry_config 会被忽略
        // 注意：实际的警告在 post_multipart 方法中发出
        // 这里我们验证 retry_config 可以被设置（虽然会被忽略）
        let mut config = MultipartRequestConfig::new();
        config.retry_config = Some(HttpRetryConfig::new());

        // 验证 retry_config 存在（虽然会被忽略）
        assert!(config.retry_config.is_some());

        // 注意：实际的警告测试需要在集成测试中使用 mockito
        // 因为需要实际调用 post_multipart 方法
    }

    // ==================== HttpClient 初始化失败测试 ====================

    #[test]
    #[serial]
    fn test_http_client_global_initialization() {
        // 测试全局客户端初始化
        // 正常情况下应该成功
        let result = HttpClient::global();

        assert!(result.is_ok());
    }

    // 注意：测试初始化失败的情况比较困难，因为需要模拟系统资源不足
    // 这种情况在实际使用中很少发生，可以在集成测试中通过其他方式测试
}
