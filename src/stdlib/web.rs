// Copyright 2025 Temuujin
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use crate::compiler::ast::Program;
use crate::compiler::lexer::Lexer;
use crate::compiler::parser::Parser;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::Value;
use bigdecimal::ToPrimitive;
use bytes::Bytes;
use futures_util::StreamExt;
use hyper::body::to_bytes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::net::SocketAddr;
use std::path::{Component, Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex, RwLock};
use std::task::{Context, Poll};
use std::time::SystemTime;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio_stream::wrappers::TcpListenerStream;

const DEFAULT_ADDR: &str = "127.0.0.1:8000";
const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

pub fn create_web_module() -> Value {
    let mut methods = HashMap::new();

    methods.insert(
        "Serve".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() < 2 || args.len() > 3 {
                return Err(
                    "Web.Serve requires 2-3 arguments (addr, handler_path, optional static_dir)"
                        .to_string(),
                );
            }

            let addr = args[0].to_display_string();
            let handler_path = args[1].to_display_string();
            let static_dir = if args.len() == 3 {
                Some(args[2].to_display_string())
            } else {
                None
            };

            serve(&addr, &handler_path, static_dir.as_deref())?;
            Ok(Value::Boolean(true))
        }))),
    );

    methods.insert(
        "ServeTls".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() < 4 || args.len() > 5 {
                return Err(
                    "Web.ServeTls requires 4-5 arguments (addr, handler_path, cert_path, key_path, optional static_dir)"
                        .to_string(),
                );
            }

            let addr = args[0].to_display_string();
            let handler_path = args[1].to_display_string();
            let cert_path = args[2].to_display_string();
            let key_path = args[3].to_display_string();
            let static_dir = if args.len() == 5 {
                Some(args[4].to_display_string())
            } else {
                None
            };

            serve_tls(
                &addr,
                &handler_path,
                &cert_path,
                &key_path,
                static_dir.as_deref(),
            )?;
            Ok(Value::Boolean(true))
        }))),
    );

    methods.insert(
        "Router".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if !args.is_empty() {
                return Err("Web.Router takes no arguments".to_string());
            }
            Ok(create_router_object())
        }))),
    );

    methods.insert(
        "Stream".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.is_empty() || args.len() > 3 {
                return Err(
                    "Web.Stream requires 1-3 arguments (stream, optional status, optional headers)"
                        .to_string(),
                );
            }

            let stream = args[0].clone();
            if !is_stream_value(&stream) {
                return Err("Web.Stream requires a Stream value".to_string());
            }

            let mut status = 200u16;
            let mut headers: Option<Value> = None;

            if args.len() >= 2 {
                match &args[1] {
                    Value::Map(_) => headers = Some(args[1].clone()),
                    _ => {
                        status = value_to_status(&args[1]).unwrap_or(200);
                    }
                }
            }

            if args.len() == 3 {
                headers = Some(args[2].clone());
            }

            Ok(build_stream_response_map(stream, status, headers))
        }))),
    );

    methods.insert(
        "Response".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.is_empty() || args.len() > 3 {
                return Err(
                    "Web.Response requires 1-3 arguments (body, optional status, optional headers)"
                        .to_string(),
                );
            }

            let body = args[0].clone();
            let mut status = 200u16;
            let mut headers: Option<Value> = None;

            if args.len() >= 2 {
                match &args[1] {
                    Value::Map(_) => headers = Some(args[1].clone()),
                    _ => {
                        status = value_to_status(&args[1]).unwrap_or(200);
                    }
                }
            }

            if args.len() == 3 {
                headers = Some(args[2].clone());
            }

            Ok(build_response_map(body, status, headers))
        }))),
    );

    methods.insert(
        "Json".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.is_empty() || args.len() > 3 {
                return Err(
                    "Web.Json requires 1-3 arguments (value, optional status, optional headers)"
                        .to_string(),
                );
            }

            let mut status = 200u16;
            let mut headers: Option<Value> = None;

            if args.len() >= 2 {
                match &args[1] {
                    Value::Map(_) => headers = Some(args[1].clone()),
                    _ => {
                        status = value_to_status(&args[1]).unwrap_or(200);
                    }
                }
            }

            if args.len() == 3 {
                headers = Some(args[2].clone());
            }

            let json_value = value_to_json(&args[0]);
            let json_body = serde_json::to_string(&json_value).unwrap_or_else(|_| "{}".to_string());

            Ok(build_response_map(
                Value::String(json_body),
                status,
                Some(merge_headers(
                    headers,
                    "Content-Type",
                    "application/json; charset=utf-8",
                )),
            ))
        }))),
    );

    methods.insert(
        "Redirect".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.is_empty() || args.len() > 2 {
                return Err(
                    "Web.Redirect requires 1-2 arguments (url, optional status)".to_string()
                );
            }

            let url = args[0].to_display_string();
            let status = if args.len() == 2 {
                value_to_status(&args[1]).unwrap_or(302)
            } else {
                302
            };

            let mut headers = HashMap::new();
            headers.insert("Location".to_string(), Value::String(url));
            Ok(build_response_map(
                Value::String(String::new()),
                status,
                Some(Value::Map(Arc::new(RwLock::new(headers)))),
            ))
        }))),
    );

    methods.insert(
        "File".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.is_empty() || args.len() > 2 {
                return Err(
                    "Web.File requires 1-2 arguments (path, optional content_type)".to_string(),
                );
            }

            let path = args[0].to_display_string();
            let mut response = HashMap::new();
            response.insert("FilePath".to_string(), Value::String(path));
            if args.len() == 2 {
                response.insert(
                    "ContentType".to_string(),
                    Value::String(args[1].to_display_string()),
                );
            }

            Ok(Value::Map(Arc::new(RwLock::new(response))))
        }))),
    );

    Value::Map(Arc::new(RwLock::new(methods)))
}

