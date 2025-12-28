use crate::compiler::ast::{Expression, Method, Statement};
use crate::runtime::value::Value as SfxValue;
use bigdecimal::{BigDecimal, FromPrimitive};
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, Linkage, Module};
use std::collections::HashMap;
use std::sync::{/*Arc,*/ RwLock};
// use std::mem::ManuallyDrop;

struct VarContext<'a> {
    param_values: HashMap<String, Value>,
    local_vars: HashMap<String, Variable>,
    obj_ptr: Option<Value>,
    method_name: &'a str,
    available_methods: &'a [Method],
    field_access_cache: HashMap<String, Value>,
}

pub struct JitCompiler {
    module: JITModule,
    ctx: codegen::Context,
    #[allow(dead_code)]
    data_description: DataDescription,
    compiled_functions: HashMap<(String, String), *const u8>,
    required_fields_cache: HashMap<(String, String), Vec<String>>,
    update_field_func: Option<cranelift_module::FuncId>,
    methods_with_set: HashMap<(String, String), bool>,
}

impl JitCompiler {
    pub fn new() -> Self {
        let mut flag_builder = cranelift::codegen::settings::builder();
        flag_builder.set("opt_level", "speed").unwrap();
        flag_builder.set("enable_verifier", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let flags = cranelift::codegen::settings::Flags::new(flag_builder);

        let mut builder = JITBuilder::with_isa(
            cranelift_native::builder().unwrap().finish(flags).unwrap(),
            cranelift_module::default_libcall_names(),
        );

        builder.symbol("jit_update_field", jit_update_field as *const u8);

        let mut module = JITModule::new(builder);
        let ctx = module.make_context();

        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(types::I64));
        sig.params.push(AbiParam::new(types::I64));
        sig.params.push(AbiParam::new(types::I64));
        sig.params.push(AbiParam::new(types::F64));

        let update_field_func = module
            .declare_function("jit_update_field", Linkage::Import, &sig)
            .ok();

        Self {
            module,
            ctx,
            data_description: DataDescription::new(),
            compiled_functions: HashMap::new(),
            required_fields_cache: HashMap::new(),
            update_field_func,
            methods_with_set: HashMap::new(),
        }
    }

