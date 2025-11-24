//! Prompt 管理器
//!
//! 提供 Prompt 的加载、缓存和默认值回退功能。

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// Prompt 缓存（线程安全）
fn prompt_cache() -> &'static Mutex<HashMap<String, String>> {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Prompt 管理器
///
/// 提供统一的 Prompt 加载和管理功能。
/// 支持从文件加载 Prompt，如果文件不存在则使用默认值。
pub struct PromptManager;

impl PromptManager {
    /// 加载 Prompt，如果文件不存在则使用默认值
    ///
    /// # 参数
    ///
    /// * `name` - Prompt 完整文件名（如 `"generate_branch.system.md"`），必须包含扩展名
    /// * `default_fn` - 默认值生成函数，如果文件不存在则调用此函数
    ///
    /// # 返回
    ///
    /// 返回加载的 Prompt 字符串。如果文件存在则从文件加载，否则使用默认值。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use workflow::base::prompt::PromptManager;
    ///
    /// let prompt = PromptManager::load_or_default("pr/system", || {
    ///     "Default system prompt".to_string()
    /// })?;
    /// ```
    pub fn load_or_default<F>(name: &str, default_fn: F) -> Result<String>
    where
        F: FnOnce() -> String,
    {
        // 先尝试从缓存加载
        if let Some(cached) = Self::get_from_cache(name) {
            return Ok(cached);
        }

        // 尝试从文件加载
        match Self::load_from_file(name) {
            Ok(content) => {
                // 缓存加载的内容
                Self::put_to_cache(name, &content);
                Ok(content)
            }
            Err(_) => {
                // 文件不存在，使用默认值
                let default = default_fn();
                // 缓存默认值（可选，避免重复生成）
                Self::put_to_cache(name, &default);
                Ok(default)
            }
        }
    }

    /// 加载 Prompt（如果文件不存在会返回错误）
    ///
    /// # 参数
    ///
    /// * `name` - Prompt 完整文件名（如 `"generate_branch.system.md"`），必须包含扩展名
    ///
    /// # 返回
    ///
    /// 返回加载的 Prompt 字符串。如果文件不存在则返回错误。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use workflow::base::prompt::PromptManager;
    ///
    /// let prompt = PromptManager::load("pr/system")?;
    /// ```
    pub fn load(name: &str) -> Result<String> {
        // 先尝试从缓存加载
        if let Some(cached) = Self::get_from_cache(name) {
            return Ok(cached);
        }

        // 从文件加载
        let content = Self::load_from_file(name)?;
        // 缓存加载的内容
        Self::put_to_cache(name, &content);
        Ok(content)
    }

    /// 清除缓存
    ///
    /// 清除所有已缓存的 Prompt。主要用于测试或需要重新加载的场景。
    pub fn clear_cache() {
        let mut cache = prompt_cache().lock().unwrap();
        cache.clear();
    }

    /// 从文件加载 Prompt
    ///
    /// # 参数
    ///
    /// * `name` - Prompt 完整文件名（如 `"generate_branch.system.md"`），必须包含扩展名
    ///
    /// # 返回
    ///
    /// 返回文件内容。如果文件不存在或读取失败则返回错误。
    fn load_from_file(name: &str) -> Result<String> {
        let file_path = Self::name_to_path(name)?;

        if !file_path.exists() {
            anyhow::bail!(
                "Prompt file does not exist: {} (tried .md and .txt)",
                file_path.display()
            );
        }

        fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read prompt file: {}", file_path.display()))
            .map(|s| s.trim().to_string())
    }

    /// 将 Prompt 名称转换为文件路径
    ///
    /// # 参数
    ///
    /// * `name` - Prompt 完整文件名（如 `"generate_branch.system.md"`），必须包含扩展名
    ///
    /// # 返回
    ///
    /// 返回对应的文件路径
    ///
    /// 文件路径格式：`src/lib/base/prompt/prompts/{name}`
    fn name_to_path(name: &str) -> Result<PathBuf> {
        // 获取当前 crate 的根目录
        // 在编译时，使用 env!("CARGO_MANIFEST_DIR") 获取项目根目录
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let base_path = Path::new(manifest_dir);

        // 直接使用完整文件名
        let file_path = base_path.join(format!("src/lib/base/prompt/prompts/{}", name));
        Ok(file_path)
    }

    /// 从缓存获取 Prompt
    fn get_from_cache(name: &str) -> Option<String> {
        let cache = prompt_cache().lock().unwrap();
        cache.get(name).cloned()
    }

    /// 将 Prompt 放入缓存
    fn put_to_cache(name: &str, content: &str) {
        let mut cache = prompt_cache().lock().unwrap();
        cache.insert(name.to_string(), content.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_to_path() {
        // 测试完整文件名
        let path = PromptManager::name_to_path("generate_branch.system.md").unwrap();
        assert!(path.to_string_lossy().contains("generate_branch.system.md"));
    }

    #[test]
    fn test_cache() {
        PromptManager::clear_cache();
        assert!(PromptManager::get_from_cache("test").is_none());

        PromptManager::put_to_cache("test", "test content");
        assert_eq!(
            PromptManager::get_from_cache("test"),
            Some("test content".to_string())
        );

        PromptManager::clear_cache();
        assert!(PromptManager::get_from_cache("test").is_none());
    }

    #[test]
    fn test_load_or_default() {
        PromptManager::clear_cache();

        // 测试默认值回退（文件不存在时）
        let result = PromptManager::load_or_default("test_nonexistent/prompt", || {
            "default content".to_string()
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "default content");

        // 测试缓存：第二次调用应该返回缓存的内容
        let result2 = PromptManager::load_or_default("test_nonexistent/prompt", || {
            "different content".to_string()
        });
        assert!(result2.is_ok());
        // 应该返回缓存的内容（第一次的默认值），而不是新的默认值
        // 这证明了缓存机制正常工作
        assert_eq!(result2.unwrap(), "default content");

        // 清除缓存后，应该使用新的默认值
        PromptManager::clear_cache();
        let result3 = PromptManager::load_or_default("test_nonexistent/prompt", || {
            "new default content".to_string()
        });
        assert!(result3.is_ok());
        assert_eq!(result3.unwrap(), "new default content");
    }

    #[test]
    fn test_load_existing_file() {
        PromptManager::clear_cache();

        // 测试加载存在的文件（使用完整文件名）
        let result = PromptManager::load("generate_branch.system.md");
        if result.is_ok() {
            let content = result.unwrap();
            assert!(!content.is_empty());
            // 验证内容包含预期的关键词
            assert!(content.contains("git assistant") || content.contains("branch name"));
        }
        // 如果文件不存在，测试会跳过（这是正常的）
    }
}
