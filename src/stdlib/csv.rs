use crate::runtime::value::Value;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

pub fn parse_csv(csv_data: &str) -> Result<Value, String> {
    let cursor = Cursor::new(csv_data);
    let mut rdr = csv::ReaderBuilder::new().has_headers(true).from_reader(cursor);

    let mut list_of_rows = Vec::new();

    // Get headers
    let headers = match rdr.headers() {
        Ok(h) => h.clone(),
        Err(e) => {
            return Err(format!("CSV Header Error: {}", e));
        }
    };

    for result in rdr.records() {
        let record = result.map_err(|e| format!("CSV Record Error: {}", e))?;
        let mut row_map = HashMap::new();

        for (i, field) in record.iter().enumerate() {
            if let Some(header_name) = headers.get(i) {
                // Try to parse as number if possible, else string
                let val = if let Ok(num) = Value::from_number_string(field) {
                    num
                } else {
                    Value::String(field.to_string())
                };
                row_map.insert(header_name.to_string(), val);
            }
        }
        list_of_rows.push(Value::Map(Arc::new(std::sync::RwLock::new(row_map))));
    }

    Ok(Value::List(Arc::new(std::sync::RwLock::new(list_of_rows))))
}

pub fn create_csv_module() -> Value {
    let mut methods = HashMap::new();

    methods.insert(
        "Parse".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("CSV.Parse requires 1 argument".to_string());
                    }

                    let csv_data = args[0].to_display_string();
                    parse_csv(&csv_data)
                })
            )
        )
    );

    // NOTE: 1-based indexing! Row 1 is the first data row (after headers).
    methods.insert(
        "ReadRows".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 3 {
                        return Err(
                            "CSV.ReadRows requires 3 arguments (filepath, start_row, count)".to_string()
                        );
                    }

                    let filepath = args[0].to_display_string();
                    let start_row = match &args[1] {
                        Value::Number(n) => {
                            let val = n.to_string().parse::<usize>().unwrap_or(1);
                            if val < 1 {
                                return Err("start_row must be >= 1 (1-based indexing)".to_string());
                            }
                            val
                        }
                        _ => {
                            return Err("start_row must be a number".to_string());
                        }
                    };
                    let count = match &args[2] {
                        Value::Number(n) => n.to_string().parse::<usize>().unwrap_or(1000),
                        _ => {
                            return Err("count must be a number".to_string());
                        }
                    };

                    use std::fs::File;

                    match File::open(&filepath) {
                        Ok(file) => {
                            let mut rdr = csv::ReaderBuilder
                                ::new()
                                .has_headers(true)
                                .from_reader(file);

                            // Get headers
                            let headers = match rdr.headers() {
                                Ok(h) => h.clone(),
                                Err(e) => {
                                    return Err(format!("CSV Header Error: {}", e));
                                }
                            };

                            let mut list_of_rows = Vec::new();
                            let mut row_count = 1; // 1-based row counting

                            // Convert 1-based to 0-based for iteration
                            let offset_zero_based = start_row - 1;

                            for result in rdr.records() {
                                // Skip until to reach start_row (1-based)
                                if row_count - 1 < offset_zero_based {
                                    row_count += 1;
                                    continue;
                                }

                                if list_of_rows.len() >= count {
                                    break;
                                }

                                match result {
                                    Ok(record) => {
                                        let mut row_map = HashMap::new();

                                        for (i, field) in record.iter().enumerate() {
                                            if let Some(header_name) = headers.get(i) {
                                                // Try to parse as number if possible, else string
                                                let val = if
                                                    let Ok(num) = Value::from_number_string(field)
                                                {
                                                    num
                                                } else {
                                                    Value::String(field.to_string())
                                                };
                                                row_map.insert(header_name.to_string(), val);
                                            }
                                        }
                                        list_of_rows.push(
                                            Value::Map(Arc::new(std::sync::RwLock::new(row_map)))
                                        );
                                    }
                                    Err(e) => {
                                        return Err(format!("CSV Record Error: {}", e));
                                    }
                                }

                                row_count += 1;
                            }

                            Ok(Value::List(Arc::new(std::sync::RwLock::new(list_of_rows))))
                        }
                        Err(e) => Err(format!("Failed to open file: {}", e)),
                    }
                })
            )
        )
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}
