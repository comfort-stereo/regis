use crate::bytecode::{BytecodeChunk, BytecodeInstruction};
use crate::list::List;
use crate::value::Value;
use crate::vm_error::VmError;

use std::collections::HashMap;

static DEBUG: bool = false;

pub struct Vm {
    stack: Vec<Value>,
    scopes: ScopeManager,
}

impl Vm {
    pub fn new() -> Vm {
        Vm {
            stack: Vec::new(),
            scopes: ScopeManager::new(),
        }
    }

    pub fn run_chunk(&mut self, chunk: BytecodeChunk) -> Result<(), VmError> {
        let mut line = 0;
        while line < chunk.instructions().len() {
            let instruction = chunk.get(line);
            let mut next = line + 1;

            if DEBUG {
                println!("DEBUG: {} -> {:?}:", line, instruction);
            }

            match instruction {
                BytecodeInstruction::Blank => {}
                BytecodeInstruction::Pop => self.instruction_pop(),
                BytecodeInstruction::PushScope => self.instruction_push_scope(),
                BytecodeInstruction::PopScope => self.instruction_pop_scope(),
                BytecodeInstruction::Duplicate => self.instruction_duplicate(),
                BytecodeInstruction::IsNull => self.instruction_is_null(),
                BytecodeInstruction::BinaryAdd
                | BytecodeInstruction::BinaryDiv
                | BytecodeInstruction::BinaryMul
                | BytecodeInstruction::BinarySub
                | BytecodeInstruction::BinaryGt
                | BytecodeInstruction::BinaryLt
                | BytecodeInstruction::BinaryGte
                | BytecodeInstruction::BinaryLte
                | BytecodeInstruction::BinaryEq
                | BytecodeInstruction::BinaryNeq => {
                    self.instruction_binary_operation(&instruction)?
                }
                BytecodeInstruction::GetIndex => self.instruction_get_index()?,
                BytecodeInstruction::PushNull => self.instruction_push_null(),
                BytecodeInstruction::PushBoolean(value) => self.instruction_push_boolean(*value),
                BytecodeInstruction::PushNumber(value) => self.instruction_push_number(*value),
                BytecodeInstruction::PushVariable(name) => self.instruction_push_variable(name)?,
                BytecodeInstruction::CreateList => self.instruction_create_list(),
                BytecodeInstruction::InPlacePush => self.instruction_in_place_push()?,
                BytecodeInstruction::DeclareVariable(name) => {
                    self.instruction_declare_variable(name)?
                }
                BytecodeInstruction::AssignVariable(name) => {
                    self.instruction_assign_variable(name)?
                }
                BytecodeInstruction::JumpIf(destination) => {
                    if self.pop().to_boolean() {
                        next = *destination;
                    }
                }
                BytecodeInstruction::JumpUnless(destination) => {
                    if !self.pop().to_boolean() {
                        next = *destination;
                    }
                }
                BytecodeInstruction::Jump(destination) => next = *destination,
                BytecodeInstruction::Echo => self.instruction_echo(),
            }

            line = next;
        }

        Ok(())
    }

    fn instruction_declare_variable(&mut self, name: &str) -> Result<(), VmError> {
        self.scopes.declare(name)
    }

    fn instruction_assign_variable(&mut self, name: &str) -> Result<(), VmError> {
        let value = self.pop();
        self.scopes.assign(name, value)
    }

    fn instruction_pop(&mut self) {
        self.pop();
    }

    fn instruction_push_scope(&mut self) {
        self.scopes.push();
    }

    fn instruction_pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn instruction_duplicate(&mut self) {
        let value = self.tos();
        self.push(value);
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

    fn instruction_get_index(&mut self) -> Result<(), VmError> {
        let index = self.pop();
        let target = self.pop();

        self.push(match target {
            Value::List(list) => list.borrow().get(index)?,
            _ => {
                return Err(VmError::InvalidIndexAccess {
                    target_type: target.type_of(),
                    index: index.to_string(),
                })
            }
        });

        Ok(())
    }

    fn instruction_push_boolean(&mut self, value: bool) {
        self.push(Value::Boolean(value));
    }

    fn instruction_push_number(&mut self, value: f64) {
        self.push(Value::Number(value));
    }

    fn instruction_push_variable(&mut self, name: &str) -> Result<(), VmError> {
        self.push(self.var(name)?);
        Ok(())
    }

    fn instruction_create_list(&mut self) {
        self.push(Value::List(List::create()));
    }

    fn instruction_in_place_push(&mut self) -> Result<(), VmError> {
        let value = self.pop();
        let target = self.tos();
        match target {
            Value::List(list) => {
                list.borrow_mut().push(value);
                Ok(())
            }
            _ => Err(VmError::UndefinedBinaryOperation {
                operation: format!("{:?}", BytecodeInstruction::InPlacePush),
                target_type: target.type_of(),
                other_type: value.type_of(),
            }),
        }
    }

    fn instruction_binary_operation(
        &mut self,
        instruction: &BytecodeInstruction,
    ) -> Result<(), VmError> {
        let right = self.pop();
        let left = self.pop();

        let result = match (left.clone(), right.clone()) {
            (Value::Number(left), Value::Number(right)) => match instruction {
                BytecodeInstruction::BinaryAdd => Some(Value::Number(left + right)),
                BytecodeInstruction::BinaryDiv => Some(Value::Number(left / right)),
                BytecodeInstruction::BinaryMul => Some(Value::Number(left * right)),
                BytecodeInstruction::BinarySub => Some(Value::Number(left - right)),
                BytecodeInstruction::BinaryGt => Some(Value::Boolean(left > right)),
                BytecodeInstruction::BinaryLt => Some(Value::Boolean(left < right)),
                BytecodeInstruction::BinaryGte => Some(Value::Boolean(left >= right)),
                BytecodeInstruction::BinaryLte => Some(Value::Boolean(left <= right)),
                _ => None,
            },
            (Value::List(left), Value::List(right)) => match instruction {
                BytecodeInstruction::BinaryAdd => Some(Value::List(left.borrow().concat(right))),
                _ => None,
            },
            _ => None,
        }
        .or_else(|| match instruction {
            BytecodeInstruction::BinaryEq => Some(Value::Boolean(left == right)),
            BytecodeInstruction::BinaryNeq => Some(Value::Boolean(left != right)),
            _ => None,
        });

        if let Some(result) = result {
            self.push(result);
            Ok(())
        } else {
            Err(VmError::UndefinedBinaryOperation {
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

    fn var(&self, name: &str) -> Result<Value, VmError> {
        let value = self.scopes.get(name);
        match value {
            Some(value) => Ok(value.clone()),
            None => Err(VmError::UndefinedVariableAccess { name: name.into() }),
        }
    }
}

struct ScopeManager {
    scopes: Vec<HashMap<String, Value>>,
}

impl ScopeManager {
    pub fn new() -> ScopeManager {
        ScopeManager {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        if self.scopes.len() < 2 {
            panic!("At least one scope must be present to pop.");
        }

        self.scopes.pop();
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }

        None
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), VmError> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.into(), value);
                return Ok(());
            }
        }

        Err(VmError::UndefinedVariableAssignment { name: name.into() })
    }

    pub fn declare(&mut self, name: &str) -> Result<(), VmError> {
        let scope = self.scopes.last_mut().unwrap();
        if scope.insert(name.into(), Value::Null).is_some() {
            return Err(VmError::VariableRedeclaration { name: name.into() });
        }

        Ok(())
    }
}