pub fn serve(addr: &str, handler_path: &str, static_dir: Option<&str>) -> Result<(), String> {
    let mut state = RouterState::new();
    state.fallback = Some(Arc::new(ScriptHandler::new(handler_path)));

    if let Some(dir) = static_dir {
        state.static_mounts.push(StaticMount::new("/", dir));
    }

    start_server(addr, Arc::new(Mutex::new(state)), None)
}

pub fn serve_tls(
    addr: &str,
    handler_path: &str,
    cert_path: &str,
    key_path: &str,
    static_dir: Option<&str>,
) -> Result<(), String> {
    let mut state = RouterState::new();
    state.fallback = Some(Arc::new(ScriptHandler::new(handler_path)));

    if let Some(dir) = static_dir {
        state.static_mounts.push(StaticMount::new("/", dir));
    }

    start_server(
        addr,
        Arc::new(Mutex::new(state)),
        Some(TlsPaths {
            cert_path: cert_path.to_string(),
            key_path: key_path.to_string(),
        }),
    )
}

fn create_router_object() -> Value {
    let state = Arc::new(Mutex::new(RouterState::new()));
    let mut methods = HashMap::new();

    methods.insert(
        "Get".to_string(),
        route_register(Some("GET"), state.clone()),
    );
    methods.insert(
        "Post".to_string(),
        route_register(Some("POST"), state.clone()),
    );
    methods.insert(
        "Put".to_string(),
        route_register(Some("PUT"), state.clone()),
    );
    methods.insert(
        "Patch".to_string(),
        route_register(Some("PATCH"), state.clone()),
    );
    methods.insert(
        "Delete".to_string(),
        route_register(Some("DELETE"), state.clone()),
    );
    methods.insert("Any".to_string(), route_register(None, state.clone()));

    let state_use = state.clone();
    methods.insert(
        "Use".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Router.Use requires 1 argument (middleware_path)".to_string());
            }

            let handler = ScriptHandler::new(&args[0].to_display_string());
            let mut state = state_use.lock().expect("lock poisoned");
            state.middleware.push(Arc::new(handler));
            Ok(Value::Boolean(true))
        }))),
    );

    let state_static = state.clone();
    methods.insert(
        "Static".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 2 {
                return Err("Router.Static requires 2 arguments (mount_path, dir)".to_string());
            }

            let mount_path = args[0].to_display_string();
            let dir = args[1].to_display_string();
            let mut state = state_static.lock().expect("lock poisoned");
            state
                .static_mounts
                .push(StaticMount::new(&mount_path, &dir));
            Ok(Value::Boolean(true))
        }))),
    );

    let state_nf = state.clone();
    methods.insert(
        "NotFound".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Router.NotFound requires 1 argument (handler_path)".to_string());
            }

            let handler = ScriptHandler::new(&args[0].to_display_string());
            let mut state = state_nf.lock().expect("lock poisoned");
            state.not_found = Some(Arc::new(handler));
            Ok(Value::Boolean(true))
        }))),
    );

    let state_serve = state.clone();
    methods.insert(
        "Serve".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() > 1 {
                return Err("Router.Serve requires 0-1 arguments (addr)".to_string());
            }

            let addr = if args.is_empty() {
                DEFAULT_ADDR.to_string()
            } else {
                args[0].to_display_string()
            };

            start_server(&addr, state_serve.clone(), None)?;
            Ok(Value::Boolean(true))
        }))),
    );

    let state_serve_tls = state.clone();
    methods.insert(
        "ServeTls".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() < 3 || args.len() > 4 {
                return Err(
                    "Router.ServeTls requires 3-4 arguments (addr, cert_path, key_path, optional static_dir)"
                        .to_string(),
                );
            }

            let addr = args[0].to_display_string();
            let cert_path = args[1].to_display_string();
            let key_path = args[2].to_display_string();
            let static_dir = if args.len() == 4 {
                Some(args[3].to_display_string())
            } else {
                None
            };

            if let Some(dir) = static_dir {
                let mut state = state_serve_tls.lock().expect("lock poisoned");
                state
                    .static_mounts
                    .push(StaticMount::new("/", &dir));
            }

            start_server(
                &addr,
                state_serve_tls.clone(),
                Some(TlsPaths {
                    cert_path,
                    key_path,
                }),
            )?;
            Ok(Value::Boolean(true))
        }))),
    );

    Value::Map(Arc::new(RwLock::new(methods)))
}

fn route_register(method: Option<&'static str>, state: Arc<Mutex<RouterState>>) -> Value {
    let method_string = method.map(|m| m.to_string());
    Value::NativeFunction(Arc::new(Box::new(move |args| {
        if args.len() != 2 {
            return Err(
                "Router route registration requires 2 arguments (path, handler_path)".to_string(),
            );
        }

        let path = args[0].to_display_string();
        let handler_path = args[1].to_display_string();
        let handler = Arc::new(ScriptHandler::new(&handler_path));
        let route = Route::new(method_string.clone(), &path, handler);

        let mut state = state.lock().expect("lock poisoned");
        state.routes.push(route);
        Ok(Value::Boolean(true))
    })))
}

#[derive(Clone)]
struct Route {
    method: Option<String>,
    pattern: RoutePattern,
    handler: Arc<ScriptHandler>,
}

impl Route {
    fn new(method: Option<String>, pattern: &str, handler: Arc<ScriptHandler>) -> Self {
        Self {
            method,
            pattern: RoutePattern::new(pattern),
            handler,
        }
    }
}

#[derive(Clone)]
struct StaticMount {
    mount_path: String,
    dir: PathBuf,
}

