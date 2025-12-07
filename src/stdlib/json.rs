use crate::runtime::value::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub fn convert_json_to_object(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Boolean(false),
        serde_json::Value::Bool(b) => Value::Boolean(b),
        serde_json::Value::Number(n) => {
            Value::from_number_string(&n.to_string()).unwrap_or(Value::default_number())
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            let list: Vec<Value> = arr.into_iter().map(convert_json_to_object).collect();
            Value::List(Arc::new(std::sync::RwLock::new(list)))
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k, convert_json_to_object(v));
            }
            Value::Map(Arc::new(std::sync::RwLock::new(map)))
        }
    }
}

pub fn create_json_module() -> Value {
    let mut methods = HashMap::new();

    methods.insert(
        "Parse".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("JSON.Parse requires 1 argument".to_string());
                    }

                    let json_str = args[0].to_display_string();

                    match serde_json::from_str(&json_str) {
                        Ok(json_val) => Ok(convert_json_to_object(json_val)),
                        Err(e) => Err(format!("JSON Parse Error: {}", e)),
                    }
                })
            )
        )
    );

    methods.insert(
        "Stringify".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("JSON.Stringify requires 1 argument".to_string());
                    }
                    Ok(Value::String(args[0].to_display_string()))
                })
            )
        )
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}
