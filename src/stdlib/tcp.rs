use crate::runtime::value::Value;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

pub fn create_tcp_module() -> Value {
    let mut methods = HashMap::new();

    // TCP.Connect("127.0.0.1:8080")
    methods.insert(
        "Connect".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("TCP.Connect requires 1 argument (address:port)".to_string());
            }

            let addr = args[0].to_display_string();

            match TcpStream::connect(&addr) {
                Ok(stream) => Ok(create_tcp_connection_object(stream)),
                Err(e) => Err(format!("TCP connection failed: {}", e)),
            }
        }))),
    );

    // TCP.Listen("127.0.0.1:8080")
    methods.insert(
        "Listen".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("TCP.Listen requires 1 argument (address:port)".to_string());
            }

            let addr = args[0].to_display_string();

            match TcpListener::bind(&addr) {
                Ok(listener) => Ok(create_tcp_listener_object(listener)),
                Err(e) => Err(format!("TCP bind failed: {}", e)),
            }
        }))),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}

fn create_tcp_connection_object(stream: TcpStream) -> Value {
    let stream_arc = Arc::new(Mutex::new(stream));
    let mut methods = HashMap::new();

    // Connection.Send("data")
    let stream_send = stream_arc.clone();
    methods.insert(
        "Send".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Connection.Send requires 1 argument (data)".to_string());
            }

            let data = args[0].to_display_string();
            let mut stream_guard = stream_send.lock().unwrap();

            match stream_guard.write_all(data.as_bytes()) {
                Ok(_) => {
                    stream_guard.flush().ok();
                    Ok(Value::Boolean(true))
                }
                Err(e) => Err(format!("Failed to send data: {}", e)),
            }
        }))),
    );

    // Connection.Receive(buffer_size)
    let stream_recv = stream_arc.clone();
    methods.insert(
        "Receive".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            let buffer_size = if args.is_empty() {
                1024
            } else {
                match &args[0] {
                    Value::Number(n) => {
                        use bigdecimal::ToPrimitive;
                        n.to_usize().unwrap_or(1024)
                    }
                    _ => 1024,
                }
            };

            let mut stream_guard = stream_recv.lock().unwrap();
            let mut buffer = vec![0u8; buffer_size];

            match stream_guard.read(&mut buffer) {
                Ok(n) => {
                    buffer.truncate(n);
                    match String::from_utf8(buffer) {
                        Ok(s) => Ok(Value::String(s)),
                        Err(_) => Err("Received non-UTF8 data".to_string()),
                    }
                }
                Err(e) => Err(format!("Failed to receive data: {}", e)),
            }
        }))),
    );

    // Connection.Close()
    let stream_close = stream_arc.clone();
    methods.insert(
        "Close".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            drop(stream_close.lock().unwrap());
            Ok(Value::Boolean(true))
        }))),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}

fn create_tcp_listener_object(listener: TcpListener) -> Value {
    let listener_arc = Arc::new(Mutex::new(listener));
    let mut methods = HashMap::new();

    // Listener.Accept() -> returns Connection object
    let listener_accept = listener_arc.clone();
    methods.insert(
        "Accept".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if !args.is_empty() {
                return Err("Listener.Accept requires no arguments".to_string());
            }

            let listener_guard = listener_accept.lock().unwrap();
            match listener_guard.accept() {
                Ok((stream, _addr)) => Ok(create_tcp_connection_object(stream)),
                Err(e) => Err(format!("Failed to accept connection: {}", e)),
            }
        }))),
    );

    // Listener.Close()
    let listener_close = listener_arc.clone();
    methods.insert(
        "Close".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            drop(listener_close.lock().unwrap());
            Ok(Value::Boolean(true))
        }))),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}
