mod builtins;
mod capture;
mod function;
mod list;
mod native;
mod object;
mod rid;
mod value;

pub use function::Function;
pub use list::List;
pub use object::Object;
pub use value::{Value, ValueType};

use std::collections::HashMap;

use crate::ast::base::AstModule;
use crate::ast::location::Location;
use crate::ast::Ast;
use crate::bytecode::{
    Bytecode, Environment, ExportLocation, Instruction, Module, Procedure, StackLocation,
    VariableVariant,
};
use crate::error::{RegisError, RegisErrorVariant};
use crate::path::CanonicalPath;
use crate::shared::{SharedImmutable, SharedMutable};

use capture::Capture;
use function::ProcedureVariant;
use native::{ExternalCallContext, ExternalProcedure, ExternalProcedureCallback};

static DEBUG: bool = false;

#[derive(Debug)]
pub struct Interpreter {
    stack: Vec<StackValue>,
    frames: Vec<Frame>,
    modules: HashMap<CanonicalPath, LoadedModule>,
    environment: Environment,
    globals: Vec<Value>,
}

impl Interpreter {
    pub fn new(main: CanonicalPath) -> Self {
        let mut result = Self {
            stack: Vec::new(),
            frames: vec![Frame::new(0, FrameVariant::Module(main.clone()))],
            modules: HashMap::new(),
            environment: Environment::new(main),
            globals: Vec::new(),
        };

        result.add_default_globals();
        result
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
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
        self.add_global(
            name,
            Value::Function(Function::new(ProcedureVariant::External(procedure.into())).into()),
        );
    }

    fn add_default_globals(&mut self) {
        self.add_global_function("@print".into(), 1, builtins::print);
        self.add_global_function("@len".into(), 1, builtins::len);
        self.add_global_function("@import".into(), 1, builtins::import);
    }

