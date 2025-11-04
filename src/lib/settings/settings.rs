use std::sync::{OnceLock, RwLock};
use std::env;

/// 应用程序设置
/// 单例模式，从环境变量读取配置
#[derive(Clone)]
pub struct Settings {
    // ==================== 用户配置 ====================
    /// 用户邮箱
    pub email: String,

    // ==================== Jira 配置 ====================
    /// Jira API Token
    pub jira_api_token: String,
    /// Jira 服务地址
    pub jira_service_address: String,

    // ==================== GitHub 配置 ====================
    /// GitHub 分支前缀
    pub gh_branch_prefix: Option<String>,

    // ==================== 日志配置 ====================
    /// 操作完成后是否删除日志
    pub log_delete_when_operation_completed: bool,
    /// 日志输出文件夹名称
    pub log_output_folder_name: String,

    // ==================== 代理配置 ====================
    /// 是否禁用代理检查
    pub disable_check_proxy: bool,

    // ==================== LLM/AI 配置 ====================
    /// OpenAI API Key
    pub openai_key: Option<String>,
    /// LLM 代理 URL
    pub llm_proxy_url: Option<String>,
    /// LLM 代理 Key
    pub llm_proxy_key: Option<String>,
    /// DeepSeek API Key
    pub deepseek_key: Option<String>,
    /// LLM Provider (openai, deepseek, proxy)
    pub llm_provider: String,

    // ==================== Codeup 配置 ====================
    /// Codeup 项目 ID
    pub codeup_project_id: Option<u64>,
    /// Codeup CSRF Token
    pub codeup_csrf_token: Option<String>,
    /// Codeup Cookie
    pub codeup_cookie: Option<String>,
}

// 使用 RwLock 存储 Settings，支持运行时重新加载
static INSTANCE: OnceLock<RwLock<Settings>> = OnceLock::new();

// 线程局部缓存，避免频繁读取锁
thread_local! {
    static CACHED: std::cell::Cell<*const Settings> = std::cell::Cell::new(std::ptr::null());
}

impl Settings {

    /// 获取单例实例
    /// 首次调用时会从环境变量初始化
    ///
    /// 注意：为了支持运行时重新加载，使用 RwLock 包装
    /// 使用线程局部缓存来避免频繁读取锁，但在 reload() 后会自动刷新
    pub fn get() -> &'static Self {
        let lock = Self::get_lock();

