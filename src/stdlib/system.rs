use crate::runtime::value::Value;
use std::collections::HashMap;
use std::sync::Arc;
use system::system_output;

pub fn create_system_module() -> Value {
    let mut methods = HashMap::new();
    methods.insert(
        // Dangerious
        "Execute".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("System.Execute requires 1 argument (command)".to_string());
            }

            let command_str = args[0].to_display_string();
            match system_output(&command_str) {
                Ok(output) => {
                    let mut result = HashMap::new();

                    // Exit code
                    let exit_code = output.status.code().unwrap_or(-1);
                    use bigdecimal::BigDecimal;
                    result.insert(
                        "ExitCode".to_string(),
                        Value::Number(BigDecimal::from(exit_code as i64)),
                    );

                    // Success
                    result.insert(
                        "Success".to_string(),
                        Value::Boolean(output.status.success()),
                    );

                    // Stdout
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    result.insert("Output".to_string(), Value::String(stdout));

                    // Stderr
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    result.insert("Error".to_string(), Value::String(stderr));

                    Ok(Value::Map(Arc::new(std::sync::RwLock::new(result))))
                }
                Err(e) => Err(format!("Failed to execute command: {}", e)),
            }
        }))),
    );

    methods.insert(
        "Run".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("System.Run requires 1 argument (script_path)".to_string());
            }

            let script_path = args[0].to_display_string();
            let command = format!("cargo run --quiet -- run {}", script_path);
            match system_output(&command) {
                Ok(output) => {
                    let mut result = HashMap::new();

                    let exit_code = output.status.code().unwrap_or(-1);
                    use bigdecimal::BigDecimal;
                    result.insert(
                        "ExitCode".to_string(),
                        Value::Number(BigDecimal::from(exit_code as i64)),
                    );
                    result.insert(
                        "Success".to_string(),
                        Value::Boolean(output.status.success()),
                    );

                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    result.insert("Output".to_string(), Value::String(stdout));

                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    result.insert("Error".to_string(), Value::String(stderr));

                    Ok(Value::Map(Arc::new(std::sync::RwLock::new(result))))
                }
                Err(e) => Err(format!("Failed to run script: {}", e)),
            }
        }))),
    );

    methods.insert(
        "Info".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if !args.is_empty() {
                return Err("System.Info takes no arguments".to_string());
            }

            let mut info = HashMap::new();

            // OS Type
            let os_type = if cfg!(target_os = "windows") {
                "Windows"
            } else if cfg!(target_os = "linux") {
                "Linux"
            } else if cfg!(target_os = "macos") {
                "macOS"
            } else {
                std::env::consts::OS
            };
            info.insert("OS".to_string(), Value::String(os_type.to_string()));

            // OS Family
            info.insert(
                "Family".to_string(),
                Value::String(std::env::consts::FAMILY.to_string()),
            );

            // Architecture
            info.insert(
                "Arch".to_string(),
                Value::String(std::env::consts::ARCH.to_string()),
            );

            // Hostname
            if let Ok(hostname) = hostname::get() {
                info.insert(
                    "Hostname".to_string(),
                    Value::String(hostname.to_string_lossy().to_string()),
                );
            } else {
                info.insert("Hostname".to_string(), Value::String("Unknown".to_string()));
            }

            // Number of CPUs
            use bigdecimal::BigDecimal;
            let cpu_count = num_cpus::get() as i64;
            info.insert(
                "CPUs".to_string(),
                Value::Number(BigDecimal::from(cpu_count)),
            );

            Ok(Value::Map(Arc::new(std::sync::RwLock::new(info))))
        }))),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}
