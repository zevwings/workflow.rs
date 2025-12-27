#![allow(clippy::test_attr_in_doctest)]

//! 环境变量隔离守卫
//!
//! 管理测试期间的环境变量修改，自动恢复原始值。
//!
//! # 使用示例
//!
//! ```rust
//! use tests::common::guards::EnvGuard;
//!
//! #[test]
//! fn test_with_env_isolation() {
//!     let mut guard = EnvGuard::new();
//!
//!     // 设置环境变量
//!     guard.set("TEST_VAR", "test_value");
//!
//!     // 测试代码...
//!
//!     // Drop时自动恢复环境变量
//! }
//! ```

use std::collections::HashMap;

/// 环境变量隔离守卫
///
/// 管理测试期间的环境变量修改，自动恢复原始值。
/// 使用 RAII 模式确保环境变量在作用域结束时恢复到原始值。
pub struct EnvGuard {
    /// 记录的环境变量原始值
    /// Key: 环境变量名
    /// Value: 原始值（None 表示原本不存在）
    original_vars: HashMap<String, Option<String>>,
}

impl EnvGuard {
    /// 创建新的环境变量守卫
    ///
    /// # 返回
    ///
    /// 返回新的 `EnvGuard` 实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::guards::EnvGuard;
    ///
    /// let mut guard = EnvGuard::new();
    /// ```
    pub fn new() -> Self {
        Self {
            original_vars: HashMap::new(),
        }
    }

    /// 设置环境变量（自动记录原始值）
    ///
    /// 如果环境变量原本不存在，记录为 `None`。
    /// 如果环境变量原本存在，记录原始值。
    ///
    /// # 参数
    ///
    /// * `key` - 环境变量名
    /// * `value` - 环境变量值
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::guards::EnvGuard;
    ///
    /// let mut guard = EnvGuard::new();
    /// guard.set("HOME", "/tmp/test");
    /// ```
    pub fn set(&mut self, key: &str, value: &str) {
        // 记录原始值（如果存在）
        if !self.original_vars.contains_key(key) {
            let original = std::env::var(key).ok();
            self.original_vars.insert(key.to_string(), original);
        }

        // 设置新值
        std::env::set_var(key, value);
    }

    /// 移除环境变量（自动记录原始值）
    ///
    /// 如果环境变量原本不存在，记录为 `None`。
    /// 如果环境变量原本存在，记录原始值。
    ///
    /// # 参数
    ///
    /// * `key` - 环境变量名
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::guards::EnvGuard;
    ///
    /// let mut guard = EnvGuard::new();
    /// guard.remove("HOME");
    /// ```
    pub fn remove(&mut self, key: &str) {
        // 记录原始值（如果存在）
        if !self.original_vars.contains_key(key) {
            let original = std::env::var(key).ok();
            self.original_vars.insert(key.to_string(), original);
        }

        // 移除环境变量
        std::env::remove_var(key);
    }

