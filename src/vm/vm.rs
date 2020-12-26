use crate::bytecode::{Bytecode, Instruction, Procedure};
use crate::shared::SharedImmutable;

use super::dict::Dict;
use super::error::VmError;
use super::function::Function;
use super::list::List;
use super::value::Value;
use super::VmErrorVariant;

static DEBUG: bool = false;

#[derive(Debug)]
pub struct Vm {
    stack: Vec<Value>,
    calls: Vec<Call>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            calls: Vec::new(),
        }
    }

    pub fn run(&mut self, bytecode: &Bytecode) -> Result<(), VmError> {
        self.run_with_arguments(bytecode, 0)
    }

    pub fn run_with_arguments(
        &mut self,
        bytecode: &Bytecode,
        argument_count: usize,
    ) -> Result<(), VmError> {
        // Any arguments should be allocated on the stack already.
        // Allocate space for all other variables.
        for _ in argument_count..bytecode.variable_count() {
            self.push(Value::Null);
        }

        let mut position = 0;
        let end = bytecode.size();

        while position < end {
            let instruction = bytecode
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
                    if self.pop().to_boolean() {
                        next = *destination;
                    }
                }
                Instruction::JumpUnless(destination) => {
                    if !self.pop().to_boolean() {
                        next = *destination;
                    }
                }
                Instruction::Return => break,
                Instruction::IsNull => self.instruction_is_null(),
                Instruction::PushNull => self.instruction_push_null(),
                Instruction::PushBoolean(value) => self.instruction_push_boolean(*value),
                Instruction::PushNumber(value) => self.instruction_push_number(*value),
                Instruction::PushString(value) => self.instruction_push_string(value.clone()),
                Instruction::PushVariable(address) => self.instruction_push_variable(*address),
                Instruction::AssignVariable(address) => self.instruction_assign_variable(*address),
                Instruction::CreateList(size) => self.instruction_create_list(*size),
                Instruction::CreateDict(size) => self.instruction_create_dict(*size),
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

    fn size(&self) -> usize {
        self.stack.len()
    }

    fn get(&self, position: usize) -> Value {
        self.stack[position].clone()
    }

    fn set(&mut self, position: usize, value: Value) {
        self.stack[position] = value;
    }

    fn get_variable(&self, address: usize) -> Value {
        let position = self.top_call_position();
        self.get(position + address).clone()
    }

    fn set_variable(&mut self, address: usize, value: Value) {
        let position = self.top_call_position();
        self.set(position + address, value);
    }

    fn push(&mut self, value: Value) {
        if DEBUG {
            println!("DEBUG:   Push -> {:?}", value);
        }

        self.stack.push(value);

        if DEBUG {
            println!("DEBUG:   Size -> {:?}", self.stack.len());
        }
    }

    fn pop(&mut self) -> Value {
        let result = self
            .stack
            .pop()
            .unwrap_or_else(|| panic!("No values exist to be popped off the stack."));

        if DEBUG {
            println!("DEBUG:   Pop  -> {:?}", result);
            println!("DEBUG:   TOS  -> {:?}", self.stack.last());
            println!("DEBUG:   Size -> {:?}", self.stack.len());
        }

        result
    }

    fn top(&self) -> Value {
        let result = self
            .stack
            .last()
            .unwrap_or_else(|| panic!("No value at top of stack."));

        if DEBUG {
            println!("DEBUG:   TOS  -> {:?}", self.stack.last());
            println!("DEBUG:   Size -> {:?}", self.stack.len());
        }

        result.clone()
    }

    fn push_call(&mut self, function: SharedImmutable<Function>, argument_count: usize) {
        let position = self.size();
        let call = Call::new(function, position - argument_count);
        self.calls.push(call);
    }

    fn pop_call(&mut self) -> Call {
        let call = self.calls.pop().unwrap();
        // Pop the result of the function call off the top of the stack.
        let result = self.pop();
        // Pop and discard all variables allocated for the function call.
        while self.stack.len() > call.position() {
            self.pop();
        }

        // Push the result back to the top of the stack.
        self.push(result);
        call
    }

    fn top_call(&self) -> Option<&Call> {
        self.calls.last()
    }

    fn top_call_position(&self) -> usize {
        self.top_call().map_or(0, |call| call.position())
    }

    fn instruction_pop(&mut self) {
        self.pop();
    }

    fn instruction_duplicate(&mut self) {
        let value = self.top();
        self.push(value);
    }

    fn instruction_duplicate_top(&mut self, count: usize) {
        for i in self.size() - count..self.size() {
            self.push(self.stack[i].clone());
        }
    }

    fn instruction_is_null(&mut self) {
        let value = self.pop();
        self.push(Value::Boolean(match value {
            Value::Null => true,
            _ => false,
        }));
    }

    fn instruction_push_null(&mut self) {
        self.push(Value::Null);
    }

    fn instruction_push_boolean(&mut self, value: bool) {
        self.push(Value::Boolean(value));
    }

    fn instruction_push_number(&mut self, value: f64) {
        self.push(Value::Number(value));
    }

    fn instruction_push_string(&mut self, value: SharedImmutable<String>) {
        self.push(Value::String(value));
    }

    fn instruction_push_variable(&mut self, address: usize) {
        self.push(self.get_variable(address));
    }

    fn instruction_assign_variable(&mut self, address: usize) {
        let value = self.pop();
        self.set_variable(address, value);
    }

    fn instruction_create_list(&mut self, size: usize) {
        let mut list = List::new();
        list.reserve(size);
        for _ in 0..size {
            list.push(self.pop());
        }

        self.push(Value::List(list.into()));
    }

    fn instruction_create_dict(&mut self, size: usize) {
        let mut dict = Dict::new();
        dict.reserve(size);
        for _ in 0..size {
            let key = self.pop();
            let value = self.pop();
            dict.set(value.clone(), key.clone());
        }

        self.push(Value::Dict(dict.into()));
    }

    fn instruction_create_function(&mut self, procedure: SharedImmutable<Procedure>) {
        self.push(Value::Function(Function::new(procedure).into()));
    }

    fn instruction_call(&mut self, argument_count: usize) -> Result<(), VmError> {
        let target = self.pop();
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

        self.push_call(function.clone(), argument_count);
        self.run_with_arguments(function.bytecode(), argument_count)?;
        self.pop_call();

        Ok(())
    }

    fn instruction_binary_operation(&mut self, instruction: &Instruction) -> Result<(), VmError> {
        let right = self.pop();
        let left = self.pop();

        let result = match (left.clone(), right.clone()) {
            (Value::Number(left), Value::Number(right)) => match instruction {
                Instruction::BinaryAdd => Some(Value::Number(left + right)),
                Instruction::BinaryDiv => Some(Value::Number(left / right)),
                Instruction::BinaryMul => Some(Value::Number(left * right)),
                Instruction::BinarySub => Some(Value::Number(left - right)),
                Instruction::BinaryGt => Some(Value::Boolean(left > right)),
                Instruction::BinaryLt => Some(Value::Boolean(left < right)),
                Instruction::BinaryGte => Some(Value::Boolean(left >= right)),
                Instruction::BinaryLte => Some(Value::Boolean(left <= right)),
                _ => None,
            },
            (Value::List(left), Value::List(right)) => match instruction {
                Instruction::BinaryAdd => Some(Value::List(left.borrow().concat(&right))),
                _ => None,
            },
            (Value::List(left), right) => match instruction {
                Instruction::BinaryPush => {
                    left.borrow_mut().push(right);
                    Some(Value::List(left))
                }
                _ => None,
            },
            (Value::Dict(left), Value::Dict(right)) => match instruction {
                Instruction::BinaryAdd => Some(Value::Dict(left.borrow().concat(&right))),
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
            self.push(result);
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
        let index = self.pop();
        let target = self.pop();
        let value = match target {
            Value::List(list) => list.borrow().get(index)?,
            Value::Dict(dict) => dict.borrow().get(index),
            _ => {
                return Err(VmError::new(
                    None,
                    VmErrorVariant::InvalidIndexAccess {
                        target_type: target.type_of(),
                        index: index.to_string(),
                    },
                ));
            }
        };

        self.push(value);
        Ok(())
    }

    fn instruction_set_index(&mut self) -> Result<(), VmError> {
        let value = self.pop();
        let index = self.pop();
        let target = self.pop();

        match target {
            Value::List(list) => list.borrow_mut().set(index, value)?,
            Value::Dict(dict) => dict.borrow_mut().set(index, value),
            _ => {
                return Err(VmError::new(
                    None,
                    VmErrorVariant::InvalidIndexAssignment {
                        target_type: target.type_of(),
                        index: index.to_string(),
                    },
                ));
            }
        }

        Ok(())
    }

    fn instruction_echo(&mut self) {
        println!("{}", self.pop().to_string());
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
