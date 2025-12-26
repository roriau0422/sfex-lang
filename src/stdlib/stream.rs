use crate::runtime::value::Value;
use bigdecimal::BigDecimal;
use std::collections::HashMap;
use std::sync::Arc;

struct StreamState {
    items: Vec<Value>,
    index: usize,
    exhausted: bool,
    generator: Option<Value>,
}

pub fn create_stream_module() -> Value {
    let mut methods = HashMap::new();

    methods.insert(
        "Create".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|_args| {
            Ok(create_stream_object(vec![], None))
        }))),
    );

    methods.insert(
        "FromList".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("Stream.FromList requires 1 argument (list)".to_string());
            }

            match &args[0] {
                Value::List(list) => {
                    let items = list.read().expect("lock poisoned").clone();
                    Ok(create_stream_object(items, None))
                }
                _ => Err("Argument must be a List".to_string()),
            }
        }))),
    );

    methods.insert(
        "Range".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 2 {
                return Err("Stream.Range requires 2 arguments (start, end)".to_string());
            }

            use bigdecimal::ToPrimitive;
            let start = match &args[0] {
                Value::Number(n) => n.to_i64().ok_or("Start must be an integer")?,
                Value::FastNumber(f) => *f as i64,
                _ => {
                    return Err("Start must be a number".to_string());
                }
            };

            let end = match &args[1] {
                Value::Number(n) => n.to_i64().ok_or("End must be an integer")?,
                Value::FastNumber(f) => *f as i64,
                _ => {
                    return Err("End must be a number".to_string());
                }
            };

            let current = Arc::new(std::sync::RwLock::new(start));
            let current_clone = current.clone();

            let generator = Value::NativeFunction(Arc::new(Box::new(move |_args| {
                let mut curr = current_clone.write().expect("lock poisoned");
                if *curr > end {
                    Ok(Value::Option(Box::new(None)))
                } else {
                    let val = *curr;
                    *curr += 1;
                    Ok(Value::Option(Box::new(Some(Value::Number(
                        BigDecimal::from(val),
                    )))))
                }
            })));

            Ok(create_stream_object(vec![], Some(generator)))
        }))),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}

