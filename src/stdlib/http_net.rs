use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::Value;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;

pub fn create_http_module(interpreter: &Interpreter) -> Value {
    let mut methods = HashMap::new();
    let runtime = interpreter.runtime.clone();

    let runtime_get = runtime.clone();
    methods.insert(
        "Get".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(move |args| {
                    if args.is_empty() || args.len() > 2 {
                        return Err(
                            "HTTP.Get requires 1-2 arguments (url, optional headers map)".to_string()
                        );
                    }

                    let url = args[0].to_display_string();
                    let runtime = runtime_get.clone();

                    let result = runtime.block_on(async {
                        let client = Client::new();
                        let mut request = client.get(&url);

                        if args.len() == 2 {
                            if let Value::Map(headers_map) = &args[1] {
                                for (key, value) in headers_map
                                    .read()
                                    .expect("lock poisoned")
                                    .iter() {
                                    request = request.header(key, value.to_display_string());
                                }
                            }
                        }

                        request.send().await
                    });

                    match result {
                        Ok(response) => {
                            let response_obj = runtime_get.block_on(
                                create_response_object(response)
                            );
                            Ok(response_obj)
                        }
                        Err(e) => Err(format!("HTTP Error: {}", e)),
                    }
                })
            )
        )
    );

    let runtime_post = runtime.clone();
    methods.insert(
        "Post".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(move |args| {
                    if args.is_empty() || args.len() > 3 {
                        return Err(
                            "HTTP.Post requires 1-3 arguments (url, optional body, optional headers)".to_string()
                        );
                    }

                    let url = args[0].to_display_string();
                    let runtime = runtime_post.clone();

                    let result = runtime.block_on(async {
                        let client = Client::new();
                        let mut request = client.post(&url);

                        if args.len() >= 2 {
                            let body_str = args[1].to_display_string();
                            request = request.body(body_str);
                            request = request.header("Content-Type", "application/json");
                        }

                        if args.len() == 3 {
                            if let Value::Map(headers_map) = &args[2] {
                                for (key, value) in headers_map
                                    .read()
                                    .expect("lock poisoned")
                                    .iter() {
                                    request = request.header(key, value.to_display_string());
                                }
                            }
                        }

                        request.send().await
                    });

                    match result {
                        Ok(response) => {
                            let response_obj = runtime_post.block_on(
                                create_response_object(response)
                            );
                            Ok(response_obj)
                        }
                        Err(e) => Err(format!("HTTP Error: {}", e)),
                    }
                })
            )
        )
    );

    let runtime_put = runtime.clone();
    methods.insert(
        "Put".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(move |args| {
                    if args.is_empty() || args.len() > 3 {
                        return Err(
                            "HTTP.Put requires 1-3 arguments (url, optional body, optional headers)".to_string()
                        );
                    }

                    let url = args[0].to_display_string();
                    let runtime = runtime_put.clone();

                    let result = runtime.block_on(async {
                        let client = Client::new();
                        let mut request = client.put(&url);

                        if args.len() >= 2 {
                            let body_str = args[1].to_display_string();
                            request = request.body(body_str);
                            request = request.header("Content-Type", "application/json");
                        }

                        if args.len() == 3 {
                            if let Value::Map(headers_map) = &args[2] {
                                for (key, value) in headers_map
                                    .read()
                                    .expect("lock poisoned")
                                    .iter() {
                                    request = request.header(key, value.to_display_string());
                                }
                            }
                        }

                        request.send().await
                    });

                    match result {
                        Ok(response) => {
                            let response_obj = runtime_put.block_on(
                                create_response_object(response)
                            );
                            Ok(response_obj)
                        }
                        Err(e) => Err(format!("HTTP Error: {}", e)),
                    }
                })
            )
        )
    );

    let runtime_delete = runtime.clone();
    methods.insert(
        "Delete".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(move |args| {
                    if args.is_empty() || args.len() > 2 {
                        return Err(
                            "HTTP.Delete requires 1-2 arguments (url, optional headers)".to_string()
                        );
                    }

                    let url = args[0].to_display_string();
                    let runtime = runtime_delete.clone();

                    let result = runtime.block_on(async {
                        let client = Client::new();
                        let mut request = client.delete(&url);

                        if args.len() == 2 {
                            if let Value::Map(headers_map) = &args[1] {
                                for (key, value) in headers_map
                                    .read()
                                    .expect("lock poisoned")
                                    .iter() {
                                    request = request.header(key, value.to_display_string());
                                }
                            }
                        }

                        request.send().await
                    });

                    match result {
                        Ok(response) => {
                            let response_obj = runtime_delete.block_on(
                                create_response_object(response)
                            );
                            Ok(response_obj)
                        }
                        Err(e) => Err(format!("HTTP Error: {}", e)),
                    }
                })
            )
        )
    );

    let runtime_patch = runtime.clone();
    methods.insert(
        "Patch".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(move |args| {
                    if args.is_empty() || args.len() > 3 {
                        return Err(
                            "HTTP.Patch requires 1-3 arguments (url, optional body, optional headers)".to_string()
                        );
                    }

                    let url = args[0].to_display_string();
                    let runtime = runtime_patch.clone();

                    let result = runtime.block_on(async {
                        let client = Client::new();
                        let mut request = client.patch(&url);

                        if args.len() >= 2 {
                            let body_str = args[1].to_display_string();
                            request = request.body(body_str);
                            request = request.header("Content-Type", "application/json");
                        }

                        if args.len() == 3 {
                            if let Value::Map(headers_map) = &args[2] {
                                for (key, value) in headers_map
                                    .read()
                                    .expect("lock poisoned")
                                    .iter() {
                                    request = request.header(key, value.to_display_string());
                                }
                            }
                        }

                        request.send().await
                    });

                    match result {
                        Ok(response) => {
                            let response_obj = runtime_patch.block_on(
                                create_response_object(response)
                            );
                            Ok(response_obj)
                        }
                        Err(e) => Err(format!("HTTP Error: {}", e)),
                    }
                })
            )
        )
    );

    let runtime_getstream = runtime.clone();
    methods.insert(
        "GetStream".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(move |args| {
                    if args.is_empty() || args.len() > 2 {
                        return Err(
                            "HTTP.GetStream requires 1-2 arguments (url, optional headers)".to_string()
                        );
                    }

                    let url = args[0].to_display_string();
                    let runtime = runtime_getstream.clone();

                    let result = runtime.block_on(async {
                        let client = Client::new();
                        let mut request = client.get(&url);

                        if args.len() == 2 {
                            if let Value::Map(headers_map) = &args[1] {
                                for (key, value) in headers_map
                                    .read()
                                    .expect("lock poisoned")
                                    .iter() {
                                    request = request.header(key, value.to_display_string());
                                }
                            }
                        }

                        request.send().await
                    });

                    match result {
                        Ok(response) => {
                            let stream_obj = runtime_getstream.block_on(
                                create_stream_object(response, runtime_getstream.clone())
                            );
                            Ok(stream_obj)
                        }
                        Err(e) => Err(format!("HTTP Error: {}", e)),
                    }
                })
            )
        )
    );

    let runtime_poststream = runtime.clone();
    methods.insert(
        "PostStream".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(move |args| {
                    if args.is_empty() || args.len() > 3 {
                        return Err(
                            "HTTP.PostStream requires 1-3 arguments (url, optional body, optional headers)".to_string()
                        );
                    }

                    let url = args[0].to_display_string();
                    let runtime = runtime_poststream.clone();

                    let result = runtime.block_on(async {
                        let client = Client::new();
                        let mut request = client.post(&url);

                        if args.len() >= 2 {
                            let body_str = args[1].to_display_string();
                            request = request.body(body_str);
                            request = request.header("Content-Type", "application/json");
                        }

                        if args.len() == 3 {
                            if let Value::Map(headers_map) = &args[2] {
                                for (key, value) in headers_map
                                    .read()
                                    .expect("lock poisoned")
                                    .iter() {
                                    request = request.header(key, value.to_display_string());
                                }
                            }
                        }

                        request.send().await
                    });

                    match result {
                        Ok(response) => {
                            let stream_obj = runtime_poststream.block_on(
                                create_stream_object(response, runtime_poststream.clone())
                            );
                            Ok(stream_obj)
                        }
                        Err(e) => Err(format!("HTTP Error: {}", e)),
                    }
                })
            )
        )
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}

