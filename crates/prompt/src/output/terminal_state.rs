//! 终端渲染状态管理
//!
//! 统一管理 spinner 和 progress bar 的终端渲染状态，
//! 提供暂停和恢复接口，确保与日志输出不冲突。

use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Mutex, OnceLock,
    },
};

use crossterm::{
    cursor,
    terminal::{Clear, ClearType},
    QueueableCommand,
};

/// 渲染器信息
struct RendererInfo {
    redraw_callback: Box<dyn Fn() + Send + Sync>,
}

/// 全局终端渲染状态
struct TerminalState {
    /// 是否有活跃的渲染器
    active: AtomicBool,
    /// 是否处于暂停状态
    suspended: AtomicBool,
    /// 活跃渲染器计数（支持嵌套）
    active_count: AtomicUsize,
    /// 当前渲染器信息
    renderer: Mutex<Option<RendererInfo>>,
}

impl TerminalState {
    fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            suspended: AtomicBool::new(false),
            active_count: AtomicUsize::new(0),
            renderer: Mutex::new(None),
        }
    }
}

static TERMINAL_STATE: OnceLock<TerminalState> = OnceLock::new();

fn get_state() -> &'static TerminalState {
    TERMINAL_STATE.get_or_init(TerminalState::new)
}

/// 注册渲染器为活跃状态
///
/// # 参数
///
/// * `redraw_callback` - 重绘回调函数
pub fn register_renderer<F>(redraw_callback: F)
where
    F: Fn() + Send + Sync + 'static,
{
    let state = get_state();
    let count = state.active_count.fetch_add(1, Ordering::SeqCst);

    // 只有第一个渲染器才设置回调
    if count == 0 {
        state.active.store(true, Ordering::SeqCst);
        if let Ok(mut renderer) = state.renderer.lock() {
            *renderer = Some(RendererInfo {
                redraw_callback: Box::new(redraw_callback),
            });
        }
    }
}

/// 注销渲染器
///
/// 递减活跃渲染器计数。当最后一个渲染器注销时，
/// 清理全局状态并重置暂停标志。
pub fn unregister_renderer() {
    let state = get_state();
    let count = state.active_count.fetch_sub(1, Ordering::SeqCst);

    // 只有最后一个渲染器才清理状态
    if count == 1 {
        state.active.store(false, Ordering::SeqCst);
        state.suspended.store(false, Ordering::SeqCst);
        if let Ok(mut renderer) = state.renderer.lock() {
            *renderer = None;
        }
    }
}

/// 暂停终端渲染（供外部调用，如 tracing layer）
///
/// 清除当前行，为日志输出腾出空间。
pub fn suspend() {
    let state = get_state();

    // 只有当有活跃渲染器且未暂停时才执行
    if !state.active.load(Ordering::SeqCst) {
        return;
    }

    if state.suspended.swap(true, Ordering::SeqCst) {
        return; // 已经暂停
    }

    // 清除当前行
    let mut stderr = io::stderr();
    let _ = stderr.queue(cursor::MoveToColumn(0));
    let _ = stderr.queue(Clear(ClearType::CurrentLine));
    let _ = stderr.flush();
}

/// 恢复终端渲染（供外部调用，如 tracing layer）
///
/// 调用渲染器的重绘回调。
pub fn resume() {
    let state = get_state();

    // 只有当有活跃渲染器且已暂停时才执行
    if !state.active.load(Ordering::SeqCst) {
        return;
    }

    if !state.suspended.swap(false, Ordering::SeqCst) {
        return; // 未暂停
    }

    // 调用重绘回调
    if let Ok(renderer) = state.renderer.lock() {
        if let Some(ref info) = *renderer {
            (info.redraw_callback)();
        }
    }
}

/// 检查是否处于暂停状态
pub fn is_suspended() -> bool {
    get_state().suspended.load(Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_state_singleton() {
        let state1 = get_state();
        let state2 = get_state();
        assert!(std::ptr::eq(state1, state2));
    }

    #[test]
    fn test_renderer_registration_increments_count() {
        let state = get_state();
        let initial_count = state.active_count.load(Ordering::SeqCst);

        register_renderer(|| {});
        let after_register = state.active_count.load(Ordering::SeqCst);
        assert_eq!(after_register, initial_count + 1);

        unregister_renderer();
        let after_unregister = state.active_count.load(Ordering::SeqCst);
        assert_eq!(after_unregister, initial_count);
    }

    #[test]
    fn test_suspend_and_resume_basic_flow() {
        register_renderer(|| {});

        let state = get_state();
        if state.active.load(Ordering::SeqCst) {
            let was_suspended = state.suspended.load(Ordering::SeqCst);
            suspend();
            if !was_suspended {
                assert!(state.suspended.load(Ordering::SeqCst));
            }
            resume();
        }

        unregister_renderer();
    }
}