pub fn create_stream_object(items: Vec<Value>, generator: Option<Value>) -> Value {
    let state = Arc::new(std::sync::RwLock::new(StreamState {
        items,
        index: 0,
        exhausted: false,
        generator,
    }));

    let mut stream_map = HashMap::new();

    let state_next = state.clone();
    stream_map.insert(
        "Next".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            let mut s = state_next.write().expect("lock poisoned");

            if s.index < s.items.len() {
                let item = s.items[s.index].clone();
                s.index += 1;
                return Ok(Value::Option(Box::new(Some(item))));
            }

            if let Some(ref gen_fn) = s.generator {
                if !s.exhausted {
                    match gen_fn {
                        Value::NativeFunction(f) => match f(vec![]) {
                            Ok(Value::Option(opt)) => {
                                if opt.as_ref().is_none() {
                                    s.exhausted = true;
                                }
                                Ok(Value::Option(opt))
                            }
                            Ok(value) => Ok(Value::Option(Box::new(Some(value)))),
                            Err(e) => {
                                s.exhausted = true;
                                Err(e)
                            }
                        },
                        _ => {
                            s.exhausted = true;
                            Err("Generator must be a function".to_string())
                        }
                    }
                } else {
                    Ok(Value::Option(Box::new(None)))
                }
            } else {
                s.exhausted = true;
                Ok(Value::Option(Box::new(None)))
            }
        }))),
    );

    let state_has = state.clone();
    stream_map.insert(
        "HasMore".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            let s = state_has.read().expect("lock poisoned");
            let has_buffered = s.index < s.items.len();
            let has_generator = !s.exhausted && s.generator.is_some();
            Ok(Value::Boolean(has_buffered || has_generator))
        }))),
    );

    let state_list = state.clone();
    stream_map.insert(
        "ToList".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            let mut result = Vec::new();

            let next_fn = {
                let s = state_list.read().expect("lock poisoned");
                if let Some(generator_fn) = &s.generator {
                    Some(generator_fn.clone())
                } else {
                    None
                }
            };

            loop {
                let buffered_item = {
                    let mut s = state_list.write().expect("lock poisoned");
                    if s.index < s.items.len() {
                        let item = s.items[s.index].clone();
                        s.index += 1;
                        Some(item)
                    } else {
                        None
                    }
                };

                if let Some(item) = buffered_item {
                    result.push(item);
                    continue;
                }

                if let Some(ref generator_fn) = next_fn {
                    let mut s = state_list.write().expect("lock poisoned");
                    if s.exhausted {
                        break;
                    }

                    match generator_fn {
                        Value::NativeFunction(f) => match f(vec![]) {
                            Ok(Value::Option(opt)) => {
                                if let Some(value) = opt.as_ref() {
                                    result.push(value.clone());
                                } else {
                                    s.exhausted = true;
                                    break;
                                }
                            }
                            Ok(value) => {
                                result.push(value);
                            }
                            Err(_) => {
                                s.exhausted = true;
                                break;
                            }
                        },
                        _ => {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }

            Ok(Value::List(Arc::new(std::sync::RwLock::new(result))))
        }))),
    );

    let state_close = state.clone();
    stream_map.insert(
        "Close".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            let mut s = state_close.write().expect("lock poisoned");
            s.exhausted = true;
            s.items.clear();
            s.generator = None;
            Ok(Value::Boolean(true))
        }))),
    );

    let state_gen = state.clone();
    stream_map.insert(
        "SetGenerator".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("SetGenerator requires 1 argument (function)".to_string());
            }

            match &args[0] {
                Value::NativeFunction(_) => {
                    let mut s = state_gen.write().expect("lock poisoned");
                    s.generator = Some(args[0].clone());
                    s.exhausted = false;
                    Ok(Value::Boolean(true))
                }
                _ => Err("Argument must be a function".to_string()),
            }
        }))),
    );

    let state_reset = state.clone();
    stream_map.insert(
        "Reset".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            let mut s = state_reset.write().expect("lock poisoned");
            s.index = 0;
            s.exhausted = false;
            Ok(Value::Boolean(true))
        }))),
    );

    let stream_rc = Arc::new(std::sync::RwLock::new(stream_map));
    let stream_value = Value::Map(stream_rc.clone());

    let stream_for_map = stream_value.clone();
    stream_rc.write().expect("lock poisoned").insert(
        "Map".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Map requires 1 argument (function)".to_string());
            }
            let map_fn = args[0].clone();
            create_map_stream(stream_for_map.clone(), map_fn)
        }))),
    );

    let stream_for_filter = stream_value.clone();
    stream_rc.write().expect("lock poisoned").insert(
        "Filter".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Filter requires 1 argument (function)".to_string());
            }
            let filter_fn = args[0].clone();
            create_filter_stream(stream_for_filter.clone(), filter_fn)
        }))),
    );

    let stream_for_take = stream_value.clone();
    stream_rc.write().expect("lock poisoned").insert(
        "Take".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Take requires 1 argument (count)".to_string());
            }
            use bigdecimal::ToPrimitive;
            let count = match &args[0] {
                Value::Number(n) => n.to_usize().ok_or("Count must be a positive integer")?,
                Value::FastNumber(f) => *f as usize,
                _ => {
                    return Err("Count must be a number".to_string());
                }
            };
            create_take_stream(stream_for_take.clone(), count)
        }))),
    );

    let stream_for_skip = stream_value.clone();
    stream_rc.write().expect("lock poisoned").insert(
        "Skip".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Skip requires 1 argument (count)".to_string());
            }
            use bigdecimal::ToPrimitive;
            let count = match &args[0] {
                Value::Number(n) => n.to_usize().ok_or("Count must be a positive integer")?,
                Value::FastNumber(f) => *f as usize,
                _ => {
                    return Err("Count must be a number".to_string());
                }
            };
            create_skip_stream(stream_for_skip.clone(), count)
        }))),
    );

    stream_value
}

