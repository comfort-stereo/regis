mod closure;
mod error;
mod function;
mod list;
mod object;
mod rid;
mod value;

pub use error::{VmError, VmErrorVariant};
pub use function::Function;
pub use list::List;
pub use object::Object;
pub use value::{Value, ValueType};

use crate::bytecode::{Bytecode, Instruction, Procedure, VariableVariant};
use crate::shared::{SharedImmutable, SharedMutable};

use closure::Capture;

static DEBUG: bool = true;

#[derive(Debug, Clone)]
enum StackValue {
    Value(Value),
    Capture(SharedMutable<Capture>),
}

impl StackValue {
    pub fn get(&self) -> Value {
        match self {
            StackValue::Value(value) => value.clone(),
            StackValue::Capture(capture) => capture.borrow().value.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Vm {
    stack: Vec<StackValue>,
    calls: Vec<Call>,
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            calls: Vec::new(),
        }
    }

    pub fn run_module(&mut self, bytecode: &Bytecode) -> Result<(), VmError> {
        // Allocate space for all variables.
        for _ in 0..bytecode.variables().len() {
            self.push_value(Value::Null);
        }

        // Run the bytecode instructions.
        self.run_instructions(bytecode)
    }

    pub fn run_function(
        &mut self,
        function: &SharedImmutable<Function>,
        argument_count: usize,
    ) -> Result<(), VmError> {
        // Push a new call onto the stack. Store the position we return to to after its finished.
        {
            let position = self.top_position() - argument_count;
            self.calls.push(Call::new(function.clone(), position));
        }

        // Arguments should be allocated on the stack already.
        {
            if argument_count > function.bytecode().parameters().len() {
                // If there are extra arguments for the function, pop them off and discard them.
                for _ in function.bytecode().parameters().len()..argument_count {
                    self.pop_value();
                }
            } else {
                // If there are missing arguments for the function push null to replace them.
                for _ in argument_count..function.bytecode().parameters().len() {
                    self.push_value(Value::Null);
                }
            }
        }

        // Push all captured variables onto the stack.
        for capture in function.captures() {
            self.push_capture(capture.clone());
        }

        // Allocate space for local variables.
        for _ in function.captures().len()..function.bytecode().variables().len() {
            self.push_value(Value::Null);
        }

        // Run the bytecode instructions.
        self.run_instructions(&function.bytecode())?;

        // Pop the function call and discard all allocated variables.
        {
            let call = self.calls.pop().unwrap();
            // Pop the result of the function call off the top of the stack.
            let result = self.pop_value();
            // Pop and discard all variables allocated for the function call.
            while self.stack.len() > call.position() {
                self.pop_value();
            }

            // Push the result back to the top of the stack.
            self.push_value(result);
        }

        Ok(())
    }

    fn run_instructions(&mut self, bytecode: &Bytecode) -> Result<(), VmError> {
        let mut position = 0;
        let end = bytecode.instructions().len();

        while position < end {
            let instruction = bytecode
                .instructions()
                .get(position)
                .expect("Undefined bytecode position reached.");
            let mut next = position + 1;

            if DEBUG {
                println!("DEBUG: {} -> {:?}:", position, instruction);
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
                Instruction::PushCapturedVariable(address) => {
                    self.instruction_push_variable(*address)
                }
                Instruction::AssignCapturedVariable(address) => {
                    self.instruction_assign_variable(*address)
                }
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
                | Instruction::BinaryNeq
                | Instruction::BinaryPush => self.instruction_binary_operation(&instruction)?,
                Instruction::GetIndex => self.instruction_get_index()?,
                Instruction::SetIndex => self.instruction_set_index()?,
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
        match &self.stack[position] {
            StackValue::Value(..) => self.stack[position] = StackValue::Value(value),
            StackValue::Capture(capture) => capture.borrow_mut().value = value,
        }
    }

    fn capture_value(&mut self, position: usize) -> SharedMutable<Capture> {
        match self.stack[position].clone() {
            StackValue::Value(value) => {
                let capture = SharedMutable::new(Capture { value });
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
            println!("DEBUG:   Push -> {:?}", value);
        }

        self.stack.push(value);

        if DEBUG {
            println!("DEBUG:   Size -> {:?}", self.stack.len());
        }
    }

    fn pop_value(&mut self) -> Value {
        let result = self
            .stack
            .pop()
            .unwrap_or_else(|| panic!("No values exist to be popped off the stack."));

        if DEBUG {
            println!("DEBUG:   Pop  -> {:?}", result);
            println!("DEBUG:   TOS  -> {:?}", self.stack.last());
            println!("DEBUG:   Size -> {:?}", self.stack.len());
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
        let position = self.top_call_position();
        self.get_value(position + address)
    }

    fn set_variable(&mut self, address: usize, value: Value) {
        let position = self.top_call_position();
        self.set_value(position + address, value);
    }

    fn top_call(&self) -> Option<&Call> {
        self.calls.last()
    }

    fn top_call_position(&self) -> usize {
        self.top_call().map_or(0, |call| call.position())
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
        for variable in procedure.bytecode().variables() {
            if let VariableVariant::Capture { offset } = variable.variant {
                captures.push(self.capture_value(self.top_position() - offset));
            }
        }

        self.push_value(Value::Function(Function::new(procedure, captures).into()));
    }

    fn instruction_call(&mut self, argument_count: usize) -> Result<(), VmError> {
        let target = self.pop_value();
        let function = match target {
            Value::Function(function) => function,
            _ => {
                return Err(VmError::new(
                    None,
                    VmErrorVariant::UndefinedUnaryOperation {
                        operation: format!("{:?}", Instruction::Call(argument_count)),
                        target_type: target.type_of(),
                    },
                ));
            }
        };

        self.run_function(&function, argument_count)
    }

    fn instruction_binary_operation(&mut self, instruction: &Instruction) -> Result<(), VmError> {
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
            (Value::List(left), right) => match instruction {
                Instruction::BinaryPush => {
                    left.borrow_mut().push(right.clone());
                    Some(Value::List(left.clone()))
                }
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
            Err(VmError::new(
                None,
                VmErrorVariant::UndefinedBinaryOperation {
                    operation: format!("{:?}", instruction),
                    target_type: left.type_of(),
                    other_type: right.type_of(),
                },
            ))
        }
    }

    fn instruction_get_index(&mut self) -> Result<(), VmError> {
        let index = self.pop_value();
        let target = self.pop_value();
        let value = match target {
            Value::List(list) => list.borrow().get(index)?,
            Value::Object(object) => object.borrow().get(index),
            _ => {
                return Err(VmError::new(
                    None,
                    VmErrorVariant::TypeError {
                        message: format!("Type '{}' is not indexable.", target.type_of()),
                    },
                ));
            }
        };

        self.push_value(value);
        Ok(())
    }

    fn instruction_set_index(&mut self) -> Result<(), VmError> {
        let value = self.pop_value();
        let index = self.pop_value();
        let target = self.pop_value();

        match target {
            Value::List(list) => list.borrow_mut().set(index, value)?,
            Value::Object(object) => object.borrow_mut().set(index, value),
            _ => {
                return Err(VmError::new(
                    None,
                    VmErrorVariant::TypeError {
                        message: format!("Type '{}' is not indexable.", target.type_of()),
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

#[derive(Debug)]
struct Call {
    function: SharedImmutable<Function>,
    position: usize,
}

impl Call {
    fn new(function: SharedImmutable<Function>, position: usize) -> Self {
        Self { function, position }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    // pub fn function(&self) -> &SharedImmutable<Function> {
    //     &self.function
    // }
}
