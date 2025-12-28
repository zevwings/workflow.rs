#![allow(clippy::test_attr_in_doctest)]

//! 测试环境快照和恢复
//!
//! 提供测试环境状态的快照和恢复功能，用于在测试中保存和恢复环境状态。
//!
//! ## 使用示例
//!
//! ```rust
//! use tests::common::snapshot::TestSnapshot;
//!
//! #[test]
//! fn test_with_snapshot() {
//!     // 捕获当前环境状态
//!     let snapshot = TestSnapshot::capture();
//!
//!     // 修改环境变量
//!     std::env::set_var("TEST_VAR", "test_value");
//!
//!     // 执行测试...
//!
//!     // 恢复环境状态
//!     snapshot.restore();
//! }
//! ```

use color_eyre::Result;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

/// 测试环境快照
///
/// 捕获测试环境的当前状态，包括环境变量和当前工作目录。
/// 可以在测试后恢复这些状态。
pub struct TestSnapshot {
    /// 环境变量快照（变量名 -> 值，None 表示变量不存在）
    env_vars: HashMap<String, Option<String>>,
    /// 当前工作目录
    current_dir: PathBuf,
}

impl TestSnapshot {
    /// 捕获当前环境状态
    ///
    /// 记录所有环境变量和当前工作目录。
    ///
    /// # 返回
    ///
    /// 返回包含当前环境状态的 `TestSnapshot`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// let snapshot = TestSnapshot::capture();
    /// ```
    pub fn capture() -> Self {
        let mut env_vars = HashMap::new();

        // 捕获所有环境变量
        for (key, value) in env::vars() {
            env_vars.insert(key, Some(value));
        }

        // 捕获当前工作目录
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        Self {
            env_vars,
            current_dir,
        }
    }

    /// 恢复环境状态
    ///
    /// 将环境变量和当前工作目录恢复到快照时的状态。
    ///
    /// # 示例
    ///
    /// ```rust
    /// let snapshot = TestSnapshot::capture();
    /// // ... 修改环境 ...
    /// snapshot.restore();
    /// ```
    pub fn restore(&self) -> Result<()> {
        use color_eyre::eyre::Context;

        // 恢复当前工作目录
        env::set_current_dir(&self.current_dir).wrap_err_with(|| {
            format!(
                "Failed to restore current directory to {}",
                self.current_dir.display()
            )
        })?;

        // 获取当前所有环境变量
        let current_env_vars: HashMap<String, Option<String>> =
            env::vars().map(|(k, v)| (k, Some(v))).collect();

        // 恢复快照中的环境变量
        for (key, snapshot_value) in &self.env_vars {
            if let Some(value) = snapshot_value {
                // 变量在快照中存在，恢复其值
                env::set_var(key, value);
            } else {
                // 变量在快照中不存在，删除它
                env::remove_var(key);
            }
        }

        // 删除快照中不存在的环境变量（在快照后新增的变量）
        for key in current_env_vars.keys() {
            if !self.env_vars.contains_key(key) {
                env::remove_var(key);
            }
        }

        Ok(())
    }

    /// 获取快照时的环境变量值
    ///
    /// # 参数
    ///
    /// * `key` - 环境变量名
    ///
    /// # 返回
    ///
    /// 如果变量在快照中存在，返回 `Some(值)`；如果不存在，返回 `None`。
    pub fn get_env_var(&self, key: &str) -> Option<&String> {
        self.env_vars.get(key).and_then(|v| v.as_ref())
    }

    /// 获取快照时的当前工作目录
    #[allow(dead_code)]
    pub fn current_dir(&self) -> &PathBuf {
        &self.current_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试TestSnapshot捕获和恢复环境变量
    ///
    /// ## 测试目的
    /// 验证 `TestSnapshot` 能够正确捕获和恢复环境变量。
    ///
    /// ## 测试场景
    /// 1. 捕获当前环境状态
    /// 2. 设置新的环境变量
    /// 3. 恢复环境状态
    /// 4. 验证环境变量已恢复
    ///
    /// ## 预期结果
    /// - 环境变量正确恢复
    #[test]
    fn test_snapshot_env_vars() -> Result<()> {
        // 保存原始值
        let original_value = env::var("TEST_SNAPSHOT_VAR").ok();

        // 捕获快照
        let snapshot = TestSnapshot::capture();

        // 修改环境变量
        env::set_var("TEST_SNAPSHOT_VAR", "test_value");

        // 验证修改成功
        assert_eq!(env::var("TEST_SNAPSHOT_VAR").unwrap(), "test_value");

        // 恢复环境
        snapshot.restore()?;

        // 验证环境变量已恢复
        match original_value {
            Some(val) => assert_eq!(env::var("TEST_SNAPSHOT_VAR").unwrap(), val),
            None => assert!(env::var("TEST_SNAPSHOT_VAR").is_err()),
        }

        Ok(())
    }

    /// 测试TestSnapshot捕获和恢复当前工作目录
    ///
    /// ## 测试目的
    /// 验证 `TestSnapshot` 能够正确捕获和恢复当前工作目录。
    ///
    /// ## 测试场景
    /// 1. 捕获当前环境状态
    /// 2. 修改当前工作目录
    /// 3. 恢复环境状态
    /// 4. 验证当前工作目录已恢复
    ///
    /// ## 预期结果
    /// - 当前工作目录正确恢复
    #[test]
    fn test_snapshot_current_dir() -> Result<()> {
        // 捕获快照
        let original_dir = env::current_dir()?;
        let snapshot = TestSnapshot::capture();

        // 修改当前工作目录（如果可能）
        if let Ok(temp_dir) = std::fs::canonicalize(".") {
            let parent = temp_dir.parent();
            if let Some(parent) = parent {
                env::set_current_dir(parent)?;

                // 恢复环境
                snapshot.restore()?;

                // 验证当前工作目录已恢复
                assert_eq!(env::current_dir()?, original_dir);
            }
        }

        Ok(())
    }

    /// 测试TestSnapshot获取环境变量值
    ///
    /// ## 测试目的
    /// 验证 `TestSnapshot::get_env_var()` 能够正确获取快照时的环境变量值。
    ///
    /// ## 测试场景
    /// 1. 设置环境变量
    /// 2. 捕获快照
    /// 3. 修改环境变量
    /// 4. 从快照获取原始值
    ///
    /// ## 预期结果
    /// - 快照中的值与设置的值一致
    #[test]
    fn test_snapshot_get_env_var() {
        // 设置环境变量
        env::set_var("TEST_SNAPSHOT_GET_VAR", "original_value");

        // 捕获快照
        let snapshot = TestSnapshot::capture();

        // 修改环境变量
        env::set_var("TEST_SNAPSHOT_GET_VAR", "modified_value");

        // 从快照获取原始值
        assert_eq!(
            snapshot.get_env_var("TEST_SNAPSHOT_GET_VAR"),
            Some(&"original_value".to_string())
        );

        // 清理
        env::remove_var("TEST_SNAPSHOT_GET_VAR");
    }
}
