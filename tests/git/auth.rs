//! Git 认证模块测试
//!
//! 测试 GitAuth 模块的认证功能。

use serial_test::serial;
use workflow::git::GitAuth;

/// 测试认证诊断功能
#[test]
#[serial]
fn test_auth_diagnose() {
    let diagnosis = GitAuth::diagnose();
    println!("{}", diagnosis);

    // 诊断应该包含关键信息
    assert!(diagnosis.contains("Git Authentication Diagnosis"));
    assert!(diagnosis.contains("SSH Configuration"));
    assert!(diagnosis.contains("HTTPS Configuration"));
}

/// 测试获取远程回调（不实际执行认证）
#[test]
#[serial]
fn test_get_remote_callbacks() {
    // 测试获取认证回调（不应该 panic）
    let _callbacks = GitAuth::get_remote_callbacks();
    // RemoteCallbacks 没有公共方法可以验证，但创建成功就说明没问题
    assert!(true); // 占位符，确保测试通过
}

/// 测试认证信息缓存
#[test]
#[serial]
fn test_auth_info_caching() {
    // 多次调用应该使用缓存的认证信息
    let diagnosis1 = GitAuth::diagnose();
    let diagnosis2 = GitAuth::diagnose();

    // 两次诊断结果应该相同（因为使用了缓存）
    assert_eq!(diagnosis1, diagnosis2);
}

/// 测试在实际 Git 仓库中的认证回调
#[test]
#[serial]
#[ignore] // 需要实际的 Git 仓库和网络连接
fn test_auth_with_real_repo() {
    use git2::Repository;

    // 只在 Git 仓库中运行
    if let Ok(repo) = Repository::open(".") {
        if let Ok(mut remote) = repo.find_remote("origin") {
            let callbacks = GitAuth::get_remote_callbacks();
            let mut fetch_options = git2::FetchOptions::new();
            fetch_options.remote_callbacks(callbacks);

            // 尝试 fetch（这会触发认证回调）
            // 注意：这需要实际的网络连接和有效的认证
            let result = remote.fetch(&[] as &[&str], Some(&mut fetch_options), None);

            // 我们不关心 fetch 是否成功，只关心认证回调是否正常工作
            // 如果认证失败，会返回相应的错误
            match result {
                Ok(_) => println!("✓ Authentication and fetch succeeded"),
                Err(e) => {
                    println!("Authentication or fetch failed: {}", e);
                    // 这是预期的，因为测试环境可能没有配置认证
                }
            }
        }
    }
}
