use crate::runtime::value::Value;
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;

pub fn create_system_module() -> Value {
    let mut methods = HashMap::new();

    // System.Execute(command) - Execute shell command and return output
    methods.insert(
        "Execute".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("System.Execute requires 1 argument (command)".to_string());
            }

            let command_str = args[0].to_display_string();

            // Determine shell based on OS
            let (shell, flag) = if cfg!(target_os = "windows") {
                ("cmd", "/C")
            } else {
                ("sh", "-c")
            };

            match Command::new(shell).arg(flag).arg(&command_str).output() {
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

    // System.Run(script) - Run a SFX script and return result
    methods.insert(
        "Run".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("System.Run requires 1 argument (script_path)".to_string());
            }

            let script_path = args[0].to_display_string();

            // Execute: cargo run --quiet -- run <script_path>
            let command = format!("cargo run --quiet -- run {}", script_path);

            let (shell, flag) = if cfg!(target_os = "windows") {
                ("cmd", "/C")
            } else {
                ("sh", "-c")
            };

            match Command::new(shell).arg(flag).arg(&command).output() {
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

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}