impl StaticMount {
    fn new(mount_path: &str, dir: &str) -> Self {
        Self {
            mount_path: normalize_path(mount_path),
            dir: resolve_path(dir),
        }
    }
}

struct RouterState {
    routes: Vec<Route>,
    middleware: Vec<Arc<ScriptHandler>>,
    static_mounts: Vec<StaticMount>,
    not_found: Option<Arc<ScriptHandler>>,
    fallback: Option<Arc<ScriptHandler>>,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl RouterState {
    fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create web runtime");
        Self {
            routes: Vec::new(),
            middleware: Vec::new(),
            static_mounts: Vec::new(),
            not_found: None,
            fallback: None,
            runtime: Arc::new(runtime),
        }
    }
}

struct ScriptHandler {
    path: PathBuf,
    state: Mutex<ScriptState>,
}

struct ScriptState {
    modified: Option<SystemTime>,
    program: Option<Program>,
    last_error: Option<String>,
}

impl ScriptHandler {
    fn new(path: &str) -> Self {
        Self {
            path: resolve_path(path),
            state: Mutex::new(ScriptState {
                modified: None,
                program: None,
                last_error: None,
            }),
        }
    }

    fn ensure_current(&self) -> Result<Program, String> {
        let mut state = self.state.lock().expect("lock poisoned");
        let metadata = fs::metadata(&self.path)
            .map_err(|e| format!("Failed to read handler '{}': {}", self.path.display(), e))?;
        let modified = metadata.modified().ok();
        let needs_reload = match (modified, state.modified) {
            (Some(new_time), Some(old_time)) => new_time > old_time,
            (Some(_), None) => true,
            (None, _) => state.program.is_none(),
        };

        if needs_reload || state.program.is_none() {
            match load_program(&self.path) {
                Ok(program) => {
                    state.program = Some(program);
                    state.modified = modified;
                    state.last_error = None;
                }
                Err(err) => {
                    state.last_error = Some(err);
                }
            }
        }

        if let Some(err) = &state.last_error {
            return Err(err.clone());
        }

        state
            .program
            .clone()
            .ok_or_else(|| "Handler script not loaded".to_string())
    }
}

#[derive(Clone)]
struct RoutePattern {
    segments: Vec<RouteSegment>,
}

#[derive(Clone)]
enum RouteSegment {
    Literal(String),
    Param(String),
    Wildcard(String),
}

impl RoutePattern {
    fn new(pattern: &str) -> Self {
        let normalized = normalize_path(pattern);
        if normalized == "*" || normalized == "/*" {
            return Self {
                segments: vec![RouteSegment::Wildcard("wildcard".to_string())],
            };
        }

        let segments = split_path(&normalized)
            .into_iter()
            .map(|segment| {
                if segment == "*" {
                    RouteSegment::Wildcard("wildcard".to_string())
                } else if let Some(name) = segment.strip_prefix(':') {
                    RouteSegment::Param(name.to_string())
                } else if segment.starts_with('{') && segment.ends_with('}') && segment.len() > 2 {
                    RouteSegment::Param(segment[1..segment.len() - 1].to_string())
                } else {
                    RouteSegment::Literal(segment)
                }
            })
            .collect();

        Self { segments }
    }

    fn matches(&self, path: &str) -> Option<HashMap<String, String>> {
        let normalized = normalize_path(path);
        let segments = split_path(&normalized);
        let mut params = HashMap::new();

        let mut index = 0usize;
        for segment in &self.segments {
            match segment {
                RouteSegment::Wildcard(name) => {
                    let rest = segments[index..].join("/");
                    if !name.is_empty() {
                        params.insert(name.clone(), rest);
                    }
                    return Some(params);
                }
                RouteSegment::Param(name) => {
                    if index >= segments.len() {
                        return None;
                    }
                    params.insert(name.clone(), segments[index].clone());
                    index += 1;
                }
                RouteSegment::Literal(lit) => {
                    if index >= segments.len() || segments[index] != *lit {
                        return None;
                    }
                    index += 1;
                }
            }
        }

        if index == segments.len() {
            Some(params)
        } else {
            None
        }
    }
}

struct RequestContext {
    method: String,
    path: String,
    raw_path: String,
    version: String,
    headers: HashMap<String, String>,
    headers_raw: HashMap<String, String>,
    body: Vec<u8>,
    remote_addr: String,
    query: HashMap<String, String>,
    cookies: HashMap<String, String>,
}

struct ResponseData {
    status: u16,
    headers: HashMap<String, String>,
    body: ResponseBody,
}

impl ResponseData {
    fn new(status: u16, body: Vec<u8>) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: ResponseBody::Bytes(body),
        }
    }
}

enum ResponseBody {
    Bytes(Vec<u8>),
    Stream(Value),
}

struct TlsPaths {
    cert_path: String,
    key_path: String,
}

struct PlainStreamWithAddr {
    addr: SocketAddr,
    stream: tokio::net::TcpStream,
}

impl PlainStreamWithAddr {
    fn remote_addr(&self) -> SocketAddr {
        self.addr
    }
}

impl AsyncRead for PlainStreamWithAddr {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for PlainStreamWithAddr {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().stream).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}

struct TlsStreamWithAddr {
    addr: SocketAddr,
    stream: tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
}

impl TlsStreamWithAddr {
    fn remote_addr(&self) -> SocketAddr {
        self.addr
    }
}

impl AsyncRead for TlsStreamWithAddr {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for TlsStreamWithAddr {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().stream).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}

