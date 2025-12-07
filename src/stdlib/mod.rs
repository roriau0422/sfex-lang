pub mod channel;
pub mod csv;
pub mod data;
pub mod env;
pub mod error;
pub mod file;
pub mod html;
pub mod http_net;
pub mod json;
pub mod llm;
pub mod math;
pub mod stream;
pub mod system;
pub mod task;
pub mod tcp;
pub mod time;
pub mod toml;
pub mod udp;
pub mod websocket;
pub mod xml;

use crate::runtime::interpreter::Interpreter;

pub fn register_stdlib(interpreter: &mut Interpreter) {
    let file_module = file::create_file_module();
    interpreter.define_global("File", file_module);

    let json_module = json::create_json_module();
    interpreter.define_global("JSON", json_module);

    let html_module = html::create_html_module();
    interpreter.define_global("HTML", html_module);

    let xml_module = xml::create_xml_module();
    interpreter.define_global("XML", xml_module);

    let toml_module = toml::create_toml_module();
    interpreter.define_global("TOML", toml_module);

    let csv_module = csv::create_csv_module();
    interpreter.define_global("CSV", csv_module);

    let http_module = http_net::create_http_module(interpreter);
    interpreter.define_global("HTTP", http_module);

    let websocket_module = websocket::create_websocket_module(interpreter);
    interpreter.define_global("WebSocket", websocket_module);

    let tcp_module = tcp::create_tcp_module();
    interpreter.define_global("TCP", tcp_module);

    let udp_module = udp::create_udp_module();
    interpreter.define_global("UDP", udp_module);

    let env_module = env::create_env_module();
    interpreter.define_global("Env", env_module);

    let data_module = data::create_data_module();
    interpreter.define_global("Data", data_module);

    let system_module = system::create_system_module();
    interpreter.define_global("System", system_module);

    let time_module = time::create_time_module();
    interpreter.define_global("Time", time_module);

    let llm_module = llm::create_llm_module();
    interpreter.define_global("LLM", llm_module);

    let stream_module = stream::create_stream_module();
    interpreter.define_global("Stream", stream_module);

    let task_module = task::create_task_module(interpreter);
    interpreter.define_global("Task", task_module);

    let channel_module = channel::create_channel_module(interpreter);
    interpreter.define_global("Channel", channel_module);

    let error_module = error::create_error_module();
    interpreter.define_global("Error", error_module);

    let math_module = math::create_math_module();
    interpreter.define_global("Math", math_module);

    // FastNumber() creates fast floating-point numbers
    use crate::runtime::value::Value;
    use std::sync::Arc;
    let fast_number_fn = Value::NativeFunction(Arc::new(Box::new(|args| {
        if args.len() != 1 {
            return Err("FastNumber requires 1 argument (number to convert)".to_string());
        }

        match &args[0] {
            Value::Number(n) => {
                use bigdecimal::ToPrimitive;
                let f64_val = n.to_f64().ok_or("Number too large for FastNumber")?;
                Ok(Value::FastNumber(f64_val))
            }
            Value::FastNumber(f) => {
                Ok(Value::FastNumber(*f))
            }
            Value::String(s) => {
                let f64_val = s
                    .parse::<f64>()
                    .map_err(|_| format!("Cannot convert '{}' to FastNumber", s))?;
                Ok(Value::FastNumber(f64_val))
            }
            _ => Err(format!(
                "Cannot convert {} to FastNumber",
                args[0].to_display_string()
            )),
        }
    })));
    interpreter.define_global("FastNumber", fast_number_fn);

    // WeakRef() constructor creates weak references to Lists/Maps
    let weak_ref_fn = Value::NativeFunction(Arc::new(Box::new(|args| {
        if args.len() != 1 {
            return Err("WeakRef requires 1 argument (List or Map to reference)".to_string());
        }

        args[0].to_weak_ref()
    })));
    interpreter.define_global("WeakRef", weak_ref_fn);

    // Some() constructor - creates Option with a value
    let some_fn = Value::NativeFunction(Arc::new(Box::new(|args| {
        if args.len() != 1 {
            return Err("Some requires 1 argument (value to wrap)".to_string());
        }

        Ok(Value::Option(Box::new(Some(args[0].clone()))))
    })));
    interpreter.define_global("Some", some_fn);

    // None - singleton value representing absence
    let none_value = Value::Option(Box::new(None));
    interpreter.define_global("None", none_value);
}
