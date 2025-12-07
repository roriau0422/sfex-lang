use crate::runtime::value::Value;
use crate::stdlib::{ csv, html, json, toml, xml };
use file_format::FileFormat;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

fn sanitize_content(bytes: &[u8]) -> String {
    if bytes.starts_with(b"\xEF\xBB\xBF") {
        String::from_utf8_lossy(&bytes[3..]).to_string()
    } else {
        String::from_utf8_lossy(bytes).to_string()
    }
}

fn guess_format_priority(content: &str, filepath: Option<&str>) -> Vec<&'static str> {
    let mut candidates = Vec::new();
    let trimmed = content.trim();

    if let Some(path_str) = filepath {
        if let Some(ext) = Path::new(path_str).extension() {
            match ext.to_string_lossy().to_lowercase().as_str() {
                "json" => candidates.push("JSON"),
                "xml" => candidates.push("XML"),
                "html" | "htm" => candidates.push("HTML"),
                "toml" => candidates.push("TOML"),
                "csv" => candidates.push("CSV"),
                "yaml" | "yml" => candidates.push("YAML"),
                _ => {}
            }
        }
    }

    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        if !candidates.contains(&"JSON") {
            candidates.push("JSON");
        }
    }

    if trimmed.starts_with('<') {
        if trimmed.to_lowercase().starts_with("<!doctype html") || trimmed.contains("</html>") {
            if !candidates.contains(&"HTML") {
                candidates.push("HTML");
            }
        } else {
            if !candidates.contains(&"XML") {
                candidates.push("XML");
            }
            if !candidates.contains(&"HTML") {
                candidates.push("HTML");
            }
        }
    }

    if
        trimmed
            .lines()
            .take(5)
            .any(|l| {
                let line = l.trim();
                (line.starts_with('[') && line.ends_with(']')) ||
                    (line.contains('=') && !line.starts_with('<'))
            })
    {
        if !candidates.contains(&"TOML") {
            candidates.push("TOML");
        }
    }

    if trimmed.contains('\n') && (trimmed.contains(',') || trimmed.contains(';')) {
        if candidates.is_empty() || candidates.contains(&"CSV") {
            if !candidates.contains(&"CSV") {
                candidates.push("CSV");
            }
        }
    }

    candidates
}

fn get_media_type_for_format(format: &str, fallback: &str) -> String {
    match format {
        "JSON" | "JavaScript Object Notation" => "application/json".to_string(),
        "XML" | "Extensible Markup Language" => "text/xml".to_string(),
        "HTML" | "HyperText Markup Language" => "text/html".to_string(),
        "TOML" | "Tom's Obvious Minimal Language" => "application/toml".to_string(),
        "CSV" | "Comma-Separated Values" => "text/csv".to_string(),
        "YAML" | "YAML Ain't Markup Language" => "application/yaml".to_string(),
        _ => fallback.to_string(),
    }
}

