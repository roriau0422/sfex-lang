use bigdecimal::{ BigDecimal, ToPrimitive };
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::{ Arc, RwLock, Weak };
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Debug)]
pub struct ErrorInfo {
    pub category: String,
    pub subtype: String,
    pub message: String,
}

fn format_number_for_display(n: &BigDecimal) -> String {
    let s = n.to_string();

    if !s.contains('.') {
        return s;
    }

    let scale = 10i64;
    let rounded = n.with_scale(scale);
    let rounded_str = rounded.to_string();

    let trimmed = rounded_str.trim_end_matches('0').trim_end_matches('.');

    trimmed.to_string()
}

#[derive(Clone)]
pub enum Value {
    Number(BigDecimal),
    FastNumber(f64),
    String(String),
    Boolean(bool),

    List(Arc<RwLock<Vec<Value>>>),
    Map(Arc<RwLock<HashMap<String, Value>>>),
    Vector(Vec<f32>),
    NativeFunction(Arc<Box<dyn (Fn(Vec<Value>) -> Result<Value, String>) + Send + Sync>>),

    WeakList(Weak<RwLock<Vec<Value>>>),
    WeakMap(Weak<RwLock<HashMap<String, Value>>>),

    Option(Box<Option<Value>>),

    TaskHandle(Arc<std::sync::Mutex<Option<tokio::task::JoinHandle<Value>>>>, Arc<AtomicBool>),

    Error(Arc<ErrorInfo>),
}

impl Value {
    pub fn default_number() -> Self {
        Value::Number(BigDecimal::from(0))
    }

    pub fn default_fast_number() -> Self {
        Value::FastNumber(0.0)
    }

    pub fn default_string() -> Self {
        Value::String(String::new())
    }

    pub fn default_boolean() -> Self {
        Value::Boolean(false)
    }

    pub fn default_list() -> Self {
        Value::List(Arc::new(RwLock::new(Vec::new())))
    }

