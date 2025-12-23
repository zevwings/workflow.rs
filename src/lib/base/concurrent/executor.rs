//! 并发任务执行器实现

use color_eyre::{eyre::eyre, Result};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// 任务列表类型别名
type TaskList<T, E> = Vec<(String, Box<dyn Fn() -> Result<T, E> + Send + Sync>)>;

/// 任务结果
#[derive(Debug, Clone)]
pub enum TaskResult<T, E> {
    /// 任务成功完成
    Success(T),
    /// 任务执行失败
    Failure(E),
}

/// 并发任务执行器
///
/// 用于并行执行多个任务，支持并发数限制和结果收集。
///
/// # 示例
///
/// ```rust
/// use workflow::base::concurrent::{ConcurrentExecutor, TaskResult};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let executor = ConcurrentExecutor::new(5); // 最大并发数 5
///
/// let tasks: Vec<(String, Box<dyn Fn() -> Result<String, String> + Send + Sync>)> = vec![
///     ("task1".to_string(), Box::new(|| -> Result<String, String> { Ok("result1".to_string()) })),
///     ("task2".to_string(), Box::new(|| -> Result<String, String> { Ok("result2".to_string()) })),
/// ];
///
/// let results = executor.execute(tasks)?;
/// for (name, result) in results {
///     match result {
///         TaskResult::Success(value) => println!("{}: success - {}", name, value),
///         TaskResult::Failure(err) => println!("{}: failed - {}", name, err),
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct ConcurrentExecutor {
    /// 最大并发数
    max_concurrent: usize,
}