async fn create_stream_object(
    response: reqwest::Response,
    _runtime: Arc<tokio::runtime::Runtime>
) -> Value {
    let mut stream_map = HashMap::new();

    let status = response.status().as_u16();
    stream_map.insert(
        "Status".to_string(),
        Value::from_number_string(&status.to_string()).unwrap_or(Value::default_number())
    );

    stream_map.insert(
        "StatusText".to_string(),
        Value::String(response.status().canonical_reason().unwrap_or("Unknown").to_string())
    );

    if let Some(length) = response.content_length() {
        stream_map.insert(
            "ContentLength".to_string(),
            Value::from_number_string(&length.to_string()).unwrap_or(Value::default_number())
        );
    }

    let mut headers_map = HashMap::new();
    for (key, value) in response.headers() {
        if let Ok(v) = value.to_str() {
            headers_map.insert(key.to_string(), Value::String(v.to_string()));
        }
    }
    stream_map.insert(
        "Headers".to_string(),
        Value::Map(Arc::new(std::sync::RwLock::new(headers_map)))
    );

    let body = response.bytes().await.unwrap_or_default();
    let body_arc = Arc::new(std::sync::RwLock::new(body.to_vec()));
    let position_arc = Arc::new(std::sync::RwLock::new(0usize));

    let body_ref = body_arc.clone();
    let pos_ref = position_arc.clone();
    stream_map.insert(
        "ReadChunk".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(move |args| {
                    if args.len() != 1 {
                        return Err("ReadChunk requires 1 argument (size in bytes)".to_string());
                    }

                    let size = match &args[0] {
                        Value::Number(n) => n.to_string().parse::<usize>().unwrap_or(4096),
                        _ => {
                            return Err("size must be a number".to_string());
                        }
                    };

                    let body = body_ref.read().expect("lock poisoned");
                    let mut pos = pos_ref.write().expect("lock poisoned");

                    if *pos >= body.len() {
                        return Ok(Value::String(String::new()));
                    }

                    let end = (*pos + size).min(body.len());
                    let chunk = &body[*pos..end];
                    *pos = end;

                    let data = String::from_utf8_lossy(chunk).to_string();
                    Ok(Value::String(data))
                })
            )
        )
    );

    stream_map.insert(
        "Close".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| { Ok(Value::Boolean(true)) })))
    );

    Value::Map(Arc::new(std::sync::RwLock::new(stream_map)))
}