    /// 设置多个环境变量
    ///
    /// # 参数
    ///
    /// * `vars` - 环境变量键值对列表
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::guards::EnvGuard;
    ///
    /// let mut guard = EnvGuard::new();
    /// guard.set_many(&[("HOME", "/tmp/test"), ("PATH", "/usr/bin")]);
    /// ```
    pub fn set_many(&mut self, vars: &[(&str, &str)]) {
        for (key, value) in vars {
            self.set(key, value);
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        // 恢复所有环境变量到原始值
        for (key, original_value) in &self.original_vars {
            match original_value {
                Some(value) => {
                    // 恢复原始值
                    std::env::set_var(key, value);
                }
                None => {
                    // 移除环境变量（原本不存在）
                    std::env::remove_var(key);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试EnvGuard设置和恢复环境变量
    ///
    /// ## 测试目的
    /// 验证 `EnvGuard::set()` 方法能够设置环境变量，并在drop时自动恢复原始值。
    ///
    /// ## 测试场景
    /// 1. 保存原始HOME环境变量值
    /// 2. 创建EnvGuard并设置HOME为测试值
    /// 3. 验证HOME已设置为测试值
    /// 4. Drop guard
    /// 5. 验证HOME已恢复为原始值
    ///
    /// ## 预期结果
    /// - 设置时，环境变量被正确设置
    /// - Drop后，环境变量恢复为原始值（或移除，如果原本不存在）
    #[test]
    fn test_env_guard_set_and_restore() -> color_eyre::Result<()> {
        let original_home = std::env::var("HOME").ok();

        {
            let mut guard = EnvGuard::new();
            guard.set("HOME", "/tmp/test");

            // 验证环境变量已设置
            let home = std::env::var("HOME")
                .map_err(|e| color_eyre::eyre::eyre!("HOME should exist: {}", e))?;
            assert_eq!(home, "/tmp/test");
        }

        // 验证环境变量已恢复
        match original_home {
            Some(ref value) => {
                let home = std::env::var("HOME")
                    .map_err(|e| color_eyre::eyre::eyre!("HOME should exist: {}", e))?;
                assert_eq!(home, *value);
            }
            None => assert!(std::env::var("HOME").is_err()),
        }
        Ok(())
    }

    /// 测试EnvGuard移除和恢复环境变量
    ///
    /// ## 测试目的
    /// 验证 `EnvGuard::remove()` 方法能够移除环境变量，并在drop时自动恢复原始值。
    ///
    /// ## 测试场景
    /// 1. 设置一个测试环境变量
    /// 2. 创建EnvGuard并移除该环境变量
    /// 3. 验证环境变量已移除
    /// 4. Drop guard
    /// 5. 验证环境变量已恢复为原始值
    ///
    /// ## 预期结果
    /// - 移除时，环境变量被正确移除
    /// - Drop后，环境变量恢复为原始值
    #[test]
    fn test_env_guard_remove_and_restore() -> color_eyre::Result<()> {
        // 设置一个测试环境变量
        std::env::set_var("TEST_ENV_VAR", "original_value");

        {
            let mut guard = EnvGuard::new();
            guard.remove("TEST_ENV_VAR");

            // 验证环境变量已移除
            assert!(std::env::var("TEST_ENV_VAR").is_err());
        }

        // 验证环境变量已恢复
        let test_var = std::env::var("TEST_ENV_VAR")
            .map_err(|e| color_eyre::eyre::eyre!("TEST_ENV_VAR should exist: {}", e))?;
        assert_eq!(test_var, "original_value");

        // 清理
        std::env::remove_var("TEST_ENV_VAR");
        Ok(())
    }

    /// 测试EnvGuard批量设置环境变量
    ///
    /// ## 测试目的
    /// 验证 `EnvGuard::set_many()` 方法能够批量设置多个环境变量，并在drop时自动恢复所有原始值。
    ///
    /// ## 测试场景
    /// 1. 保存原始HOME和PATH环境变量值
    /// 2. 创建EnvGuard并批量设置多个环境变量
    /// 3. 验证所有环境变量已设置
    /// 4. Drop guard
    /// 5. 验证所有环境变量已恢复为原始值
    ///
    /// ## 预期结果
    /// - 批量设置时，所有环境变量被正确设置
    /// - Drop后，所有环境变量恢复为原始值
    #[test]
    fn test_env_guard_set_many() -> color_eyre::Result<()> {
        let original_home = std::env::var("HOME").ok();
        let original_path = std::env::var("PATH").ok();

        {
            let mut guard = EnvGuard::new();
            guard.set_many(&[("HOME", "/tmp/test"), ("PATH", "/usr/bin")]);

            // 验证环境变量已设置
            let home = std::env::var("HOME")
                .map_err(|e| color_eyre::eyre::eyre!("HOME should exist: {}", e))?;
            let path = std::env::var("PATH")
                .map_err(|e| color_eyre::eyre::eyre!("PATH should exist: {}", e))?;
            assert_eq!(home, "/tmp/test");
            assert_eq!(path, "/usr/bin");
        }

        // 验证环境变量已恢复
        match original_home {
            Some(ref value) => {
                let home = std::env::var("HOME")
                    .map_err(|e| color_eyre::eyre::eyre!("HOME should exist: {}", e))?;
                assert_eq!(home, *value);
            }
            None => assert!(std::env::var("HOME").is_err()),
        }
        match original_path {
            Some(ref value) => {
                let path = std::env::var("PATH")
                    .map_err(|e| color_eyre::eyre::eyre!("PATH should exist: {}", e))?;
                assert_eq!(path, *value);
            }
            None => assert!(std::env::var("PATH").is_err()),
        }
        Ok(())
    }
}