fn start_server(
    addr: &str,
    state: Arc<Mutex<RouterState>>,
    tls: Option<TlsPaths>,
) -> Result<(), String> {
    let addr = addr.to_string();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to start async runtime: {}", e))?;

    runtime.block_on(async move {
        if let Some(tls_paths) = tls {
            let config = load_tls_config(&tls_paths)?;
            run_server_tls(&addr, state, config).await
        } else {
            run_server_plain(&addr, state).await
        }
    })
}

async fn run_server_plain(addr: &str, state: Arc<Mutex<RouterState>>) -> Result<(), String> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind {}: {}", addr, e))?;
    println!("SFX web server listening on http://{}", addr);

    let incoming = TcpListenerStream::new(listener).map(|conn| {
        conn.map(|stream| {
            let addr = stream
                .peer_addr()
                .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
            PlainStreamWithAddr { addr, stream }
        })
    });

    let make_svc = make_service_fn(move |conn: &PlainStreamWithAddr| {
        let state = state.clone();
        let remote = conn.remote_addr().to_string();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handle_http_request(req, state.clone(), remote.clone())
            }))
        }
    });

    Server::builder(hyper::server::accept::from_stream(incoming))
        .serve(make_svc)
        .await
        .map_err(|e| format!("Server error: {}", e))
}

async fn run_server_tls(
    addr: &str,
    state: Arc<Mutex<RouterState>>,
    tls_config: Arc<ServerConfig>,
) -> Result<(), String> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind {}: {}", addr, e))?;
    println!("SFX web server listening on https://{}", addr);

    let acceptor = TlsAcceptor::from(tls_config);
    let incoming = TcpListenerStream::new(listener).then(move |conn| {
        let acceptor = acceptor.clone();
        async move {
            let stream = conn?;
            let addr = stream
                .peer_addr()
                .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
            let tls_stream = acceptor
                .accept(stream)
                .await
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            Ok::<_, io::Error>(TlsStreamWithAddr {
                addr,
                stream: tls_stream,
            })
        }
    });

    let make_svc = make_service_fn(move |conn: &TlsStreamWithAddr| {
        let state = state.clone();
        let remote = conn.remote_addr().to_string();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handle_http_request(req, state.clone(), remote.clone())
            }))
        }
    });

    Server::builder(hyper::server::accept::from_stream(incoming))
        .http2_only(false)
        .serve(make_svc)
        .await
        .map_err(|e| format!("Server error: {}", e))
}

async fn handle_http_request(
    req: Request<Body>,
    state: Arc<Mutex<RouterState>>,
    remote_addr: String,
) -> Result<Response<Body>, hyper::Error> {
    let request = match build_request_context(req, remote_addr).await {
        Ok(request) => request,
        Err(err) => {
            let response = ResponseData::new(400, format!("Bad Request: {}", err).into_bytes());
            return Ok(build_hyper_response(response));
        }
    };

    let is_head = request.method == "HEAD";
    let mut response = handle_request(&request, state);
    if is_head {
        response.body = ResponseBody::Bytes(Vec::new());
    }
    Ok(build_hyper_response(response))
}

async fn build_request_context(
    req: Request<Body>,
    remote_addr: String,
) -> Result<RequestContext, String> {
    let (parts, body) = req.into_parts();
    let method = parts.method.as_str().to_uppercase();
    let raw_path = parts
        .uri
        .path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| "/".to_string());
    let path = normalize_path(parts.uri.path());
    let query = parts.uri.query().unwrap_or("");

    let mut headers = HashMap::new();
    let mut headers_raw = HashMap::new();
    for (name, value) in parts.headers.iter() {
        let key = name.as_str().to_string();
        let value_str = value.to_str().unwrap_or("").to_string();
        headers_raw.insert(key.clone(), value_str.clone());
        headers.insert(key.to_lowercase(), value_str);
    }

    let body_bytes = to_bytes(body)
        .await
        .map_err(|e| format!("Failed to read body: {}", e))?;
    if body_bytes.len() > MAX_BODY_SIZE {
        return Err("Body too large".to_string());
    }

    let query_map = parse_query(query);
    let cookies = headers
        .get("cookie")
        .map(|value| parse_cookies(value))
        .unwrap_or_default();

    Ok(RequestContext {
        method,
        path,
        raw_path,
        version: format!("{:?}", parts.version),
        headers,
        headers_raw,
        body: body_bytes.to_vec(),
        remote_addr,
        query: query_map,
        cookies,
    })
}

fn build_hyper_response(response: ResponseData) -> Response<Body> {
    let mut builder = Response::builder().status(response.status);
    let headers = normalize_response_headers(&response);
    for (key, value) in headers.iter() {
        builder = builder.header(key, value);
    }

    match response.body {
        ResponseBody::Bytes(body) => builder
            .body(Body::from(body))
            .unwrap_or_else(|_| Response::new(Body::from("Response build error"))),
        ResponseBody::Stream(stream_value) => {
            let body = build_stream_body(stream_value);
            builder
                .body(body)
                .unwrap_or_else(|_| Response::new(Body::from("Response build error")))
        }
    }
}

fn normalize_response_headers(response: &ResponseData) -> HashMap<String, String> {
    let mut headers = response.headers.clone();

    if !header_exists(&headers, "Content-Type") {
        headers.insert(
            "Content-Type".to_string(),
            "text/plain; charset=utf-8".to_string(),
        );
    }

    if matches!(response.body, ResponseBody::Bytes(_)) {
        if !header_exists(&headers, "Content-Length") {
            if let ResponseBody::Bytes(body) = &response.body {
                headers.insert("Content-Length".to_string(), body.len().to_string());
            }
        }
    } else {
        headers.retain(|k, _| k.to_lowercase() != "content-length");
    }

    headers
}

