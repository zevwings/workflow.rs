//! 日志系统集成
//!
//! 提供与 tracing 集成的日志缓冲和显示功能。

use chrono::Utc;
use std::sync::{Arc, Mutex};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer};

/// 日志条目
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub level: Level,
    pub message: String,
    pub module: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 日志缓冲区
#[derive(Clone)]
pub struct LogBuffer {
    entries: Arc<Mutex<Vec<LogEntry>>>,
    max_entries: usize,
}

impl LogBuffer {
    /// 创建新的日志缓冲区
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            max_entries,
        }
    }

    /// 添加日志条目
    pub fn add_entry(&self, entry: LogEntry) {
        let mut entries = self.entries.lock().unwrap();
        entries.push(entry);

        // 限制缓冲区大小
        if entries.len() > self.max_entries {
            entries.remove(0);
        }
    }

    /// 获取所有日志条目
    pub fn get_entries(&self) -> Vec<LogEntry> {
        self.entries.lock().unwrap().clone()
    }

    /// 清空日志缓冲区
    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }

    /// 获取日志条目数量
    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    /// 检查缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.lock().unwrap().is_empty()
    }
}

/// Ratatui Tracing Layer
///
/// 收集 tracing 事件并存储到 LogBuffer 中，供 LogViewer 使用。
pub struct RatatuiLayer {
    buffer: LogBuffer,
}

impl RatatuiLayer {
    /// 创建新的 RatatuiLayer
    pub fn new(max_entries: usize) -> Self {
        Self {
            buffer: LogBuffer::new(max_entries),
        }
    }

    /// 获取日志缓冲区
    pub fn buffer(&self) -> LogBuffer {
        self.buffer.clone()
    }
}

impl<S: Subscriber> Layer<S> for RatatuiLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut message = String::new();
        event.record(&mut MessageVisitor(&mut message));

        let entry = LogEntry {
            level: *event.metadata().level(),
            message,
            module: event.metadata().module_path().map(|s| s.to_string()),
            timestamp: Utc::now(),
        };

        self.buffer.add_entry(entry);
    }
}

/// 消息访问器，用于从 tracing Event 中提取消息
struct MessageVisitor<'a>(&'a mut String);

impl<'a> tracing::field::Visit for MessageVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            *self.0 = format!("{:?}", value);
        } else {
            // 处理其他字段
            if !self.0.is_empty() {
                self.0.push(' ');
            }
            self.0.push_str(&format!("{}={:?}", field.name(), value));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            *self.0 = value.to_string();
        } else {
            // 处理其他字段
            if !self.0.is_empty() {
                self.0.push(' ');
            }
            self.0.push_str(&format!("{}={}", field.name(), value));
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        self.0.push_str(&format!("{}={}", field.name(), value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        self.0.push_str(&format!("{}={}", field.name(), value));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        self.0.push_str(&format!("{}={}", field.name(), value));
    }
}
