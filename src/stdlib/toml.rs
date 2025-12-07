use crate::runtime::value::Value;
use std::collections::HashMap;
use std::sync::Arc;
use toml::{ Table, Value as TomlValue };

pub fn convert_toml_to_object(toml: TomlValue) -> Value {
    match toml {
        TomlValue::String(s) => Value::String(s),
        TomlValue::Integer(i) => {
            Value::from_number_string(&i.to_string()).unwrap_or(Value::default_number())
        }
        TomlValue::Float(f) => {
            Value::from_number_string(&f.to_string()).unwrap_or(Value::default_number())
        }
        TomlValue::Boolean(b) => Value::Boolean(b),
        TomlValue::Datetime(d) => Value::String(d.to_string()),
        TomlValue::Array(arr) => {
            let list: Vec<Value> = arr.into_iter().map(convert_toml_to_object).collect();
            Value::List(Arc::new(std::sync::RwLock::new(list)))
        }
        TomlValue::Table(table) => {
            let mut map = HashMap::new();
            for (k, v) in table {
                map.insert(k, convert_toml_to_object(v));
            }
            Value::Map(Arc::new(std::sync::RwLock::new(map)))
        }
    }
}

pub fn create_toml_module() -> Value {
    let mut methods = HashMap::new();

    methods.insert(
        "Parse".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("TOML.Parse requires 1 argument".to_string());
                    }

                    let toml_str = args[0].to_display_string();

                    match toml::from_str::<Table>(&toml_str) {
                        Ok(table) => Ok(convert_toml_to_object(TomlValue::Table(table))),
                        Err(e) => Err(format!("TOML Parse Error: {}", e)),
                    }
                })
            )
        )
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}