fn build_stream_body(stream_value: Value) -> Body {
    let (sender, receiver) = tokio::sync::mpsc::channel::<Result<Bytes, io::Error>>(8);
    tokio::task::spawn_blocking(move || {
        let _ = send_stream_chunks(stream_value, sender);
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(receiver);
    Body::wrap_stream(stream)
}

fn handle_request(request: &RequestContext, state: Arc<Mutex<RouterState>>) -> ResponseData {
    let (routes, middleware, static_mounts, not_found, fallback, runtime) = {
        let state = state.lock().expect("lock poisoned");
        (
            state.routes.clone(),
            state.middleware.clone(),
            state.static_mounts.clone(),
            state.not_found.clone(),
            state.fallback.clone(),
            state.runtime.clone(),
        )
    };

    if let Some(response) = try_static(&request.path, &static_mounts) {
        return response;
    }

    if let Some((handler, params)) = find_route(&routes, &request.method, &request.path) {
        if let Ok(Some(response)) = run_middleware(&middleware, request, &params, &runtime) {
            return response;
        }

        return match execute_script(&handler, request, &params, &runtime) {
            Ok(Some(response)) => response,
            Ok(None) => ResponseData::new(204, Vec::new()),
            Err(err) => ResponseData::new(500, err.into_bytes()),
        };
    }

    if let Some(handler) = fallback {
        let empty_params = HashMap::new();
        if let Ok(Some(response)) = run_middleware(&middleware, request, &empty_params, &runtime) {
            return response;
        }

        return match execute_script(&handler, request, &empty_params, &runtime) {
            Ok(Some(response)) => response,
            Ok(None) => ResponseData::new(204, Vec::new()),
            Err(err) => ResponseData::new(500, err.into_bytes()),
        };
    }

    if let Some(handler) = not_found {
        let empty_params = HashMap::new();
        if let Ok(Some(response)) = run_middleware(&middleware, request, &empty_params, &runtime) {
            return response;
        }

        return match execute_script(&handler, request, &empty_params, &runtime) {
            Ok(Some(response)) => response,
            Ok(None) => ResponseData::new(404, b"Not Found".to_vec()),
            Err(err) => ResponseData::new(500, err.into_bytes()),
        };
    }

    ResponseData::new(404, b"Not Found".to_vec())
}

fn run_middleware(
    middleware: &[Arc<ScriptHandler>],
    request: &RequestContext,
    params: &HashMap<String, String>,
    runtime: &Arc<tokio::runtime::Runtime>,
) -> Result<Option<ResponseData>, String> {
    for handler in middleware {
        if let Some(response) = execute_script(handler, request, params, runtime)? {
            return Ok(Some(response));
        }
    }
    Ok(None)
}

fn find_route(
    routes: &[Route],
    method: &str,
    path: &str,
) -> Option<(Arc<ScriptHandler>, HashMap<String, String>)> {
    let method = method.to_uppercase();
    let method_lookup = if method == "HEAD" {
        "GET"
    } else {
        method.as_str()
    };

    for route in routes {
        if let Some(route_method) = &route.method {
            if route_method != method_lookup {
                continue;
            }
        }

        if let Some(params) = route.pattern.matches(path) {
            return Some((route.handler.clone(), params));
        }
    }

    None
}

fn execute_script(
    handler: &Arc<ScriptHandler>,
    request: &RequestContext,
    params: &HashMap<String, String>,
    runtime: &Arc<tokio::runtime::Runtime>,
) -> Result<Option<ResponseData>, String> {
    let program = handler.ensure_current()?;
    let mut interpreter = Interpreter::new_with_shared_runtime(runtime.clone());

    interpreter.define_global("Request", build_request_value(request, params));
    interpreter.define_global("Params", build_params_value(params));
    interpreter.define_global("Response", Value::Boolean(false));

    interpreter
        .run(program)
        .map_err(|e| format!("Runtime error: {}", e))?;

    if let Some(response) = interpreter.get_global("Response") {
        if matches!(response, Value::Boolean(false)) {
            return Ok(None);
        }
        return Ok(Some(response_from_value(&response)?));
    }

    Ok(None)
}

fn build_request_value(request: &RequestContext, params: &HashMap<String, String>) -> Value {
    let mut request_map = HashMap::new();

    request_map.insert("Method".to_string(), Value::String(request.method.clone()));
    request_map.insert("Path".to_string(), Value::String(request.path.clone()));
    request_map.insert(
        "RawPath".to_string(),
        Value::String(request.raw_path.clone()),
    );
    request_map.insert(
        "Version".to_string(),
        Value::String(request.version.clone()),
    );
    request_map.insert(
        "RemoteAddr".to_string(),
        Value::String(request.remote_addr.clone()),
    );

    let headers_value = build_headers_value(&request.headers_raw, &request.headers);
    request_map.insert("Headers".to_string(), headers_value);

    let mut query_map = HashMap::new();
    for (key, value) in &request.query {
        query_map.insert(key.clone(), Value::String(value.clone()));
    }
    request_map.insert(
        "Query".to_string(),
        Value::Map(Arc::new(RwLock::new(query_map))),
    );

    let mut cookies_map = HashMap::new();
    for (key, value) in &request.cookies {
        cookies_map.insert(key.clone(), Value::String(value.clone()));
    }
    request_map.insert(
        "Cookies".to_string(),
        Value::Map(Arc::new(RwLock::new(cookies_map))),
    );

    let body = String::from_utf8_lossy(&request.body).to_string();
    request_map.insert("Body".to_string(), Value::String(body));

    request_map.insert("Params".to_string(), build_params_value(params));

    if let Some(length) = request.headers.get("content-length") {
        if let Ok(length_num) = length.parse::<i64>() {
            request_map.insert(
                "ContentLength".to_string(),
                Value::from_number_string(&length_num.to_string())
                    .unwrap_or(Value::default_number()),
            );
        }
    }

    Value::Map(Arc::new(RwLock::new(request_map)))
}

fn build_headers_value(
    headers_raw: &HashMap<String, String>,
    headers_lower: &HashMap<String, String>,
) -> Value {
    let mut headers_map = HashMap::new();
    for (key, value) in headers_raw {
        headers_map.insert(key.clone(), Value::String(value.clone()));
    }
    for (key, value) in headers_lower {
        headers_map
            .entry(key.clone())
            .or_insert_with(|| Value::String(value.clone()));
    }

    Value::Map(Arc::new(RwLock::new(headers_map)))
}

fn build_params_value(params: &HashMap<String, String>) -> Value {
    let mut map = HashMap::new();
    for (key, value) in params {
        map.insert(key.clone(), Value::String(value.clone()));
    }
    Value::Map(Arc::new(RwLock::new(map)))
}

fn response_from_value(value: &Value) -> Result<ResponseData, String> {
    if is_stream_value(value) {
        return Ok(ResponseData {
            status: 200,
            headers: HashMap::new(),
            body: ResponseBody::Stream(value.clone()),
        });
    }

    match value {
        Value::Map(map) => response_from_map(map),
        Value::List(_) | Value::Vector(_) => {
            let json_value = value_to_json(value);
            let json_body = serde_json::to_string(&json_value).unwrap_or_else(|_| "[]".to_string());
            let mut response = ResponseData::new(200, json_body.into_bytes());
            response.headers.insert(
                "Content-Type".to_string(),
                "application/json; charset=utf-8".to_string(),
            );
            Ok(response)
        }
        Value::String(s) => Ok(ResponseData::new(200, s.as_bytes().to_vec())),
        Value::Boolean(b) => Ok(ResponseData::new(200, b.to_string().into_bytes())),
        Value::Number(n) => Ok(ResponseData::new(200, n.to_string().into_bytes())),
        Value::FastNumber(f) => Ok(ResponseData::new(200, f.to_string().into_bytes())),
        Value::Error(err) => Ok(ResponseData::new(
            500,
            format!("Error.{}.{}: {}", err.category, err.subtype, err.message).into_bytes(),
        )),
        _ => Ok(ResponseData::new(
            200,
            value.to_display_string().into_bytes(),
        )),
    }
}

fn response_from_map(map: &Arc<RwLock<HashMap<String, Value>>>) -> Result<ResponseData, String> {
    let map = map.read().expect("lock poisoned");
    if is_stream_map(&map) {
        return Ok(ResponseData {
            status: 200,
            headers: HashMap::new(),
            body: ResponseBody::Stream(Value::Map(Arc::new(RwLock::new(map.clone())))),
        });
    }
    let is_response_map = map.contains_key("Status")
        || map.contains_key("StatusCode")
        || map.contains_key("Headers")
        || map.contains_key("Body")
        || map.contains_key("FilePath")
        || map.contains_key("File")
        || map.contains_key("Stream");

    if !is_response_map {
        let json_value = value_to_json(&Value::Map(Arc::new(RwLock::new(map.clone()))));
        let json_body = serde_json::to_string(&json_value).unwrap_or_else(|_| "{}".to_string());
        let mut response = ResponseData::new(200, json_body.into_bytes());
        response.headers.insert(
            "Content-Type".to_string(),
            "application/json; charset=utf-8".to_string(),
        );
        return Ok(response);
    }

    let status = map
        .get("Status")
        .or_else(|| map.get("StatusCode"))
        .and_then(value_to_status)
        .unwrap_or(200);

    let mut headers = HashMap::new();
    if let Some(Value::Map(header_map)) = map.get("Headers") {
        let header_map = header_map.read().expect("lock poisoned");
        for (key, value) in header_map.iter() {
            headers.insert(key.clone(), value.to_display_string());
        }
    }

    if let Some(content_type) = map.get("ContentType") {
        headers.insert("Content-Type".to_string(), content_type.to_display_string());
    }

    if let Some(content_type) = map.get("Content-Type") {
        headers.insert("Content-Type".to_string(), content_type.to_display_string());
    }

    if let Some(file_path) = map.get("FilePath").or_else(|| map.get("File")) {
        let file_path = file_path.to_display_string();
        let bytes = fs::read(&file_path)
            .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;
        if !header_exists(&headers, "Content-Type") {
            headers.insert(
                "Content-Type".to_string(),
                guess_mime_type(Path::new(&file_path)).to_string(),
            );
        }
        let mut response = ResponseData::new(status, bytes);
        response.headers = headers;
        return Ok(response);
    }

    if let Some(stream_value) = map.get("Stream") {
        if is_stream_value(stream_value) {
            return Ok(ResponseData {
                status,
                headers,
                body: ResponseBody::Stream(stream_value.clone()),
            });
        }
    }

    let body_value = map
        .get("Body")
        .cloned()
        .unwrap_or(Value::String(String::new()));
    if is_stream_value(&body_value) {
        return Ok(ResponseData {
            status,
            headers,
            body: ResponseBody::Stream(body_value),
        });
    }

    let mut response = match body_value {
        Value::List(_) | Value::Vector(_) | Value::Map(_) => {
            let json_value = value_to_json(&body_value);
            let json_body = serde_json::to_string(&json_value).unwrap_or_else(|_| "{}".to_string());
            let response = ResponseData::new(status, json_body.into_bytes());
            if !header_exists(&headers, "Content-Type") {
                headers.insert(
                    "Content-Type".to_string(),
                    "application/json; charset=utf-8".to_string(),
                );
            }
            response
        }
        Value::String(s) => ResponseData::new(status, s.into_bytes()),
        Value::Boolean(b) => ResponseData::new(status, b.to_string().into_bytes()),
        Value::Number(n) => ResponseData::new(status, n.to_string().into_bytes()),
        Value::FastNumber(f) => ResponseData::new(status, f.to_string().into_bytes()),
        other => ResponseData::new(status, other.to_display_string().into_bytes()),
    };

    response.headers = headers;
    Ok(response)
}

fn is_stream_value(value: &Value) -> bool {
    match value {
        Value::Map(map) => {
            let map = map.read().expect("lock poisoned");
            map.get("Next").map(is_native_fn).unwrap_or(false)
        }
        _ => false,
    }
}

fn is_stream_map(map: &HashMap<String, Value>) -> bool {
    map.get("Next").map(is_native_fn).unwrap_or(false)
}

fn is_native_fn(value: &Value) -> bool {
    matches!(value, Value::NativeFunction(_))
}

fn send_stream_chunks(
    stream_value: Value,
    sender: tokio::sync::mpsc::Sender<Result<Bytes, io::Error>>,
) -> Result<(), String> {
    loop {
        let next = stream_next_value(&stream_value)?;
        let Some(value) = next else {
            break;
        };

        let bytes = chunk_bytes_from_value(value);
        if sender.blocking_send(Ok(Bytes::from(bytes))).is_err() {
            break;
        }
    }
    Ok(())
}

fn stream_next_value(stream_value: &Value) -> Result<Option<Value>, String> {
    let Value::Map(map) = stream_value else {
        return Err("Stream value must be a Map".to_string());
    };
    let next_fn = {
        let map = map.read().expect("lock poisoned");
        map.get("Next").cloned()
    }
    .ok_or_else(|| "Stream is missing Next".to_string())?;

    match next_fn {
        Value::NativeFunction(func) => match func(vec![])? {
            Value::Option(opt) => Ok(opt.as_ref().clone()),
            value => Ok(Some(value)),
        },
        _ => Err("Stream Next must be a function".to_string()),
    }
}

fn chunk_bytes_from_value(value: Value) -> Vec<u8> {
    match value {
        Value::List(_) | Value::Vector(_) | Value::Map(_) => {
            serde_json::to_string(&value_to_json(&value))
                .unwrap_or_else(|_| value.to_display_string())
                .into_bytes()
        }
        Value::Error(err) => {
            format!("Error.{}.{}: {}", err.category, err.subtype, err.message).into_bytes()
        }
        other => other.to_display_string().into_bytes(),
    }
}

fn parse_query(query: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = parts.next().unwrap_or("");
        map.insert(url_decode(key), url_decode(value));
    }
    map
}

fn parse_cookies(cookie_header: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for part in cookie_header.split(';') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.splitn(2, '=');
        let key = parts.next().unwrap_or("").trim();
        let value = parts.next().unwrap_or("").trim();
        if !key.is_empty() {
            map.insert(key.to_string(), value.to_string());
        }
    }
    map
}

