use std::collections::HashMap;

use crate::ast::parser::parse_module;
use crate::compiler::bytecode::{Bytecode, Instruction};
use crate::compiler::compile_module;
use crate::dict::Dict;
use crate::function::Function;
use crate::interpreter_error::InterpreterError;
use crate::list::List;
use crate::shared::SharedImmutable;
use crate::value::Value;

static DEBUG: bool = false;

#[derive(Debug)]
pub struct Interpreter {
    stack: Vec<Value>,
    frames: Vec<StackFrame>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            frames: vec![StackFrame::new(0)],
        }
    }

    pub fn run_module(&mut self, code: &str) -> Result<(), InterpreterError> {
        let ast = match parse_module(&code) {
            Ok(ast) => ast,
            Err(error) => {
                return Err(InterpreterError::ParseError { error });
            }
        };

        let bytecode = compile_module(&ast);
        self.run_bytecode(&bytecode)
    }

    fn run_bytecode(&mut self, bytecode: &Bytecode) -> Result<(), InterpreterError> {
        let mut line = 0;
        let end = bytecode.len();

        while line < end {
            let instruction = bytecode.get(line);
            let mut next = line + 1;

            if DEBUG {
                println!("DEBUG: {} -> {:?}:", line, instruction);
            }

            match instruction {
                Instruction::Blank => {}
                Instruction::Pop => self.instruction_pop(),
                Instruction::PushScope => self.instruction_push_scope(),
                Instruction::PopScope => self.instruction_pop_scope(),
                Instruction::Duplicate => self.instruction_duplicate(),
                Instruction::DuplicateTop(count) => self.instruction_duplicate_top(*count),
                Instruction::IsNull => self.instruction_is_null(),
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
                Instruction::PushNull => self.instruction_push_null(),
                Instruction::PushBoolean(value) => self.instruction_push_boolean(*value),
                Instruction::PushNumber(value) => self.instruction_push_number(*value),
                Instruction::PushString(value) => self.instruction_push_string(value.clone()),
                Instruction::PushVariable(name) => self.instruction_push_variable(&name)?,
                Instruction::CreateList(size) => self.instruction_create_list(*size),
                Instruction::CreateDict(size) => self.instruction_create_dict(*size),
                Instruction::CreateFunction(function) => {
                    self.instruction_create_function(function.clone())
                }
                Instruction::Call(argument_count) => self.instruction_call(*argument_count)?,
                Instruction::DeclareVariable(name) => {
                    self.instruction_declare_variable(name.clone())?
                }
                Instruction::AssignVariable(name) => {
                    self.instruction_assign_variable(name.clone())?
                }
                Instruction::GetIndex => self.instruction_get_index()?,
                Instruction::SetIndex => self.instruction_set_index()?,
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
                Instruction::Jump(destination) => next = *destination,
                Instruction::Return => break,
                Instruction::Echo => self.instruction_echo(),
            }

            line = next;
        }

        Ok(())
    }

    fn push_frame(&mut self, frame: StackFrame) {
        self.frames.push(frame);
    }

    fn pop_frame(&mut self) -> StackFrame {
        let frame = self
            .frames
            .pop()
            .expect("At least one stack frame must be left on the stack.");
        // Remove all values allocated for the frame except the return value.
        self.stack.resize(frame.position() + 1, Value::Null);
        frame
    }

    fn frame(&self) -> &StackFrame {
        self.frames.last().unwrap()
    }

    fn frame_mut(&mut self) -> &mut StackFrame {
        self.frames.last_mut().unwrap()
    }

    fn instruction_declare_variable(
        &mut self,
        name: SharedImmutable<String>,
    ) -> Result<(), InterpreterError> {
        self.frame_mut().declare(name)
    }

    fn instruction_assign_variable(
        &mut self,
        name: SharedImmutable<String>,
    ) -> Result<(), InterpreterError> {
        let value = self.pop();
        self.frame_mut().assign(name, value)
    }

    fn instruction_get_index(&mut self) -> Result<(), InterpreterError> {
        let index = self.pop();
        let target = self.pop();

        self.push(match target {
            Value::List(list) => list.borrow().get(index)?,
            Value::Dict(dict) => dict.borrow().get(index),
            _ => {
                return Err(InterpreterError::InvalidIndexAccess {
                    target_type: target.type_of(),
                    index: index.to_string(),
                })
            }
        });

        Ok(())
    }

    fn instruction_set_index(&mut self) -> Result<(), InterpreterError> {
        let value = self.pop();
        let index = self.pop();
        let target = self.pop();

        match target {
            Value::List(list) => list.borrow_mut().set(index, value)?,
            Value::Dict(dict) => dict.borrow_mut().set(index, value),
            _ => {
                return Err(InterpreterError::InvalidIndexAssignment {
                    target_type: target.type_of(),
                    index: index.to_string(),
                })
            }
        }

        Ok(())
    }

    fn instruction_pop(&mut self) {
        self.pop();
    }

    fn instruction_push_scope(&mut self) {
        self.frame_mut().push_scope();
    }

    fn instruction_pop_scope(&mut self) {
        self.frame_mut().pop_scope();
    }

    fn instruction_duplicate(&mut self) {
        let value = self.tos();
        self.push(value);
    }

    fn instruction_duplicate_top(&mut self, count: usize) {
        for i in self.stack.len() - count..self.stack.len() {
            self.push(self.stack[i].clone());
        }
    }

    fn instruction_echo(&mut self) {
        println!("{}", self.pop().to_string());
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

    fn instruction_push_variable(
        &mut self,
        name: &SharedImmutable<String>,
    ) -> Result<(), InterpreterError> {
        self.push(self.var(name)?);
        Ok(())
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

    fn instruction_create_function(&mut self, function: SharedImmutable<Function>) {
        self.push(Value::Function(function));
    }

    fn instruction_call(&mut self, argument_count: usize) -> Result<(), InterpreterError> {
        let target = self.pop();
        let function = match target {
            Value::Function(function) => function,
            _ => {
                return Err(InterpreterError::UndefinedUnaryOperation {
                    operation: format!("{:?}", Instruction::Call(argument_count)),
                    target_type: target.type_of(),
                })
            }
        };

        let mut frame = StackFrame::new(self.stack.len() - argument_count);
        if let Some(name) = function.name() {
            frame.declare(name.clone())?;
            frame.assign(name.clone(), Value::Function(function.clone()))?;
        }

        for parameter in function.parameters() {
            frame.declare(parameter.clone())?
        }

        for parameter in function.parameters()[0..argument_count].iter() {
            frame.assign(parameter.clone(), self.pop())?
        }

        self.push_frame(frame);
        self.run_bytecode(function.bytecode())?;
        self.pop_frame();

        Ok(())
    }

    fn instruction_binary_operation(
        &mut self,
        instruction: &Instruction,
    ) -> Result<(), InterpreterError> {
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
            Err(InterpreterError::UndefinedBinaryOperation {
                operation: format!("{:?}", instruction),
                target_type: left.type_of(),
                other_type: right.type_of(),
            })
        }
    }

    fn peek(&self) -> Option<&Value> {
        self.stack.last()
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
            println!("DEBUG:   TOS  -> {:?}", self.peek());
            println!("DEBUG:   Size -> {:?}", self.stack.len());
        }

        result
    }

    fn tos(&self) -> Value {
        let result = self
            .stack
            .last()
            .unwrap_or_else(|| panic!("No value at top of stack."));

        if DEBUG {
            println!("DEBUG:   TOS  -> {:?}", self.peek());
            println!("DEBUG:   Size -> {:?}", self.stack.len());
        }

        result.clone()
    }

    fn var(&self, name: &SharedImmutable<String>) -> Result<Value, InterpreterError> {
        let value = self.frame().get(name);
        match value {
            Some(value) => Ok(value.clone()),
            None => Err(InterpreterError::UndefinedVariableAccess {
                name: (**name).clone(),
            }),
        }
    }
}

#[derive(Debug)]
struct StackFrame {
    position: usize,
    scopes: Vec<HashMap<SharedImmutable<String>, Value>>,
}

impl StackFrame {
    fn new(position: usize) -> Self {
        Self {
            position,
            scopes: vec![HashMap::new()],
        }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() == 1 {
            panic!("Cannot pop, only one scope is present on the stack.");
        }

        self.scopes.pop();
    }

    pub fn get(&self, name: &SharedImmutable<String>) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }

        None
    }

    pub fn assign(
        &mut self,
        name: SharedImmutable<String>,
        value: Value,
    ) -> Result<(), InterpreterError> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(&name) {
                scope.insert(name, value);
                return Ok(());
            }
        }

        Err(InterpreterError::UndefinedVariableAssignment {
            name: (*name).clone(),
        })
    }

    pub fn declare(&mut self, name: SharedImmutable<String>) -> Result<(), InterpreterError> {
        let scope = self.scopes.last_mut().unwrap();
        if scope.insert(name.clone(), Value::Null).is_some() {
            return Err(InterpreterError::VariableRedeclaration {
                name: (*name).clone(),
            });
        }

        Ok(())
    }
}
