use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

pub fn create_channel_module(interpreter: &Interpreter) -> Value {
    let mut methods = HashMap::new();
    let runtime = interpreter.runtime.clone();

    // Channel.Create(buffer_size) - Create a new channel
    // Returns a Map with "Send" and "Receive" methods
    methods.insert(
        "Create".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            let buffer_size = if args.is_empty() {
                10 // Default buffer size
            } else {
                match &args[0] {
                    Value::Number(n) => {
                        use bigdecimal::ToPrimitive;
                        n.to_usize().ok_or("Invalid buffer size")?
                    }
                    _ => return Err("Buffer size must be a number".to_string()),
                }
            };

            // Create the channel
            let (tx, rx) = mpsc::channel::<Value>(buffer_size);

            // Wrap receiver in Arc Mutex so it can be shared across Receive calls
            let rx_shared = Arc::new(tokio::sync::Mutex::new(rx));

            // Create Send function
            let tx_clone = tx.clone();
            let send_fn = Value::NativeFunction(Arc::new(Box::new(move |args| {
                if args.len() != 1 {
                    return Err("Send requires 1 argument (value to send)".to_string());
                }

                let value = args[0].clone();
                let tx = tx_clone.clone();

                // Use blocking_send since we are in a sync context
                tx.blocking_send(value)
                    .map_err(|_| "Channel closed".to_string())?;

                Ok(Value::Boolean(true))
            })));

            // Create Receive function
            let runtime_clone = runtime.clone();
            let rx_clone = rx_shared.clone();
            let receive_fn = Value::NativeFunction(Arc::new(Box::new(move |args| {
                if !args.is_empty() {
                    return Err("Receive requires 0 arguments".to_string());
                }

                let rx = rx_clone.clone();
                // Use runtime to block on async receive
                let result = runtime_clone.block_on(async {
                    let mut rx_guard = rx.lock().await;
                    rx_guard.recv().await
                });

                result.ok_or("Channel closed".to_string())
            })));

            // Create TryReceive function with timeout
            let runtime_try = runtime.clone();
            let rx_try = rx_shared.clone();
            let try_receive_fn = Value::NativeFunction(Arc::new(Box::new(move |args| {
                if args.len() != 1 {
                    return Err("TryReceive requires 1 argument (timeout in seconds)".to_string());
                }

                let timeout_secs = match &args[0] {
                    Value::Number(n) => {
                        use bigdecimal::ToPrimitive;
                        n.to_f64().ok_or("Invalid timeout")?
                    }
                    Value::FastNumber(f) => *f,
                    _ => return Err("Timeout must be a number".to_string()),
                };

                let rx = rx_try.clone();
                let duration = std::time::Duration::from_secs_f64(timeout_secs);

                // Use runtime to block on async receive with timeout
                let result = runtime_try.block_on(async {
                    let mut rx_guard = rx.lock().await;
                    tokio::time::timeout(duration, rx_guard.recv()).await
                });

                match result {
                    Ok(Some(value)) => {
                        // Received a value - wrap in Some
                        Ok(Value::Option(Box::new(Some(value))))
                    }
                    Ok(None) => {
                        // Channel closed - return None
                        Ok(Value::Option(Box::new(None)))
                    }
                    Err(_) => {
                        // Timeout - return None
                        Ok(Value::Option(Box::new(None)))
                    }
                }
            })));

            // Return a Map with Send, Receive, and TryReceive methods
            let mut channel_map = HashMap::new();
            channel_map.insert("Send".to_string(), send_fn);
            channel_map.insert("Receive".to_string(), receive_fn);
            channel_map.insert("TryReceive".to_string(), try_receive_fn);

            Ok(Value::Map(Arc::new(std::sync::RwLock::new(channel_map))))
        }))),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}
