//! Spinner 加载指示器

mod builder;
mod format;
#[allow(clippy::module_inception)]
mod spinner;

pub use builder::SpinnerBuilder;
pub use spinner::Spinner;

/// 便捷函数
pub fn spinner(message: impl Into<String>) -> SpinnerBuilder {
    SpinnerBuilder::new(message)
}

// ============================================================================
// 宏定义
// ============================================================================

/// 格式化加载指示器宏
///
/// 提供格式化字符串的便捷方式，避免手动使用 `format!`。
///
/// # 示例
///
/// ```rust,no_run
/// use toolkit::spinner;
///
/// # fn main() {
/// let spinner = spinner!("正在处理 {}...", "文件");
/// # }
/// ```
#[macro_export]
macro_rules! spinner {
    ($($arg:tt)*) => {
        $crate::spinner(format!($($arg)*))
    };
}

/// 使用 spinner 执行操作并在成功时显示结果
///
/// 该宏会：
/// 1. 显示一个 spinner 加载指示器
/// 2. 执行操作函数（返回 `Result<T, E>`）
/// 3. 如果操作成功，调用显示函数显示结果
/// 4. 如果操作失败，只停止 spinner（不显示错误）
///
/// # 参数
///
/// - `$message`: spinner 显示的消息文本
/// - `$operation_fn`: 操作函数，返回 `Result<T, E>`
/// - `$display_fn`: 显示函数，接受 `&T` 作为参数，用于显示操作成功的结果
///
/// # 示例
///
/// ```rust,no_run
/// use toolkit::spinner_then;
///
/// # struct OperationResult;
/// # struct DisplayService;
/// # impl DisplayService {
/// #     fn github(_: &OperationResult) {}
/// # }
/// # struct OperationService;
/// # impl OperationService {
/// #     fn verify_github_config(&self, _: &()) -> Result<OperationResult, String> {
/// #         Ok(OperationResult)
/// #     }
/// # }
/// # let operation_service = OperationService;
/// # let settings = ();
///
/// spinner_then!(
///     "Verifying GitHub configuration...",
///     || operation_service.verify_github_config(&settings.github),
///     DisplayService::github
/// );
/// ```
#[macro_export]
macro_rules! spinner_then {
    ($message:expr, $operation_fn:expr, $display_fn:expr $(,)?) => {{
        let spinner = $crate::Spinner::new($message);
        let spinner_instance = spinner.start();
        let result = $operation_fn();
        spinner_instance.stop();
        if let Ok(ref val) = result {
            $display_fn(val);
        }
        result
    }};
}
