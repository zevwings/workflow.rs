#![allow(clippy::field_reassign_with_default)]

//! 测试数据缓存系统
//!
//! 提供测试数据缓存功能，优化测试数据生成性能。

use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 是否启用缓存
    pub enabled: bool,
    /// 缓存过期时间（TTL）
    pub ttl: Option<Duration>,
    /// 最大缓存大小
    pub max_size: Option<usize>,
    /// 缓存淘汰策略
    pub eviction_policy: EvictionPolicy,
}

/// 缓存淘汰策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
    /// 最近最少使用（LRU）
    Lru,
    /// 先进先出（FIFO）
    Fifo,
    /// 基于时间（TTL）
    #[allow(dead_code)]
    Ttl,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl: Some(Duration::from_secs(3600)), // 默认 1 小时
            max_size: Some(1000),
            eviction_policy: EvictionPolicy::Lru,
        }
    }
}

/// 缓存的测试数据项
#[derive(Debug, Clone)]
struct CachedItem {
    /// 缓存的数据
    data: Value,
    /// 创建时间
    created_at: DateTime<Utc>,
    /// 访问次数
    access_count: usize,
    /// 最后访问时间
    last_accessed: DateTime<Utc>,
}

impl CachedItem {
    fn new(data: Value) -> Self {
        let now = Utc::now();
        Self {
            data,
            created_at: now,
            access_count: 0,
            last_accessed: now,
        }
    }

    fn access(&mut self) -> &Value {
        self.access_count += 1;
        self.last_accessed = Utc::now();
        &self.data
    }

    fn is_expired(&self, ttl: Option<Duration>) -> bool {
        if let Some(ttl) = ttl {
            let elapsed =
                Utc::now().signed_duration_since(self.created_at).to_std().unwrap_or_default();
            elapsed > ttl
        } else {
            false
        }
    }
}

/// 测试数据缓存
pub struct TestDataCache {
    /// 缓存存储
    cache: HashMap<String, CachedItem>,
    /// 缓存配置
    config: CacheConfig,
    /// 缓存统计信息
    stats: CacheStats,
    /// FIFO 队列（用于 FIFO 淘汰策略）
    fifo_queue: Vec<String>,
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// 缓存命中次数
    pub hits: usize,
    /// 缓存未命中次数
    pub misses: usize,
    /// 缓存淘汰次数
    pub evictions: usize,
    /// 当前缓存大小
    pub size: usize,
}

impl TestDataCache {
    /// 创建新的缓存实例
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: HashMap::new(),
            config,
            stats: CacheStats {
                hits: 0,
                misses: 0,
                evictions: 0,
                size: 0,
            },
            fifo_queue: Vec::new(),
        }
    }

    /// 获取缓存数据
    pub fn get(&mut self, key: &str) -> Option<&Value> {
        if !self.config.enabled {
            self.stats.misses += 1;
            return None;
        }

        // 先检查是否存在和是否过期
        let should_remove = self
            .cache
            .get(key)
            .map(|item| item.is_expired(self.config.ttl))
            .unwrap_or(false);

        if should_remove {
            self.cache.remove(key);
            self.stats.misses += 1;
            self.stats.size = self.cache.len();
            return None;
        }

        // 更新访问信息并返回数据
        if let Some(item) = self.cache.get_mut(key) {
            let data = item.access();
            self.stats.hits += 1;
            Some(data)
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// 存储缓存数据
    pub fn put(&mut self, key: String, value: Value) {
        if !self.config.enabled {
            return;
        }

        // 检查是否需要淘汰
        if let Some(max_size) = self.config.max_size {
            if self.cache.len() >= max_size && !self.cache.contains_key(&key) {
                self.evict();
            }
        }

        // 存储新数据
        let item = CachedItem::new(value);
        self.cache.insert(key.clone(), item);
        self.stats.size = self.cache.len();

        // 更新 FIFO 队列
        if self.config.eviction_policy == EvictionPolicy::Fifo && !self.fifo_queue.contains(&key) {
            self.fifo_queue.push(key);
        }
    }

    /// 使缓存项失效
    pub fn invalidate(&mut self, key: &str) {
        self.cache.remove(key);
        self.fifo_queue.retain(|k| k != key);
        self.stats.size = self.cache.len();
    }

    /// 清空所有缓存
    pub fn clear(&mut self) {
        self.cache.clear();
        self.fifo_queue.clear();
        self.stats.size = 0;
    }

    /// 获取缓存统计信息
    pub fn get_stats(&self) -> &CacheStats {
        &self.stats
    }

    /// 执行缓存淘汰
    fn evict(&mut self) {
        let key_to_remove = match self.config.eviction_policy {
            EvictionPolicy::Lru => {
                // 找到最近最少使用的项
                self.cache
                    .iter()
                    .min_by_key(|(_, item)| item.last_accessed)
                    .map(|(key, _)| key.clone())
            }
            EvictionPolicy::Fifo => {
                // 移除队列中最旧的项
                self.fifo_queue.pop()
            }
            EvictionPolicy::Ttl => {
                // 找到第一个过期的项
                self.cache.iter().find_map(|(key, item)| {
                    if item.is_expired(self.config.ttl) {
                        Some(key.clone())
                    } else {
                        None
                    }
                })
            }
        };

        if let Some(key) = key_to_remove {
            self.cache.remove(&key);
            self.stats.evictions += 1;
        }

        self.stats.size = self.cache.len();
    }

    /// 清理过期项
    #[allow(dead_code)]
    pub fn cleanup_expired(&mut self) {
        // 先收集所有过期的键
        let expired_keys: Vec<String> = self
            .cache
            .iter()
            .filter_map(|(key, item)| {
                if item.is_expired(self.config.ttl) {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        // 然后移除它们
        for key in expired_keys {
            self.cache.remove(&key);
            self.stats.evictions += 1;
        }

        self.stats.size = self.cache.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cache_get_put() {
        let mut cache = TestDataCache::new(CacheConfig::default());
        let key = "test_key";
        let value = json!({"test": "data"});

        // 首次获取应该未命中
        assert!(cache.get(key).is_none());
        assert_eq!(cache.stats.misses, 1);

        // 存储数据
        cache.put(key.to_string(), value.clone());

        // 再次获取应该命中
        let cached = cache.get(key);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), &value);
        assert_eq!(cache.stats.hits, 1);
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = TestDataCache::new(CacheConfig::default());
        let key = "test_key";
        let value = json!({"test": "data"});

        cache.put(key.to_string(), value);
        assert!(cache.get(key).is_some());

        cache.invalidate(key);
        assert!(cache.get(key).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = TestDataCache::new(CacheConfig::default());

        cache.put("key1".to_string(), json!({"data": 1}));
        cache.put("key2".to_string(), json!({"data": 2}));

        assert_eq!(cache.stats.size, 2);

        cache.clear();
        assert_eq!(cache.stats.size, 0);
    }

    #[test]
    fn test_cache_max_size() {
        let mut config = CacheConfig::default();
        config.max_size = Some(2);
        config.eviction_policy = EvictionPolicy::Fifo;

        let mut cache = TestDataCache::new(config);

        cache.put("key1".to_string(), json!({"data": 1}));
        cache.put("key2".to_string(), json!({"data": 2}));
        assert_eq!(cache.stats.size, 2);

        // 添加第三个项应该触发淘汰
        cache.put("key3".to_string(), json!({"data": 3}));
        assert_eq!(cache.stats.size, 2);
        assert!(cache.stats.evictions > 0);
    }
}
