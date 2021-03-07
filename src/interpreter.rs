mod builtins;
mod capture;
mod function;
mod list;
mod native;
mod object;
mod rid;
mod value;

pub use self::function::Function;
pub use self::list::List;
pub use self::object::Object;
pub use self::value::{Value, ValueType};

use std::collections::HashMap;

use crate::bytecode::{
    Bytecode, Environment, ExportLocation, Instruction, Module, Procedure, StackLocation,
    VariableVariant,
};
use crate::error::{RegisError, RegisErrorVariant};
use crate::lexer::Symbol;
use crate::parser::Parser;
use crate::shared::{SharedImmutable, SharedMutable};
use crate::source::{CanonicalPath, Location};

use self::capture::Capture;
use self::function::ProcedureVariant;
use self::native::{ExternalCallContext, ExternalProcedure, ExternalProcedureCallback};
use self::rid::Rid;

static DEBUG: bool = false;

#[derive(Debug)]
pub struct Interpreter {
    stack: Vec<StackValue>,
    frames: Vec<Frame>,
    modules: HashMap<CanonicalPath, LoadedModule>,
    environment: Environment,
    globals: Vec<Value>,
    next_id: Rid,
}

#[allow(clippy::unnecessary_wraps)]
impl Interpreter {
    pub fn new(main: CanonicalPath) -> Self {
        let mut result = Self {
            stack: Vec::new(),
            frames: vec![Frame::new(0, FrameVariant::Module(main.clone()))],
            modules: HashMap::new(),
            environment: Environment::new(main),
            globals: Vec::new(),
            next_id: Rid::new(),
        };

        result.add_default_globals();
        result
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    pub fn generate_id(&mut self) -> Rid {
        let id = self.next_id;
        self.next_id = self.next_id.next();
        id
    }

    pub fn add_global(&mut self, name: String, value: Value) {
        let address = self.environment.add_global(name.into());
        if address == self.globals.len() {
            self.globals.push(value);
        } else {
            self.globals[address] = value;
        }
    }

    pub fn add_global_function(
        &mut self,
        name: String,
        arity: usize,
        callback: ExternalProcedureCallback,
    ) {
        let procedure = ExternalProcedure::new(SharedImmutable::new(name.clone()), arity, callback);
        let function = Value::Function(
            Function::new(
                self.generate_id(),
                ProcedureVariant::External(procedure.into()),
            )
            .into(),
        );

        self.add_global(name, function);
    }

    fn add_default_globals(&mut self) {
        self.add_global_function("@print".into(), 1, builtins::print);
        self.add_global_function("@println".into(), 1, builtins::println);
        self.add_global_function("@len".into(), 1, builtins::len);
        self.add_global_function("@import".into(), 1, builtins::import);
        self.add_global_function("@sleep".into(), 1, builtins::sleep);
    }

    pub fn load_module(&mut self, path: &CanonicalPath) -> Result<(), RegisError> {
        if self.modules.contains_key(&path) {
            return Ok(());
        }

        if let Ok(source) = path.read() {
            let ast = match Parser::new(&source).parse() {
                Ok(ast) => ast,
                Err(error) => {
                    return Err(RegisError::new(
                        Some(Location::new(Some(path.clone()), *error.span())),
                        RegisErrorVariant::ParseError {
                            message: error.to_string(),
                        },
                    ));
                }
            };

            let module = Module::build(
                path.clone(),
                &ast,
                self.environment().for_module(path.clone()),
            )
            .into();

            self.run_module(module)
        } else {
            Err(RegisError::new(
                None,
                RegisErrorVariant::ModuleDoesNotExistError {
                    path: path.to_string(),
                },
            ))
        }
    }

    fn run_module(&mut self, module: SharedImmutable<Module>) -> Result<(), RegisError> {
        // Add the module to the set of loaded modules.
        let loaded = LoadedModule::new(self.generate_id(), module.clone());
        self.modules.insert(module.path().clone(), loaded);

        // Push a new module frame onto the stack. Store the position we return to to after its
        // evalutated.
        self.frames.push(Frame::new(
            self.top(),
            FrameVariant::Module(module.path().clone()),
        ));

        // Allocate space for all local variables.
        for _ in 0..module.environment().variables().len() {
            self.push_value(Value::Null);
        }

        // Run the bytecode instructions.
        self.run_bytecode(module.bytecode(), module.environment())?;

        // Pop the module frame.
        let frame = self.frames.pop().unwrap();

        // Discard all local variables allocated for the module.
        self.pop_values_to(frame.position());

        Ok(())
    }

    fn run_function(
        &mut self,
        function: &SharedImmutable<Function>,
        argument_count: usize,
    ) -> Result<(), RegisError> {
        let procedure = match function.procedure() {
            ProcedureVariant::Internal(internal) => internal,
            ProcedureVariant::External(external) => {
                let result = self.call_external_procedure(external, argument_count)?;
                self.push_value(result);
                return Ok(());
            }
        };

        let parameter_count = procedure.environment().parameters().len();
        if parameter_count > argument_count {
            return Err(RegisError::new(
                None,
                RegisErrorVariant::ArgumentCountError {
                    function_name: function.name().map(|name| name.clone_inner()),
                    required: parameter_count,
                    actual: argument_count,
                },
            ));
        }

        // Arguments should be allocated on the stack already.
        if argument_count > parameter_count {
            // If there are extra arguments for the function, pop them off and discard them.
            self.pop_values(argument_count - parameter_count);
        }

        // Push a new stack frame for the call. Store the position we return to to after its
        // evalutated.
        {
            let position = self.top() - parameter_count;
            self.frames
                .push(Frame::new(position, FrameVariant::Call(function.clone())));
        }

        // Initialize all variables.
        self.push_stack_values(function.init());

        // Run the bytecode instructions.
        self.run_bytecode(procedure.bytecode(), procedure.environment())?;

        // Pop the function call frame and discard all allocated variables.
        {
            let frame = self.frames.pop().unwrap();
            // Pop the result of the function call off the top of the stack.
            let result = self.pop_value();
            // Pop and discard all variables allocated for the function call.
            self.pop_values_to(frame.position());
            // Push the result back to the top of the stack.
            self.push_value(result);
        }

        Ok(())
    }

    fn call_external_procedure(
        &mut self,
        procedure: &ExternalProcedure,
        argument_count: usize,
    ) -> Result<Value, RegisError> {
        if argument_count < procedure.arity() {
            let name = procedure.name();
            return Err(RegisError::new(
                None,
                RegisErrorVariant::ArgumentCountError {
                    function_name: Some(name.clone_inner()),
                    required: procedure.arity(),
                    actual: argument_count,
                },
            ));
        }
        let mut arguments = Vec::with_capacity(procedure.arity());
        for _ in 0..argument_count {
            arguments.push(self.pop_value());
        }

        procedure.call(
            &arguments[..argument_count],
            &mut ExternalCallContext { interpreter: self },
        )
    }

    fn run_bytecode(
        &mut self,
        bytecode: &Bytecode,
        environment: &Environment,
    ) -> Result<(), RegisError> {
        let mut ptr = Some(0);
        let instructions = bytecode.instructions();

        while let Some(start) = ptr {
            if start >= instructions.len() {
                break;
            }

            ptr.take();

            for (i, instruction) in instructions[start..].iter().enumerate() {
                let result = match instruction {
                    Instruction::Blank => Ok(()),
                    Instruction::Pop => self.instruction_pop(),
                    Instruction::Duplicate => self.instruction_duplicate(),
                    Instruction::DuplicateTop(count) => self.instruction_duplicate_top(*count),
                    Instruction::Jump(destination) => {
                        ptr.replace(*destination);
                        break;
                    }
                    Instruction::JumpIf(destination) => {
                        if self.pop_value().to_boolean() {
                            ptr.replace(*destination);
                            break;
                        }

                        Ok(())
                    }
                    Instruction::JumpUnless(destination) => {
                        if !self.pop_value().to_boolean() {
                            ptr.replace(*destination);
                            break;
                        }

                        Ok(())
                    }
                    Instruction::Return => return Ok(()),
                    Instruction::IsNull => self.instruction_is_null(),
                    Instruction::PushNull => self.instruction_push_null(),
                    Instruction::PushBoolean(value) => self.instruction_push_boolean(*value),
                    Instruction::PushInt(value) => self.instruction_push_int(*value),
                    Instruction::PushFloat(value) => self.instruction_push_float(*value),
                    Instruction::PushString(value) => self.instruction_push_string(value.clone()),
                    Instruction::PushVariable(address) => self.instruction_push_variable(*address),
                    Instruction::AssignVariable(address) => {
                        self.instruction_assign_variable(*address)
                    }
                    Instruction::PushExport(location) => self.instruction_push_export(location),
                    Instruction::AssignExport(location) => self.instruction_assign_export(location),
                    Instruction::PushGlobal(address) => self.instruction_push_global(*address),
                    Instruction::CreateList(size) => self.instruction_create_list(*size),
                    Instruction::CreateObject(size) => self.instruction_create_object(*size),
                    Instruction::CreateFunction(procedure) => {
                        self.instruction_create_function(procedure.clone())
                    }
                    Instruction::Call(argument_count) => self.instruction_call(*argument_count),
                    Instruction::UnaryNeg => self.instruction_unary_neg(),
                    Instruction::UnaryBitNot => self.instruction_unary_bit_not(),
                    Instruction::UnaryNot => self.instruction_unary_not(),
                    Instruction::BinaryAdd => self.instruction_binary_add(),
                    Instruction::BinarySub => self.instruction_binary_sub(),
                    Instruction::BinaryMul => self.instruction_binary_mul(),
                    Instruction::BinaryDiv => self.instruction_binary_div(),
                    Instruction::BinaryShl => self.instruction_binary_shl(),
                    Instruction::BinaryShr => self.instruction_binary_shr(),
                    Instruction::BinaryBitAnd => self.instruction_binary_bit_and(),
                    Instruction::BinaryBitOr => self.instruction_binary_bit_or(),
                    Instruction::BinaryLt => self.instruction_binary_lt(),
                    Instruction::BinaryGt => self.instruction_binary_gt(),
                    Instruction::BinaryLte => self.instruction_binary_lte(),
                    Instruction::BinaryGte => self.instruction_binary_gte(),
                    Instruction::BinaryEq => self.instruction_binary_eq(),
                    Instruction::BinaryNeq => self.instruction_binary_neq(),
                    Instruction::GetIndex => self.instruction_get_index(),
                    Instruction::SetIndex => self.instruction_set_index(),
                };

                if let Err(error) = result {
                    let location = error.location().clone().unwrap_or_else(|| {
                        Location::new(
                            Some(environment.path().clone()),
                            bytecode.spans()[start + i],
                        )
                    });
                    let variant = error.variant().clone();

                    return Err(RegisError::new(Some(location), variant));
                }
            }
        }

        Ok(())
    }

    fn top(&self) -> usize {
        self.stack.len()
    }

    fn get_value(&self, position: usize) -> Value {
        self.stack[position].get()
    }

    fn set_value(&mut self, position: usize, value: Value) {
        match &mut self.stack[position] {
            StackValue::Value(..) => self.stack[position] = StackValue::Value(value),
            StackValue::Capture(capture) => capture.borrow_mut().set(value),
        }
    }

    fn get_variable_position_from_stack_location(
        &self,
        StackLocation { ascend, address }: &StackLocation,
    ) -> usize {
        if *ascend >= self.frames.len() {
            *address
        } else {
            self.frames
                .get(self.frames.len() - 1 - ascend)
                .map_or(0, |frame| frame.position())
                + address
        }
    }

    fn capture_value(&mut self, position: usize) -> SharedMutable<Capture> {
        match self.stack[position].clone() {
            StackValue::Value(value) => {
                let capture = SharedMutable::new(Capture::new(value));
                self.stack[position] = StackValue::Capture(capture.clone());
                capture
            }
            StackValue::Capture(capture) => capture,
        }
    }

    fn push_value(&mut self, value: Value) {
        self.push_stack_value(StackValue::Value(value));
    }

    fn push_stack_value(&mut self, value: StackValue) {
        if DEBUG {
            println!("DEBUG:   Push -> {:#?}", value);
        }

        self.stack.push(value);

        if DEBUG {
            println!("DEBUG:   Size -> {:#?}", self.stack.len());
        }
    }

    fn push_stack_values(&mut self, values: &[StackValue]) {
        if DEBUG {
            for value in values {
                println!("DEBUG:   Push -> {:#?}", value);
            }
        }

        self.stack.extend_from_slice(values);

        if DEBUG {
            println!("DEBUG:   Size -> {:#?}", self.stack.len());
        }
    }

    fn pop_value(&mut self) -> Value {
        let result = self
            .stack
            .pop()
            .unwrap_or_else(|| panic!("No values exist to be popped off the stack."));

        if DEBUG {
            println!("DEBUG:   Pop  -> {:#?}", result);
            println!("DEBUG:   TOS  -> {:#?}", self.stack.last());
            println!("DEBUG:   Size -> {:#?}", self.stack.len());
        }

        result.get()
    }

    fn pop_values(&mut self, count: usize) {
        self.pop_values_to(self.top() - count);
    }

    fn pop_values_to(&mut self, position: usize) {
        if DEBUG {
            for value in self.stack.iter().rev().take(self.top() - position) {
                println!("DEBUG:   Pop -> {:#?}", value);
            }
        }

        self.stack.truncate(position);

        if DEBUG {
            println!("DEBUG:   Size -> {:#?}", self.stack.len());
        }
    }

    fn top_value(&self) -> Value {
        let result = self
            .stack
            .last()
            .unwrap_or_else(|| panic!("No value at top of stack."));

        if DEBUG {
            println!("DEBUG:   TOS  -> {:?}", self.stack.last());
            println!("DEBUG:   Size -> {:?}", self.stack.len());
        }

        result.get()
    }

    fn get_variable(&self, address: usize) -> Value {
        let position = self.top_frame_position();
        self.get_value(position + address)
    }

    fn set_variable(&mut self, address: usize, value: Value) {
        let position = self.top_frame_position();
        self.set_value(position + address, value);
    }

    fn top_frame(&self) -> Option<&Frame> {
        self.frames.last()
    }

    fn top_frame_position(&self) -> usize {
        self.top_frame().map_or(0, |frame| frame.position())
    }

    fn instruction_pop(&mut self) -> Result<(), RegisError> {
        self.pop_value();
        Ok(())
    }

    fn instruction_duplicate(&mut self) -> Result<(), RegisError> {
        let value = self.top_value();
        self.push_value(value);
        Ok(())
    }

    fn instruction_duplicate_top(&mut self, count: usize) -> Result<(), RegisError> {
        for i in self.top() - count..self.top() {
            self.push_value(self.stack[i].get());
        }

        Ok(())
    }

    fn instruction_is_null(&mut self) -> Result<(), RegisError> {
        let value = self.pop_value();
        self.push_value(Value::Boolean(matches!(value, Value::Null)));
        Ok(())
    }

    fn instruction_push_null(&mut self) -> Result<(), RegisError> {
        self.push_value(Value::Null);
        Ok(())
    }

    fn instruction_push_boolean(&mut self, value: bool) -> Result<(), RegisError> {
        self.push_value(Value::Boolean(value));
        Ok(())
    }

    fn instruction_push_int(&mut self, value: i64) -> Result<(), RegisError> {
        self.push_value(Value::Int(value));
        Ok(())
    }

    fn instruction_push_float(&mut self, value: f64) -> Result<(), RegisError> {
        self.push_value(Value::Float(value));
        Ok(())
    }

    fn instruction_push_string(
        &mut self,
        value: SharedImmutable<String>,
    ) -> Result<(), RegisError> {
        self.push_value(Value::String(value));
        Ok(())
    }

    fn instruction_push_variable(&mut self, address: usize) -> Result<(), RegisError> {
        self.push_value(self.get_variable(address));
        Ok(())
    }

    fn instruction_assign_variable(&mut self, address: usize) -> Result<(), RegisError> {
        let value = self.pop_value();
        self.set_variable(address, value);
        Ok(())
    }

    fn instruction_push_export(
        &mut self,
        ExportLocation {
            path: module,
            export,
        }: &ExportLocation,
    ) -> Result<(), RegisError> {
        let value = self
            .modules
            .get(module)
            .map(|module| {
                module
                    .exports()
                    .borrow()
                    .get(&Value::String(export.clone()))
            })
            .unwrap_or_else(|| {
                panic!(
                    "Attempted to push export variable {} which does not exist.",
                    export,
                )
            });

        self.push_value(value);
        Ok(())
    }

    fn instruction_assign_export(
        &mut self,
        ExportLocation {
            path: module,
            export,
        }: &ExportLocation,
    ) -> Result<(), RegisError> {
        let value = self.pop_value();
        self.modules
            .get(module)
            .map(|module| {
                module
                    .exports()
                    .borrow_mut()
                    .set(Value::String(export.clone()), value);
            })
            .unwrap_or_else(|| {
                panic!(
                    "Attempted to assign export variable {} to module {} which does not exist.",
                    export, module,
                )
            });
        Ok(())
    }

    fn instruction_push_global(&mut self, address: usize) -> Result<(), RegisError> {
        self.push_value(self.globals[address].clone());
        Ok(())
    }

    fn instruction_create_list(&mut self, size: usize) -> Result<(), RegisError> {
        let mut list = List::new(self.generate_id());
        list.reserve(size);
        for _ in 0..size {
            list.push(self.pop_value());
        }

        self.push_value(Value::List(list.into()));
        Ok(())
    }

    fn instruction_create_object(&mut self, size: usize) -> Result<(), RegisError> {
        let mut object = Object::new(self.generate_id());
        object.reserve(size);
        for _ in 0..size {
            let key = self.pop_value();
            let value = self.pop_value();
            object.set(value.clone(), key.clone());
        }

        self.push_value(Value::Object(object.into()));
        Ok(())
    }

    fn instruction_create_function(
        &mut self,
        procedure: SharedImmutable<Procedure>,
    ) -> Result<(), RegisError> {
        let init = procedure
            .environment()
            .variables()
            .iter()
            .map(|variable| match &variable.variant {
                VariableVariant::Local => StackValue::Value(Value::Null),
                VariableVariant::Capture { location } => StackValue::Capture(
                    self.capture_value(self.get_variable_position_from_stack_location(location)),
                ),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let function = Value::Function(
            Function::with_init(
                self.generate_id(),
                ProcedureVariant::Internal(procedure),
                init,
            )
            .into(),
        );

        self.push_value(function);
        Ok(())
    }

    fn instruction_call(&mut self, argument_count: usize) -> Result<(), RegisError> {
        let target = self.pop_value();
        let function = match target {
            Value::Function(function) => function,
            _ => {
                return Err(RegisError::new(
                    None,
                    RegisErrorVariant::TypeError {
                        message: format!("Type '{}' is not callable.", target.type_of()),
                    },
                ));
            }
        };

        self.run_function(&function, argument_count)
    }

    fn run_errorable_unary_operation<O: Fn(&mut Self, Value) -> Result<Value, RegisError>>(
        &mut self,
        operation: O,
    ) -> Result<(), RegisError> {
        let right = self.pop_value();
        operation(self, right).map(|result| self.push_value(result))
    }

    fn run_non_errorable_unary_operation<O: Fn(&mut Self, Value) -> Value>(
        &mut self,
        operation: O,
    ) {
        let right = self.pop_value();
        let result = operation(self, right);
        self.push_value(result);
    }

    fn run_errorable_binary_operation<
        O: Fn(&mut Self, Value, Value) -> Result<Value, RegisError>,
    >(
        &mut self,
        operation: O,
    ) -> Result<(), RegisError> {
        let right = self.pop_value();
        let left = self.pop_value();
        operation(self, left, right).map(|result| self.push_value(result))
    }

    fn run_non_errorable_binary_operation<O: Fn(&mut Self, Value, Value) -> Value>(
        &mut self,
        operation: O,
    ) {
        let right = self.pop_value();
        let left = self.pop_value();
        let result = operation(self, left, right);
        self.push_value(result);
    }

    fn instruction_unary_neg(&mut self) -> Result<(), RegisError> {
        self.run_errorable_unary_operation(|_, right| {
            Ok(match right {
                Value::Int(int) => Value::Int(-int),
                Value::Float(float) => Value::Float(-float),
                _ => return Err(unary_operation_error(Symbol::Sub.text(), right)),
            })
        })
    }

    fn instruction_unary_bit_not(&mut self) -> Result<(), RegisError> {
        self.run_errorable_unary_operation(|_, right| {
            Ok(match right {
                Value::Int(int) => Value::Int(!int),
                _ => return Err(unary_operation_error(Symbol::BitNot.text(), right)),
            })
        })
    }

    fn instruction_unary_not(&mut self) -> Result<(), RegisError> {
        self.run_non_errorable_unary_operation(|_, right| Value::Boolean(!right.to_boolean()));
        Ok(())
    }

    fn instruction_binary_add(&mut self) -> Result<(), RegisError> {
        self.run_errorable_binary_operation(|this, left, right| {
            Ok(match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left.wrapping_add(right)),
                (Value::Int(left), Value::Float(right)) => Value::Float(left as f64 + right),
                (Value::Float(left), Value::Float(right)) => Value::Float(left + right),
                (Value::Float(left), Value::Int(right)) => Value::Float(left + right as f64),
                (Value::List(left), Value::List(right)) => {
                    Value::List(left.borrow().concat(&right.borrow(), this.generate_id()))
                }
                (Value::Object(left), Value::Object(right)) => {
                    Value::Object(left.borrow().concat(&right.borrow(), this.generate_id()))
                }
                (Value::String(left), right) => {
                    Value::String(format!("{}{}", left, right.to_string()).into())
                }
                (left, Value::String(right)) => {
                    Value::String(format!("{}{}", left.to_string(), right).into())
                }
                (left, right) => {
                    return Err(binary_operation_error(Symbol::Add.text(), left, right))
                }
            })
        })
    }

    fn instruction_binary_sub(&mut self) -> Result<(), RegisError> {
        self.run_errorable_binary_operation(|_, left, right| {
            Ok(match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left.wrapping_sub(right)),
                (Value::Int(left), Value::Float(right)) => Value::Float(left as f64 - right),
                (Value::Float(left), Value::Float(right)) => Value::Float(left - right),
                (Value::Float(left), Value::Int(right)) => Value::Float(left - right as f64),
                (left, right) => {
                    return Err(binary_operation_error(Symbol::Sub.text(), left, right))
                }
            })
        })
    }

    fn instruction_binary_mul(&mut self) -> Result<(), RegisError> {
        self.run_errorable_binary_operation(|_, left, right| {
            Ok(match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left.wrapping_mul(right)),
                (Value::Int(left), Value::Float(right)) => Value::Float(left as f64 * right),
                (Value::Float(left), Value::Float(right)) => Value::Float(left * right),
                (Value::Float(left), Value::Int(right)) => Value::Float(left * right as f64),
                (left, right) => {
                    return Err(binary_operation_error(Symbol::Mul.text(), left, right))
                }
            })
        })
    }

    fn instruction_binary_div(&mut self) -> Result<(), RegisError> {
        self.run_errorable_binary_operation(|_, left, right| {
            Ok(match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left.wrapping_div(right)),
                (Value::Int(left), Value::Float(right)) => Value::Float(left as f64 / right),
                (Value::Float(left), Value::Float(right)) => Value::Float(left / right),
                (Value::Float(left), Value::Int(right)) => Value::Float(left / right as f64),
                (left, right) => {
                    return Err(binary_operation_error(Symbol::Div.text(), left, right))
                }
            })
        })
    }

    fn instruction_binary_shl(&mut self) -> Result<(), RegisError> {
        self.run_errorable_binary_operation(|_, left, right| {
            Ok(match (left, right) {
                (Value::Int(left), Value::Int(right)) => {
                    // TODO: Check to make right hand side is correct.
                    Value::Int(left.wrapping_shl(right as u32))
                }
                (Value::List(left), right) => {
                    left.borrow_mut().push(right);
                    Value::List(left)
                }
                (Value::Object(left), right) => {
                    left.borrow_mut().set(right, Value::Null);
                    Value::Object(left)
                }
                (left, right) => {
                    return Err(binary_operation_error(Symbol::Shl.text(), left, right))
                }
            })
        })
    }

    fn instruction_binary_shr(&mut self) -> Result<(), RegisError> {
        self.run_errorable_binary_operation(|_, left, right| {
            Ok(match (left, right) {
                (Value::Int(left), Value::Int(right)) => {
                    // TODO: Check to make right hand side is correct.
                    Value::Int(left.wrapping_shr(right as u32))
                }
                (left, right) => {
                    return Err(binary_operation_error(Symbol::Shr.text(), left, right))
                }
            })
        })
    }

    fn instruction_binary_bit_and(&mut self) -> Result<(), RegisError> {
        self.run_errorable_binary_operation(|_, left, right| {
            Ok(match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left & right),
                (left, right) => {
                    return Err(binary_operation_error(Symbol::BitAnd.text(), left, right))
                }
            })
        })
    }

    fn instruction_binary_bit_or(&mut self) -> Result<(), RegisError> {
        self.run_errorable_binary_operation(|_, left, right| {
            Ok(match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left | right),
                (left, right) => {
                    return Err(binary_operation_error(Symbol::BitOr.text(), left, right))
                }
            })
        })
    }

    fn instruction_binary_lt(&mut self) -> Result<(), RegisError> {
        self.run_errorable_binary_operation(|_, left, right| {
            Ok(match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Boolean(left < right),
                (Value::Int(left), Value::Float(right)) => Value::Boolean((left as f64) < right),
                (Value::Float(left), Value::Float(right)) => Value::Boolean(left < right),
                (Value::Float(left), Value::Int(right)) => Value::Boolean(left < (right as f64)),
                (left, right) => {
                    return Err(binary_operation_error(Symbol::Lt.text(), left, right))
                }
            })
        })
    }

    fn instruction_binary_gt(&mut self) -> Result<(), RegisError> {
        self.run_errorable_binary_operation(|_, left, right| {
            Ok(match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Boolean(left > right),
                (Value::Int(left), Value::Float(right)) => Value::Boolean((left as f64) > right),
                (Value::Float(left), Value::Float(right)) => Value::Boolean(left > right),
                (Value::Float(left), Value::Int(right)) => Value::Boolean(left > (right as f64)),
                (left, right) => {
                    return Err(binary_operation_error(Symbol::Gt.text(), left, right))
                }
            })
        })
    }

    fn instruction_binary_lte(&mut self) -> Result<(), RegisError> {
        self.run_errorable_binary_operation(|_, left, right| {
            Ok(match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Boolean(left <= right),
                (Value::Int(left), Value::Float(right)) => Value::Boolean((left as f64) <= right),
                (Value::Float(left), Value::Float(right)) => Value::Boolean(left <= right),
                (Value::Float(left), Value::Int(right)) => Value::Boolean(left <= (right as f64)),
                (left, right) => {
                    return Err(binary_operation_error(Symbol::Lte.text(), left, right))
                }
            })
        })
    }

    fn instruction_binary_gte(&mut self) -> Result<(), RegisError> {
        self.run_errorable_binary_operation(|_, left, right| {
            Ok(match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Boolean(left >= right),
                (Value::Int(left), Value::Float(right)) => Value::Boolean((left as f64) >= right),
                (Value::Float(left), Value::Float(right)) => Value::Boolean(left >= right),
                (Value::Float(left), Value::Int(right)) => Value::Boolean(left >= (right as f64)),
                (left, right) => {
                    return Err(binary_operation_error(Symbol::Gte.text(), left, right))
                }
            })
        })
    }

    fn instruction_binary_eq(&mut self) -> Result<(), RegisError> {
        self.run_non_errorable_binary_operation(|_, left, right| Value::Boolean(left == right));
        Ok(())
    }

    fn instruction_binary_neq(&mut self) -> Result<(), RegisError> {
        self.run_non_errorable_binary_operation(|_, left, right| Value::Boolean(left != right));
        Ok(())
    }

    fn instruction_get_index(&mut self) -> Result<(), RegisError> {
        let index = self.pop_value();
        let target = self.pop_value();
        let value = match target {
            Value::String(string) => {
                if let Value::Int(int) = index {
                    let positive = int as usize;
                    if int < 0 || positive >= string.len() {
                        Value::Null
                    } else {
                        let character = if string.is_ascii() {
                            string.as_bytes()[positive] as char
                        } else {
                            string.chars().nth(positive).unwrap()
                        };

                        Value::String(character.to_string().into())
                    }
                } else {
                    return Err(RegisError::new(
                        None,
                        RegisErrorVariant::TypeError {
                            message: format!(
                                "String cannot be indexed by type '{}', only '{}' is allowed.",
                                index.type_of(),
                                ValueType::Int
                            ),
                        },
                    ));
                }
            }
            Value::List(list) => list.borrow().get(&index)?,
            Value::Object(object) => object.borrow().get(&index),
            _ => {
                return Err(RegisError::new(
                    None,
                    RegisErrorVariant::TypeError {
                        message: format!("Cannot get index of type '{}'.", target.type_of()),
                    },
                ));
            }
        };

        self.push_value(value);
        Ok(())
    }

    fn instruction_set_index(&mut self) -> Result<(), RegisError> {
        let value = self.pop_value();
        let index = self.pop_value();
        let target = self.pop_value();

        match target {
            Value::List(list) => list.borrow_mut().set(index, value)?,
            Value::Object(object) => object.borrow_mut().set(index, value),
            _ => {
                return Err(RegisError::new(
                    None,
                    RegisErrorVariant::TypeError {
                        message: format!("Cannot set index of type '{}'.", target.type_of()),
                    },
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum StackValue {
    Value(Value),
    Capture(SharedMutable<Capture>),
}

impl StackValue {
    pub fn get(&self) -> Value {
        match self {
            StackValue::Value(value) => value.clone(),
            StackValue::Capture(capture) => capture.borrow().get().clone(),
        }
    }
}

#[derive(Debug)]
struct Frame {
    position: usize,
    variant: FrameVariant,
}

#[derive(Debug)]
enum FrameVariant {
    Call(SharedImmutable<Function>),
    Module(CanonicalPath),
}

impl Frame {
    fn new(position: usize, variant: FrameVariant) -> Self {
        Self { position, variant }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn variant(&self) -> &FrameVariant {
        &self.variant
    }
}

#[derive(Debug)]
struct LoadedModule {
    module: SharedImmutable<Module>,
    exports: SharedMutable<Object>,
}

impl LoadedModule {
    pub fn new(id: Rid, module: SharedImmutable<Module>) -> Self {
        Self {
            module,
            exports: Object::new(id).into(),
        }
    }

    // pub fn module(&self) -> &Module {
    //     &self.module
    // }

    pub fn exports(&self) -> &SharedMutable<Object> {
        &self.exports
    }
}

fn unary_operation_error(operator: &'static str, right: Value) -> RegisError {
    RegisError::new(
        None,
        RegisErrorVariant::UndefinedUnaryOperation {
            operator: operator.into(),
            right_type: right.type_of(),
        },
    )
}

fn binary_operation_error(operator: &'static str, left: Value, right: Value) -> RegisError {
    RegisError::new(
        None,
        RegisErrorVariant::UndefinedBinaryOperation {
            operator: operator.into(),
            left_type: left.type_of(),
            right_type: right.type_of(),
        },
    )
}