fn create_map_stream(parent_stream: Value, map_fn: Value) -> Result<Value, String> {
    let mut stream_map = HashMap::new();

    let parent_next = parent_stream.clone();
    let map_fn_next = map_fn.clone();
    stream_map.insert(
        "Next".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            if let Value::Map(map) = &parent_next {
                if let Some(next_method) = map.read().expect("lock poisoned").get("Next") {
                    if let Value::NativeFunction(f) = next_method {
                        match f(vec![]) {
                            Ok(Value::Option(opt)) => {
                                if let Some(item) = opt.as_ref() {
                                    if let Value::NativeFunction(map_f) = &map_fn_next {
                                        match map_f(vec![item.clone()]) {
                                            Ok(mapped_value) => {
                                                Ok(Value::Option(Box::new(Some(mapped_value))))
                                            }
                                            Err(e) => Err(e),
                                        }
                                    } else {
                                        Err("Map function must be a function".to_string())
                                    }
                                } else {
                                    Ok(Value::Option(Box::new(None)))
                                }
                            }
                            Ok(_) => Err("Parent stream Next() must return Option".to_string()),
                            Err(e) => Err(e),
                        }
                    } else {
                        Err("Parent stream Next must be a function".to_string())
                    }
                } else {
                    Err("Parent stream missing Next method".to_string())
                }
            } else {
                Err("Parent is not a stream".to_string())
            }
        }))),
    );

    let parent_has = parent_stream.clone();
    stream_map.insert(
        "HasMore".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            if let Value::Map(map) = &parent_has {
                if let Some(has_more_method) = map.read().expect("lock poisoned").get("HasMore") {
                    if let Value::NativeFunction(f) = has_more_method {
                        return f(vec![]);
                    }
                }
            }
            Ok(Value::Boolean(false))
        }))),
    );

    let parent_list = parent_stream.clone();
    let map_fn_list = map_fn.clone();
    stream_map.insert(
        "ToList".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            let mut result = Vec::new();

            loop {
                if let Value::Map(map) = &parent_list {
                    if let Some(next_method) = map.read().expect("lock poisoned").get("Next") {
                        if let Value::NativeFunction(f) = next_method {
                            match f(vec![]) {
                                Ok(Value::Option(opt)) => {
                                    if let Some(item) = opt.as_ref() {
                                        if let Value::NativeFunction(map_f) = &map_fn_list {
                                            match map_f(vec![item.clone()]) {
                                                Ok(mapped) => result.push(mapped),
                                                Err(_) => {
                                                    break;
                                                }
                                            }
                                        }
                                    } else {
                                        break;
                                    }
                                }
                                _ => {
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            Ok(Value::List(Arc::new(std::sync::RwLock::new(result))))
        }))),
    );

    add_transform_methods(&mut stream_map, parent_stream.clone());

    Ok(Value::Map(Arc::new(std::sync::RwLock::new(stream_map))))
}

fn create_filter_stream(parent_stream: Value, filter_fn: Value) -> Result<Value, String> {
    let mut stream_map = HashMap::new();

    let parent_next = parent_stream.clone();
    let filter_fn_next = filter_fn.clone();
    stream_map.insert(
        "Next".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            loop {
                if let Value::Map(map) = &parent_next {
                    if let Some(next_method) = map.read().expect("lock poisoned").get("Next") {
                        if let Value::NativeFunction(f) = next_method {
                            match f(vec![]) {
                                Ok(Value::Option(opt)) => {
                                    if let Some(item) = opt.as_ref() {
                                        if let Value::NativeFunction(filter_f) = &filter_fn_next {
                                            match filter_f(vec![item.clone()]) {
                                                Ok(Value::Boolean(true)) => {
                                                    return Ok(Value::Option(Box::new(Some(
                                                        item.clone(),
                                                    ))));
                                                }
                                                Ok(Value::Boolean(false)) => {
                                                    continue;
                                                }
                                                Ok(_) => {
                                                    return Err(
                                                        "Filter function must return Boolean"
                                                            .to_string(),
                                                    );
                                                }
                                                Err(e) => {
                                                    return Err(e);
                                                }
                                            }
                                        } else {
                                            return Err("Filter must be a function".to_string());
                                        }
                                    } else {
                                        return Ok(Value::Option(Box::new(None)));
                                    }
                                }
                                Ok(_) => {
                                    return Err(
                                        "Parent stream Next() must return Option".to_string()
                                    );
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                            }
                        } else {
                            return Err("Parent stream Next must be a function".to_string());
                        }
                    } else {
                        return Err("Parent stream missing Next method".to_string());
                    }
                } else {
                    return Err("Parent is not a stream".to_string());
                }
            }
        }))),
    );

    let parent_has = parent_stream.clone();
    stream_map.insert(
        "HasMore".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            if let Value::Map(map) = &parent_has {
                if let Some(has_more_method) = map.read().expect("lock poisoned").get("HasMore") {
                    if let Value::NativeFunction(f) = has_more_method {
                        return f(vec![]);
                    }
                }
            }
            Ok(Value::Boolean(false))
        }))),
    );

    let stream_rc = Arc::new(std::sync::RwLock::new(stream_map));
    let stream_value = Value::Map(stream_rc.clone());

    let stream_for_list = stream_value.clone();
    stream_rc.write().expect("lock poisoned").insert(
        "ToList".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            let mut result = Vec::new();

            loop {
                if let Value::Map(map) = &stream_for_list {
                    if let Some(next_method) = map.read().expect("lock poisoned").get("Next") {
                        if let Value::NativeFunction(f) = next_method {
                            match f(vec![]) {
                                Ok(Value::Option(opt)) => {
                                    if let Some(item) = opt.as_ref() {
                                        result.push(item.clone());
                                    } else {
                                        break;
                                    }
                                }
                                _ => {
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            Ok(Value::List(Arc::new(std::sync::RwLock::new(result))))
        }))),
    );

    add_close_method(
        &mut *stream_rc.write().expect("lock poisoned"),
        parent_stream.clone(),
    );
    add_transform_methods(
        &mut *stream_rc.write().expect("lock poisoned"),
        parent_stream.clone(),
    );

    Ok(stream_value)
}

fn create_take_stream(parent_stream: Value, count: usize) -> Result<Value, String> {
    let taken = Arc::new(std::sync::RwLock::new(0usize));
    let mut stream_map = HashMap::new();

    let parent_next = parent_stream.clone();
    let taken_next = taken.clone();
    stream_map.insert(
        "Next".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            let mut taken_count = taken_next.write().expect("lock poisoned");

            if *taken_count >= count {
                return Ok(Value::Option(Box::new(None)));
            }

            if let Value::Map(map) = &parent_next {
                if let Some(next_method) = map.read().expect("lock poisoned").get("Next") {
                    if let Value::NativeFunction(f) = next_method {
                        match f(vec![]) {
                            Ok(Value::Option(opt)) => {
                                if opt.as_ref().is_some() {
                                    *taken_count += 1;
                                }
                                Ok(Value::Option(opt))
                            }
                            other => other,
                        }
                    } else {
                        Err("Parent stream Next must be a function".to_string())
                    }
                } else {
                    Err("Parent stream missing Next method".to_string())
                }
            } else {
                Err("Parent is not a stream".to_string())
            }
        }))),
    );

    let parent_has = parent_stream.clone();
    let taken_has = taken.clone();
    stream_map.insert(
        "HasMore".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            let taken_count = taken_has.read().expect("lock poisoned");

            if *taken_count >= count {
                return Ok(Value::Boolean(false));
            }

            if let Value::Map(map) = &parent_has {
                if let Some(has_more_method) = map.read().expect("lock poisoned").get("HasMore") {
                    if let Value::NativeFunction(f) = has_more_method {
                        return f(vec![]);
                    }
                }
            }
            Ok(Value::Boolean(false))
        }))),
    );

    let stream_rc = Arc::new(std::sync::RwLock::new(stream_map));
    let stream_value = Value::Map(stream_rc.clone());

    let stream_for_list = stream_value.clone();
    stream_rc.write().expect("lock poisoned").insert(
        "ToList".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            let mut result = Vec::new();

            loop {
                if let Value::Map(map) = &stream_for_list {
                    if let Some(next_method) = map.read().expect("lock poisoned").get("Next") {
                        if let Value::NativeFunction(f) = next_method {
                            match f(vec![]) {
                                Ok(Value::Option(opt)) => {
                                    if let Some(item) = opt.as_ref() {
                                        result.push(item.clone());
                                    } else {
                                        break;
                                    }
                                }
                                _ => {
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            Ok(Value::List(Arc::new(std::sync::RwLock::new(result))))
        }))),
    );

    add_close_method(
        &mut *stream_rc.write().expect("lock poisoned"),
        parent_stream.clone(),
    );
    add_transform_methods(
        &mut *stream_rc.write().expect("lock poisoned"),
        parent_stream.clone(),
    );

    Ok(stream_value)
}

fn create_skip_stream(parent_stream: Value, count: usize) -> Result<Value, String> {
    let skipped = Arc::new(std::sync::RwLock::new(0usize));
    let mut stream_map = HashMap::new();

    let parent_next = parent_stream.clone();
    let skipped_next = skipped.clone();
    stream_map.insert(
        "Next".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            let mut skipped_count = skipped_next.write().expect("lock poisoned");

            while *skipped_count < count {
                if let Value::Map(map) = &parent_next {
                    if let Some(next_method) = map.read().expect("lock poisoned").get("Next") {
                        if let Value::NativeFunction(f) = next_method {
                            match f(vec![]) {
                                Ok(Value::Option(opt)) => {
                                    if opt.as_ref().is_some() {
                                        *skipped_count += 1;
                                    } else {
                                        return Ok(Value::Option(Box::new(None)));
                                    }
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                                Ok(_) => {
                                    return Err(
                                        "Parent stream Next() must return Option".to_string()
                                    );
                                }
                            }
                        } else {
                            return Err("Parent stream Next must be a function".to_string());
                        }
                    } else {
                        return Err("Parent stream missing Next method".to_string());
                    }
                } else {
                    return Err("Parent is not a stream".to_string());
                }
            }

            if let Value::Map(map) = &parent_next {
                if let Some(next_method) = map.read().expect("lock poisoned").get("Next") {
                    if let Value::NativeFunction(f) = next_method {
                        return f(vec![]);
                    }
                }
            }
            Ok(Value::Option(Box::new(None)))
        }))),
    );

    let parent_has = parent_stream.clone();
    stream_map.insert(
        "HasMore".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            if let Value::Map(map) = &parent_has {
                if let Some(has_more_method) = map.read().expect("lock poisoned").get("HasMore") {
                    if let Value::NativeFunction(f) = has_more_method {
                        return f(vec![]);
                    }
                }
            }
            Ok(Value::Boolean(false))
        }))),
    );

    let stream_rc = Arc::new(std::sync::RwLock::new(stream_map));
    let stream_value = Value::Map(stream_rc.clone());

    let stream_for_list = stream_value.clone();
    stream_rc.write().expect("lock poisoned").insert(
        "ToList".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            let mut result = Vec::new();

            loop {
                if let Value::Map(map) = &stream_for_list {
                    if let Some(next_method) = map.read().expect("lock poisoned").get("Next") {
                        if let Value::NativeFunction(f) = next_method {
                            match f(vec![]) {
                                Ok(Value::Option(opt)) => {
                                    if let Some(item) = opt.as_ref() {
                                        result.push(item.clone());
                                    } else {
                                        break;
                                    }
                                }
                                _ => {
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            Ok(Value::List(Arc::new(std::sync::RwLock::new(result))))
        }))),
    );

    add_close_method(
        &mut *stream_rc.write().expect("lock poisoned"),
        parent_stream.clone(),
    );
    add_transform_methods(
        &mut *stream_rc.write().expect("lock poisoned"),
        parent_stream.clone(),
    );

    Ok(stream_value)
}

fn add_close_method(stream_map: &mut HashMap<String, Value>, parent_stream: Value) {
    let parent_close = parent_stream.clone();
    stream_map.insert(
        "Close".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            if let Value::Map(map) = &parent_close {
                if let Some(close_method) = map.read().expect("lock poisoned").get("Close") {
                    if let Value::NativeFunction(f) = close_method {
                        return f(vec![]);
                    }
                }
            }
            Ok(Value::Boolean(true))
        }))),
    );
}