fn url_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                output.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                if let (Some(a), Some(b)) = (hex_value(bytes[i + 1]), hex_value(bytes[i + 2])) {
                    output.push(a * 16 + b);
                    i += 3;
                } else {
                    output.push(bytes[i]);
                    i += 1;
                }
            }
            byte => {
                output.push(byte);
                i += 1;
            }
        }
    }

    String::from_utf8_lossy(&output).to_string()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn try_static(path: &str, mounts: &[StaticMount]) -> Option<ResponseData> {
    for mount in mounts {
        if let Some(relative_path) = strip_mount(path, &mount.mount_path) {
            if let Some(file_path) = resolve_static_path(&mount.dir, &relative_path) {
                if let Ok(bytes) = fs::read(&file_path) {
                    let mut response = ResponseData::new(200, bytes);
                    response.headers.insert(
                        "Content-Type".to_string(),
                        guess_mime_type(&file_path).to_string(),
                    );
                    return Some(response);
                }
            }
        }
    }
    None
}

fn strip_mount(path: &str, mount_path: &str) -> Option<String> {
    if mount_path == "/" {
        return Some(path.trim_start_matches('/').to_string());
    }
    if path == mount_path {
        return Some(String::new());
    }
    let prefix = format!("{}/", mount_path.trim_end_matches('/'));
    if path.starts_with(&prefix) {
        return Some(path[prefix.len()..].to_string());
    }
    None
}

