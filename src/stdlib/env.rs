use crate::runtime::value::Value;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

pub fn create_env_module() -> Value {
    let mut methods = HashMap::new();

    methods.insert(
        "Get".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.is_empty() || args.len() > 2 {
                return Err("Env.Get requires 1 or 2 arguments (key, optional default)".to_string());
            }

            let key = args[0].to_display_string();
            let default = if args.len() == 2 {
                args[1].to_display_string()
            } else {
                String::new()
            };

            match env::var(&key) {
                Ok(value) => Ok(Value::String(value)),
                Err(_) => Ok(Value::String(default)),
            }
        }))),
    );

    methods.insert(
        "Has".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("Env.Has requires 1 argument (key)".to_string());
            }

            let key = args[0].to_display_string();
            Ok(Value::Boolean(env::var(&key).is_ok()))
        }))),
    );

    methods.insert(
        "All".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if !args.is_empty() {
                return Err("Env.All requires no arguments".to_string());
            }

            let mut env_map = HashMap::new();
            for (key, value) in env::vars() {
                env_map.insert(key, Value::String(value));
            }

            Ok(Value::Map(Arc::new(std::sync::RwLock::new(env_map))))
        }))),
    );

    methods.insert(
        "Load".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("Env.Load requires 1 argument (filepath)".to_string());
            }

            let filepath = args[0].to_display_string();

            match std::fs::read_to_string(&filepath) {
                Ok(content) => {
                    let mut count = 0;
                    for line in content.lines() {
                        let line = line.trim();

                        if line.is_empty() || line.starts_with('#') {
                            continue;
                        }

                        if let Some(eq_pos) = line.find('=') {
                            let key = line[..eq_pos].trim();
                            let mut value = line[eq_pos + 1..].trim();

                            if (value.starts_with('"') && value.ends_with('"'))
                                || (value.starts_with('\'') && value.ends_with('\''))
                            {
                                value = &value[1..value.len() - 1];
                            }

                            unsafe {
                                env::set_var(key, value);
                            }
                            count += 1;
                        }
                    }

                    use bigdecimal::BigDecimal;
                    Ok(Value::Number(BigDecimal::from(count as i64)))
                }
                Err(e) => Err(format!("Failed to load .env file: {}", e)),
            }
        }))),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}