pub fn create_data_module() -> Value {
    let mut methods = HashMap::new();

    methods.insert(
        "Detect".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("Data.Detect requires 1 argument".to_string());
                    }
                    let filepath = args[0].to_display_string();

                    match std::fs::File::open(&filepath) {
                        Ok(mut file) => {
                            let mut buffer = [0u8; 8192];
                            let bytes_read = file.read(&mut buffer).unwrap_or(0);

                            let format = FileFormat::from_bytes(&buffer[..bytes_read]);
                            let base_name = format.name();
                            let base_media_type = format.media_type();
                            let base_kind = format.kind();

                            let content_sample = sanitize_content(&buffer[..bytes_read]);
                            let priorities = guess_format_priority(
                                &content_sample,
                                Some(&filepath)
                            );

                            let (final_format, final_media_type) = if
                                base_name == "Plain Text" ||
                                base_media_type == "application/octet-stream"
                            {
                                if let Some(best_guess) = priorities.first() {
                                    (
                                        best_guess.to_string(),
                                        get_media_type_for_format(best_guess, base_media_type),
                                    )
                                } else {
                                    (base_name.to_string(), base_media_type.to_string())
                                }
                            } else {
                                (base_name.to_string(), base_media_type.to_string())
                            };

                            let mut info = HashMap::new();
                            info.insert("Format".to_string(), Value::String(final_format));
                            info.insert("MediaType".to_string(), Value::String(final_media_type));
                            info.insert(
                                "Kind".to_string(),
                                Value::String(format!("{:?}", base_kind))
                            );
                            info.insert(
                                "Extension".to_string(),
                                Value::String(format.extension().to_string())
                            );

                            let candidates: Vec<Value> = priorities
                                .iter()
                                .map(|s| Value::String(s.to_string()))
                                .collect();
                            info.insert(
                                "Candidates".to_string(),
                                Value::List(Arc::new(std::sync::RwLock::new(candidates)))
                            );

                            Ok(Value::Map(Arc::new(std::sync::RwLock::new(info))))
                        }
                        Err(e) => Err(format!("Failed to read file: {}", e)),
                    }
                })
            )
        )
    );

    methods.insert(
        "DetectFromString".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("Data.DetectFromString requires 1 argument".to_string());
                    }
                    let content = args[0].to_display_string();

                    let priorities = guess_format_priority(&content, None);
                    let best_guess = priorities.first().cloned().unwrap_or("Plain Text");
                    let media_type = get_media_type_for_format(best_guess, "text/plain");

                    let mut info = HashMap::new();
                    info.insert("Format".to_string(), Value::String(best_guess.to_string()));
                    info.insert("MediaType".to_string(), Value::String(media_type));
                    info.insert("Kind".to_string(), Value::String("Text".to_string()));

                    Ok(Value::Map(Arc::new(std::sync::RwLock::new(info))))
                })
            )
        )
    );

    methods.insert(
        "Parse".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("Data.Parse requires 1 argument".to_string());
                    }
                    let filepath = args[0].to_display_string();

                    const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;
                    let file_size = match std::fs::metadata(&filepath) {
                        Ok(m) => m.len(),
                        Err(e) => {
                            return Err(format!("Metadata error: {}", e));
                        }
                    };
                    if file_size > MAX_FILE_SIZE {
                        return Err("File too large".to_string());
                    }

                    let raw_bytes = match std::fs::read(&filepath) {
                        Ok(b) => b,
                        Err(e) => {
                            return Err(format!("Read error: {}", e));
                        }
                    };
                    let content = sanitize_content(&raw_bytes);

                    let priorities = guess_format_priority(&content, Some(&filepath));

                    for format in priorities {
                        match format {
                            "JSON" => {
                                if
                                    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(
                                        &content
                                    )
                                {
                                    return Ok(json::convert_json_to_object(parsed));
                                }
                            }
                            "TOML" => {
                                if let Ok(table) = ::toml::from_str::<::toml::Table>(&content) {
                                    return Ok(
                                        toml::convert_toml_to_object(::toml::Value::Table(table))
                                    );
                                }
                            }
                            "XML" => {
                                if let Ok(doc) = xml::parse_xml(&content) {
                                    return Ok(doc);
                                }
                            }
                            "HTML" => {
                                if let Ok(doc) = html::parse_html(&content) {
                                    return Ok(doc);
                                }
                            }
                            "CSV" => {
                                if let Ok(res) = csv::parse_csv(&content) {
                                    return Ok(res);
                                }
                            }
                            _ => {}
                        }
                    }

                    Ok(Value::String(content))
                })
            )
        )
    );

    methods.insert(
        "Describe".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("Data.Describe requires 1 argument".to_string());
                    }
                    let filepath = args[0].to_display_string();

                    let size = match std::fs::metadata(&filepath) {
                        Ok(m) => m.len(),
                        Err(e) => {
                            return Err(format!("IO Error: {}", e));
                        }
                    };

                    match std::fs::File::open(&filepath) {
                        Ok(mut file) => {
                            let mut buffer = [0u8; 8192];
                            let bytes_read = file.read(&mut buffer).unwrap_or(0);

                            let format = FileFormat::from_bytes(&buffer[..bytes_read]);
                            let base_name = format.name();
                            let base_media_type = format.media_type();

                            let content_sample = sanitize_content(&buffer[..bytes_read]);
                            let priorities = guess_format_priority(
                                &content_sample,
                                Some(&filepath)
                            );

                            let (final_format, final_media_type) = if
                                base_name == "Plain Text" ||
                                base_media_type == "application/octet-stream"
                            {
                                if let Some(best_guess) = priorities.first() {
                                    (
                                        best_guess.to_string(),
                                        get_media_type_for_format(best_guess, base_media_type),
                                    )
                                } else {
                                    (base_name.to_string(), base_media_type.to_string())
                                }
                            } else {
                                (base_name.to_string(), base_media_type.to_string())
                            };

                            let mut description = HashMap::new();
                            description.insert(
                                "Format".to_string(),
                                Value::String(final_format.clone())
                            );
                            description.insert(
                                "MediaType".to_string(),
                                Value::String(final_media_type)
                            );
                            description.insert(
                                "Extension".to_string(),
                                Value::String(format.extension().to_string())
                            );

                            use bigdecimal::BigDecimal;
                            description.insert(
                                "Size".to_string(),
                                Value::Number(BigDecimal::from(size as i64))
                            );

                            let parseable = matches!(
                                final_format.as_str(),
                                "JSON" | "XML" | "HTML" | "TOML" | "CSV"
                            );
                            description.insert("Parseable".to_string(), Value::Boolean(parseable));

                            Ok(Value::Map(Arc::new(std::sync::RwLock::new(description))))
                        }
                        Err(e) => Err(format!("Failed to read file: {}", e)),
                    }
                })
            )
        )
    );

    methods.insert(
        "Structure".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.is_empty() || args.len() > 2 {
                        return Err("Data.Structure requires 1 or 2 arguments".to_string());
                    }

                    let data = &args[0];
                    let max_depth = if args.len() == 2 {
                        match &args[1] {
                            Value::Number(n) => n.to_string().parse::<usize>().unwrap_or(10),
                            _ => 10,
                        }
                    } else {
                        10
                    };

                    fn analyze_structure(value: &Value, depth: usize, max_depth: usize) -> Value {
                        if depth > max_depth {
                            return Value::String("...".to_string());
                        }

                        match value {
                            Value::Map(m) => {
                                let map = m.read().unwrap();
                                let mut s = HashMap::new();
                                for (k, v) in map.iter() {
                                    s.insert(k.clone(), analyze_structure(v, depth + 1, max_depth));
                                }
                                Value::Map(Arc::new(std::sync::RwLock::new(s)))
                            }
                            Value::List(l) => {
                                let list = l.read().unwrap();
                                let count = list.len();
                                let mut s = HashMap::new();
                                s.insert("type".to_string(), Value::String("List".to_string()));
                                s.insert(
                                    "count".to_string(),
                                    Value::Number(bigdecimal::BigDecimal::from(count as i64))
                                );
                                if !list.is_empty() {
                                    s.insert(
                                        "sample_item".to_string(),
                                        analyze_structure(&list[0], depth + 1, max_depth)
                                    );
                                }
                                Value::Map(Arc::new(std::sync::RwLock::new(s)))
                            }
                            Value::String(_) => Value::String("String".to_string()),
                            Value::Number(_) => Value::String("Number".to_string()),
                            Value::Boolean(_) => Value::String("Boolean".to_string()),
                            _ =>
                                Value::String(
                                    format!("{:?}", value)
                                        .split('(')
                                        .next()
                                        .unwrap_or("Unknown")
                                        .to_string()
                                ),
                        }
                    }

                    Ok(analyze_structure(data, 0, max_depth))
                })
            )
        )
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}
