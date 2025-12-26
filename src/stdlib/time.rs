use crate::runtime::value::Value;
use chrono::{DateTime, Datelike, Local, TimeZone, Timelike, Utc};
use std::collections::HashMap;
use std::sync::Arc;

pub fn create_time_module() -> Value {
    let mut methods = HashMap::new();

    methods.insert(
        "Now".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if !args.is_empty() {
                return Err("Time.Now requires 0 arguments".to_string());
            }

            let now = chrono::Utc::now();
            let timestamp = now.timestamp();

            use bigdecimal::BigDecimal;
            Ok(Value::Number(BigDecimal::from(timestamp)))
        }))),
    );

    methods.insert(
        "Precise".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if !args.is_empty() {
                return Err("Time.Precise requires 0 arguments".to_string());
            }

            let now = std::time::SystemTime::now();
            let duration = now
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| "System time error".to_string())?;

            let seconds = duration.as_secs() as f64;
            let nanos = duration.subsec_nanos() as f64;
            let precise_time = seconds + nanos / 1_000_000_000.0;

            Ok(Value::FastNumber(precise_time))
        }))),
    );

    methods.insert(
        "LocalTime".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() > 1 {
                return Err(
                    "Time.LocalTime requires 0-1 arguments (optional timestamp)".to_string()
                );
            }

            let dt: DateTime<Local> = if args.is_empty() {
                Local::now()
            } else {
                let timestamp = match &args[0] {
                    Value::Number(n) => n
                        .to_string()
                        .parse::<i64>()
                        .map_err(|_| "Invalid timestamp".to_string())?,
                    _ => {
                        return Err("Timestamp must be a number".to_string());
                    }
                };

                Local
                    .timestamp_opt(timestamp, 0)
                    .single()
                    .ok_or("Invalid timestamp")?
            };

            Ok(create_datetime_map(dt))
        }))),
    );

    methods.insert(
        "GMTime".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() > 1 {
                return Err("Time.GMTime requires 0-1 arguments (optional timestamp)".to_string());
            }

            let dt: DateTime<Utc> = if args.is_empty() {
                Utc::now()
            } else {
                let timestamp = match &args[0] {
                    Value::Number(n) => n
                        .to_string()
                        .parse::<i64>()
                        .map_err(|_| "Invalid timestamp".to_string())?,
                    _ => {
                        return Err("Timestamp must be a number".to_string());
                    }
                };

                Utc.timestamp_opt(timestamp, 0)
                    .single()
                    .ok_or("Invalid timestamp")?
            };

            Ok(create_datetime_map(dt))
        }))),
    );

    methods.insert(
        "Format".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 2 {
                return Err("Time.Format requires 2 arguments (datetime, format)".to_string());
            }

            let dt_map = match &args[0] {
                Value::Map(m) => m,
                _ => {
                    return Err("First argument must be a datetime map".to_string());
                }
            };

            let format = args[1].to_display_string();

            let map_borrow = dt_map.read().expect("lock poisoned");

            let year = get_number_field(&map_borrow, "Year")? as i32;
            let month = get_number_field(&map_borrow, "Month")? as u32;
            let day = get_number_field(&map_borrow, "Day")? as u32;
            let hour = get_number_field(&map_borrow, "Hour")? as u32;
            let minute = get_number_field(&map_borrow, "Minute")? as u32;
            let second = get_number_field(&map_borrow, "Second")? as u32;

            let dt = Local
                .with_ymd_and_hms(year, month, day, hour, minute, second)
                .single()
                .ok_or("Invalid datetime components")?;

            let formatted = dt.format(&format).to_string();
            Ok(Value::String(formatted))
        }))),
    );

    methods.insert(
        "Sleep".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("Time.Sleep requires 1 argument (seconds)".to_string());
            }

            let seconds = match &args[0] {
                Value::Number(n) => n
                    .to_string()
                    .parse::<f64>()
                    .map_err(|_| "Invalid sleep duration".to_string())?,
                Value::FastNumber(f) => *f,
                _ => {
                    return Err("Sleep duration must be a number".to_string());
                }
            };

            if seconds < 0.0 {
                return Err("Sleep duration cannot be negative".to_string());
            }

            let millis = (seconds * 1000.0) as u64;
            std::thread::sleep(std::time::Duration::from_millis(millis));

            Ok(Value::Boolean(true))
        }))),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}

fn create_datetime_map<Tz: TimeZone>(dt: DateTime<Tz>) -> Value
where
    Tz::Offset: std::fmt::Display,
{
    let mut dt_map = HashMap::new();

    use bigdecimal::BigDecimal;

    dt_map.insert(
        "Year".to_string(),
        Value::Number(BigDecimal::from(dt.year())),
    );
    dt_map.insert(
        "Month".to_string(),
        Value::Number(BigDecimal::from(dt.month() as i32)),
    );
    dt_map.insert(
        "Day".to_string(),
        Value::Number(BigDecimal::from(dt.day() as i32)),
    );
    dt_map.insert(
        "Hour".to_string(),
        Value::Number(BigDecimal::from(dt.hour() as i32)),
    );
    dt_map.insert(
        "Minute".to_string(),
        Value::Number(BigDecimal::from(dt.minute() as i32)),
    );
    dt_map.insert(
        "Second".to_string(),
        Value::Number(BigDecimal::from(dt.second() as i32)),
    );

    let weekday = dt.weekday().number_from_monday();
    dt_map.insert(
        "Weekday".to_string(),
        Value::Number(BigDecimal::from(weekday as i32)),
    );

    dt_map.insert(
        "YearDay".to_string(),
        Value::Number(BigDecimal::from(dt.ordinal() as i32)),
    );

    dt_map.insert(
        "Timestamp".to_string(),
        Value::Number(BigDecimal::from(dt.timestamp())),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(dt_map)))
}

fn get_number_field(map: &HashMap<String, Value>, field: &str) -> Result<i64, String> {
    match map.get(field) {
        Some(Value::Number(n)) => n
            .to_string()
            .parse::<i64>()
            .map_err(|_| format!("Invalid {} value", field)),
        Some(_) => Err(format!("{} must be a number", field)),
        None => Err(format!("Missing {} field", field)),
    }
}