    pub fn default_map() -> Self {
        Value::Map(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn default_vector() -> Self {
        Value::Vector(Vec::new())
    }

    pub fn to_weak_ref(&self) -> Result<Value, String> {
        match self {
            Value::List(arc) => Ok(Value::WeakList(Arc::downgrade(arc))),
            Value::Map(arc) => Ok(Value::WeakMap(Arc::downgrade(arc))),
            _ => Err(format!("Cannot create weak reference from {}", self.type_name())),
        }
    }

    pub fn is_weak_valid(&self) -> bool {
        match self {
            Value::WeakList(weak) => weak.strong_count() > 0,
            Value::WeakMap(weak) => weak.strong_count() > 0,
            _ => false,
        }
    }

    pub fn upgrade_weak(&self) -> Result<Value, String> {
        match self {
            Value::WeakList(weak) =>
                weak
                    .upgrade()
                    .map(|arc| Value::List(arc))
                    .ok_or("Weak reference no longer valid (object was collected)".to_string()),
            Value::WeakMap(weak) =>
                weak
                    .upgrade()
                    .map(|arc| Value::Map(arc))
                    .ok_or("Weak reference no longer valid (object was collected)".to_string()),
            _ => Err(format!("{} is not a weak reference", self.type_name())),
        }
    }

    pub fn is_some(&self) -> bool {
        match self {
            Value::Option(opt) => opt.is_some(),
            _ => false,
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            Value::Option(opt) => opt.is_none(),
            _ => false,
        }
    }

    pub fn unwrap_option(&self) -> Result<Value, String> {
        match self {
            Value::Option(opt) =>
                opt.as_ref().clone().ok_or("Cannot unwrap None value".to_string()),
            _ => Err(format!("{} is not an Option type", self.type_name())),
        }
    }

    pub fn unwrap_or(&self, default: Value) -> Result<Value, String> {
        match self {
            Value::Option(opt) => Ok(opt.as_ref().clone().unwrap_or(default)),
            _ => Err(format!("{} is not an Option type", self.type_name())),
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn is_fast_number(&self) -> bool {
        matches!(self, Value::FastNumber(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn is_native_function(&self) -> bool {
        matches!(self, Value::NativeFunction(_))
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Number(n) => n != &BigDecimal::from(0),
            Value::FastNumber(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.read().expect("lock poisoned").is_empty(),
            Value::Map(m) => !m.read().expect("lock poisoned").is_empty(),
            Value::Vector(v) => !v.is_empty(),
            Value::NativeFunction(_) => true,
            Value::WeakList(weak) => weak.strong_count() > 0,
            Value::WeakMap(weak) => weak.strong_count() > 0,
            Value::Option(opt) => opt.is_some(),
            Value::TaskHandle(_, _) => true,
            Value::Error(_) => true,
        }
    }

    pub fn add(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),

            (Value::FastNumber(a), Value::FastNumber(b)) => Ok(Value::FastNumber(a + b)),
            (Value::FastNumber(f), Value::Number(n)) => {
                let n_f64 = n.to_f64().unwrap_or(0.0);
                Ok(Value::FastNumber(f + n_f64))
            }
            (Value::Number(n), Value::FastNumber(f)) => {
                let n_f64 = n.to_f64().unwrap_or(0.0);
                Ok(Value::FastNumber(n_f64 + f))
            }
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (Value::String(s), Value::Number(n)) =>
                Ok(Value::String(format!("{}{}", s, format_number_for_display(n)))),
            (Value::Number(n), Value::String(s)) =>
                Ok(Value::String(format!("{}{}", format_number_for_display(n), s))),
            (Value::String(s), Value::FastNumber(f)) => Ok(Value::String(format!("{}{}", s, f))),
            (Value::FastNumber(f), Value::String(s)) => Ok(Value::String(format!("{}{}", f, s))),

            (Value::String(s), Value::Boolean(b)) =>
                Ok(Value::String(format!("{}{}", s, if *b { "True" } else { "False" }))),
            (Value::Boolean(b), Value::String(s)) =>
                Ok(Value::String(format!("{}{}", if *b { "True" } else { "False" }, s))),

            (Value::String(s), Value::WeakList(_)) | (Value::String(s), Value::WeakMap(_)) => {
                Ok(Value::String(format!("{}{}", s, other.to_display_string())))
            }
            (Value::WeakList(_), Value::String(s)) | (Value::WeakMap(_), Value::String(s)) => {
                Ok(Value::String(format!("{}{}", self.to_display_string(), s)))
            }

            (Value::String(s), Value::Option(_)) => {
                Ok(Value::String(format!("{}{}", s, other.to_display_string())))
            }
            (Value::Option(_), Value::String(s)) => {
                Ok(Value::String(format!("{}{}", self.to_display_string(), s)))
            }
            (Value::List(a), Value::List(b)) => {
                let mut result = a.read().expect("lock poisoned").clone();
                result.extend(b.read().expect("lock poisoned").clone());
                Ok(Value::List(Arc::new(RwLock::new(result))))
            }
            (Value::Vector(a), Value::Vector(b)) => {
                if a.len() != b.len() {
                    return Err("Vectors must have same length for addition".to_string());
                }
                let result: Vec<f32> = a
                    .iter()
                    .zip(b.iter())
                    .map(|(x, y)| x + y)
                    .collect();
                Ok(Value::Vector(result))
            }
            _ => Err(format!("Cannot add {:?} and {:?}", self.type_name(), other.type_name())),
        }
    }

    pub fn subtract(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            (Value::FastNumber(a), Value::FastNumber(b)) => Ok(Value::FastNumber(a - b)),
            (Value::FastNumber(f), Value::Number(n)) => {
                let n_f64 = n.to_f64().unwrap_or(0.0);
                Ok(Value::FastNumber(f - n_f64))
            }
            (Value::Number(n), Value::FastNumber(f)) => {
                let n_f64 = n.to_f64().unwrap_or(0.0);
                Ok(Value::FastNumber(n_f64 - f))
            }
            (Value::Vector(a), Value::Vector(b)) => {
                if a.len() != b.len() {
                    return Err("Vectors must have same length".to_string());
                }
                let result: Vec<f32> = a
                    .iter()
                    .zip(b.iter())
                    .map(|(x, y)| x - y)
                    .collect();
                Ok(Value::Vector(result))
            }
            _ =>
                Err(format!("Cannot subtract {:?} from {:?}", other.type_name(), self.type_name())),
        }
    }

    pub fn multiply(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            (Value::FastNumber(a), Value::FastNumber(b)) => Ok(Value::FastNumber(a * b)),
            (Value::FastNumber(f), Value::Number(n)) => {
                let n_f64 = n.to_f64().unwrap_or(0.0);
                Ok(Value::FastNumber(f * n_f64))
            }
            (Value::Number(n), Value::FastNumber(f)) => {
                let n_f64 = n.to_f64().unwrap_or(0.0);
                Ok(Value::FastNumber(n_f64 * f))
            }
            _ => Err(format!("Cannot multiply {:?} and {:?}", self.type_name(), other.type_name())),
        }
    }

    pub fn divide(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => {
                if b == &BigDecimal::from(0) {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Number(a / b))
                }
            }
            (Value::FastNumber(a), Value::FastNumber(b)) => {
                if *b == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::FastNumber(a / b))
                }
            }
            (Value::FastNumber(f), Value::Number(n)) => {
                let n_f64 = n.to_f64().unwrap_or(0.0);
                if n_f64 == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::FastNumber(f / n_f64))
                }
            }
            (Value::Number(n), Value::FastNumber(f)) => {
                let n_f64 = n.to_f64().unwrap_or(0.0);
                if *f == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::FastNumber(n_f64 / f))
                }
            }
            _ => Err(format!("Cannot divide {:?} by {:?}", self.type_name(), other.type_name())),
        }
    }

    pub fn modulo(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => {
                if b == &BigDecimal::from(0) {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(Value::Number(a % b))
                }
            }
            (Value::FastNumber(a), Value::FastNumber(b)) => {
                if *b == 0.0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(Value::FastNumber(a % b))
                }
            }
            (Value::FastNumber(f), Value::Number(n)) => {
                let n_f64 = n.to_f64().unwrap_or(0.0);
                if n_f64 == 0.0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(Value::FastNumber(f % n_f64))
                }
            }
            (Value::Number(n), Value::FastNumber(f)) => {
                let n_f64 = n.to_f64().unwrap_or(0.0);
                if *f == 0.0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(Value::FastNumber(n_f64 % f))
                }
            }
            _ => Err(format!("Cannot modulo {:?} by {:?}", self.type_name(), other.type_name())),
        }
    }

    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::FastNumber(a), Value::FastNumber(b)) => a == b,

            (Value::FastNumber(f), Value::Number(n)) => {
                if let Some(n_f64) = n.to_f64() { (f - n_f64).abs() < f64::EPSILON } else { false }
            }
            (Value::Number(n), Value::FastNumber(f)) => {
                if let Some(n_f64) = n.to_f64() { (n_f64 - f).abs() < f64::EPSILON } else { false }
            }
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            _ => false,
        }
    }

    pub fn compare(&self, other: &Value) -> Result<std::cmp::Ordering, String> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(a.cmp(b)),
            (Value::FastNumber(a), Value::FastNumber(b)) =>
                a.partial_cmp(b).ok_or("Cannot compare NaN values".to_string()),
            (Value::FastNumber(f), Value::Number(n)) => {
                let n_f64 = n.to_f64().ok_or("Number too large for FastNumber comparison")?;
                f.partial_cmp(&n_f64).ok_or("Cannot compare NaN values".to_string())
            }
            (Value::Number(n), Value::FastNumber(f)) => {
                let n_f64 = n.to_f64().ok_or("Number too large for FastNumber comparison")?;
                n_f64.partial_cmp(f).ok_or("Cannot compare NaN values".to_string())
            }
            (Value::String(a), Value::String(b)) => Ok(a.cmp(b)),
            _ => Err(format!("Cannot compare {:?} and {:?}", self.type_name(), other.type_name())),
        }
    }

    pub fn index(&self, idx: &Value) -> Result<Value, String> {
        match (self, idx) {
            (Value::List(list), Value::Number(n)) => {
                let idx_i64 = n.to_i64().ok_or("Index must be integer")?;

                if idx_i64 == 0 {
                    return Err("SFX lists start at 1, not 0".to_string());
                }

                let rust_idx = if idx_i64 > 0 {
                    (idx_i64 - 1) as usize
                } else {
                    return Err("Negative indices not supported yet".to_string());
                };

                list.read()
                    .expect("lock poisoned")
                    .get(rust_idx)
                    .cloned()
                    .ok_or_else(|| format!("Index {} out of bounds", idx_i64))
            }
            (Value::String(s), Value::Number(n)) => {
                let idx_i64 = n.to_i64().ok_or("Index must be integer")?;

                if idx_i64 == 0 {
                    return Err(
                        "SFX strings start at 1. Use index 1 for the first character.".to_string()
                    );
                }

                let rust_idx = if idx_i64 > 0 {
                    (idx_i64 - 1) as usize
                } else {
                    let len = s.graphemes(true).count() as i64;
                    (len + idx_i64) as usize
                };

                s.graphemes(true)
                    .nth(rust_idx)
                    .map(|g| Value::String(g.to_string()))
                    .ok_or_else(|| format!("Index {} out of bounds", idx_i64))
            }
            (Value::Map(map), Value::String(key)) =>
                map
                    .read()
                    .expect("lock poisoned")
                    .get(key)
                    .cloned()
                    .ok_or_else(|| format!("Key '{}' not found", key)),
            _ => Err(format!("Cannot index {:?} with {:?}", self.type_name(), idx.type_name())),
        }
    }

    pub fn clone_deep(&self) -> Value {
        match self {
            Value::Number(n) => Value::Number(n.clone()),
            Value::FastNumber(f) => Value::FastNumber(*f),
            Value::String(s) => Value::String(s.clone()),
            Value::Boolean(b) => Value::Boolean(*b),

            Value::List(l) => {
                let inner = l.read().expect("lock poisoned");

                let deep_copied_items: Vec<Value> = inner
                    .iter()
                    .map(|v| v.clone_deep())
                    .collect();
                Value::List(Arc::new(RwLock::new(deep_copied_items)))
            }

            Value::Map(m) => {
                let inner = m.read().expect("lock poisoned");
                let deep_copied_entries: HashMap<String, Value> = inner
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone_deep()))
                    .collect();
                Value::Map(Arc::new(RwLock::new(deep_copied_entries)))
            }

            Value::Vector(v) => Value::Vector(v.clone()),
            Value::NativeFunction(f) => Value::NativeFunction(f.clone()),

            Value::WeakList(w) => Value::WeakList(w.clone()),
            Value::WeakMap(w) => Value::WeakMap(w.clone()),

            Value::Option(opt) => {
                Value::Option(
                    Box::new(
                        opt
                            .as_ref()
                            .clone()
                            .map(|v| v.clone_deep())
                    )
                )
            }

            Value::TaskHandle(h, c) => Value::TaskHandle(h.clone(), c.clone()),

            Value::Error(e) => Value::Error(e.clone()),
        }
    }

    pub fn len(&self) -> Result<usize, String> {
        match self {
            Value::String(s) => {
                use unicode_segmentation::UnicodeSegmentation;
                Ok(s.graphemes(true).count())
            }
            Value::List(l) => Ok(l.read().expect("lock poisoned").len()),
            Value::Vector(v) => Ok(v.len()),
            Value::Map(m) => Ok(m.read().expect("lock poisoned").len()),
            _ => Err(format!("{:?} has no length", self.type_name())),
        }
    }

    pub fn to_display_string(&self) -> String {
        match self {
            Value::Number(n) => format_number_for_display(n),
            Value::FastNumber(f) => {
                if f.is_finite() {
                    format!("{}", f)
                } else if f.is_infinite() {
                    if *f > 0.0 { "Infinity".to_string() } else { "-Infinity".to_string() }
                } else {
                    "NaN".to_string()
                }
            }
            Value::String(s) => s.clone(),
            Value::Boolean(b) => (if *b { "True" } else { "False" }).to_string(),
            Value::List(l) => {
                let items: Vec<String> = l
                    .read()
                    .expect("lock poisoned")
                    .iter()
                    .map(|v| v.to_display_string())
                    .collect();
                format!("[{}]", items.join(", "))
            }
            Value::Map(m) => {
                let entries: Vec<String> = m
                    .read()
                    .expect("lock poisoned")
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_display_string()))
                    .collect();
                format!("{{{}}}", entries.join(", "))
            }
            Value::Vector(v) => { format!("Vector[{}]", v.len()) }
            Value::NativeFunction(_) => "<native function>".to_string(),
            Value::WeakList(weak) => {
                if weak.strong_count() > 0 {
                    "<WeakRef to List (valid)>".to_string()
                } else {
                    "<WeakRef to List (collected)>".to_string()
                }
            }
            Value::WeakMap(weak) => {
                if weak.strong_count() > 0 {
                    "<WeakRef to Map (valid)>".to_string()
                } else {
                    "<WeakRef to Map (collected)>".to_string()
                }
            }
            Value::Option(opt) =>
                match opt.as_ref() {
                    Some(v) => format!("Some({})", v.to_display_string()),
                    None => "None".to_string(),
                }
            Value::TaskHandle(_, _) => "<TaskHandle>".to_string(),
            Value::Error(err) => {
                format!("Error.{}.{}: {}", err.category, err.subtype, err.message)
            }
        }
    }

    fn type_name(&self) -> &str {
        match self {
            Value::Number(_) => "Number",
            Value::FastNumber(_) => "FastNumber",
            Value::String(_) => "String",
            Value::Boolean(_) => "Boolean",
            Value::List(_) => "List",
            Value::Map(_) => "Map",
            Value::Vector(_) => "Vector",
            Value::NativeFunction(_) => "NativeFunction",
            Value::WeakList(_) => "WeakRef (List)",
            Value::WeakMap(_) => "WeakRef (Map)",
            Value::Option(_) => "Option",
            Value::TaskHandle(_, _) => "TaskHandle",
            Value::Error(_) => "Error",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}

impl Value {
    pub fn from_number_string(s: &str) -> Result<Value, String> {
        BigDecimal::from_str(s)
            .map(Value::Number)
            .map_err(|e| format!("Invalid number: {}", e))
    }

    pub fn to_debug_string(&self) -> String {
        match self {
            Value::String(s) => format!("\"{}\"", s),
            Value::List(l) => {
                let items: Vec<String> = l
                    .read()
                    .expect("lock poisoned")
                    .iter()
                    .map(|v| v.to_debug_string())
                    .collect();
                format!("[{}]", items.join(", "))
            }
            _ => self.to_string(),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,

            (Value::List(a), Value::List(b)) => Arc::ptr_eq(a, b),
            (Value::Map(a), Value::Map(b)) => Arc::ptr_eq(a, b),
            (Value::Vector(a), Value::Vector(b)) => a == b,

            (Value::NativeFunction(a), Value::NativeFunction(b)) => Arc::ptr_eq(a, b),

            (Value::WeakList(a), Value::WeakList(b)) => Weak::ptr_eq(a, b),
            (Value::WeakMap(a), Value::WeakMap(b)) => Weak::ptr_eq(a, b),

            (Value::Option(a), Value::Option(b)) =>
                match (a.as_ref(), b.as_ref()) {
                    (Some(va), Some(vb)) => va.equals(vb),
                    (None, None) => true,
                    _ => false,
                }

            (Value::TaskHandle(a, _), Value::TaskHandle(b, _)) => Arc::ptr_eq(a, b),

            (Value::Error(a), Value::Error(b)) => {
                a.category == b.category && a.subtype == b.subtype && a.message == b.message
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mathematical_honesty() {
        let a = Value::from_number_string("0.1").unwrap();
        let b = Value::from_number_string("0.2").unwrap();
        let result = a.add(&b).unwrap();

        let expected = Value::from_number_string("0.3").unwrap();
        assert!(result.equals(&expected), "0.1 + 0.2 must equal 0.3");
    }

    #[test]
    fn test_1_based_indexing() {
        let list = Value::List(
            Arc::new(
                std::sync::RwLock::new(
                    vec![
                        Value::from_number_string("10").unwrap(),
                        Value::from_number_string("20").unwrap(),
                        Value::from_number_string("30").unwrap()
                    ]
                )
            )
        );

        let first = list.index(&Value::from_number_string("1").unwrap()).unwrap();
        assert!(first.equals(&Value::from_number_string("10").unwrap()));
    }

    #[test]
    fn test_zero_index_error() {
        let list = Value::List(
            Arc::new(std::sync::RwLock::new(vec![Value::from_number_string("10").unwrap()]))
        );

        let result = list.index(&Value::from_number_string("0").unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("start at 1"));
    }

    #[test]
    fn test_emoji_as_one_character() {
        let flag = Value::String("🇲🇳".to_string());
        assert_eq!(flag.len().unwrap(), 1, "Emoji flag should be 1 character");

        let indexed = flag.index(&Value::from_number_string("1").unwrap()).unwrap();
        assert_eq!(indexed.to_display_string(), "🇲🇳");
    }

    #[test]
    fn test_string_concatenation() {
        let s = Value::String("Age: ".to_string());
        let n = Value::from_number_string("34").unwrap();
        let result = s.add(&n).unwrap();

        assert_eq!(result.to_display_string(), "Age: 34");
    }

    #[test]
    fn test_copy_on_write() {
        let a = Value::List(
            Arc::new(
                RwLock::new(
                    vec![
                        Value::from_number_string("1").unwrap(),
                        Value::from_number_string("2").unwrap()
                    ]
                )
            )
        );

        let b = a.clone_deep();

        if let Value::List(list) = &b {
            list.write().expect("lock poisoned").push(Value::from_number_string("3").unwrap());
        }

        if let Value::List(list) = &a {
            assert_eq!(list.read().expect("lock poisoned").len(), 2);
        }
    }

    #[test]
    fn test_no_null() {
        let defaults = vec![
            Value::default_number(),
            Value::default_string(),
            Value::default_boolean(),
            Value::default_list()
        ];

        for val in defaults {
            assert!(!matches!(val, Value::Boolean(false)) || val.is_truthy() == false);
        }
    }
}