        // 检查线程局部缓存是否有效
        let cached_ptr = CACHED.with(|c| c.get());
        if !cached_ptr.is_null() {
            unsafe { &*cached_ptr }
        } else {
            // 从锁读取并缓存
            let guard = lock.read().unwrap();
            let settings = Box::leak(Box::new((*guard).clone()));
            let ptr = settings as *const Settings;
            CACHED.with(|c| c.set(ptr));
            settings
        }
    }

    /// 获取内部锁实例
    fn get_lock() -> &'static RwLock<Settings> {
        INSTANCE.get_or_init(|| {
            // 如果初始化失败，创建一个包含默认值的 Settings
            // 这样在 setup 阶段不会 panic
            let settings = Self::from_env().unwrap_or_else(|_| {
                Self {
                    email: String::new(),
                    jira_api_token: String::new(),
                    jira_service_address: String::new(),
                    gh_branch_prefix: None,
                    log_delete_when_operation_completed: false,
                    log_output_folder_name: "logs".to_string(),
                    disable_check_proxy: false,
                    openai_key: None,
                    llm_proxy_url: None,
                    llm_proxy_key: None,
                    deepseek_key: None,
                    llm_provider: "openai".to_string(),
                    codeup_project_id: None,
                    codeup_csrf_token: None,
                    codeup_cookie: None,
                }
            });
            RwLock::new(settings)
        })
    }

    /// 重新加载环境变量
    /// 强制从环境变量重新读取配置并更新单例实例
    /// 同时清除线程局部缓存，确保下次 get() 时使用新值
    ///
    /// 如果环境变量未设置（例如在 setup 阶段），返回错误但不 panic
    pub fn reload() -> Result<(), String> {
        let lock = Self::get_lock();
        let new_settings = Self::from_env()?;

        // 更新锁中的值
        *lock.write().unwrap() = new_settings;

        // 清除所有线程的缓存，强制下次 get() 时重新读取
        CACHED.with(|c| c.set(std::ptr::null()));

        Ok(())
    }

    /// 从环境变量初始化设置
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            // ==================== 用户配置 ====================
            email: Self::load_user_config()?,

            // ==================== Jira 配置 ====================
            jira_api_token: Self::load_jira_api_token()?,
            jira_service_address: Self::load_jira_service_address()?,

            // ==================== GitHub 配置 ====================
            gh_branch_prefix: Self::load_github_config(),

            // ==================== 日志配置 ====================
            log_delete_when_operation_completed: Self::load_log_delete_when_completed(),
            log_output_folder_name: Self::load_log_output_folder_name(),

            // ==================== 代理配置 ====================
            disable_check_proxy: Self::load_proxy_config(),

            // ==================== LLM/AI 配置 ====================
            openai_key: Self::load_llm_openai_key(),
            llm_proxy_url: Self::load_llm_proxy_url(),
            llm_proxy_key: Self::load_llm_proxy_key(),
            deepseek_key: Self::load_llm_deepseek_key(),
            llm_provider: Self::load_llm_provider(),

            // ==================== Codeup 配置 ====================
            codeup_project_id: Self::load_codeup_project_id(),
            codeup_csrf_token: Self::load_codeup_csrf_token(),
            codeup_cookie: Self::load_codeup_cookie(),
        })
    }

    // ==================== 用户配置 ====================
    fn load_user_config() -> Result<String, String> {
        env::var("EMAIL").map_err(|_| "EMAIL environment variable not set".to_string())
    }

    // ==================== Jira 配置 ====================
    fn load_jira_api_token() -> Result<String, String> {
        env::var("JIRA_API_TOKEN").map_err(|_| "JIRA_API_TOKEN environment variable not set".to_string())
    }

    fn load_jira_service_address() -> Result<String, String> {
        env::var("JIRA_SERVICE_ADDRESS").map_err(|_| "JIRA_SERVICE_ADDRESS environment variable not set".to_string())
    }

    // ==================== GitHub 配置 ====================
    fn load_github_config() -> Option<String> {
        env::var("GH_BRANCH_PREFIX").ok()
    }

    // ==================== 日志配置 ====================
    fn load_log_delete_when_completed() -> bool {
        env::var("LOG_DELETE_WHEN_OPERATION_COMPLETED")
            .unwrap_or_else(|_| "0".to_string())
            .parse::<u8>()
            .unwrap_or(0)
            == 1
    }

    fn load_log_output_folder_name() -> String {
        env::var("LOG_OUTPUT_FOLDER_NAME").unwrap_or_else(|_| "logs".to_string())
    }

    // ==================== 代理配置 ====================
    fn load_proxy_config() -> bool {
        env::var("DISABLE_CHECK_PROXY")
            .unwrap_or_else(|_| "0".to_string())
            .parse::<u8>()
            .unwrap_or(0)
            == 1
    }

    // ==================== LLM/AI 配置 ====================
    fn load_llm_openai_key() -> Option<String> {
        env::var("LLM_OPENAI_KEY").ok()
    }

    fn load_llm_proxy_url() -> Option<String> {
        // 1. 优先从当前进程的环境变量读取
        if let Ok(url) = env::var("LLM_PROXY_URL") {
            if !url.is_empty() {
                return Some(url);
            }
        }

        // 2. 从 shell 配置文件读取
        if let Ok(shell_config_env) = crate::EnvFile::load() {
            if let Some(url) = shell_config_env.get("LLM_PROXY_URL") {
                if !url.is_empty() {
                    return Some(url.clone());
                }
            }
        }

        None
    }

    fn load_llm_proxy_key() -> Option<String> {
        // 1. 优先从当前进程的环境变量读取
        if let Ok(key) = env::var("LLM_PROXY_KEY") {
            if !key.is_empty() {
                return Some(key);
            }
        }

        // 2. 从 shell 配置文件读取
        if let Ok(shell_config_env) = crate::EnvFile::load() {
            if let Some(key) = shell_config_env.get("LLM_PROXY_KEY") {
                if !key.is_empty() {
                    return Some(key.clone());
                }
            }
        }

        None
    }

    fn load_llm_deepseek_key() -> Option<String> {
        env::var("LLM_DEEPSEEK_KEY").ok()
    }

    fn load_llm_provider() -> String {
        env::var("LLM_PROVIDER").unwrap_or_else(|_| "openai".to_string())
    }

    // ==================== Codeup 配置 ====================
    fn load_codeup_project_id() -> Option<u64> {
        env::var("CODEUP_PROJECT_ID")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
    }

    fn load_codeup_csrf_token() -> Option<String> {
        env::var("CODEUP_CSRF_TOKEN").ok()
    }

    fn load_codeup_cookie() -> Option<String> {
        env::var("CODEUP_COOKIE").ok()
    }

    /// 重新加载环境变量（仅用于测试）
    /// 在测试中，可以强制替换实例
    #[cfg(test)]
    pub fn reload_for_test() {
        let new_settings = Self::from_env().expect("Failed to initialize Settings");
        *INSTANCE.get_or_init(|| RwLock::new(new_settings.clone()))
            .write().unwrap() = new_settings;
        CACHED.with(|c| c.set(std::ptr::null()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_initialization() {
        // 设置测试环境变量
        env::set_var("EMAIL", "test@example.com");
        env::set_var("JIRA_API_TOKEN", "test-token");
        env::set_var("JIRA_SERVICE_ADDRESS", "https://test.atlassian.net");

        // 测试初始化
        let settings = Settings::from_env().unwrap();
        assert_eq!(settings.email, "test@example.com");
        assert_eq!(settings.jira_api_token, "test-token");
        assert_eq!(settings.jira_service_address, "https://test.atlassian.net");
        assert_eq!(settings.log_output_folder_name, "logs");
        assert_eq!(settings.llm_provider, "openai"); // 默认值

        // 清理
        env::remove_var("EMAIL");
        env::remove_var("JIRA_API_TOKEN");
        env::remove_var("JIRA_SERVICE_ADDRESS");
    }

    #[test]
    fn test_boolean_flags() {
        env::set_var("EMAIL", "test@example.com");
        env::set_var("JIRA_API_TOKEN", "test-token");
        env::set_var("JIRA_SERVICE_ADDRESS", "https://test.atlassian.net");

        env::set_var("LOG_DELETE_WHEN_OPERATION_COMPLETED", "1");
        env::set_var("DISABLE_CHECK_PROXY", "1");

        let settings = Settings::from_env().unwrap();
        assert!(settings.log_delete_when_operation_completed);
        assert!(settings.disable_check_proxy);

        // 清理
        env::remove_var("EMAIL");
        env::remove_var("JIRA_API_TOKEN");
        env::remove_var("JIRA_SERVICE_ADDRESS");
        env::remove_var("LOG_DELETE_WHEN_OPERATION_COMPLETED");
        env::remove_var("DISABLE_CHECK_PROXY");
    }

    #[test]
    fn test_llm_provider() {
        env::set_var("EMAIL", "test@example.com");
        env::set_var("JIRA_API_TOKEN", "test-token");
        env::set_var("JIRA_SERVICE_ADDRESS", "https://test.atlassian.net");

        // 测试默认值
        let settings = Settings::from_env().unwrap();
        assert_eq!(settings.llm_provider, "openai");

        // 测试自定义值
        env::set_var("LLM_PROVIDER", "deepseek");
        let settings = Settings::from_env().unwrap();
        assert_eq!(settings.llm_provider, "deepseek");

        // 清理
        env::remove_var("EMAIL");
        env::remove_var("JIRA_API_TOKEN");
        env::remove_var("JIRA_SERVICE_ADDRESS");
        env::remove_var("LLM_PROVIDER");
    }
}

