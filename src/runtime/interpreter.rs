use super::value::{ErrorInfo, Value};
use crate::compiler::ast::*;
use crate::stdlib;
use bigdecimal::FromPrimitive;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub enum RuntimeError {
    UndefinedVariable(String),
    UndefinedConcept(String),
    UndefinedMethod(String),
    TypeError(String),
    IndexError(String),
    Custom(String),
}

#[derive(Clone)]
pub enum ExecutionResult {
    Done,
    Return(Value),
    Break,
    Continue,
}

#[derive(Clone)]
pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return true;
            }
        }
        false
    }

    pub fn clone_deep(&self) -> Self {
        let deep_scopes = self
            .scopes
            .iter()
            .map(|scope| {
                scope
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone_deep()))
                    .collect()
            })
            .collect();

        Self {
            scopes: deep_scopes,
        }
    }
}

pub struct Interpreter {
    pub env: Environment,
    concepts: HashMap<String, Concept>,
    situations: HashMap<String, Situation>,
    pub active_situations: Vec<String>,
    current_line: usize,
    trace: bool,
    pub runtime: std::sync::Arc<tokio::runtime::Runtime>,
    proceed_stack: Vec<(Vec<Method>, usize, Value, Vec<(String, Value)>)>,
    observer_depth: usize,

    profiler: crate::jit::Profiler,
    jit_compiler: crate::jit::JitCompiler,
}

impl Interpreter {
    pub fn new() -> Self {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

        let mut interpreter = Self {
            env: Environment::new(),
            concepts: HashMap::new(),
            situations: HashMap::new(),
            active_situations: Vec::new(),
            current_line: 0,
            trace: false,
            runtime: std::sync::Arc::new(runtime),
            proceed_stack: Vec::new(),
            observer_depth: 0,
            profiler: crate::jit::Profiler::new(),
            jit_compiler: crate::jit::JitCompiler::new(),
        };

        stdlib::register_stdlib(&mut interpreter);

        interpreter
    }

    fn new_with_shared_runtime(runtime: std::sync::Arc<tokio::runtime::Runtime>) -> Self {
        let mut interpreter = Self {
            env: Environment::new(),
            concepts: HashMap::new(),
            situations: HashMap::new(),
            active_situations: Vec::new(),
            current_line: 0,
            trace: false,
            runtime,
            proceed_stack: Vec::new(),
            observer_depth: 0,
            profiler: crate::jit::Profiler::new(),
            jit_compiler: crate::jit::JitCompiler::new(),
        };

        stdlib::register_stdlib(&mut interpreter);

        interpreter
    }

    pub fn define_global(&mut self, name: &str, value: Value) {
        self.env.define(name.to_string(), value);
    }

    pub fn enable_trace(&mut self) {
        self.trace = true;
    }

    pub fn run(&mut self, program: Program) -> Result<(), RuntimeError> {
        for concept in program.concepts {
            self.concepts.insert(concept.name.clone(), concept);
        }
        for situation in program.situations {
            self.situations.insert(situation.name.clone(), situation);
        }

        self.execute_story(&program.story)?;
        Ok(())
    }

    fn execute_story(&mut self, story: &Story) -> Result<(), RuntimeError> {
        match self.execute_block_no_scope(&story.body)? {
            ExecutionResult::Done
            | ExecutionResult::Return(_)
            | ExecutionResult::Break
            | ExecutionResult::Continue => Ok(()),
        }
    }

    fn load_module(&mut self, path: &str) -> Result<(), RuntimeError> {
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let resolved = crate::project::resolve_module_path(path, &cwd)
            .unwrap_or_else(|| std::path::PathBuf::from(path));

        let source = std::fs::read_to_string(&resolved).map_err(|e| {
            RuntimeError::Custom(format!(
                "Failed to read module '{}': {}",
                resolved.display(),
                e
            ))
        })?;

        let mut lexer = crate::compiler::lexer::Lexer::new(&source);
        let tokens = lexer.tokenize().map_err(|e| {
            RuntimeError::Custom(format!("Lexer error in module '{}': {}", path, e))
        })?;

        let mut parser = crate::compiler::parser::Parser::new(tokens);
        let program = parser.parse().map_err(|e| {
            RuntimeError::Custom(format!("Parser error in module '{}': {}", path, e))
        })?;

        for concept in program.concepts {
            if self.concepts.contains_key(&concept.name) {}
            self.concepts.insert(concept.name.clone(), concept);
        }

        for situation in program.situations {
            self.situations.insert(situation.name.clone(), situation);
        }

        self.execute_story(&program.story)?;

        Ok(())
    }