async fn create_response_object(response: reqwest::Response) -> Value {
    let mut response_map = HashMap::new();

    let status = response.status().as_u16();
    response_map.insert(
        "Status".to_string(),
        Value::from_number_string(&status.to_string()).unwrap_or(Value::default_number())
    );

    response_map.insert(
        "StatusText".to_string(),
        Value::String(response.status().canonical_reason().unwrap_or("Unknown").to_string())
    );

    let mut headers_map = HashMap::new();
    for (key, value) in response.headers() {
        if let Ok(v) = value.to_str() {
            headers_map.insert(key.to_string(), Value::String(v.to_string()));
        }
    }
    response_map.insert(
        "Headers".to_string(),
        Value::Map(Arc::new(std::sync::RwLock::new(headers_map)))
    );

    const WARN_SIZE: u64 = 100 * 1024 * 1024;
    if let Some(content_length) = response.content_length() {
        if content_length > WARN_SIZE {
            eprintln!(
                "\n⚠️  WARNING: Large HTTP response ({:.2} MB)",
                (content_length as f64) / (1024.0 * 1024.0)
            );
            eprintln!("   → High memory usage: Response loaded entirely into RAM");
            eprintln!("   → Network bottleneck: Download may take significant time");
            eprintln!("   → Consider: HTTP.GetStream() for chunked processing\n");
        }
    }

    match response.text().await {
        Ok(body) => {
            response_map.insert("Body".to_string(), Value::String(body));
        }
        Err(e) => {
            response_map.insert(
                "Body".to_string(),
                Value::String(format!("Error reading body: {}", e))
            );
        }
    }

    Value::Map(Arc::new(std::sync::RwLock::new(response_map)))
}
