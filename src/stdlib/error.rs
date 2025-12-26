use crate::runtime::value::{ErrorInfo, Value};
use std::collections::HashMap;
use std::sync::Arc;

pub fn create_error_module() -> Value {
    let mut categories = HashMap::new();

    // Error.System - System-level errors
    categories.insert("System".to_string(), create_system_category());

    // Error.Logic - Logic/programming errors
    categories.insert("Logic".to_string(), create_logic_category());

    // Error.Lookup - Lookup/not found errors
    categories.insert("Lookup".to_string(), create_lookup_category());

    // Error.Validation - Validation/constraint errors
    categories.insert("Validation".to_string(), create_validation_category());

    // Error.Panic - Panic/crash errors
    categories.insert("Panic".to_string(), create_panic_category());

    // Helper methods on Error itself
    // Error.IsError(value) - Check if a value is an error
    categories.insert(
        "IsError".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("Error.IsError requires 1 argument (value to check)".to_string());
            }
            Ok(Value::Boolean(matches!(args[0], Value::Error(_))))
        }))),
    );

    // Error.GetMessage(error) - Get error message
    categories.insert(
        "GetMessage".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("Error.GetMessage requires 1 argument (error value)".to_string());
            }
            match &args[0] {
                Value::Error(err) => Ok(Value::String(err.message.clone())),
                _ => Err("Argument must be an Error".to_string()),
            }
        }))),
    );

    // Error.GetCategory(error) - Get error category
    categories.insert(
        "GetCategory".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("Error.GetCategory requires 1 argument (error value)".to_string());
            }
            match &args[0] {
                Value::Error(err) => Ok(Value::String(err.category.clone())),
                _ => Err("Argument must be an Error".to_string()),
            }
        }))),
    );

    // Error.GetSubtype(error) - Get error subtype
    categories.insert(
        "GetSubtype".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("Error.GetSubtype requires 1 argument (error value)".to_string());
            }
            match &args[0] {
                Value::Error(err) => Ok(Value::String(err.subtype.clone())),
                _ => Err("Argument must be an Error".to_string()),
            }
        }))),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(categories)))
}

fn create_system_category() -> Value {
    let mut subtypes = HashMap::new();

    // Error.System.FileNotFound(message)
    subtypes.insert(
        "FileNotFound".to_string(),
        create_error_constructor("System", "FileNotFound"),
    );

    // Error.System.NetworkError(message)
    subtypes.insert(
        "NetworkError".to_string(),
        create_error_constructor("System", "NetworkError"),
    );

    // Error.System.PermissionDenied(message)
    subtypes.insert(
        "PermissionDenied".to_string(),
        create_error_constructor("System", "PermissionDenied"),
    );

    // Error.System.Timeout(message)
    subtypes.insert(
        "Timeout".to_string(),
        create_error_constructor("System", "Timeout"),
    );

    // Error.System.ResourceExhausted(message)
    subtypes.insert(
        "ResourceExhausted".to_string(),
        create_error_constructor("System", "ResourceExhausted"),
    );

    // Error.System.IOError(message)
    subtypes.insert(
        "IOError".to_string(),
        create_error_constructor("System", "IOError"),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(subtypes)))
}

fn create_logic_category() -> Value {
    let mut subtypes = HashMap::new();

    // Error.Logic.DivisionByZero(message)
    subtypes.insert(
        "DivisionByZero".to_string(),
        create_error_constructor("Logic", "DivisionByZero"),
    );

    // Error.Logic.InvalidOperation(message)
    subtypes.insert(
        "InvalidOperation".to_string(),
        create_error_constructor("Logic", "InvalidOperation"),
    );

    // Error.Logic.NullReference(message)
    subtypes.insert(
        "NullReference".to_string(),
        create_error_constructor("Logic", "NullReference"),
    );

    // Error.Logic.InvalidState(message)
    subtypes.insert(
        "InvalidState".to_string(),
        create_error_constructor("Logic", "InvalidState"),
    );

    // Error.Logic.NotImplemented(message)
    subtypes.insert(
        "NotImplemented".to_string(),
        create_error_constructor("Logic", "NotImplemented"),
    );

    // Error.Logic.Assertion(message)
    subtypes.insert(
        "Assertion".to_string(),
        create_error_constructor("Logic", "Assertion"),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(subtypes)))
}

