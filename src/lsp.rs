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

use crate::compiler::lexer::Lexer;
use crate::compiler::parser::Parser;
use serde_json::{Value as JsonValue, json};
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, BufWriter, Write};

struct LspState {
    documents: HashMap<String, String>,
}

impl LspState {
    fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }
}

pub fn run() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());
    let mut state = LspState::new();

    loop {
        let message = read_message(&mut reader)?;
        let Some(message) = message else { break };

        if !handle_message(&message, &mut state, &mut writer)? {
            break;
        }
    }

    Ok(())
}

fn handle_message(
    message: &JsonValue,
    state: &mut LspState,
    writer: &mut impl Write,
) -> io::Result<bool> {
    let method = message.get("method").and_then(|m| m.as_str());

    match method {
        Some("initialize") => {
            let id = message.get("id").cloned().unwrap_or(JsonValue::Null);
            let result = json!({
                "capabilities": {
                    "textDocumentSync": {
                        "openClose": true,
                        "change": 1
                    }
                },
                "serverInfo": {
                    "name": "SFX LSP",
                    "version": env!("CARGO_PKG_VERSION")
                }
            });
            write_response(writer, id, result)?;
        }
        Some("shutdown") => {
            let id = message.get("id").cloned().unwrap_or(JsonValue::Null);
            write_response(writer, id, JsonValue::Null)?;
        }
        Some("exit") => {
            return Ok(false);
        }
        Some("textDocument/didOpen") => {
            if let Some(params) = message.get("params") {
                if let (Some(uri), Some(text)) = (
                    params.pointer("/textDocument/uri").and_then(|v| v.as_str()),
                    params
                        .pointer("/textDocument/text")
                        .and_then(|v| v.as_str()),
                ) {
                    state.documents.insert(uri.to_string(), text.to_string());
                    publish_diagnostics(writer, uri, text)?;
                }
            }
        }
        Some("textDocument/didChange") => {
            if let Some(params) = message.get("params") {
                let uri = params
                    .pointer("/textDocument/uri")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if let Some(changes) = params.pointer("/contentChanges").and_then(|v| v.as_array())
                {
                    if let Some(last) = changes.last() {
                        if let Some(text) = last.get("text").and_then(|v| v.as_str()) {
                            state.documents.insert(uri.to_string(), text.to_string());
                            publish_diagnostics(writer, uri, text)?;
                        }
                    }
                }
            }
        }
        _ => {}
    }

    Ok(true)
}

fn read_message(reader: &mut impl BufRead) -> io::Result<Option<JsonValue>> {
    let mut content_length: Option<usize> = None;
    let mut line = String::new();

    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some(rest) = trimmed.strip_prefix("Content-Length:") {
            content_length = rest.trim().parse::<usize>().ok();
        }
    }

    let Some(length) = content_length else {
        return Ok(None);
    };

    let mut body = vec![0u8; length];
    reader.read_exact(&mut body)?;
    let message = serde_json::from_slice(&body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    Ok(Some(message))
}

fn write_message(writer: &mut impl Write, payload: &JsonValue) -> io::Result<()> {
    let body = serde_json::to_vec(payload)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()?;
    Ok(())
}

fn write_response(writer: &mut impl Write, id: JsonValue, result: JsonValue) -> io::Result<()> {
    let response = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    });
    write_message(writer, &response)
}

fn publish_diagnostics(writer: &mut impl Write, uri: &str, text: &str) -> io::Result<()> {
    let diagnostics = build_diagnostics(text);
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": uri,
            "diagnostics": diagnostics
        }
    });
    write_message(writer, &notification)
}

fn build_diagnostics(text: &str) -> Vec<JsonValue> {
    let mut lexer = Lexer::new(text);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(err) => {
            return vec![make_diagnostic(err.to_string(), err.line, err.column)];
        }
    };

    let mut parser = Parser::new(tokens);
    if let Err(err) = parser.parse() {
        let (line, column) = err.location();
        return vec![make_diagnostic(err.to_string(), line, column)];
    }

    Vec::new()
}

fn make_diagnostic(message: String, line: usize, column: usize) -> JsonValue {
    let line_idx = line.saturating_sub(1);
    let col_idx = column.saturating_sub(1);
    json!({
        "range": {
            "start": { "line": line_idx, "character": col_idx },
            "end": { "line": line_idx, "character": col_idx.saturating_add(1) }
        },
        "severity": 1,
        "source": "sfx",
        "message": message
    })
}