    pub fn load_module(&mut self, path: &CanonicalPath) -> Result<(), RegisError> {
        if self.modules.contains_key(&path) {
            return Ok(());
        }

        if let Ok(source) = path.read() {
            let ast = match Ast::<AstModule>::parse_module(&source) {
                Ok(ast) => ast,
                Err(error) => {
                    return Err(RegisError {
                        location: Some(Location {
                            path: Some(path.clone()),
                            ..error.location
                        }),
                        variant: RegisErrorVariant::ParseError {
                            expected: error.expected,
                        },
                    });
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
            Err(RegisError {
                location: None,
                variant: RegisErrorVariant::ModuleDoesNotExistError {
                    path: path.to_string(),
                },
            })
        }
    }

    fn run_module(&mut self, module: SharedImmutable<Module>) -> Result<(), RegisError> {
        // Add the module to the set of loaded modules.
        let loaded = LoadedModule::new(module.clone());
        self.modules.insert(module.path().clone(), loaded);

        // Push a new module frame onto the stack. Store the position we return to to after its
        // evalutated.
        self.frames.push(Frame::new(
            self.top_position(),
            FrameVariant::Module(module.path().clone()),
        ));

        // Allocate space for all local variables.
        for _ in 0..module.environment().variables().len() {
            self.push_value(Value::Null);
        }

        // Run the bytecode instructions.
        self.run_instructions(module.bytecode())?;

        // Pop the module frame.
        let frame = self.frames.pop().unwrap();

        // Discard all local variables allocated for the module.
        while self.stack.len() > frame.position() {
            self.pop_value();
        }

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

        if argument_count > procedure.environment().parameters().len() {
            return Err(RegisError::new(
                None,
                RegisErrorVariant::ArgumentCountError {
                    function_name: function.name().map(|name| name.clone_inner()),
                    required: procedure.environment().parameters().len(),
                    actual: argument_count,
                },
            ));
        }

        // Push a new stack frame for the call. Store the position we return to to after its
        // evalutated.
        {
            let position = self.top_position() - argument_count;
            self.frames
                .push(Frame::new(position, FrameVariant::Call(function.clone())));
        }

        // Arguments should be allocated on the stack already.
        if argument_count > procedure.environment().parameters().len() {
            // If there are extra arguments for the function, pop them off and discard them.
            for _ in procedure.environment().parameters().len()..argument_count {
                self.pop_value();
            }
        }

        // Push all captured variables onto the stack.
        for capture in function.captures() {
            self.push_capture(capture.clone());
        }

        // Allocate space for all other local variables.
        for _ in function.captures().len()..procedure.environment().variables().len() {
            self.push_value(Value::Null);
        }

        // Run the bytecode instructions.
        self.run_instructions(procedure.bytecode())?;

        // Pop the function call frame and discard all allocated variables.
        {
            let frame = self.frames.pop().unwrap();
            // Pop the result of the function call off the top of the stack.
            let result = self.pop_value();
            // Pop and discard all variables allocated for the function call.
            while self.stack.len() > frame.position() {
                self.pop_value();
            }

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

    fn run_instructions(&mut self, bytecode: &Bytecode) -> Result<(), RegisError> {
        let mut position = 0;
        let end = bytecode.instructions().len();

        while position < end {
            let instruction = bytecode
                .instructions()
                .get(position)
                .expect("Undefined bytecode position reached.");
            let mut next = position + 1;

            if DEBUG {
                println!("DEBUG: {} -> {:#?}:", position, instruction);
            }

            match instruction {
                Instruction::Blank => {}
                Instruction::Pop => self.instruction_pop(),
                Instruction::Duplicate => self.instruction_duplicate(),
                Instruction::DuplicateTop(count) => self.instruction_duplicate_top(*count),
                Instruction::Jump(destination) => next = *destination,
                Instruction::JumpIf(destination) => {
                    if self.pop_value().to_boolean() {
                        next = *destination;
                    }
                }
                Instruction::JumpUnless(destination) => {
                    if !self.pop_value().to_boolean() {
                        next = *destination;
                    }
                }
                Instruction::Return => break,
                Instruction::IsNull => self.instruction_is_null(),
                Instruction::PushNull => self.instruction_push_null(),
                Instruction::PushBoolean(value) => self.instruction_push_boolean(*value),
                Instruction::PushInt(value) => self.instruction_push_int(*value),
                Instruction::PushFloat(value) => self.instruction_push_float(*value),
                Instruction::PushString(value) => self.instruction_push_string(value.clone()),
                Instruction::PushVariable(address) => self.instruction_push_variable(*address),
                Instruction::AssignVariable(address) => self.instruction_assign_variable(*address),
                Instruction::PushExport(location) => self.instruction_push_export(location),
                Instruction::AssignExport(location) => self.instruction_assign_export(location),
                Instruction::PushGlobal(address) => self.instruction_push_global(*address),
                Instruction::CreateList(size) => self.instruction_create_list(*size),
                Instruction::CreateObject(size) => self.instruction_create_object(*size),
                Instruction::CreateFunction(function) => {
                    self.instruction_create_function(function.clone())
                }
                Instruction::Call(argument_count) => self.instruction_call(*argument_count)?,
                Instruction::BinaryAdd
                | Instruction::BinaryDiv
                | Instruction::BinaryMul
                | Instruction::BinarySub
                | Instruction::BinaryGt
                | Instruction::BinaryLt
                | Instruction::BinaryGte
                | Instruction::BinaryLte
                | Instruction::BinaryEq
                | Instruction::BinaryNeq => self.instruction_binary_operation(&instruction)?,
                Instruction::GetIndex => self.instruction_get_index()?,
                Instruction::SetIndex => self.instruction_set_index()?,
                Instruction::Push => self.instruction_push()?,
                Instruction::Echo => self.instruction_echo(),
            }

            position = next;
        }

        Ok(())
    }

    fn top_position(&self) -> usize {
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

    fn push_capture(&mut self, capture: SharedMutable<Capture>) {
        self.push_stack_value(StackValue::Capture(capture));
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

    fn instruction_pop(&mut self) {
        self.pop_value();
    }

    fn instruction_duplicate(&mut self) {
        let value = self.top_value();
        self.push_value(value);
    }

    fn instruction_duplicate_top(&mut self, count: usize) {
        for i in self.top_position() - count..self.top_position() {
            self.push_value(self.stack[i].get());
        }
    }

    fn instruction_is_null(&mut self) {
        let value = self.pop_value();
        self.push_value(Value::Boolean(matches!(value, Value::Null)));
    }

    fn instruction_push_null(&mut self) {
        self.push_value(Value::Null);
    }

    fn instruction_push_boolean(&mut self, value: bool) {
        self.push_value(Value::Boolean(value));
    }

    fn instruction_push_int(&mut self, value: i64) {
        self.push_value(Value::Int(value));
    }

    fn instruction_push_float(&mut self, value: f64) {
        self.push_value(Value::Float(value));
    }

    fn instruction_push_string(&mut self, value: SharedImmutable<String>) {
        self.push_value(Value::String(value));
    }

    fn instruction_push_variable(&mut self, address: usize) {
        self.push_value(self.get_variable(address));
    }

    fn instruction_assign_variable(&mut self, address: usize) {
        let value = self.pop_value();
        self.set_variable(address, value);
    }

    fn instruction_push_export(
        &mut self,
        ExportLocation {
            path: module,
            export,
        }: &ExportLocation,
    ) {
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
    }

    fn instruction_assign_export(
        &mut self,
        ExportLocation {
            path: module,
            export,
        }: &ExportLocation,
    ) {
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
    }

    fn instruction_push_global(&mut self, address: usize) {
        self.push_value(self.globals[address].clone());
    }

    fn instruction_create_list(&mut self, size: usize) {
        let mut list = List::new();
        list.reserve(size);
        for _ in 0..size {
            list.push(self.pop_value());
        }

        self.push_value(Value::List(list.into()));
    }

    fn instruction_create_object(&mut self, size: usize) {
        let mut object = Object::new();
        object.reserve(size);
        for _ in 0..size {
            let key = self.pop_value();
            let value = self.pop_value();
            object.set(value.clone(), key.clone());
        }

        self.push_value(Value::Object(object.into()));
    }

    fn instruction_create_function(&mut self, procedure: SharedImmutable<Procedure>) {
        let mut captures = Vec::new();
        for variable in procedure.environment().variables() {
            if let VariableVariant::Capture { location, .. } = &variable.variant {
                if location.ascend != 0 {
                    captures.push(
                        self.capture_value(
                            self.get_variable_position_from_stack_location(location),
                        ),
                    );
                }
            }
        }

        self.push_value(Value::Function(
            Function::with_captures(ProcedureVariant::Internal(procedure), captures).into(),
        ));
    }

    fn instruction_call(&mut self, argument_count: usize) -> Result<(), RegisError> {
        let target = self.pop_value();
        let function = match target {
            Value::Function(function) => function,
            _ => {
                return Err(RegisError::new(
                    None,
                    RegisErrorVariant::UndefinedUnaryOperation {
                        operation: format!("{:?}", Instruction::Call(argument_count)),
                        target_type: target.type_of(),
                    },
                ));
            }
        };

        self.run_function(&function, argument_count)
    }

    fn instruction_binary_operation(
        &mut self,
        instruction: &Instruction,
    ) -> Result<(), RegisError> {
        let right = self.pop_value();
        let left = self.pop_value();

        let result = match (&left, &right) {
            (Value::Int(left), Value::Int(right)) => match instruction {
                Instruction::BinaryAdd => Some(Value::Int(left + right)),
                Instruction::BinaryDiv => Some(Value::Float((*left) as f64 / (*right) as f64)),
                Instruction::BinaryMul => Some(Value::Int(left * right)),
                Instruction::BinarySub => Some(Value::Int(left - right)),
                Instruction::BinaryGt => Some(Value::Boolean(left > right)),
                Instruction::BinaryLt => Some(Value::Boolean(left < right)),
                Instruction::BinaryGte => Some(Value::Boolean(left >= right)),
                Instruction::BinaryLte => Some(Value::Boolean(left <= right)),
                _ => None,
            },
            (Value::Float(left), Value::Float(right)) => match instruction {
                Instruction::BinaryAdd => Some(Value::Float(left + right)),
                Instruction::BinaryDiv => Some(Value::Float(left / right)),
                Instruction::BinaryMul => Some(Value::Float(left * right)),
                Instruction::BinarySub => Some(Value::Float(left - right)),
                Instruction::BinaryGt => Some(Value::Boolean(left > right)),
                Instruction::BinaryLt => Some(Value::Boolean(left < right)),
                Instruction::BinaryGte => Some(Value::Boolean(left >= right)),
                Instruction::BinaryLte => Some(Value::Boolean(left <= right)),
                _ => None,
            },
            (Value::Int(left), Value::Float(right)) => match instruction {
                Instruction::BinaryAdd => Some(Value::Float((*left) as f64 + right)),
                Instruction::BinaryDiv => Some(Value::Float((*left) as f64 / right)),
                Instruction::BinaryMul => Some(Value::Float((*left) as f64 * right)),
                Instruction::BinarySub => Some(Value::Float((*left) as f64 - right)),
                Instruction::BinaryGt => Some(Value::Boolean((*left) as f64 > *right)),
                Instruction::BinaryLt => Some(Value::Boolean((*(left) as f64) < *right)),
                Instruction::BinaryGte => Some(Value::Boolean((*left) as f64 >= *right)),
                Instruction::BinaryLte => Some(Value::Boolean((*left) as f64 <= *right)),
                _ => None,
            },
            (Value::Float(left), Value::Int(right)) => match instruction {
                Instruction::BinaryAdd => Some(Value::Float(left + (*right) as f64)),
                Instruction::BinaryDiv => Some(Value::Float(left / (*right) as f64)),
                Instruction::BinaryMul => Some(Value::Float(left * (*right) as f64)),
                Instruction::BinarySub => Some(Value::Float(left - (*right) as f64)),
                Instruction::BinaryGt => Some(Value::Boolean(*left > (*right) as f64)),
                Instruction::BinaryLt => Some(Value::Boolean(*left < (*right) as f64)),
                Instruction::BinaryGte => Some(Value::Boolean(*left >= (*right) as f64)),
                Instruction::BinaryLte => Some(Value::Boolean(*left <= (*right) as f64)),
                _ => None,
            },
            (Value::List(left), Value::List(right)) => match instruction {
                Instruction::BinaryAdd => Some(Value::List(left.borrow().concat(&right))),
                _ => None,
            },
            (Value::Object(left), Value::Object(right)) => match instruction {
                Instruction::BinaryAdd => Some(Value::Object(left.borrow().concat(&right))),
                _ => None,
            },
            (Value::String(left), right) => match instruction {
                Instruction::BinaryAdd => Some(Value::String(
                    format!("{}{}", *left, right.to_string()).into(),
                )),
                _ => None,
            },
            (left, Value::String(right)) => match instruction {
                Instruction::BinaryAdd => Some(Value::String(
                    format!("{}{}", left.to_string(), *right).into(),
                )),
                _ => None,
            },
            _ => None,
        }
        .or_else(|| match instruction {
            Instruction::BinaryEq => Some(Value::Boolean(left == right)),
            Instruction::BinaryNeq => Some(Value::Boolean(left != right)),
            _ => None,
        });

        if let Some(result) = result {
            self.push_value(result);
            Ok(())
        } else {
            Err(RegisError::new(
                None,
                RegisErrorVariant::UndefinedBinaryOperation {
                    operation: format!("{:?}", instruction),
                    target_type: left.type_of(),
                    other_type: right.type_of(),
                },
            ))
        }
    }

    fn instruction_get_index(&mut self) -> Result<(), RegisError> {
        let index = self.pop_value();
        let target = self.pop_value();
        let value = match target {
            Value::List(list) => list.borrow().get(&index)?,
            Value::Object(object) => object.borrow().get(&index),
            _ => {
                return Err(RegisError::new(
                    None,
                    RegisErrorVariant::TypeError {
                        message: format!("Type '{}' is not indexable.", target.type_of()),
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
                        message: format!("Type '{}' is not indexable.", target.type_of()),
                    },
                ));
            }
        }

        Ok(())
    }

    fn instruction_push(&mut self) -> Result<(), RegisError> {
        let value = self.pop_value();
        let target = self.pop_value();

        match target {
            Value::List(list) => {
                list.borrow_mut().push(value);
            }
            Value::Object(object) => {
                object.borrow_mut().set(value, Value::Null);
            }
            _ => {
                return Err(RegisError::new(
                    None,
                    RegisErrorVariant::TypeError {
                        message: format!(
                            "Operator '[]=' is not defined for type {}.",
                            target.type_of()
                        ),
                    },
                ));
            }
        }

        Ok(())
    }

    fn instruction_echo(&mut self) {
        println!("{}", self.pop_value());
    }
}

#[derive(Debug, Clone)]
enum StackValue {
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
    pub fn new(module: SharedImmutable<Module>) -> Self {
        Self {
            module,
            exports: Object::new().into(),
        }
    }

    // pub fn module(&self) -> &Module {
    //     &self.module
    // }

    pub fn exports(&self) -> &SharedMutable<Object> {
        &self.exports
    }
}
