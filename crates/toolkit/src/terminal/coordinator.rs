//! Spinner 协调器
//!
//! 提供全局的 spinner 暂停/恢复回调注册机制。

use std::sync::{Mutex, OnceLock};

/// Spinner 处理器类型
type SuspendHandler = Box<dyn Fn() + Send + Sync>;
type ResumeHandler = Box<dyn Fn() + Send + Sync>;

/// Spinner 处理器
pub struct SpinnerHandlers {
    /// 暂停 spinner 的回调
    pub suspend: SuspendHandler,
    /// 恢复 spinner 的回调
    pub resume: ResumeHandler,
}

/// 全局 spinner 处理器
static SPINNER_HANDLERS: OnceLock<Mutex<Option<SpinnerHandlers>>> = OnceLock::new();

fn get_handlers() -> &'static Mutex<Option<SpinnerHandlers>> {
    SPINNER_HANDLERS.get_or_init(|| Mutex::new(None))
}

/// 注册 spinner 处理器
///
/// 在应用初始化时调用，注册 spinner 的暂停和恢复回调。
///
/// # 参数
///
/// * `suspend` - 暂停 spinner 的回调函数
/// * `resume` - 恢复 spinner 的回调函数
pub fn register_spinner_handlers<S, R>(suspend: S, resume: R)
where
    S: Fn() + Send + Sync + 'static,
    R: Fn() + Send + Sync + 'static,
{
    if let Ok(mut guard) = get_handlers().lock() {
        *guard = Some(SpinnerHandlers {
            suspend: Box::new(suspend),
            resume: Box::new(resume),
        });
    }
}

/// 暂停 spinner
///
/// 如果已注册处理器，调用暂停回调。
pub fn suspend_spinner() {
    if let Ok(guard) = get_handlers().lock() {
        if let Some(ref handlers) = *guard {
            (handlers.suspend)();
        }
    }
}

/// 恢复 spinner
///
/// 如果已注册处理器，调用恢复回调。
pub fn resume_spinner() {
    if let Ok(guard) = get_handlers().lock() {
        if let Some(ref handlers) = *guard {
            (handlers.resume)();
        }
    }
}