fn resolve_static_path(base: &Path, relative: &str) -> Option<PathBuf> {
    let mut resolved = PathBuf::from(base);
    for component in Path::new(relative).components() {
        match component {
            Component::Normal(part) => resolved.push(part),
            Component::CurDir => {}
            Component::ParentDir => return None,
            _ => return None,
        }
    }

    if resolved.is_dir() {
        resolved.push("index.html");
    }

    if resolved.is_file() {
        Some(resolved)
    } else {
        None
    }
}

fn load_tls_config(paths: &TlsPaths) -> Result<Arc<ServerConfig>, String> {
    let cert_file = fs::read(&paths.cert_path)
        .map_err(|e| format!("Failed to read cert {}: {}", paths.cert_path, e))?;
    let key_file = fs::read(&paths.key_path)
        .map_err(|e| format!("Failed to read key {}: {}", paths.key_path, e))?;

    let mut cert_reader = std::io::Cursor::new(cert_file);
    let mut key_reader = std::io::Cursor::new(key_file);

    let certs = certs(&mut cert_reader)
        .map_err(|_| "Failed to parse certificate".to_string())?
        .into_iter()
        .map(Certificate)
        .collect::<Vec<_>>();
    if certs.is_empty() {
        return Err("No certificates found".to_string());
    }

    let mut keys = pkcs8_private_keys(&mut key_reader)
        .map_err(|_| "Failed to parse private key".to_string())?
        .into_iter()
        .map(PrivateKey)
        .collect::<Vec<_>>();

    if keys.is_empty() {
        let mut key_reader = std::io::Cursor::new(
            fs::read(&paths.key_path)
                .map_err(|e| format!("Failed to read key {}: {}", paths.key_path, e))?,
        );
        keys = rsa_private_keys(&mut key_reader)
            .map_err(|_| "Failed to parse RSA key".to_string())?
            .into_iter()
            .map(PrivateKey)
            .collect::<Vec<_>>();
    }

    let key = keys
        .into_iter()
        .next()
        .ok_or_else(|| "No private keys found".to_string())?;

    let mut config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| format!("TLS config error: {}", e))?;

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Ok(Arc::new(config))
}

