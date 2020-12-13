use crate::bytecode::{BytecodeChunk, BytecodeInstruction};
use crate::list::List;
use crate::value::Value;

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

    pub fn run_chunk(&mut self, chunk: BytecodeChunk) {
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
                    self.instruction_binary_operation(&instruction);
                }
                BytecodeInstruction::PushNull => self.instruction_push_null(),
                BytecodeInstruction::PushBoolean(value) => self.instruction_push_boolean(*value),
                BytecodeInstruction::PushNumber(value) => self.instruction_push_number(*value),
                BytecodeInstruction::PushVariable(name) => self.instruction_push_variable(name),
                BytecodeInstruction::CreateList => self.instruction_create_list(),
                BytecodeInstruction::InPlaceAppend => self.instruction_in_place_append(),
                BytecodeInstruction::DeclareVariable(name) => {
                    self.instruction_declare_variable(name);
                }
                BytecodeInstruction::AssignVariable(name) => self.instruction_assign_variable(name),
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
    }

    fn instruction_declare_variable(&mut self, name: &String) {
        self.scopes.declare(name);
    }

    fn instruction_assign_variable(&mut self, name: &String) {
        let value = self.pop();
        self.scopes.assign(name, value);
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

    fn instruction_push_boolean(&mut self, value: bool) {
        self.push(Value::Boolean(value));
    }

    fn instruction_push_number(&mut self, value: f64) {
        self.push(Value::Number(value));
    }

    fn instruction_push_variable(&mut self, name: &String) {
        self.push(self.var(name));
    }

    fn instruction_create_list(&mut self) {
        self.push(Value::List(List::create()));
    }

    fn instruction_in_place_append(&mut self) {
        let value = self.pop();
        let target = self.tos();
        match target {
            Value::List(list) => {
                list.borrow_mut().append(value);
            }
            _ => {
                panic!("Cannot append to value of type: '{}'", value.type_name())
            }
        }
    }

    fn instruction_binary_operation(&mut self, instruction: &BytecodeInstruction) {
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
            _ => None,
        }
        .or_else(|| match instruction {
            BytecodeInstruction::BinaryEq => Some(Value::Boolean(left == right)),
            BytecodeInstruction::BinaryNeq => Some(Value::Boolean(left != right)),
            _ => None,
        });

        if let Some(result) = result {
            self.push(result);
        } else {
            panic!(
                "Binary operation {:?} is not defined for values {:?} and {:?}.",
                instruction, left, right
            );
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
            .unwrap_or_else(|| panic!("No values at top of stack."));

        if DEBUG {
            println!("DEBUG:   TOS  -> {:?}", self.peek());
            println!("DEBUG:   Size -> {:?}", self.stack.len());
        }

        result.clone()
    }

    fn var(&self, name: &String) -> Value {
        self.scopes
            .get(name)
            .unwrap_or_else(|| panic!("Undefined variable access: {}", name))
            .clone()
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
            panic!("At least one scope must be ");
        }

        self.scopes.pop();
    }

    pub fn get(&self, name: &String) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }

        None
    }

    pub fn assign(&mut self, name: &String, value: Value) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.into(), value);
                return;
            }
        }

        panic!("Assignment to undefined variable: {}", name);
    }

    pub fn declare(&mut self, name: &String) {
        let scope = self.scopes.last_mut().unwrap();
        if scope.insert(name.into(), Value::Null).is_some() {
            panic!("Redeclaration of variable: {}", name);
        }
    }
}