    fn execute_statement(&mut self, stmt: &Statement) -> Result<ExecutionResult, RuntimeError> {
        match stmt {
            Statement::Use { module_path, .. } => {
                self.load_module(module_path)?;
                Ok(ExecutionResult::Done)
            }
            Statement::Create {
                concept_name,
                instance_name,
                initial_fields,
                ..
            } => {
                let concept = self
                    .concepts
                    .get(concept_name)
                    .ok_or_else(|| RuntimeError::UndefinedConcept(concept_name.clone()))?
                    .clone();

                let mut instance_data = HashMap::new();
                instance_data.insert("_concept".to_string(), Value::String(concept_name.clone()));

                for field in &concept.fields {
                    instance_data.insert(field.clone(), Value::default_number());
                }

                let instance =
                    Value::Map(std::sync::Arc::new(std::sync::RwLock::new(instance_data)));

                // Store the instance (use shallow clone so we can modify it afterwards)
                if !self.env.assign(instance_name, instance.clone()) {
                    self.env.define(instance_name.clone(), instance.clone());
                }

                // Set initial field values if provided (modifies the shared Arc)
                if let Value::Map(m) = &instance {
                    for (field_name, field_expr) in initial_fields {
                        let field_value = self.evaluate_expression(field_expr)?;
                        m.write()
                            .expect("lock poisoned")
                            .insert(field_name.clone(), field_value);
                    }
                }

                Ok(ExecutionResult::Done)
            }

            Statement::Assignment { target, value, .. } => {
                let val = self.evaluate_expression(value)?;
                if !self.env.assign(target, val.clone_deep()) {
                    self.env.define(target.clone(), val.clone_deep());
                }
                Ok(ExecutionResult::Done)
            }

            Statement::Set { target, value, .. } => {
                let val = self.evaluate_expression(value)?;
                match target {
                    Expression::Identifier(name) => {
                        if self.env.assign(name, val.clone()) {
                        } else {
                            let this_val = self.env.get("This");
                            let mut updated = false;

                            if let Some(Value::Map(m)) = this_val {
                                if m.read().expect("lock poisoned").contains_key(name) {
                                    m.write().expect("lock poisoned").insert(name.clone(), val);
                                    updated = true;
                                }
                            }

                            if !updated {
                                return Err(RuntimeError::UndefinedVariable(name.clone()));
                            }
                        }
                    }
                    Expression::MemberAccess { object, member } => {
                        let obj_val = self.evaluate_expression(object)?;
                        if let Value::Map(m) = obj_val.clone() {
                            m.write()
                                .expect("lock poisoned")
                                .insert(member.clone(), val);

                            const MAX_OBSERVER_DEPTH: usize = 10;
                            if self.observer_depth < MAX_OBSERVER_DEPTH {
                                let concept_name = {
                                    let map_read = m.read().expect("lock poisoned");
                                    map_read.get("_concept").and_then(|v| {
                                        if let Value::String(s) = v {
                                            Some(s.clone())
                                        } else {
                                            None
                                        }
                                    })
                                };

                                if let Some(c_name) = concept_name {
                                    if let Some(concept) = self.concepts.get(&c_name).cloned() {
                                        if let Some(observer_body) =
                                            concept.when_observers.get(member)
                                        {
                                            let observer_code = observer_body.clone();

                                            self.observer_depth += 1;
                                            self.env.push_scope();
                                            self.env.define("This".to_string(), obj_val.clone());

                                            let _ = self.execute_block_no_scope(&observer_code)?;

                                            self.env.pop_scope();
                                            self.observer_depth -= 1;
                                        }
                                    }
                                }
                            } else {
                                return Err(
                                    RuntimeError::Custom(
                                        "When observer recursion limit reached (infinite loop detected)".to_string()
                                    )
                                );
                            }
                        } else {
                            return Err(RuntimeError::TypeError(
                                "Target is not an object".to_string(),
                            ));
                        }
                    }
                    _ => {
                        return Err(RuntimeError::Custom("Invalid set target".to_string()));
                    }
                }
                Ok(ExecutionResult::Done)
            }

            Statement::Print { value, .. } => {
                let val = self.evaluate_expression(value)?;
                println!("{}", val);
                Ok(ExecutionResult::Done)
            }

            Statement::SwitchOn { situation, .. } => {
                if !self.active_situations.contains(situation) {
                    self.active_situations.push(situation.clone());
                }
                Ok(ExecutionResult::Done)
            }

            Statement::SwitchOff { situation, .. } => {
                self.active_situations.retain(|s| s != situation);
                Ok(ExecutionResult::Done)
            }

            Statement::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let cond = self.evaluate_expression(condition)?;
                if cond.is_truthy() {
                    return self.execute_block(then_body);
                } else if let Some(else_stmts) = else_body {
                    return self.execute_block(else_stmts);
                }
                Ok(ExecutionResult::Done)
            }

            Statement::When {
                value,
                cases,
                otherwise,
                ..
            } => {
                let target_value = self.evaluate_expression(value)?;

                for (match_expr, body) in cases {
                    let match_value = self.evaluate_expression(match_expr)?;

                    if target_value.equals(&match_value) {
                        return self.execute_block(body);
                    }
                }

                if let Some(otherwise_body) = otherwise {
                    return self.execute_block(otherwise_body);
                }

                Ok(ExecutionResult::Done)
            }

            Statement::TryCatch {
                try_body,
                catch_var,
                catch_body,
                always_body,
                ..
            } => {
                let try_result = self.execute_block(try_body);

                let final_result = match try_result {
                    Err(err) => {
                        if let Some(catch) = catch_body {
                            if let Some(var_name) = catch_var {
                                let (error_type, error_message) = match &err {
                                    RuntimeError::UndefinedVariable(s) => {
                                        ("UndefinedVariable", s.clone())
                                    }
                                    RuntimeError::UndefinedConcept(s) => {
                                        ("UndefinedConcept", s.clone())
                                    }
                                    RuntimeError::UndefinedMethod(s) => {
                                        ("UndefinedMethod", s.clone())
                                    }
                                    RuntimeError::TypeError(s) => ("TypeError", s.clone()),
                                    RuntimeError::IndexError(s) => ("IndexError", s.clone()),
                                    RuntimeError::Custom(s) => ("Custom", s.clone()),
                                };

                                let mut error_map = HashMap::new();
                                error_map.insert(
                                    "type".to_string(),
                                    Value::String(error_type.to_string()),
                                );
                                error_map
                                    .insert("message".to_string(), Value::String(error_message));
                                error_map.insert(
                                    "line".to_string(),
                                    Value::Number(bigdecimal::BigDecimal::from(
                                        self.current_line as i64,
                                    )),
                                );

                                self.env.define(
                                    var_name.clone(),
                                    Value::Map(std::sync::Arc::new(std::sync::RwLock::new(
                                        error_map,
                                    ))),
                                );
                            }

                            self.execute_block(catch)
                        } else {
                            Err(err)
                        }
                    }
                    Ok(result) => Ok(result),
                };

                if let Some(always) = always_body {
                    match self.execute_block(always) {
                        Ok(_) => {}
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }

                final_result
            }

            Statement::RepeatTimes {
                count,
                variable,
                body,
                ..
            } => {
                let count_val = self.evaluate_expression(count)?;
                if let Value::Number(n) = count_val {
                    use bigdecimal::ToPrimitive;
                    if let Some(times) = n.to_i64() {
                        for i in 0..times {
                            if let Some(var_name) = variable {
                                self.env.push_scope();
                                let loop_index = Value::Number(bigdecimal::BigDecimal::from(i + 1));
                                self.env.define(var_name.clone(), loop_index);
                                let result = self.execute_block_no_scope(body)?;
                                self.env.pop_scope();
                                match result {
                                    ExecutionResult::Break => {
                                        break;
                                    }
                                    ExecutionResult::Return(v) => {
                                        return Ok(ExecutionResult::Return(v));
                                    }
                                    ExecutionResult::Continue => {
                                        continue;
                                    }
                                    ExecutionResult::Done => {}
                                }
                            } else {
                                match self.execute_block(body)? {
                                    ExecutionResult::Break => {
                                        break;
                                    }
                                    ExecutionResult::Return(v) => {
                                        return Ok(ExecutionResult::Return(v));
                                    }
                                    ExecutionResult::Continue => {
                                        continue;
                                    }
                                    ExecutionResult::Done => {}
                                }
                            }
                        }
                    }
                }
                Ok(ExecutionResult::Done)
            }

            Statement::RepeatWhile {
                condition, body, ..
            } => {
                loop {
                    let cond = self.evaluate_expression(condition)?;
                    if !cond.is_truthy() {
                        break;
                    }
                    match self.execute_block(body)? {
                        ExecutionResult::Break => {
                            break;
                        }
                        ExecutionResult::Return(v) => {
                            return Ok(ExecutionResult::Return(v));
                        }
                        ExecutionResult::Continue => {
                            continue;
                        }
                        ExecutionResult::Done => {}
                    }
                }
                Ok(ExecutionResult::Done)
            }

            Statement::ForEach {
                variable,
                iterable,
                body,
                ..
            } => {
                let collection = self.evaluate_expression(iterable)?;

                if let Value::Map(map) = &collection {
                    let has_next = map.read().expect("lock poisoned").contains_key("Next");
                    let has_hasmore = map.read().expect("lock poisoned").contains_key("HasMore");

                    if has_next && has_hasmore {
                        return self.iterate_stream(variable, collection, body);
                    }
                }

                let items: Vec<Value> = match collection {
                    Value::List(l) => l.read().expect("lock poisoned").iter().cloned().collect(),
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "Expected a list or stream".to_string(),
                        ));
                    }
                };