fn add_transform_methods(stream_map: &mut HashMap<String, Value>, parent_stream: Value) {
    let stream_value = Value::Map(Arc::new(std::sync::RwLock::new(stream_map.clone())));

    let stream_for_map = stream_value.clone();
    stream_map.insert(
        "Map".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Map requires 1 argument (function)".to_string());
            }
            create_map_stream(stream_for_map.clone(), args[0].clone())
        }))),
    );

    let stream_for_filter = stream_value.clone();
    stream_map.insert(
        "Filter".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Filter requires 1 argument (function)".to_string());
            }
            create_filter_stream(stream_for_filter.clone(), args[0].clone())
        }))),
    );

    let stream_for_take = stream_value.clone();
    stream_map.insert(
        "Take".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Take requires 1 argument (count)".to_string());
            }
            use bigdecimal::ToPrimitive;
            let count = match &args[0] {
                Value::Number(n) => n.to_usize().ok_or("Count must be a positive integer")?,
                Value::FastNumber(f) => *f as usize,
                _ => {
                    return Err("Count must be a number".to_string());
                }
            };
            create_take_stream(stream_for_take.clone(), count)
        }))),
    );

    let stream_for_skip = stream_value.clone();
    stream_map.insert(
        "Skip".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Skip requires 1 argument (count)".to_string());
            }
            use bigdecimal::ToPrimitive;
            let count = match &args[0] {
                Value::Number(n) => n.to_usize().ok_or("Count must be a positive integer")?,
                Value::FastNumber(f) => *f as usize,
                _ => {
                    return Err("Count must be a number".to_string());
                }
            };
            create_skip_stream(stream_for_skip.clone(), count)
        }))),
    );

    let parent_reset = parent_stream;
    stream_map.insert(
        "Reset".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |_args| {
            if let Value::Map(map) = &parent_reset {
                if let Some(reset_method) = map.read().expect("lock poisoned").get("Reset") {
                    if let Value::NativeFunction(f) = reset_method {
                        return f(vec![]);
                    }
                }
            }
            Ok(Value::Boolean(false))
        }))),
    );
}