impl ConcurrentExecutor {
    /// 创建新的并发执行器
    ///
    /// # 参数
    ///
    /// * `max_concurrent` - 最大并发数（同时执行的任务数）
    ///
    /// # 返回
    ///
    /// 返回 `ConcurrentExecutor` 实例
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent: max_concurrent.max(1),
        }
    }

    /// 执行多个任务（并行）
    ///
    /// # 参数
    ///
    /// * `tasks` - 任务列表，每个任务是一个元组 `(标识符, 任务函数)`
    ///
    /// # 返回
    ///
    /// 返回任务结果列表，每个结果是一个元组 `(标识符, 任务结果)`
    ///
    /// # 错误
    ///
    /// 如果线程创建或等待失败，返回相应的错误
    pub fn execute<T, E>(&self, tasks: TaskList<T, E>) -> Result<Vec<(String, TaskResult<T, E>)>>
    where
        T: Send + 'static,
        E: Send + 'static,
    {
        if tasks.is_empty() {
            return Ok(Vec::new());
        }

        // 如果只有一个任务，直接执行（避免线程开销）
        if tasks.len() == 1 {
            let (name, task) = tasks
                .into_iter()
                .next()
                .ok_or_else(|| eyre!("Expected exactly one task, but got none"))?;
            let result = match task() {
                Ok(value) => TaskResult::Success(value),
                Err(err) => TaskResult::Failure(err),
            };
            return Ok(vec![(name, result)]);
        }

        let max_concurrent = self.max_concurrent.min(tasks.len());

        // 结果通道
        let (tx, rx) = mpsc::channel();

        // 分批处理：将任务分成多个批次，每批最多 max_concurrent 个并行执行
        // 注意：我们不能直接 clone Box<dyn Fn()>，所以需要将任务移动到线程中
        let mut handles = Vec::new();
        let mut tasks_iter = tasks.into_iter();

        loop {
            let mut chunk = Vec::new();
            for _ in 0..max_concurrent {
                if let Some(task) = tasks_iter.next() {
                    chunk.push(task);
                } else {
                    break;
                }
            }

            if chunk.is_empty() {
                break;
            }

            let tx = tx.clone();

            let handle = thread::spawn(move || {
                for (name, task) in chunk {
                    let result = match task() {
                        Ok(value) => TaskResult::Success(value),
                        Err(err) => TaskResult::Failure(err),
                    };
                    tx.send((name, result)).ok();
                }
            });

            handles.push(handle);
        }

        // 关闭发送端
        drop(tx);

        // 收集结果
        let mut results = Vec::new();
        for result in rx {
            results.push(result);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().map_err(|e| eyre!("Thread join error: {:?}", e))?;
        }

        Ok(results)
    }

    /// 执行多个任务（并行），带进度回调
    ///
    /// # 参数
    ///
    /// * `tasks` - 任务列表
    /// * `on_progress` - 进度回调函数，参数为 `(任务标识符, 是否成功, 错误信息)`
    ///
    /// # 返回
    ///
    /// 返回任务结果列表
    pub fn execute_with_progress<T, E, F>(
        &self,
        tasks: TaskList<T, E>,
        on_progress: Option<Arc<Mutex<Option<F>>>>,
    ) -> Result<Vec<(String, TaskResult<T, E>)>>
    where
        T: Send + 'static,
        E: Send + 'static + ToString,
        F: Fn(&str, bool, Option<&str>) + Send + Sync + 'static,
    {
        if tasks.is_empty() {
            return Ok(Vec::new());
        }

        // 如果只有一个任务，直接执行
        if tasks.len() == 1 {
            let (name, task) = tasks
                .into_iter()
                .next()
                .ok_or_else(|| eyre!("Expected exactly one task, but got none"))?;
            let result = match task() {
                Ok(value) => {
                    if let Some(ref callback) = on_progress {
                        if let Ok(cb_guard) = callback.lock() {
                            if let Some(ref cb_fn) = *cb_guard {
                                cb_fn(&name, true, None);
                            }
                        }
                    }
                    TaskResult::Success(value)
                }
                Err(err) => {
                    let err_msg = err.to_string();
                    if let Some(ref callback) = on_progress {
                        if let Ok(cb_guard) = callback.lock() {
                            if let Some(ref cb_fn) = *cb_guard {
                                cb_fn(&name, false, Some(&err_msg));
                            }
                        }
                    }
                    TaskResult::Failure(err)
                }
            };
            return Ok(vec![(name, result)]);
        }

        let max_concurrent = self.max_concurrent.min(tasks.len());

        // 结果通道
        let (tx, rx) = mpsc::channel();

        // 分批处理
        let mut handles = Vec::new();
        let mut tasks_iter = tasks.into_iter();

        loop {
            let mut chunk = Vec::new();
            for _ in 0..max_concurrent {
                if let Some(task) = tasks_iter.next() {
                    chunk.push(task);
                } else {
                    break;
                }
            }

            if chunk.is_empty() {
                break;
            }

            let tx = tx.clone();
            let callback = on_progress.clone();

            let handle = thread::spawn(move || {
                for (name, task) in chunk {
                    let result = match task() {
                        Ok(value) => {
                            if let Some(ref cb) = callback {
                                if let Ok(cb_guard) = cb.lock() {
                                    if let Some(ref cb_fn) = *cb_guard {
                                        cb_fn(&name, true, None);
                                    }
                                }
                            }
                            TaskResult::Success(value)
                        }
                        Err(err) => {
                            let err_msg = err.to_string();
                            if let Some(ref cb) = callback {
                                if let Ok(cb_guard) = cb.lock() {
                                    if let Some(ref cb_fn) = *cb_guard {
                                        cb_fn(&name, false, Some(&err_msg));
                                    }
                                }
                            }
                            TaskResult::Failure(err)
                        }
                    };
                    tx.send((name, result)).ok();
                }
            });

            handles.push(handle);
        }

        // 关闭发送端
        drop(tx);

        // 收集结果
        let mut results = Vec::new();
        for result in rx {
            results.push(result);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().map_err(|e| eyre!("Thread join error: {:?}", e))?;
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_execute_empty() {
        let executor = ConcurrentExecutor::new(5);
        let results = executor.execute::<String, String>(Vec::new()).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_execute_single() {
        let executor = ConcurrentExecutor::new(5);
        let tasks = vec![(
            "task1".to_string(),
            Box::new(|| -> Result<String, String> { Ok("result1".to_string()) })
                as Box<dyn Fn() -> Result<String, String> + Send + Sync>,
        )];
        let results = executor.execute(tasks).unwrap();
        assert_eq!(results.len(), 1);
        match &results[0].1 {
            TaskResult::Success(value) => assert_eq!(value, "result1"),
            TaskResult::Failure(_) => panic!("Expected success"),
        }
    }

    #[test]
    fn test_execute_multiple() {
        let executor = ConcurrentExecutor::new(2);
        let tasks = vec![
            (
                "task1".to_string(),
                Box::new(|| -> Result<String, String> {
                    sleep(Duration::from_millis(10));
                    Ok("result1".to_string())
                }) as Box<dyn Fn() -> Result<String, String> + Send + Sync>,
            ),
            (
                "task2".to_string(),
                Box::new(|| -> Result<String, String> {
                    sleep(Duration::from_millis(10));
                    Ok("result2".to_string())
                }) as Box<dyn Fn() -> Result<String, String> + Send + Sync>,
            ),
            (
                "task3".to_string(),
                Box::new(|| -> Result<String, String> {
                    sleep(Duration::from_millis(10));
                    Ok("result3".to_string())
                }) as Box<dyn Fn() -> Result<String, String> + Send + Sync>,
            ),
        ];
        let results = executor.execute(tasks).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_execute_with_failure() {
        let executor = ConcurrentExecutor::new(5);
        let tasks = vec![
            (
                "task1".to_string(),
                Box::new(|| -> Result<String, String> { Ok("result1".to_string()) })
                    as Box<dyn Fn() -> Result<String, String> + Send + Sync>,
            ),
            (
                "task2".to_string(),
                Box::new(|| -> Result<String, String> { Err("error".to_string()) })
                    as Box<dyn Fn() -> Result<String, String> + Send + Sync>,
            ),
        ];
        let results = executor.execute(tasks).unwrap();
        assert_eq!(results.len(), 2);
        match &results[0].1 {
            TaskResult::Success(_) => {}
            TaskResult::Failure(_) => {}
        }
    }
}