fn create_lookup_category() -> Value {
    let mut subtypes = HashMap::new();

    // Error.Lookup.UndefinedVariable(message)
    subtypes.insert(
        "UndefinedVariable".to_string(),
        create_error_constructor("Lookup", "UndefinedVariable"),
    );

    // Error.Lookup.KeyNotFound(message)
    subtypes.insert(
        "KeyNotFound".to_string(),
        create_error_constructor("Lookup", "KeyNotFound"),
    );

    // Error.Lookup.IndexOutOfBounds(message)
    subtypes.insert(
        "IndexOutOfBounds".to_string(),
        create_error_constructor("Lookup", "IndexOutOfBounds"),
    );

    // Error.Lookup.MethodNotFound(message)
    subtypes.insert(
        "MethodNotFound".to_string(),
        create_error_constructor("Lookup", "MethodNotFound"),
    );

    // Error.Lookup.PropertyNotFound(message)
    subtypes.insert(
        "PropertyNotFound".to_string(),
        create_error_constructor("Lookup", "PropertyNotFound"),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(subtypes)))
}

fn create_validation_category() -> Value {
    let mut subtypes = HashMap::new();

    // Error.Validation.InvalidType(message)
    subtypes.insert(
        "InvalidType".to_string(),
        create_error_constructor("Validation", "InvalidType"),
    );

    // Error.Validation.OutOfBounds(message)
    subtypes.insert(
        "OutOfBounds".to_string(),
        create_error_constructor("Validation", "OutOfBounds"),
    );

    // Error.Validation.InvalidFormat(message)
    subtypes.insert(
        "InvalidFormat".to_string(),
        create_error_constructor("Validation", "InvalidFormat"),
    );

    // Error.Validation.ConstraintViolation(message)
    subtypes.insert(
        "ConstraintViolation".to_string(),
        create_error_constructor("Validation", "ConstraintViolation"),
    );

    // Error.Validation.ParseError(message)
    subtypes.insert(
        "ParseError".to_string(),
        create_error_constructor("Validation", "ParseError"),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(subtypes)))
}

fn create_panic_category() -> Value {
    let mut subtypes = HashMap::new();

    // Error.Panic.TaskPanicked(message)
    subtypes.insert(
        "TaskPanicked".to_string(),
        create_error_constructor("Panic", "TaskPanicked"),
    );

    // Error.Panic.RuntimeCrash(message)
    subtypes.insert(
        "RuntimeCrash".to_string(),
        create_error_constructor("Panic", "RuntimeCrash"),
    );

    // Error.Panic.Aborted(message)
    subtypes.insert(
        "Aborted".to_string(),
        create_error_constructor("Panic", "Aborted"),
    );

    // Error.Panic.StackOverflow(message)
    subtypes.insert(
        "StackOverflow".to_string(),
        create_error_constructor("Panic", "StackOverflow"),
    );

    // Error.Panic.OutOfMemory(message)
    subtypes.insert(
        "OutOfMemory".to_string(),
        create_error_constructor("Panic", "OutOfMemory"),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(subtypes)))
}

fn create_error_constructor(category: &str, subtype: &str) -> Value {
    let category = category.to_string();
    let subtype = subtype.to_string();

    Value::NativeFunction(Arc::new(Box::new(move |args| {
        let message = if args.is_empty() {
            format!("{}.{}", category, subtype)
        } else {
            args[0].to_display_string()
        };

        Ok(Value::Error(Arc::new(ErrorInfo {
            category: category.clone(),
            subtype: subtype.clone(),
            message,
        })))
    })))
}