    pub fn compile_method(
        &mut self,
        concept_name: &str,
        method: &Method,
        available_methods: &[Method],
    ) -> Result<*const u8, String> {
        let key = (concept_name.to_string(), method.name.clone());
        if let Some(&ptr) = self.compiled_functions.get(&key) {
            return Ok(ptr);
        }

        let this_fields = Self::find_this_fields(method, available_methods);

        self.required_fields_cache
            .insert(key.clone(), this_fields.clone());

        let has_set = Self::has_set_statements(method);
        self.methods_with_set.insert(key.clone(), has_set);
        let mut sig = self.module.make_signature();

        if has_set {
            sig.params.push(AbiParam::new(types::F64));
        }

        for _ in &this_fields {
            sig.params.push(AbiParam::new(types::F64));
        }

        for _ in &method.parameters {
            sig.params.push(AbiParam::new(types::F64));
        }

        sig.returns.push(AbiParam::new(types::F64));

        let func_id = self
            .module
            .declare_function(
                &format!("{}_{}", concept_name, method.name),
                Linkage::Export,
                &sig,
            )
            .map_err(|e| format!("Failed to declare function: {}", e))?;

        self.ctx.func.signature = sig;

        {
            let mut builder_context = FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut builder_context);

            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            let mut param_values = std::collections::HashMap::new();
            let local_vars = std::collections::HashMap::new();
            let block_params = builder.block_params(entry_block);
            let mut param_index = 0;

            let obj_ptr = if has_set {
                let ptr = block_params[param_index];
                param_index += 1;
                Some(ptr)
            } else {
                None
            };

            for field_name in &this_fields {
                if let Some(&param_value) = block_params.get(param_index) {
                    param_values.insert(format!("This.{}", field_name), param_value);
                    param_index += 1;
                }
            }

            for param_name in &method.parameters {
                if let Some(&param_value) = block_params.get(param_index) {
                    param_values.insert(param_name.clone(), param_value);
                    param_index += 1;
                }
            }

            let mut var_context = VarContext {
                param_values,
                local_vars,
                obj_ptr,
                method_name: &method.name,
                available_methods,
                field_access_cache: HashMap::new(),
            };

            let result = Self::compile_statements(
                &mut builder,
                &method.body,
                &mut var_context,
                &mut self.module,
                self.update_field_func,
            )?;

            builder.ins().return_(&[result]);
            builder.finalize();
        }

        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| format!("Failed to define function: {}", e))?;
        self.module.clear_context(&mut self.ctx);
        self.module
            .finalize_definitions()
            .map_err(|e| format!("Failed to finalize: {}", e))?;
        let code_ptr = self.module.get_finalized_function(func_id);
        self.compiled_functions.insert(key, code_ptr);
        Ok(code_ptr)
    }

    fn compile_statements(
        builder: &mut FunctionBuilder,
        statements: &[Statement],
        var_context: &mut VarContext,
        module: &mut JITModule,
        update_field_func_id: Option<cranelift_module::FuncId>,
    ) -> Result<Value, String> {
        let mut last_value = builder.ins().iconst(types::I64, 0);

        for stmt in statements {
            last_value =
                Self::compile_statement(builder, stmt, var_context, module, update_field_func_id)?;
        }

        Ok(last_value)
    }

    fn compile_statement(
        builder: &mut FunctionBuilder,
        statement: &Statement,
        var_context: &mut VarContext,
        module: &mut JITModule,
        update_field_func_id: Option<cranelift_module::FuncId>,
    ) -> Result<Value, String> {
        match statement {
            Statement::Return { value, .. } => {
                if let Some(expr) = value {
                    Self::compile_expression(
                        builder,
                        expr,
                        var_context,
                        module,
                        update_field_func_id,
                    )
                } else {
                    Ok(builder.ins().iconst(types::I64, 0))
                }
            }
            Statement::Assignment { target, value, .. } => {
                let val = Self::compile_expression(
                    builder,
                    value,
                    var_context,
                    module,
                    update_field_func_id,
                )?;

                if let Some(&var) = var_context.local_vars.get(target) {
                    builder.def_var(var, val);
                } else {
                    let var = builder.declare_var(types::F64);
                    builder.def_var(var, val);
                    var_context.local_vars.insert(target.clone(), var);
                }

                Ok(val)
            }
            Statement::Set { target, value, .. } => {
                let val = Self::compile_expression(
                    builder,
                    value,
                    var_context,
                    module,
                    update_field_func_id,
                )?;

                match target {
                    Expression::MemberAccess { object, member } => {
                        if matches!(&**object, Expression::Identifier(name) if name == "This") {
                            let obj_ptr_f64 = var_context.obj_ptr.ok_or(
                                "Set statement requires object pointer but none was provided",
                            )?;

                            let obj_ptr =
                                builder
                                    .ins()
                                    .bitcast(types::I64, MemFlags::new(), obj_ptr_f64);

                            let func_id = update_field_func_id
                                .ok_or("External update_field function not declared")?;

                            let func_ref = module.declare_func_in_func(func_id, builder.func);

                            let field_static: &'static str =
                                Box::leak(member.clone().into_boxed_str());
                            let field_ptr = field_static.as_ptr() as i64;
                            let field_len = field_static.len() as i64;

                            let field_ptr_val = builder.ins().iconst(types::I64, field_ptr);
                            let field_len_val = builder.ins().iconst(types::I64, field_len);

                            builder
                                .ins()
                                .call(func_ref, &[obj_ptr, field_ptr_val, field_len_val, val]);

                            Ok(val)
                        } else {
                            Err("Set statement target must be This.FieldName".to_string())
                        }
                    }
                    _ => Err("Set statement target must be This.FieldName".to_string()),
                }
            }
            Statement::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let then_block = builder.create_block();
                let else_block = builder.create_block();
                let merge_block = builder.create_block();

                let result_var = builder.declare_var(types::F64);

                let cond_val = Self::compile_expression(
                    builder,
                    condition,
                    var_context,
                    module,
                    update_field_func_id,
                )?;

                let zero = builder.ins().f64const(0.0);
                let cond_bool = builder.ins().fcmp(FloatCC::NotEqual, cond_val, zero);

                builder
                    .ins()
                    .brif(cond_bool, then_block, &[], else_block, &[]);

                builder.switch_to_block(then_block);
                builder.seal_block(then_block);
                let then_result = Self::compile_statements(
                    builder,
                    then_body,
                    var_context,
                    module,
                    update_field_func_id,
                )?;
                builder.def_var(result_var, then_result);
                builder.ins().jump(merge_block, &[]);

                builder.switch_to_block(else_block);
                builder.seal_block(else_block);
                let else_result = if let Some(else_stmts) = else_body {
                    Self::compile_statements(
                        builder,
                        else_stmts,
                        var_context,
                        module,
                        update_field_func_id,
                    )?
                } else {
                    builder.ins().iconst(types::I64, 0)
                };
                builder.def_var(result_var, else_result);
                builder.ins().jump(merge_block, &[]);

                builder.switch_to_block(merge_block);
                builder.seal_block(merge_block);

                let result = builder.use_var(result_var);
                Ok(result)
            }
            Statement::RepeatTimes {
                count,
                variable,
                body,
                ..
            } => {
                if variable.is_some() {
                    return Err("JIT doesn't support loop variables in RepeatTimes yet".to_string());
                }

                let loop_header = builder.create_block();
                let loop_body = builder.create_block();
                let loop_exit = builder.create_block();

                let counter_var = builder.declare_var(types::I64);

                let count_val_f64 = Self::compile_expression(
                    builder,
                    count,
                    var_context,
                    module,
                    update_field_func_id,
                )?;

                let count_val = builder.ins().fcvt_to_sint(types::I64, count_val_f64);

                let zero = builder.ins().iconst(types::I64, 0);
                builder.def_var(counter_var, zero);
                builder.ins().jump(loop_header, &[]);

                builder.switch_to_block(loop_header);
                let counter = builder.use_var(counter_var);
                let cond = builder
                    .ins()
                    .icmp(IntCC::SignedLessThan, counter, count_val);
                builder.ins().brif(cond, loop_body, &[], loop_exit, &[]);

                builder.switch_to_block(loop_body);
                let _body_result = Self::compile_statements(
                    builder,
                    body,
                    var_context,
                    module,
                    update_field_func_id,
                )?;
                let one = builder.ins().iconst(types::I64, 1);
                let counter_again = builder.use_var(counter_var);
                let next_counter = builder.ins().iadd(counter_again, one);
                builder.def_var(counter_var, next_counter);
                builder.ins().jump(loop_header, &[]);
                builder.seal_block(loop_body);

                builder.seal_block(loop_header);

                builder.switch_to_block(loop_exit);
                builder.seal_block(loop_exit);

                Ok(builder.ins().iconst(types::I64, 0))
            }
            _ => Err(format!("Unsupported statement for JIT: {:?}", statement)),
        }
    }

    fn compile_expression(
        builder: &mut FunctionBuilder,
        expr: &Expression,
        var_context: &mut VarContext,
        module: &mut JITModule,
        update_field_func_id: Option<cranelift_module::FuncId>,
    ) -> Result<Value, String> {
        match expr {
            Expression::Number(n) => {
                let num: f64 = n.parse().unwrap_or(0.0);
                Ok(builder.ins().f64const(num))
            }
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
                let lhs = Self::compile_expression(
                    builder,
                    left,
                    var_context,
                    module,
                    update_field_func_id,
                )?;
                let rhs = Self::compile_expression(
                    builder,
                    right,
                    var_context,
                    module,
                    update_field_func_id,
                )?;

                use crate::compiler::ast::BinaryOperator;
                match operator {
                    BinaryOperator::Add => Ok(builder.ins().fadd(lhs, rhs)),
                    BinaryOperator::Subtract => Ok(builder.ins().fsub(lhs, rhs)),
                    BinaryOperator::Multiply => Ok(builder.ins().fmul(lhs, rhs)),
                    BinaryOperator::Divide => Ok(builder.ins().fdiv(lhs, rhs)),
                    BinaryOperator::Modulo => {
                        Err("Modulo operator is not supported by JIT yet".to_string())
                    }

                    BinaryOperator::Equal => {
                        let cmp = builder.ins().fcmp(FloatCC::Equal, lhs, rhs);

                        let one = builder.ins().f64const(1.0);
                        let zero = builder.ins().f64const(0.0);
                        Ok(builder.ins().select(cmp, one, zero))
                    }
                    BinaryOperator::NotEqual => {
                        let cmp = builder.ins().fcmp(FloatCC::NotEqual, lhs, rhs);
                        let one = builder.ins().f64const(1.0);
                        let zero = builder.ins().f64const(0.0);
                        Ok(builder.ins().select(cmp, one, zero))
                    }
                    BinaryOperator::Greater => {
                        let cmp = builder.ins().fcmp(FloatCC::GreaterThan, lhs, rhs);
                        let one = builder.ins().f64const(1.0);
                        let zero = builder.ins().f64const(0.0);
                        Ok(builder.ins().select(cmp, one, zero))
                    }
                    BinaryOperator::Less => {
                        let cmp = builder.ins().fcmp(FloatCC::LessThan, lhs, rhs);
                        let one = builder.ins().f64const(1.0);
                        let zero = builder.ins().f64const(0.0);
                        Ok(builder.ins().select(cmp, one, zero))
                    }
                    BinaryOperator::GreaterEq => {
                        let cmp = builder.ins().fcmp(FloatCC::GreaterThanOrEqual, lhs, rhs);
                        let one = builder.ins().f64const(1.0);
                        let zero = builder.ins().f64const(0.0);
                        Ok(builder.ins().select(cmp, one, zero))
                    }
                    BinaryOperator::LessEq => {
                        let cmp = builder.ins().fcmp(FloatCC::LessThanOrEqual, lhs, rhs);
                        let one = builder.ins().f64const(1.0);
                        let zero = builder.ins().f64const(0.0);
                        Ok(builder.ins().select(cmp, one, zero))
                    }

                    _ => Err(format!("Unsupported operator: {:?}", operator)),
                }
            }
            Expression::Identifier(name) => {
                if let Some(&var) = var_context.local_vars.get(name) {
                    Ok(builder.use_var(var))
                } else if let Some(&value) = var_context.param_values.get(name) {
                    Ok(value)
                } else {
                    Err(format!("Undefined variable: {}", name))
                }
            }
            Expression::MemberAccess { object, member } => {
                if let Expression::Identifier(obj_name) = &**object {
                    if obj_name == "This" {
                        if let Some(callee) = var_context
                            .available_methods
                            .iter()
                            .find(|m| &m.name == member)
                        {
                            if callee.parameters.is_empty()
                                && Self::is_inlinable(callee, var_context.method_name)
                            {
                                let saved_local_vars = var_context.local_vars.clone();

                                let result = if callee.body.len() == 1 {
                                    if let Statement::Return {
                                        value: Some(expr), ..
                                    } = &callee.body[0]
                                    {
                                        Self::compile_expression(
                                            builder,
                                            expr,
                                            var_context,
                                            module,
                                            update_field_func_id,
                                        )
                                    } else {
                                        Ok(builder.ins().f64const(0.0))
                                    }
                                } else {
                                    let mut result_value = builder.ins().f64const(0.0);

                                    for stmt in &callee.body {
                                        match stmt {
                                            Statement::Return {
                                                value: Some(expr), ..
                                            } => {
                                                result_value = Self::compile_expression(
                                                    builder,
                                                    expr,
                                                    var_context,
                                                    module,
                                                    update_field_func_id,
                                                )?;
                                                break;
                                            }
                                            Statement::Assignment { .. } => {
                                                Self::compile_statement(
                                                    builder,
                                                    stmt,
                                                    var_context,
                                                    module,
                                                    update_field_func_id,
                                                )?;
                                            }
                                            _ => {
                                                return Err(format!(
                                                    "Inlined method {} contains unsupported statement",
                                                    member
                                                ));
                                            }
                                        }
                                    }
                                    Ok(result_value)
                                };

                                var_context.local_vars = saved_local_vars;
                                return result;
                            }
                        }

                        let key = format!("This.{}", member);

                        if let Some(&cached_value) = var_context.field_access_cache.get(&key) {
                            return Ok(cached_value);
                        }

                        if let Some(&value) = var_context.param_values.get(&key) {
                            var_context.field_access_cache.insert(key, value);
                            return Ok(value);
                        }
                    }
                }
                Err(format!(
                    "Unsupported member access: {:?}.{}",
                    object, member
                ))
            }
            Expression::UnaryOp { operator, operand } => {
                let val = Self::compile_expression(
                    builder,
                    operand,
                    var_context,
                    module,
                    update_field_func_id,
                )?;

                use crate::compiler::ast::UnaryOperator;
                match operator {
                    UnaryOperator::Minus => Ok(builder.ins().fneg(val)),
                    UnaryOperator::Not => {
                        let zero = builder.ins().f64const(0.0);
                        let is_zero = builder.ins().fcmp(FloatCC::Equal, val, zero);
                        let one = builder.ins().f64const(1.0);
                        Ok(builder.ins().select(is_zero, one, zero))
                    }
                }
            }
            Expression::MethodCall {
                object,
                method: method_name,
                arguments,
            } => {
                if !matches!(&**object, Expression::Identifier(name) if name == "This") {
                    return Err("JIT only supports method calls on This".to_string());
                }

                if !arguments.is_empty() {
                    return Err("JIT inlining only supports zero-argument methods".to_string());
                }

                if let Some(callee) = var_context
                    .available_methods
                    .iter()
                    .find(|m| &m.name == method_name)
                {
                    if Self::is_inlinable(callee, var_context.method_name) {
                        if callee.body.len() == 1 {
                            if let Statement::Return {
                                value: Some(expr), ..
                            } = &callee.body[0]
                            {
                                return Self::compile_expression(
                                    builder,
                                    expr,
                                    var_context,
                                    module,
                                    update_field_func_id,
                                );
                            }
                        }

                        Err(format!(
                            "Method {} is inlinable but too complex for current implementation",
                            method_name
                        ))
                    } else {
                        Err(format!(
                            "Method {} is not inlinable (too large or has control flow)",
                            method_name
                        ))
                    }
                } else {
                    Err(format!("Method {} not found for inlining", method_name))
                }
            }
            _ => Err(format!("Unsupported expression for JIT: {:?}", expr)),
        }
    }

    pub fn get_function(&self, concept: &str, method: &str) -> Option<*const u8> {
        let key = (concept.to_string(), method.to_string());
        self.compiled_functions.get(&key).copied()
    }

    pub fn get_required_fields_by_key(&self, concept: &str, method_name: &str) -> Vec<String> {
        let key = (concept.to_string(), method_name.to_string());
        self.required_fields_cache
            .get(&key)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_required_fields(&self, method: &Method) -> Vec<String> {
        Self::find_this_fields(method, &[])
    }

    fn has_set_statements(method: &Method) -> bool {
        for stmt in &method.body {
            if matches!(stmt, Statement::Set { .. }) {
                return true;
            }
        }
        false
    }

    fn is_inlinable(method: &Method, caller_name: &str) -> bool {
        if method.body.len() > 10 {
            return false;
        }

        for stmt in &method.body {
            match stmt {
                Statement::If { .. }
                | Statement::RepeatTimes { .. }
                | Statement::RepeatWhile { .. }
                | Statement::ForEach { .. }
                | Statement::When { .. } => {
                    return false;
                }

                Statement::Return {
                    value: Some(expr), ..
                } => {
                    if Self::contains_method_call(expr, caller_name) {
                        return false;
                    }
                }
                _ => {}
            }
        }

        true
    }

    fn contains_method_call(expr: &Expression, method_name: &str) -> bool {
        match expr {
            Expression::MethodCall { method, .. } => method == method_name,
            Expression::BinaryOp { left, right, .. } => {
                Self::contains_method_call(left, method_name)
                    || Self::contains_method_call(right, method_name)
            }
            Expression::UnaryOp { operand, .. } => Self::contains_method_call(operand, method_name),
            _ => false,
        }
    }

    pub fn method_needs_obj_ptr(&self, concept: &str, method_name: &str) -> bool {
        let key = (concept.to_string(), method_name.to_string());
        self.methods_with_set.get(&key).copied().unwrap_or(false)
    }

    fn find_this_fields(method: &Method, available_methods: &[Method]) -> Vec<String> {
        let mut fields = Vec::new();
        for stmt in &method.body {
            Self::find_fields_in_statement(stmt, &mut fields, available_methods);
        }
        fields.sort();
        fields.dedup();
        fields
    }

    fn find_fields_in_statement(
        stmt: &Statement,
        fields: &mut Vec<String>,
        available_methods: &[Method],
    ) {
        match stmt {
            Statement::Return {
                value: Some(expr), ..
            } => {
                Self::find_fields_in_expression(expr, fields, available_methods);
            }
            Statement::Assignment { value, .. } => {
                Self::find_fields_in_expression(value, fields, available_methods);
            }
            Statement::Set { value, .. } => {
                Self::find_fields_in_expression(value, fields, available_methods);
            }
            _ => {}
        }
    }

    fn find_fields_in_expression(
        expr: &Expression,
        fields: &mut Vec<String>,
        available_methods: &[Method],
    ) {
        match expr {
            Expression::MemberAccess { object, member } => {
                if let Expression::Identifier(name) = &**object {
                    if name == "This" {
                        if let Some(callee) = available_methods.iter().find(|m| &m.name == member) {
                            if callee.parameters.is_empty() {
                                for stmt in &callee.body {
                                    Self::find_fields_in_statement(stmt, fields, available_methods);
                                }
                            }
                        } else {
                            fields.push(member.clone());
                        }
                    }
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                Self::find_fields_in_expression(left, fields, available_methods);
                Self::find_fields_in_expression(right, fields, available_methods);
            }
            Expression::UnaryOp { operand, .. } => {
                Self::find_fields_in_expression(operand, fields, available_methods);
            }
            _ => {}
        }
    }
}

impl Default for JitCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn jit_update_field(
    obj_ptr: *const u8,
    field_ptr: *const u8,
    field_len: usize,
    value: f64,
) {
    let rwlock = unsafe { &*(obj_ptr as *const RwLock<HashMap<String, SfxValue>>) };
    let field_slice = unsafe { std::slice::from_raw_parts(field_ptr, field_len) };
    let field_name = unsafe { std::str::from_utf8_unchecked(field_slice) };
    let sfx_value =
        SfxValue::Number(BigDecimal::from_f64(value).unwrap_or_else(|| BigDecimal::from(0)));
    let mut map = rwlock.write().expect("lock poisoned");
    if let Some(existing_val) = map.get_mut(field_name) {
        *existing_val = sfx_value;
    } else {
        map.insert(field_name.to_string(), sfx_value);
    }
}
