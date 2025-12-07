use crate::runtime::value::Value;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

pub fn create_file_module() -> Value {
    let mut methods = HashMap::new();

    // File.Read("path")
    methods.insert(
        "Read".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("File.Read requires exactly 1 argument (path)".to_string());
                    }

                    let path = args[0].to_display_string();

                    match fs::read_to_string(&path) {
                        Ok(content) => Ok(Value::String(content)),
                        Err(_) => Ok(Value::String("".to_string())),
                    }
                })
            )
        )
    );

    // File.Write("path", "content")
    methods.insert(
        "Write".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 2 {
                        return Err("File.Write requires 2 arguments (path, content)".to_string());
                    }

                    let path = args[0].to_display_string();
                    let content = args[1].to_display_string();

                    match fs::write(&path, content) {
                        Ok(_) => Ok(Value::Boolean(true)),
                        Err(e) => Err(format!("Failed to write file: {}", e)),
                    }
                })
            )
        )
    );

    // File.Exists("path")
    methods.insert(
        "Exists".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("File.Exists requires 1 argument".to_string());
                    }
                    let path = args[0].to_display_string();
                    Ok(Value::Boolean(std::path::Path::new(&path).exists()))
                })
            )
        )
    );

    // File.List(directory) or File.List(directory, pattern)
    methods.insert(
        "List".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.is_empty() || args.len() > 2 {
                        return Err(
                            "File.List requires 1 or 2 arguments (directory, optional pattern)".to_string()
                        );
                    }

                    let directory = args[0].to_display_string();
                    let pattern = if args.len() == 2 {
                        Some(args[1].to_display_string())
                    } else {
                        None
                    };

                    match fs::read_dir(&directory) {
                        Ok(entries) => {
                            let mut files = Vec::new();

                            for entry in entries {
                                if let Ok(entry) = entry {
                                    let path = entry.path();

                                    // Only include files, not directories
                                    if path.is_file() {
                                        if let Some(file_name) = path.file_name() {
                                            let name = file_name.to_string_lossy().to_string();

                                            // Apply pattern filter if provided
                                            let matches = if let Some(ref pat) = pattern {
                                                // Simple glob pattern matching
                                                if pat.starts_with("*.") {
                                                    let ext = &pat[2..];
                                                    name.ends_with(ext)
                                                } else if pat.contains('*') {
                                                    // Simple wildcard matching
                                                    let parts: Vec<&str> = pat.split('*').collect();
                                                    if parts.len() == 2 {
                                                        name.starts_with(parts[0]) &&
                                                            name.ends_with(parts[1])
                                                    } else {
                                                        true
                                                    }
                                                } else {
                                                    name == *pat
                                                }
                                            } else {
                                                true
                                            };

                                            if matches {
                                                // Store full path
                                                files.push(
                                                    Value::String(
                                                        path.to_string_lossy().to_string()
                                                    )
                                                );
                                            }
                                        }
                                    }
                                }
                            }

                            Ok(Value::List(Arc::new(std::sync::RwLock::new(files))))
                        }
                        Err(e) => Err(format!("Failed to read directory: {}", e)),
                    }
                })
            )
        )
    );

    // NOTE: 1-based indexing! Line 1 is the first line.
    methods.insert(
        "ReadLines".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 3 {
                        return Err(
                            "File.ReadLines requires 3 arguments (path, start_line, count)".to_string()
                        );
                    }

                    let path = args[0].to_display_string();
                    let start_line = match &args[1] {
                        Value::Number(n) => {
                            let val = n.to_string().parse::<usize>().unwrap_or(1);
                            if val < 1 {
                                return Err(
                                    "start_line must be >= 1 (1-based indexing)".to_string()
                                );
                            }
                            val
                        }
                        _ => {
                            return Err("start_line must be a number".to_string());
                        }
                    };
                    let count = match &args[2] {
                        Value::Number(n) => n.to_string().parse::<usize>().unwrap_or(1000),
                        _ => {
                            return Err("count must be a number".to_string());
                        }
                    };

                    use std::io::{ BufRead, BufReader };

                    match fs::File::open(&path) {
                        Ok(file) => {
                            let reader = BufReader::new(file);
                            let mut lines = Vec::new();

                            // Convert 1-based to 0-based for internal iteration
                            let offset_zero_based = start_line - 1;

                            // Skip lines until start_line, then take count lines
                            for (i, line) in reader.lines().enumerate() {
                                if i < offset_zero_based {
                                    continue;
                                }
                                if i >= offset_zero_based + count {
                                    break;
                                }

                                match line {
                                    Ok(content) => lines.push(Value::String(content)),
                                    Err(_) => {
                                        break;
                                    }
                                }
                            }

                            Ok(Value::List(Arc::new(std::sync::RwLock::new(lines))))
                        }
                        Err(e) => Err(format!("Failed to open file: {}", e)),
                    }
                })
            )
        )
    );

    // File.CountLines(path) - Count total lines without loading file
    methods.insert(
        "CountLines".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("File.CountLines requires 1 argument (path)".to_string());
                    }

                    let path = args[0].to_display_string();

                    use std::io::{ BufRead, BufReader };

                    match fs::File::open(&path) {
                        Ok(file) => {
                            let reader = BufReader::new(file);
                            let count = reader.lines().count();

                            use bigdecimal::BigDecimal;
                            Ok(Value::Number(BigDecimal::from(count as i64)))
                        }
                        Err(e) => Err(format!("Failed to open file: {}", e)),
                    }
                })
            )
        )
    );

    // File.ReadStream(path) - Returns stream that reads file line-by-line
    methods.insert(
        "ReadStream".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("File.ReadStream requires 1 argument (path)".to_string());
                    }

                    let path = args[0].to_display_string();

                    use std::io::{ BufRead, BufReader };
                    use std::sync::{ Arc, Mutex };

                    // Open file and create BufReader
                    let file = fs::File
                        ::open(&path)
                        .map_err(|e| format!("Failed to open file: {}", e))?;

                    let lines_iter = BufReader::new(file).lines();
                    let reader = Arc::new(Mutex::new(lines_iter));

                    // Create generator function that reads next line
                    let reader_clone = reader.clone();
                    let generator = Value::NativeFunction(
                        Arc::new(
                            Box::new(move |_args| {
                                let mut reader_lock = reader_clone.lock().unwrap();
                                match reader_lock.next() {
                                    Some(Ok(line)) =>
                                        Ok(Value::Option(Box::new(Some(Value::String(line))))),
                                    Some(Err(e)) => Err(format!("Failed to read line: {}", e)),
                                    None => Ok(Value::Option(Box::new(None))),
                                }
                            })
                        )
                    );

                    // Create stream object with the generator
                    let stream = crate::stdlib::stream::create_stream_object(
                        vec![],
                        Some(generator)
                    );
                    Ok(stream)
                })
            )
        )
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}