fn load_program(path: &Path) -> Result<Program, String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read handler '{}': {}", path.display(), e))?;
    let mut lexer = Lexer::new(&source);
    let tokens = lexer
        .tokenize()
        .map_err(|e| format!("Lexer error in '{}': {}", path.display(), e))?;
    let mut parser = Parser::new(tokens);
    parser
        .parse()
        .map_err(|e| format!("Parser error in '{}': {}", path.display(), e))
}

fn resolve_path(path: &str) -> PathBuf {
    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        candidate
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(candidate)
    }
}

fn normalize_path(path: &str) -> String {
    if path.is_empty() {
        return "/".to_string();
    }
    let mut normalized = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };

    while normalized.len() > 1 && normalized.ends_with('/') {
        normalized.pop();
    }

    normalized
}

fn split_path(path: &str) -> Vec<String> {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        Vec::new()
    } else {
        trimmed.split('/').map(url_decode).collect()
    }
}

fn value_to_status(value: &Value) -> Option<u16> {
    match value {
        Value::Number(n) => n.to_u16(),
        Value::FastNumber(f) => {
            if f.is_finite() && *f >= 0.0 {
                Some(*f as u16)
            } else {
                None
            }
        }
        Value::String(s) => s.parse::<u16>().ok(),
        _ => None,
    }
}

fn build_response_map(body: Value, status: u16, headers: Option<Value>) -> Value {
    let mut map = HashMap::new();
    map.insert(
        "Status".to_string(),
        Value::from_number_string(&status.to_string()).unwrap_or(Value::default_number()),
    );
    map.insert("Body".to_string(), body);
    if let Some(headers) = headers {
        map.insert("Headers".to_string(), headers);
    }
    Value::Map(Arc::new(RwLock::new(map)))
}

fn build_stream_response_map(stream: Value, status: u16, headers: Option<Value>) -> Value {
    let mut map = HashMap::new();
    map.insert(
        "Status".to_string(),
        Value::from_number_string(&status.to_string()).unwrap_or(Value::default_number()),
    );
    map.insert("Stream".to_string(), stream);
    if let Some(headers) = headers {
        map.insert("Headers".to_string(), headers);
    }
    Value::Map(Arc::new(RwLock::new(map)))
}

fn merge_headers(headers: Option<Value>, key: &str, value: &str) -> Value {
    let mut map = HashMap::new();
    if let Some(Value::Map(existing)) = headers {
        let existing = existing.read().expect("lock poisoned");
        for (k, v) in existing.iter() {
            map.insert(k.clone(), v.clone());
        }
    }
    map.insert(key.to_string(), Value::String(value.to_string()));
    Value::Map(Arc::new(RwLock::new(map)))
}

fn header_exists(headers: &HashMap<String, String>, key: &str) -> bool {
    let target = key.to_lowercase();
    headers.keys().any(|k| k.to_lowercase() == target)
}

fn guess_mime_type(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        "csv" => "text/csv; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

fn value_to_json(value: &Value) -> JsonValue {
    match value {
        Value::Number(n) => {
            if let Some(i) = n.to_i64() {
                JsonValue::Number(i.into())
            } else if let Some(f) = n.to_f64() {
                serde_json::Number::from_f64(f)
                    .map(JsonValue::Number)
                    .unwrap_or_else(|| JsonValue::String(n.to_string()))
            } else {
                JsonValue::String(n.to_string())
            }
        }
        Value::FastNumber(f) => serde_json::Number::from_f64(*f)
            .map(JsonValue::Number)
            .unwrap_or_else(|| JsonValue::String(f.to_string())),
        Value::String(s) => JsonValue::String(s.clone()),
        Value::Boolean(b) => JsonValue::Bool(*b),
        Value::List(list) => {
            let list = list.read().expect("lock poisoned");
            JsonValue::Array(list.iter().map(value_to_json).collect())
        }
        Value::Vector(vec) => JsonValue::Array(
            vec.iter()
                .map(|v| serde_json::Number::from_f64(*v as f64))
                .map(|n| n.map(JsonValue::Number).unwrap_or(JsonValue::Null))
                .collect(),
        ),
        Value::Map(map) => {
            let map = map.read().expect("lock poisoned");
            let mut object = serde_json::Map::new();
            for (key, value) in map.iter() {
                object.insert(key.clone(), value_to_json(value));
            }
            JsonValue::Object(object)
        }
        Value::Option(opt) => match opt.as_ref() {
            Some(inner) => value_to_json(inner),
            None => JsonValue::Null,
        },
        Value::Error(err) => {
            let mut object = serde_json::Map::new();
            object.insert(
                "category".to_string(),
                JsonValue::String(err.category.clone()),
            );
            object.insert(
                "subtype".to_string(),
                JsonValue::String(err.subtype.clone()),
            );
            object.insert(
                "message".to_string(),
                JsonValue::String(err.message.clone()),
            );
            JsonValue::Object(object)
        }
        _ => JsonValue::String(value.to_display_string()),
    }
}
