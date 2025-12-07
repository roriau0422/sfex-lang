use crate::runtime::value::Value;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};

pub fn create_udp_module() -> Value {
    let mut methods = HashMap::new();

    // UDP.Bind("127.0.0.1:8080")
    methods.insert(
        "Bind".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("UDP.Bind requires 1 argument (address:port)".to_string());
            }

            let addr = args[0].to_display_string();

            match UdpSocket::bind(&addr) {
                Ok(socket) => Ok(create_udp_socket_object(socket)),
                Err(e) => Err(format!("UDP bind failed: {}", e)),
            }
        }))),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}

fn create_udp_socket_object(socket: UdpSocket) -> Value {
    let socket_arc = Arc::new(Mutex::new(socket));
    let mut methods = HashMap::new();

    // Socket.SendTo("data", "127.0.0.1:8081")
    let socket_send = socket_arc.clone();
    methods.insert(
        "SendTo".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 2 {
                return Err("Socket.SendTo requires 2 arguments (data, target_address)".to_string());
            }

            let data = args[0].to_display_string();
            let target = args[1].to_display_string();

            let socket_guard = socket_send.lock().unwrap();
            match socket_guard.send_to(data.as_bytes(), &target) {
                Ok(bytes_sent) => Ok(Value::from_number_string(&bytes_sent.to_string())
                    .unwrap_or(Value::default_number())),
                Err(e) => Err(format!("Failed to send data: {}", e)),
            }
        }))),
    );

    // Socket.ReceiveFrom(buffer_size) -> returns Map { data: "...", from: "..." }
    let socket_recv = socket_arc.clone();
    methods.insert(
        "ReceiveFrom".to_string(),
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

            let socket_guard = socket_recv.lock().unwrap();
            let mut buffer = vec![0u8; buffer_size];

            match socket_guard.recv_from(&mut buffer) {
                Ok((n, from_addr)) => {
                    buffer.truncate(n);
                    let data_str = match String::from_utf8(buffer) {
                        Ok(s) => s,
                        Err(_) => return Err("Received non-UTF8 data".to_string()),
                    };

                    let mut result = HashMap::new();
                    result.insert("Data".to_string(), Value::String(data_str));
                    result.insert("From".to_string(), Value::String(from_addr.to_string()));

                    Ok(Value::Map(Arc::new(std::sync::RwLock::new(result))))
                }
                Err(e) => Err(format!("Failed to receive data: {}", e)),
            }
        }))),
    );

    // Socket.Connect("127.0.0.1:8081") - sets default destination
    let socket_connect = socket_arc.clone();
    methods.insert(
        "Connect".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Socket.Connect requires 1 argument (address:port)".to_string());
            }

            let addr = args[0].to_display_string();
            let socket_guard = socket_connect.lock().unwrap();

            match socket_guard.connect(&addr) {
                Ok(_) => Ok(Value::Boolean(true)),
                Err(e) => Err(format!("Failed to connect: {}", e)),
            }
        }))),
    );

    // Socket.Send("data") - sends to connected address
    let socket_send_connected = socket_arc.clone();
    methods.insert(
        "Send".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Socket.Send requires 1 argument (data)".to_string());
            }

            let data = args[0].to_display_string();
            let socket_guard = socket_send_connected.lock().unwrap();

            match socket_guard.send(data.as_bytes()) {
                Ok(bytes_sent) => Ok(Value::from_number_string(&bytes_sent.to_string())
                    .unwrap_or(Value::default_number())),
                Err(e) => Err(format!("Failed to send data: {}", e)),
            }
        }))),
    );

    // Socket.Receive(buffer_size) - receives from connected address
    let socket_recv_connected = socket_arc.clone();
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

            let socket_guard = socket_recv_connected.lock().unwrap();
            let mut buffer = vec![0u8; buffer_size];

            match socket_guard.recv(&mut buffer) {
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

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}
