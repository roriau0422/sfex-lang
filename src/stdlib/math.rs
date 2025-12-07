use crate::runtime::value::Value;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;

pub fn create_math_module() -> Value {
    let mut methods = HashMap::new();

    methods.insert(
        "Random".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if !args.is_empty() {
                        return Err("Math.Random requires 0 arguments".to_string());
                    }

                    let mut rng = rand::rng();
                    let random_value: f64 = rng.random_range(0.0..1.0);

                    Ok(Value::FastNumber(random_value))
                })
            )
        )
    );

    methods.insert(
        "Round".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("Math.Round requires 1 argument (number)".to_string());
                    }

                    let number = match &args[0] {
                        Value::Number(n) =>
                            n
                                .to_string()
                                .parse::<f64>()
                                .map_err(|_| "Invalid number".to_string())?,
                        Value::FastNumber(f) => *f,
                        _ => {
                            return Err("Argument must be a number".to_string());
                        }
                    };

                    let rounded = number.round();

                    use bigdecimal::BigDecimal;
                    Ok(Value::Number(BigDecimal::from(rounded as i64)))
                })
            )
        )
    );

    methods.insert(
        "Floor".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("Math.Floor requires 1 argument (number)".to_string());
                    }

                    let number = match &args[0] {
                        Value::Number(n) =>
                            n
                                .to_string()
                                .parse::<f64>()
                                .map_err(|_| "Invalid number".to_string())?,
                        Value::FastNumber(f) => *f,
                        _ => {
                            return Err("Argument must be a number".to_string());
                        }
                    };

                    let floored = number.floor();

                    use bigdecimal::BigDecimal;
                    Ok(Value::Number(BigDecimal::from(floored as i64)))
                })
            )
        )
    );

    methods.insert(
        "Ceil".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("Math.Ceil requires 1 argument (number)".to_string());
                    }

                    let number = match &args[0] {
                        Value::Number(n) =>
                            n
                                .to_string()
                                .parse::<f64>()
                                .map_err(|_| "Invalid number".to_string())?,
                        Value::FastNumber(f) => *f,
                        _ => {
                            return Err("Argument must be a number".to_string());
                        }
                    };

                    let ceiled = number.ceil();

                    use bigdecimal::BigDecimal;
                    Ok(Value::Number(BigDecimal::from(ceiled as i64)))
                })
            )
        )
    );

    methods.insert(
        "Abs".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("Math.Abs requires 1 argument (number)".to_string());
                    }

                    match &args[0] {
                        Value::Number(n) => {
                            let abs_n = n.abs();
                            Ok(Value::Number(abs_n))
                        }
                        Value::FastNumber(f) => Ok(Value::FastNumber(f.abs())),
                        _ => Err("Argument must be a number".to_string()),
                    }
                })
            )
        )
    );

    methods.insert(
        "Min".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 2 {
                        return Err("Math.Min requires 2 arguments".to_string());
                    }

                    let a = match &args[0] {
                        Value::Number(n) =>
                            n
                                .to_string()
                                .parse::<f64>()
                                .map_err(|_| "Invalid number".to_string())?,
                        Value::FastNumber(f) => *f,
                        _ => {
                            return Err("Arguments must be numbers".to_string());
                        }
                    };

                    let b = match &args[1] {
                        Value::Number(n) =>
                            n
                                .to_string()
                                .parse::<f64>()
                                .map_err(|_| "Invalid number".to_string())?,
                        Value::FastNumber(f) => *f,
                        _ => {
                            return Err("Arguments must be numbers".to_string());
                        }
                    };

                    let min_val = a.min(b);
                    Ok(Value::FastNumber(min_val))
                })
            )
        )
    );

    methods.insert(
        "Max".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 2 {
                        return Err("Math.Max requires 2 arguments".to_string());
                    }

                    let a = match &args[0] {
                        Value::Number(n) =>
                            n
                                .to_string()
                                .parse::<f64>()
                                .map_err(|_| "Invalid number".to_string())?,
                        Value::FastNumber(f) => *f,
                        _ => {
                            return Err("Arguments must be numbers".to_string());
                        }
                    };

                    let b = match &args[1] {
                        Value::Number(n) =>
                            n
                                .to_string()
                                .parse::<f64>()
                                .map_err(|_| "Invalid number".to_string())?,
                        Value::FastNumber(f) => *f,
                        _ => {
                            return Err("Arguments must be numbers".to_string());
                        }
                    };

                    let max_val = a.max(b);
                    Ok(Value::FastNumber(max_val))
                })
            )
        )
    );

    methods.insert(
        "Pow".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 2 {
                        return Err("Math.Pow requires 2 arguments (base, exponent)".to_string());
                    }

                    let base = match &args[0] {
                        Value::Number(n) =>
                            n
                                .to_string()
                                .parse::<f64>()
                                .map_err(|_| "Invalid number".to_string())?,
                        Value::FastNumber(f) => *f,
                        _ => {
                            return Err("Arguments must be numbers".to_string());
                        }
                    };

                    let exponent = match &args[1] {
                        Value::Number(n) =>
                            n
                                .to_string()
                                .parse::<f64>()
                                .map_err(|_| "Invalid number".to_string())?,
                        Value::FastNumber(f) => *f,
                        _ => {
                            return Err("Arguments must be numbers".to_string());
                        }
                    };

                    let result = base.powf(exponent);
                    Ok(Value::FastNumber(result))
                })
            )
        )
    );

    methods.insert(
        "Sqrt".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("Math.Sqrt requires 1 argument (number)".to_string());
                    }

                    let number = match &args[0] {
                        Value::Number(n) =>
                            n
                                .to_string()
                                .parse::<f64>()
                                .map_err(|_| "Invalid number".to_string())?,
                        Value::FastNumber(f) => *f,
                        _ => {
                            return Err("Argument must be a number".to_string());
                        }
                    };

                    if number < 0.0 {
                        return Err("Cannot take square root of negative number".to_string());
                    }

                    let result = number.sqrt();
                    Ok(Value::FastNumber(result))
                })
            )
        )
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}
