use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::Value;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub fn create_websocket_module(interpreter: &Interpreter) -> Value {
    let mut methods = HashMap::new();
    let runtime = interpreter.runtime.clone();

    // WebSocket.Connect("wss://echo.websocket.org")
    let runtime_connect = runtime.clone();
    methods.insert(
        "Connect".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("WebSocket.Connect requires 1 argument (url)".to_string());
            }

            let url = args[0].to_display_string();
            let runtime = runtime_connect.clone();

            let runtime_clone = runtime.clone();
            let result = runtime.block_on(async {
                match connect_async(&url).await {
                    Ok((ws_stream, _)) => {
                        let (write, read) = ws_stream.split();
                        Ok((write, read))
                    }
                    Err(e) => Err(format!("WebSocket connection failed: {}", e)),
                }
            });

            match result {
                Ok((write, read)) => {
                    // Store the connection parts with the runtime
                    let write_arc = Arc::new(Mutex::new(write));
                    let read_arc = Arc::new(Mutex::new(read));

                    Ok(create_websocket_object(write_arc, read_arc, runtime_clone))
                }
                Err(e) => Err(e),
            }
        }))),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}

fn create_websocket_object(
    write: Arc<
        std::sync::Mutex<
            futures_util::stream::SplitSink<
                tokio_tungstenite::WebSocketStream<
                    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
                >,
                Message,
            >,
        >,
    >,
    read: Arc<
        std::sync::Mutex<
            futures_util::stream::SplitStream<
                tokio_tungstenite::WebSocketStream<
                    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
                >,
            >,
        >,
    >,
    runtime: Arc<tokio::runtime::Runtime>,
) -> Value {
    let mut methods = HashMap::new();

    // Connection.Send("message")
    let write_clone = write.clone();
    let runtime_send = runtime.clone();
    methods.insert(
        "Send".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Connection.Send requires 1 argument (message)".to_string());
            }

            let message = args[0].to_display_string();
            let write_clone2 = write_clone.clone();

            runtime_send.block_on(async {
                let mut write_guard = write_clone2.lock().unwrap();
                match write_guard.send(Message::Text(message.into())).await {
                    Ok(_) => Ok(Value::Boolean(true)),
                    Err(e) => Err(format!("Failed to send message: {}", e)),
                }
            })
        }))),
    );

    // Connection.Receive() -> returns message or "" if closed
    let read_clone = read.clone();
    let runtime_recv = runtime.clone();
    methods.insert(
        "Receive".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if !args.is_empty() {
                return Err("Connection.Receive requires no arguments".to_string());
            }

            let read_clone2 = read_clone.clone();

            runtime_recv.block_on(async {
                let mut read_guard = read_clone2.lock().unwrap();
                match read_guard.next().await {
                    Some(Ok(msg)) => match msg {
                        Message::Text(text) => Ok(Value::String(text.to_string())),
                        Message::Binary(data) => {
                            Ok(Value::String(format!("<binary {} bytes>", data.len())))
                        }
                        Message::Close(_) => Ok(Value::String(String::new())),
                        _ => Ok(Value::String(String::new())),
                    },
                    Some(Err(e)) => Err(format!("Error receiving message: {}", e)),
                    None => Ok(Value::String(String::new())), // Connection closed
                }
            })
        }))),
    );

    // Connection.Close()
    let write_close = write.clone();
    let runtime_close = runtime.clone();
    methods.insert(
        "Close".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if !args.is_empty() {
                return Err("Connection.Close requires no arguments".to_string());
            }

            let write_clone3 = write_close.clone();

            runtime_close.block_on(async {
                let mut write_guard = write_clone3.lock().unwrap();
                match write_guard.send(Message::Close(None)).await {
                    Ok(_) => Ok(Value::Boolean(true)),
                    Err(e) => Err(format!("Failed to close connection: {}", e)),
                }
            })
        }))),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}
