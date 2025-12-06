//! GitHub 相关测试
//!
//! 测试 GitHub 相关的功能，包括 URL 解析等。

use workflow::pr::helpers::extract_pull_request_id_from_url;

#[test]
fn test_extract_pull_request_id_from_url() {
    assert_eq!(
        extract_pull_request_id_from_url("https://github.com/owner/repo/pull/123").unwrap(),
        "123"
    );
    assert_eq!(
        extract_pull_request_id_from_url("https://github.com/owner/repo/pull/456/").unwrap(),
        "456"
    );
}
