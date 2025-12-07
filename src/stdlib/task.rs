use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub fn create_task_module(interpreter: &Interpreter) -> Value {
    let mut methods = HashMap::new();
    let runtime = interpreter.runtime.clone();

    // The function receives no arguments and runs in the background
    let runtime_spawn = runtime.clone();
    methods.insert(
        "Spawn".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Task.Spawn requires 1 argument (function)".to_string());
            }

            let func = args[0].clone();

            // Verify it's a function
            if !matches!(func, Value::NativeFunction(_)) {
                return Err("Argument must be a function".to_string());
            }

            // Create cancellation token
            let cancel_token = Arc::new(std::sync::atomic::AtomicBool::new(false));

            // Spawn the task on the Tokio runtime
            let handle = runtime_spawn.spawn(async move {
                // Call the function
                match &func {
                    Value::NativeFunction(f) => match f(vec![]) {
                        Ok(result) => result,
                        Err(e) => {
                            eprintln!("Task error: {}", e);
                            Value::Boolean(false)
                        }
                    },
                    _ => Value::Boolean(false),
                }
            });

            // Wrap the JoinHandle in a TaskHandle with cancellation token
            Ok(Value::TaskHandle(
                Arc::new(std::sync::Mutex::new(Some(handle))),
                cancel_token,
            ))
        }))),
    );

    // Returns a list of results in the same order
    let runtime_waitall = runtime.clone();
    methods.insert(
        "WaitAll".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Task.WaitAll requires 1 argument (list of tasks)".to_string());
            }

            let tasks = match &args[0] {
                Value::List(l) => l.read().expect("lock poisoned").clone(),
                _ => return Err("Argument must be a list of TaskHandles".to_string()),
            };

            let runtime = runtime_waitall.clone();
            let mut results = Vec::new();

            for task in tasks {
                if let Value::TaskHandle(handle_mutex, _cancel_token) = task {
                    let mut handle_lock = handle_mutex.lock().unwrap();
                    if let Some(handle) = handle_lock.take() {
                        let result = runtime.block_on(async move {
                            match handle.await {
                                Ok(value) => value,
                                Err(e) => {
                                    eprintln!("Task panicked: {}", e);
                                    Value::Boolean(false)
                                }
                            }
                        });
                        results.push(result);
                    } else {
                        return Err("Task already awaited".to_string());
                    }
                } else {
                    return Err("List must contain only TaskHandles".to_string());
                }
            }

            Ok(Value::List(Arc::new(std::sync::RwLock::new(results))))
        }))),
    );

    // Returns the result of the first task to finish
    let runtime_waitany = runtime.clone();
    methods.insert(
        "WaitAny".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Task.WaitAny requires 1 argument (list of tasks)".to_string());
            }

            let tasks = match &args[0] {
                Value::List(l) => l.read().expect("lock poisoned").clone(),
                _ => return Err("Argument must be a list of TaskHandles".to_string()),
            };

            if tasks.is_empty() {
                return Err("Cannot wait on empty task list".to_string());
            }

            let runtime = runtime_waitany.clone();
            let mut handles = Vec::new();
            for task in tasks {
                if let Value::TaskHandle(handle_mutex, _cancel_token) = task {
                    let mut handle_lock = handle_mutex.lock().unwrap();
                    if let Some(handle) = handle_lock.take() {
                        handles.push(handle);
                    } else {
                        return Err("Task already awaited".to_string());
                    }
                } else {
                    return Err("List must contain only TaskHandles".to_string());
                }
            }

            let result = runtime.block_on(async move {
                use futures_util::stream::{FuturesUnordered, StreamExt};
                let mut futures = FuturesUnordered::new();
                for handle in handles {
                    futures.push(handle);
                }

                // Get the first one to complete
                if let Some(result) = futures.next().await {
                    match result {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("Task panicked: {}", e);
                            Value::Boolean(false)
                        }
                    }
                } else {
                    Value::Boolean(false)
                }
            });

            Ok(result)
        }))),
    );

    // Task.Cancel(task_handle) - Signal task to cancel
    methods.insert(
        "Cancel".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("Task.Cancel requires 1 argument (task handle)".to_string());
            }

            match &args[0] {
                Value::TaskHandle(_handle, cancel_token) => {
                    // Set cancellation flag to true
                    cancel_token.store(true, std::sync::atomic::Ordering::Relaxed);
                    Ok(Value::Boolean(true))
                }
                _ => Err("Argument must be a TaskHandle".to_string()),
            }
        }))),
    );

    // Task.IsCancelled(task_handle) - Check if task is cancelled
    methods.insert(
        "IsCancelled".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("Task.IsCancelled requires 1 argument (task handle)".to_string());
            }

            match &args[0] {
                Value::TaskHandle(_handle, cancel_token) => {
                    // Check cancellation flag
                    let is_cancelled = cancel_token.load(std::sync::atomic::Ordering::Relaxed);
                    Ok(Value::Boolean(is_cancelled))
                }
                _ => Err("Argument must be a TaskHandle".to_string()),
            }
        }))),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}
