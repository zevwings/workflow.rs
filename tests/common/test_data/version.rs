//! 测试数据版本管理
//!
//! 提供测试数据版本控制和迁移功能。

use chrono::{DateTime, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// 版本变更类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// 添加字段
    Added,
    /// 修改字段
    Modified,
    /// 删除字段
    Removed,
}

/// 版本变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionChange {
    /// 变更的字段路径
    pub field: String,
    /// 旧值（如果有）
    pub old_value: Option<Value>,
    /// 新值（如果有）
    pub new_value: Option<Value>,
    /// 变更类型
    pub change_type: ChangeType,
}

/// 测试数据版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDataVersion {
    /// 版本号（语义化版本号，如 "1.0.0"）
    pub version: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 版本描述
    pub description: String,
    /// 变更列表
    pub changes: Vec<VersionChange>,
}

/// 兼容性检查结果
#[derive(Debug, Clone)]
pub struct CompatibilityResult {
    /// 是否兼容
    pub is_compatible: bool,
    /// 不兼容的原因
    #[allow(dead_code)]
    pub reason: String,
    /// 需要迁移的变更列表
    #[allow(dead_code)]
    pub migrations_needed: Vec<String>,
}

/// 迁移 trait
pub trait Migration: Send + Sync {
    /// 源版本
    #[allow(dead_code)]
    fn from_version(&self) -> &str;
    /// 目标版本
    #[allow(dead_code)]
    fn to_version(&self) -> &str;
    /// 执行迁移
    fn migrate(&self, data: &mut Value) -> Result<()>;
}

/// 测试数据版本管理器
pub struct TestDataVersionManager {
    /// 当前版本
    current_version: String,
    /// 版本列表
    #[allow(dead_code)]
    versions: Vec<TestDataVersion>,
    /// 迁移映射表
    migrations: HashMap<String, Box<dyn Migration>>,
}

impl TestDataVersionManager {
    /// 创建新的版本管理器
    pub fn new(current_version: String) -> Self {
        Self {
            current_version,
            versions: Vec::new(),
            migrations: HashMap::new(),
        }
    }

    /// 获取当前版本
    pub fn get_current_version(&self) -> &str {
        &self.current_version
    }

    /// 注册迁移
    pub fn register_migration(&mut self, version: &str, migration: Box<dyn Migration>) {
        self.migrations.insert(version.to_string(), migration);
    }

    /// 迁移到目标版本
    pub fn migrate_to(&mut self, target_version: &str, data: &mut Value) -> Result<()> {
        use color_eyre::eyre::Context;

        if self.current_version == target_version {
            return Ok(());
        }

        // 检查是否有迁移路径
        let migration_key = format!("{}->{}", self.current_version, target_version);
        if let Some(migration) = self.migrations.get(&migration_key) {
            migration.migrate(data).context(format!(
                "Failed to migrate from {} to {}",
                self.current_version, target_version
            ))?;
            self.current_version = target_version.to_string();
            Ok(())
        } else {
            Err(color_eyre::eyre::eyre!(
                "No migration path from {} to {}",
                self.current_version,
                target_version
            ))
        }
    }

    /// 检查版本兼容性
    pub fn check_compatibility(&self, version: &str) -> CompatibilityResult {
        // 简单的兼容性检查：主版本号相同则兼容
        let current_major = self
            .current_version
            .split('.')
            .next()
            .unwrap_or("0")
            .parse::<u32>()
            .unwrap_or(0);

        let target_major = version.split('.').next().unwrap_or("0").parse::<u32>().unwrap_or(0);

        if current_major == target_major {
            CompatibilityResult {
                is_compatible: true,
                reason: String::new(),
                migrations_needed: Vec::new(),
            }
        } else {
            CompatibilityResult {
                is_compatible: false,
                reason: format!(
                    "Major version mismatch: current={}, target={}",
                    current_major, target_major
                ),
                migrations_needed: vec![format!("{}->{}", self.current_version, version)],
            }
        }
    }

    /// 添加版本
    #[allow(dead_code)]
    pub fn add_version(&mut self, version: TestDataVersion) {
        self.versions.push(version);
    }

    /// 获取版本列表
    #[allow(dead_code)]
    pub fn get_versions(&self) -> &[TestDataVersion] {
        &self.versions
    }
}

/// 简单的字段添加迁移
pub struct AddFieldMigration {
    #[allow(dead_code)]
    from_version: String,
    #[allow(dead_code)]
    to_version: String,
    field_path: String,
    default_value: Value,
}

impl AddFieldMigration {
    pub fn new(
        from_version: String,
        to_version: String,
        field_path: String,
        default_value: Value,
    ) -> Self {
        Self {
            from_version,
            to_version,
            field_path,
            default_value,
        }
    }
}

impl Migration for AddFieldMigration {
    fn from_version(&self) -> &str {
        &self.from_version
    }

    fn to_version(&self) -> &str {
        &self.to_version
    }

    fn migrate(&self, data: &mut Value) -> Result<()> {
        // 简单的字段添加：如果字段不存在，则添加默认值
        if let Some(obj) = data.as_object_mut() {
            if !obj.contains_key(&self.field_path) {
                obj.insert(self.field_path.clone(), self.default_value.clone());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_version_manager() {
        let manager = TestDataVersionManager::new("1.0.0".to_string());
        assert_eq!(manager.get_current_version(), "1.0.0");
    }

    #[test]
    fn test_compatibility_check() {
        let manager = TestDataVersionManager::new("1.0.0".to_string());

        // 相同主版本应该兼容
        let result = manager.check_compatibility("1.2.0");
        assert!(result.is_compatible);

        // 不同主版本应该不兼容
        let result = manager.check_compatibility("2.0.0");
        assert!(!result.is_compatible);
    }

    #[test]
    fn test_add_field_migration() -> Result<()> {
        let mut manager = TestDataVersionManager::new("1.0.0".to_string());

        let migration = Box::new(AddFieldMigration::new(
            "1.0.0".to_string(),
            "1.1.0".to_string(),
            "new_field".to_string(),
            json!("default_value"),
        ));

        manager.register_migration("1.0.0->1.1.0", migration);

        let mut data = json!({"existing_field": "value"});
        manager.migrate_to("1.1.0", &mut data)?;

        assert_eq!(data["new_field"], "default_value");
        assert_eq!(manager.get_current_version(), "1.1.0");
        Ok(())
    }
}