                for item in items {
                    self.env.push_scope();
                    self.env.define(variable.clone(), item);
                    let result = self.execute_block_no_scope(body)?;
                    self.env.pop_scope();
                    match result {
                        ExecutionResult::Break => {
                            break;
                        }
                        ExecutionResult::Return(v) => {
                            return Ok(ExecutionResult::Return(v));
                        }
                        ExecutionResult::Continue => {
                            continue;
                        }
                        ExecutionResult::Done => {}
                    }
                }
                Ok(ExecutionResult::Done)
            }

            Statement::Break { .. } => Ok(ExecutionResult::Break),
            Statement::Continue { .. } => Ok(ExecutionResult::Continue),
            Statement::Return { value, .. } => {
                let val = if let Some(expr) = value {
                    self.evaluate_expression(expr)?
                } else {
                    Value::Boolean(false)
                };
                Ok(ExecutionResult::Return(val))
            }

            Statement::Expression { expr, .. } => {
                self.evaluate_expression(expr)?;
                Ok(ExecutionResult::Done)
            }
        }
    }

    fn iterate_stream(
        &mut self,
        variable: &str,
        stream: Value,
        body: &[Statement],
    ) -> Result<ExecutionResult, RuntimeError> {
        loop {
            let next_method = if let Value::Map(map) = &stream {
                map.read().expect("lock poisoned").get("Next").cloned()
            } else {
                return Err(RuntimeError::TypeError("Invalid stream object".to_string()));
            };

            let next_method = next_method.ok_or_else(|| {
                RuntimeError::TypeError("Stream missing .Next method".to_string())
            })?;

            let next_value = match next_method {
                Value::NativeFunction(f) => f(vec![]).map_err(|e| RuntimeError::Custom(e))?,
                _ => {
                    return Err(RuntimeError::TypeError(
                        "Stream.Next must be a function".to_string(),
                    ));
                }
            };

            match next_value {
                Value::Option(opt) => {
                    if let Some(item) = opt.as_ref() {
                        self.env.push_scope();
                        self.env.define(variable.to_string(), item.clone());
                        let result = self.execute_block_no_scope(body)?;
                        self.env.pop_scope();

                        match result {
                            ExecutionResult::Break => {
                                break;
                            }
                            ExecutionResult::Return(v) => {
                                return Ok(ExecutionResult::Return(v));
                            }
                            ExecutionResult::Continue => {
                                continue;
                            }
                            ExecutionResult::Done => {}
                        }
                    } else {
                        break;
                    }
                }
                _ => {
                    return Err(RuntimeError::TypeError(
                        "Stream.Next must return Option".to_string(),
                    ));
                }
            }
        }
        Ok(ExecutionResult::Done)
    }

    fn execute_block(&mut self, statements: &[Statement]) -> Result<ExecutionResult, RuntimeError> {
        self.env.push_scope();
        let result = self.execute_block_no_scope(statements);
        self.env.pop_scope();
        result
    }

    fn execute_block_no_scope(
        &mut self,
        statements: &[Statement],
    ) -> Result<ExecutionResult, RuntimeError> {
        for stmt in statements {
            self.current_line = Self::get_statement_line(stmt);
            if self.trace {
                println!("[line {}] {:?}", self.current_line, stmt);
            }
            let result = match self.execute_statement(stmt) {
                Ok(res) => res,
                Err(err) => {
                    return Err(Self::with_line(err, self.current_line));
                }
            };
            if !matches!(result, ExecutionResult::Done) {
                return Ok(result);
            }
        }
        Ok(ExecutionResult::Done)
    }

    fn with_line(err: RuntimeError, line: usize) -> RuntimeError {
        let prefix = format!("Line {}: ", line);
        match err {
            RuntimeError::UndefinedVariable(msg) => {
                RuntimeError::UndefinedVariable(format!("{}{}", prefix, msg))
            }
            RuntimeError::UndefinedConcept(msg) => {
                RuntimeError::UndefinedConcept(format!("{}{}", prefix, msg))
            }
            RuntimeError::UndefinedMethod(msg) => {
                RuntimeError::UndefinedMethod(format!("{}{}", prefix, msg))
            }
            RuntimeError::TypeError(msg) => RuntimeError::TypeError(format!("{}{}", prefix, msg)),
            RuntimeError::IndexError(msg) => RuntimeError::IndexError(format!("{}{}", prefix, msg)),
            RuntimeError::Custom(msg) => RuntimeError::Custom(format!("{}{}", prefix, msg)),
        }
    }

    fn get_statement_line(stmt: &Statement) -> usize {
        match stmt {
            Statement::Use { line, .. }
            | Statement::Assignment { line, .. }
            | Statement::Create { line, .. }
            | Statement::Set { line, .. }
            | Statement::Print { line, .. }
            | Statement::SwitchOn { line, .. }
            | Statement::SwitchOff { line, .. }
            | Statement::If { line, .. }
            | Statement::When { line, .. }
            | Statement::TryCatch { line, .. }
            | Statement::RepeatTimes { line, .. }
            | Statement::RepeatWhile { line, .. }
            | Statement::ForEach { line, .. }
            | Statement::Return { line, .. }
            | Statement::Break { line }
            | Statement::Continue { line }
            | Statement::Expression { line, .. } => *line,
        }
    }

    fn execute_method_stack(
        &mut self,
        stack: &[Method],
        this: Value,
        args: Vec<(String, Value)>,
    ) -> Result<Value, RuntimeError> {
        if stack.is_empty() {
            return Ok(Value::default_boolean());
        }

        let index = stack.len() - 1;
        let method = &stack[index];

        self.env.push_scope();
        self.env.define("This".to_string(), this.clone());

        for (i, param_name) in method.parameters.iter().enumerate() {
            if let Some((_, val)) = args.get(i) {
                self.env.define(param_name.clone(), val.clone());
            }
        }

        if index > 0 {
            self.proceed_stack
                .push((stack.to_vec(), index - 1, this.clone(), args.clone()));
        }

        let result = self.execute_block_no_scope(&method.body)?;

        if index > 0 {
            self.proceed_stack.pop();
        }

        self.env.pop_scope();

        match result {
            ExecutionResult::Return(v) => Ok(v),
            _ => Ok(Value::default_boolean()),
        }
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value, RuntimeError> {
        match expr {
            Expression::Number(n) => Value::from_number_string(n).map_err(RuntimeError::Custom),
            Expression::String(s) => Ok(Value::String(s.clone())),
            Expression::Boolean(b) => Ok(Value::Boolean(*b)),
            Expression::List(items) => {
                let mut values = Vec::new();
                for item in items {
                    values.push(self.evaluate_expression(item)?);
                }
                Ok(Value::List(std::sync::Arc::new(std::sync::RwLock::new(
                    values,
                ))))
            }
            Expression::Map(entries) => {
                let mut map = HashMap::new();
                for (key, value_expr) in entries {
                    map.insert(key.clone(), self.evaluate_expression(value_expr)?);
                }
                Ok(Value::Map(std::sync::Arc::new(std::sync::RwLock::new(map))))
            }
            Expression::Identifier(name) => {
                if let Some(val) = self.env.get(name) {
                    Ok(val)
                } else {
                    if let Some(Value::Map(m)) = self.env.get("This") {
                        if let Some(val) = m.read().expect("lock poisoned").get(name) {
                            return Ok(val.clone());
                        }
                    }
                    Err(RuntimeError::UndefinedVariable(name.clone()))
                }
            }
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                match operator {
                    BinaryOperator::Add => {
                        left_val.add(&right_val).map_err(RuntimeError::TypeError)
                    }
                    BinaryOperator::Subtract => left_val
                        .subtract(&right_val)
                        .map_err(RuntimeError::TypeError),
                    BinaryOperator::Multiply => left_val
                        .multiply(&right_val)
                        .map_err(RuntimeError::TypeError),
                    BinaryOperator::Divide => {
                        left_val.divide(&right_val).map_err(RuntimeError::TypeError)
                    }
                    BinaryOperator::Modulo => {
                        left_val.modulo(&right_val).map_err(RuntimeError::TypeError)
                    }
                    BinaryOperator::Equal => Ok(Value::Boolean(left_val.equals(&right_val))),
                    BinaryOperator::NotEqual => Ok(Value::Boolean(!left_val.equals(&right_val))),
                    BinaryOperator::Greater => {
                        let ord = left_val
                            .compare(&right_val)
                            .map_err(RuntimeError::TypeError)?;
                        Ok(Value::Boolean(ord == std::cmp::Ordering::Greater))
                    }
                    BinaryOperator::Less => {
                        let ord = left_val
                            .compare(&right_val)
                            .map_err(RuntimeError::TypeError)?;
                        Ok(Value::Boolean(ord == std::cmp::Ordering::Less))
                    }
                    BinaryOperator::GreaterEq => {
                        let ord = left_val
                            .compare(&right_val)
                            .map_err(RuntimeError::TypeError)?;
                        Ok(Value::Boolean(ord != std::cmp::Ordering::Less))
                    }
                    BinaryOperator::LessEq => {
                        let ord = left_val
                            .compare(&right_val)
                            .map_err(RuntimeError::TypeError)?;
                        Ok(Value::Boolean(ord != std::cmp::Ordering::Greater))
                    }
                    BinaryOperator::And => Ok(Value::Boolean(
                        left_val.is_truthy() && right_val.is_truthy(),
                    )),
                    BinaryOperator::Or => Ok(Value::Boolean(
                        left_val.is_truthy() || right_val.is_truthy(),
                    )),
                }
            }
            Expression::UnaryOp { operator, operand } => {
                let val = self.evaluate_expression(operand)?;
                match operator {
                    UnaryOperator::Not => Ok(Value::Boolean(!val.is_truthy())),
                    UnaryOperator::Minus => {
                        if let Value::Number(n) = val {
                            Ok(Value::Number(-n))
                        } else {
                            Err(RuntimeError::TypeError(
                                "Cannot negate non-number".to_string(),
                            ))
                        }
                    }
                }
            }
            Expression::Index { object, index } => {
                let obj = self.evaluate_expression(object)?;
                let idx = self.evaluate_expression(index)?;
                obj.index(&idx).map_err(RuntimeError::IndexError)
            }
            Expression::MemberAccess { object, member } => {
                let obj_val = self.evaluate_expression(object)?;

                if member == "Length" || member == "Size" {
                    match obj_val.len() {
                        Ok(len) => {
                            use bigdecimal::BigDecimal;
                            return Ok(Value::Number(BigDecimal::from(len as i64)));
                        }
                        Err(e) => {
                            return Err(RuntimeError::TypeError(e));
                        }
                    }
                }

                if member == "IsValid" {
                    if matches!(obj_val, Value::WeakList(_) | Value::WeakMap(_)) {
                        return Ok(Value::Boolean(obj_val.is_weak_valid()));
                    }
                }

                if member == "Get" {
                    if matches!(obj_val, Value::WeakList(_) | Value::WeakMap(_)) {
                        let weak_clone = obj_val.clone();
                        return Ok(Value::NativeFunction(std::sync::Arc::new(Box::new(
                            move |_args| weak_clone.upgrade_weak(),
                        ))));
                    }
                }

                if member == "IsSome" {
                    if matches!(obj_val, Value::Option(_)) {
                        return Ok(Value::Boolean(obj_val.is_some()));
                    }
                }

                if member == "IsNone" {
                    if matches!(obj_val, Value::Option(_)) {
                        return Ok(Value::Boolean(obj_val.is_none()));
                    }
                }

                if member == "Unwrap" {
                    if matches!(obj_val, Value::Option(_)) {
                        let opt_clone = obj_val.clone();
                        return Ok(Value::NativeFunction(std::sync::Arc::new(Box::new(
                            move |_args| opt_clone.unwrap_option(),
                        ))));
                    }
                }

                if member == "UnwrapOr" {
                    if matches!(obj_val, Value::Option(_)) {
                        let opt_clone = obj_val.clone();
                        return Ok(Value::NativeFunction(std::sync::Arc::new(Box::new(
                            move |args| {
                                if args.len() != 1 {
                                    return Err(
                                        "UnwrapOr requires 1 argument (default value)".to_string()
                                    );
                                }
                                opt_clone.unwrap_or(args[0].clone())
                            },
                        ))));
                    }
                }

                if member == "Await" {
                    if matches!(obj_val, Value::TaskHandle(_, _)) {
                        if let Value::TaskHandle(handle_mutex, _cancel_token) = obj_val {
                            let runtime_clone = self.runtime.clone();
                            return Ok(Value::NativeFunction(std::sync::Arc::new(Box::new(
                                move |_args| {
                                    let mut handle_lock = handle_mutex.lock().unwrap();
                                    if let Some(handle) = handle_lock.take() {
                                        runtime_clone.block_on(async move {
                                            match handle.await {
                                                Ok(value) => Ok(value),
                                                Err(e) => Err(format!("Task panicked: {}", e)),
                                            }
                                        })
                                    } else {
                                        Err("Task already awaited".to_string())
                                    }
                                },
                            ))));
                        }
                    }
                }

                if let Value::Map(m) = &obj_val {
                    if let Some(val) = m.read().expect("lock poisoned").get(member) {
                        return Ok(val.clone());
                    }
                }

                let concept_name = if let Value::Map(m) = &obj_val {
                    m.read()
                        .expect("lock poisoned")
                        .get("_concept")
                        .map(|v| v.to_string())
                } else {
                    None
                };

                if let Some(c_name) = concept_name {
                    let mut method_stack: Vec<Method> = Vec::new();

                    if let Some(concept) = self.concepts.get(&c_name) {
                        if let Some(method_def) = concept.methods.iter().find(|m| m.name == *member)
                        {
                            method_stack.push(method_def.clone());
                        }
                    }

                    for situation_name in &self.active_situations {
                        if let Some(situation) = self.situations.get(situation_name) {
                            if let Some(adj) = situation
                                .adjustments
                                .iter()
                                .find(|a| a.concept_name == c_name)
                            {
                                if let Some(method_def) =
                                    adj.methods.iter().find(|m| m.name == *member)
                                {
                                    method_stack.push(method_def.clone());
                                }
                            }
                        }
                    }

                    if !method_stack.is_empty() {
                        self.profiler.record_call(&c_name, member);

                        if let Some(cached_ptr) = self.jit_compiler.get_function(&c_name, member) {
                            let needs_obj_ptr =
                                self.jit_compiler.method_needs_obj_ptr(&c_name, member);

                            let required_fields = self
                                .jit_compiler
                                .get_required_fields_by_key(&c_name, member);

                            let obj_ptr_count = if needs_obj_ptr { 1 } else { 0 };
                            let total_args = obj_ptr_count + required_fields.len();
                            let mut jit_args: Vec<f64> = Vec::with_capacity(total_args);

                            if needs_obj_ptr {
                                if let Value::Map(m) = &obj_val {
                                    //let obj_ptr = m as *const _ as *const u8 as i64;
                                    let obj_ptr = Arc::as_ptr(m) as *const u8 as i64;
                                    jit_args.push(f64::from_bits(obj_ptr as u64));
                                }
                            }

                            if let Value::Map(m) = &obj_val {
                                let map = m.read().expect("lock poisoned");
                                for field_name in &required_fields {
                                    let val = map
                                        .get(field_name)
                                        .cloned()
                                        .unwrap_or(Value::Number(bigdecimal::BigDecimal::from(0)));
                                    jit_args.push(Self::value_to_f64(&val)?);
                                }
                            }

                            let result = Self::call_jit_function(cached_ptr, &jit_args)?;
                            return Ok(Value::Number(
                                bigdecimal::BigDecimal::from_f64(result)
                                    .unwrap_or_else(|| bigdecimal::BigDecimal::from(0)),
                            ));
                        }

                        let should_compile = self.profiler.should_jit(&c_name, member);
                        if should_compile && !method_stack.is_empty() {
                            let base_method = &method_stack[0];

                            let available_methods = self
                                .concepts
                                .get(&c_name)
                                .map(|c| c.methods.as_slice())
                                .unwrap_or(&[]);
                            match self.jit_compiler.compile_method(
                                &c_name,
                                base_method,
                                available_methods,
                            ) {
                                Ok(_func_ptr) => {
                                    self.profiler.mark_compiled(&c_name, member);

                                    if let Some(cached_ptr) =
                                        self.jit_compiler.get_function(&c_name, member)
                                    {
                                        let needs_obj_ptr =
                                            self.jit_compiler.method_needs_obj_ptr(&c_name, member);

                                        let required_fields = self
                                            .jit_compiler
                                            .get_required_fields_by_key(&c_name, member);

                                        let obj_ptr_count = if needs_obj_ptr { 1 } else { 0 };
                                        let total_args = obj_ptr_count + required_fields.len();
                                        let mut jit_args: Vec<f64> = Vec::with_capacity(total_args);

                                        if needs_obj_ptr {
                                            if let Value::Map(m) = &obj_val {
                                                //let obj_ptr = m as *const _ as *const u8 as i64;
                                                let obj_ptr = Arc::as_ptr(m) as *const u8 as i64;
                                                jit_args.push(f64::from_bits(obj_ptr as u64));
                                            }
                                        }

                                        if let Value::Map(m) = &obj_val {
                                            let map = m.read().expect("lock poisoned");
                                            for field_name in &required_fields {
                                                let val = map.get(field_name).cloned().unwrap_or(
                                                    Value::Number(bigdecimal::BigDecimal::from(0)),
                                                );
                                                jit_args.push(Self::value_to_f64(&val)?);
                                            }
                                        }

                                        let result =
                                            Self::call_jit_function(cached_ptr, &jit_args)?;
                                        return Ok(Value::Number(
                                            bigdecimal::BigDecimal::from_f64(result)
                                                .unwrap_or_else(|| bigdecimal::BigDecimal::from(0)),
                                        ));
                                    }
                                }
                                Err(e) => {
                                    println!(
                                        "JIT compilation failed for {}.{}: {}",
                                        c_name, member, e
                                    );
                                    println!("   Falling back to interpreter");
                                }
                            }
                        }

                        return self.execute_method_stack(&method_stack, obj_val, Vec::new());
                    }
                }
                Err(RuntimeError::UndefinedMethod(format!(
                    "Property or Method '{}' not found",
                    member
                )))
            }

            Expression::FunctionCall { name, arguments } => {
                let callee_val = self
                    .env
                    .get(name)
                    .ok_or_else(|| RuntimeError::UndefinedVariable(name.clone()))?;

                if let Value::NativeFunction(func) = callee_val {
                    let mut args = Vec::new();
                    for arg_expr in arguments {
                        args.push(self.evaluate_expression(arg_expr)?);
                    }

                    match func(args) {
                        Ok(v) => Ok(v),
                        Err(msg) => Err(RuntimeError::Custom(msg)),
                    }
                } else {
                    Err(RuntimeError::TypeError(format!(
                        "Identifier '{}' is not a callable function",
                        name
                    )))
                }
            }

            Expression::Call { callee, arguments } => {
                let callee_val = self.evaluate_expression(callee)?;

                if let Value::NativeFunction(func) = callee_val {
                    let mut args = Vec::new();
                    for arg_expr in arguments {
                        args.push(self.evaluate_expression(arg_expr)?);
                    }

                    match func(args) {
                        Ok(v) => Ok(v),
                        Err(msg) => Err(RuntimeError::Custom(msg)),
                    }
                } else {
                    Err(RuntimeError::TypeError(
                        "Expression is not a callable function".to_string(),
                    ))
                }
            }

            Expression::DoInBackground { body } => {
                let active_situations = self.active_situations.clone();
                let body = body.clone();
                let concepts = self.concepts.clone();
                let situations = self.situations.clone();
                let env = self.env.clone_deep();
                let runtime_outer = self.runtime.clone();
                let runtime_inner = runtime_outer.clone();

                let cancel_token = Arc::new(std::sync::atomic::AtomicBool::new(false));

                let handle = runtime_outer.spawn(async move {
                    tokio::task::spawn_blocking(move || {
                        let mut task_interpreter =
                            Interpreter::new_with_shared_runtime(runtime_inner);
                        task_interpreter.concepts = concepts;
                        task_interpreter.situations = situations;
                        task_interpreter.active_situations = active_situations;
                        task_interpreter.env = env;

                        let mut result = Value::default_boolean();
                        for statement in body {
                            let line = Self::get_statement_line(&statement);
                            task_interpreter.current_line = line;
                            match task_interpreter.execute_statement(&statement) {
                                Ok(ExecutionResult::Return(v)) => {
                                    result = v;
                                    break;
                                }
                                Ok(ExecutionResult::Break) => {
                                    break;
                                }
                                Ok(ExecutionResult::Continue) => {
                                    continue;
                                }
                                Ok(ExecutionResult::Done) => {}
                                Err(e) => {
                                    let e = Self::with_line(e, line);
                                    let (category, subtype, message) = match &e {
                                        RuntimeError::UndefinedVariable(msg) => {
                                            ("Lookup", "UndefinedVariable", msg.clone())
                                        }
                                        RuntimeError::UndefinedConcept(msg) => {
                                            ("Lookup", "UndefinedVariable", msg.clone())
                                        }
                                        RuntimeError::UndefinedMethod(msg) => {
                                            ("Lookup", "MethodNotFound", msg.clone())
                                        }
                                        RuntimeError::TypeError(msg) => {
                                            ("Validation", "InvalidType", msg.clone())
                                        }
                                        RuntimeError::IndexError(msg) => {
                                            ("Lookup", "IndexOutOfBounds", msg.clone())
                                        }
                                        RuntimeError::Custom(msg) => {
                                            ("Logic", "InvalidOperation", msg.clone())
                                        }
                                    };
                                    result = Value::Error(Arc::new(ErrorInfo {
                                        category: category.to_string(),
                                        subtype: subtype.to_string(),
                                        message,
                                    }));
                                    break;
                                }
                            }
                        }
                        result
                    })
                    .await
                    .unwrap_or_else(|e| {
                        Value::Error(Arc::new(ErrorInfo {
                            category: "Panic".to_string(),
                            subtype: "TaskPanicked".to_string(),
                            message: format!("Task panicked: {:?}", e),
                        }))
                    })
                });

                Ok(Value::TaskHandle(
                    Arc::new(std::sync::Mutex::new(Some(handle))),
                    cancel_token,
                ))
            }

            Expression::Proceed { arguments } => {
                if let Some((stack, index, this, args)) = self.proceed_stack.last().cloned() {
                    let mut new_args = Vec::new();
                    for arg_expr in arguments {
                        new_args.push(self.evaluate_expression(arg_expr)?);
                    }

                    let final_args = if !new_args.is_empty() {
                        new_args
                            .into_iter()
                            .enumerate()
                            .map(|(i, v)| (format!("arg{}", i), v))
                            .collect()
                    } else {
                        args
                    };

                    let lower_stack = &stack[0..=index];
                    self.execute_method_stack(lower_stack, this, final_args)
                } else {
                    Err(
                        RuntimeError::Custom(
                            "Proceed() can only be called within an adjustment method that has a lower layer to call".to_string()
                        )
                    )
                }
            }

            Expression::MethodCall {
                object,
                method,
                arguments,
            } => {
                let obj_val = self.evaluate_expression(object)?;

                let concept_name = if let Value::Map(m) = &obj_val {
                    let map_read = m.read().expect("lock poisoned");
                    map_read.get("_concept").and_then(|v| {
                        if let Value::String(s) = v {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                } else {
                    None
                };

                if let Some(c_name) = concept_name {
                    let mut method_stack: Vec<Method> = Vec::new();

                    if let Some(concept) = self.concepts.get(&c_name) {
                        if let Some(method_def) = concept.methods.iter().find(|m| m.name == *method)
                        {
                            method_stack.push(method_def.clone());
                        }
                    }

                    for situation_name in &self.active_situations {
                        if let Some(situation) = self.situations.get(situation_name) {
                            if let Some(adj) = situation
                                .adjustments
                                .iter()
                                .find(|a| a.concept_name == c_name)
                            {
                                if let Some(method_def) =
                                    adj.methods.iter().find(|m| m.name == *method)
                                {
                                    method_stack.push(method_def.clone());
                                }
                            }
                        }
                    }

                    if method_stack.is_empty() {
                        return Err(RuntimeError::Custom(format!(
                            "Method '{}' not found on concept '{}'",
                            method, c_name
                        )));
                    }

                    let mut args = Vec::new();
                    for (param_name, arg_expr) in arguments {
                        let val = self.evaluate_expression(arg_expr)?;
                        args.push((param_name.clone(), val));
                    }

                    self.profiler.record_call(&c_name, method);

                    if let Some(cached_ptr) = self.jit_compiler.get_function(&c_name, method) {
                        let needs_obj_ptr = self.jit_compiler.method_needs_obj_ptr(&c_name, method);

                        let required_fields = self
                            .jit_compiler
                            .get_required_fields_by_key(&c_name, method);

                        let obj_ptr_count = if needs_obj_ptr { 1 } else { 0 };
                        let total_args = obj_ptr_count + required_fields.len() + args.len();
                        let mut jit_args: Vec<f64> = Vec::with_capacity(total_args);

                        if needs_obj_ptr {
                            if let Value::Map(m) = &obj_val {
                                //let obj_ptr = m as *const _ as *const u8 as i64;
                                let obj_ptr = Arc::as_ptr(m) as *const u8 as i64;
                                jit_args.push(f64::from_bits(obj_ptr as u64));
                            }
                        }

                        if let Value::Map(m) = &obj_val {
                            let map_read = m.read().expect("lock poisoned");
                            for field_name in &required_fields {
                                if let Some(field_val) = map_read.get(field_name) {
                                    jit_args.push(Self::value_to_f64(field_val)?);
                                } else {
                                    jit_args.push(0.0);
                                }
                            }
                        }

                        for (_, val) in &args {
                            jit_args.push(Self::value_to_f64(val)?);
                        }

                        let result = Self::call_jit_function(cached_ptr, &jit_args)?;
                        return Ok(Value::Number(
                            bigdecimal::BigDecimal::from_f64(result)
                                .unwrap_or_else(|| bigdecimal::BigDecimal::from(0)),
                        ));
                    }

                    let should_compile = self.profiler.should_jit(&c_name, method);
                    if should_compile && !method_stack.is_empty() {
                        let base_method = &method_stack[0];

                        let available_methods = self
                            .concepts
                            .get(&c_name)
                            .map(|c| c.methods.as_slice())
                            .unwrap_or(&[]);
                        match self.jit_compiler.compile_method(
                            &c_name,
                            base_method,
                            available_methods,
                        ) {
                            Ok(_func_ptr) => {
                                self.profiler.mark_compiled(&c_name, method);

                                if let Some(cached_ptr) =
                                    self.jit_compiler.get_function(&c_name, method)
                                {
                                    let needs_obj_ptr =
                                        self.jit_compiler.method_needs_obj_ptr(&c_name, method);

                                    let required_fields = self
                                        .jit_compiler
                                        .get_required_fields_by_key(&c_name, method);

                                    let obj_ptr_count = if needs_obj_ptr { 1 } else { 0 };
                                    let total_args =
                                        obj_ptr_count + required_fields.len() + args.len();
                                    let mut jit_args: Vec<f64> = Vec::with_capacity(total_args);

                                    if needs_obj_ptr {
                                        if let Value::Map(m) = &obj_val {
                                            //let obj_ptr = m as *const _ as *const u8 as i64;
                                            let obj_ptr = Arc::as_ptr(m) as *const u8 as i64;
                                            jit_args.push(f64::from_bits(obj_ptr as u64));
                                        }
                                    }

                                    if let Value::Map(m) = &obj_val {
                                        let map_read = m.read().expect("lock poisoned");
                                        for field_name in &required_fields {
                                            if let Some(field_val) = map_read.get(field_name) {
                                                jit_args.push(Self::value_to_f64(field_val)?);
                                            } else {
                                                jit_args.push(0.0);
                                            }
                                        }
                                    }

                                    for (_, val) in &args {
                                        jit_args.push(Self::value_to_f64(val)?);
                                    }

                                    let result = Self::call_jit_function(cached_ptr, &jit_args)?;
                                    return Ok(Value::Number(
                                        bigdecimal::BigDecimal::from_f64(result)
                                            .unwrap_or_else(|| bigdecimal::BigDecimal::from(0)),
                                    ));
                                }
                            }
                            Err(e) => {
                                self.profiler.mark_compiled(&c_name, method);

                                if !e.contains("side effects") {
                                    eprintln!(
                                        "JIT compilation failed for {}.{}: {}",
                                        c_name, method, e
                                    );
                                }
                            }
                        }
                    }

                    self.execute_method_stack(&method_stack, obj_val, args)
                } else {
                    Err(RuntimeError::TypeError(
                        "Object does not have a concept".to_string(),
                    ))
                }
            }
        }
    }

    fn value_to_f64(val: &Value) -> Result<f64, RuntimeError> {
        match val {
            Value::Number(n) => n
                .to_string()
                .parse::<f64>()
                .map_err(|_| RuntimeError::TypeError("Cannot convert number to f64".to_string())),
            Value::FastNumber(f) => Ok(*f),
            _ => Err(RuntimeError::TypeError(format!(
                "Cannot convert {:?} to f64 for JIT",
                val
            ))),
        }
    }

    fn call_jit_function(func_ptr: *const u8, args: &[f64]) -> Result<f64, RuntimeError> {
        unsafe {
            match args.len() {
                0 => {
                    let func: extern "C" fn() -> f64 = std::mem::transmute(func_ptr);
                    Ok(func())
                }
                1 => {
                    let func: extern "C" fn(f64) -> f64 = std::mem::transmute(func_ptr);
                    Ok(func(args[0]))
                }
                2 => {
                    let func: extern "C" fn(f64, f64) -> f64 = std::mem::transmute(func_ptr);
                    Ok(func(args[0], args[1]))
                }
                3 => {
                    let func: extern "C" fn(f64, f64, f64) -> f64 = std::mem::transmute(func_ptr);
                    Ok(func(args[0], args[1], args[2]))
                }
                4 => {
                    let func: extern "C" fn(f64, f64, f64, f64) -> f64 =
                        std::mem::transmute(func_ptr);
                    Ok(func(args[0], args[1], args[2], args[3]))
                }
                5 => {
                    let func: extern "C" fn(f64, f64, f64, f64, f64) -> f64 =
                        std::mem::transmute(func_ptr);
                    Ok(func(args[0], args[1], args[2], args[3], args[4]))
                }
                6 => {
                    let func: extern "C" fn(f64, f64, f64, f64, f64, f64) -> f64 =
                        std::mem::transmute(func_ptr);
                    Ok(func(args[0], args[1], args[2], args[3], args[4], args[5]))
                }
                7 => {
                    let func: extern "C" fn(f64, f64, f64, f64, f64, f64, f64) -> f64 =
                        std::mem::transmute(func_ptr);
                    Ok(func(
                        args[0], args[1], args[2], args[3], args[4], args[5], args[6],
                    ))
                }
                8 => {
                    let func: extern "C" fn(f64, f64, f64, f64, f64, f64, f64, f64) -> f64 =
                        std::mem::transmute(func_ptr);
                    Ok(func(
                        args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7],
                    ))
                }
                9 => {
                    let func: extern "C" fn(f64, f64, f64, f64, f64, f64, f64, f64, f64) -> f64 =
                        std::mem::transmute(func_ptr);
                    Ok(func(
                        args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7],
                        args[8],
                    ))
                }
                10 => {
                    let func: extern "C" fn(
                        f64,
                        f64,
                        f64,
                        f64,
                        f64,
                        f64,
                        f64,
                        f64,
                        f64,
                        f64,
                    ) -> f64 = std::mem::transmute(func_ptr);
                    Ok(func(
                        args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7],
                        args[8], args[9],
                    ))
                }
                _ => Err(RuntimeError::Custom(format!(
                    "JIT doesn't support {} arguments yet (max 10)",
                    args.len()
                ))),
            }
        }
    }
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::UndefinedVariable(msg) => write!(f, "Undefined variable: {}", msg),
            RuntimeError::UndefinedConcept(msg) => write!(f, "Undefined concept: {}", msg),
            RuntimeError::UndefinedMethod(msg) => write!(f, "Undefined method: {}", msg),
            RuntimeError::TypeError(msg) => write!(f, "Type error: {}", msg),
            RuntimeError::IndexError(msg) => write!(f, "Index error: {}", msg),
            RuntimeError::Custom(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

impl std::error::Error for RuntimeError {}
