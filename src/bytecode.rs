use crate::parser::{AssignmentOperator, AstNode, AstNodeVariant, BinaryOperator};
use std::fmt::Formatter;
use std::fmt::{Debug, Result};

use std::collections::{BTreeMap, HashSet};
use std::hash::Hash;

#[derive(Debug, Clone)]
pub enum BytecodeInstruction {
    Blank,
    Pop,
    Duplicate,
    PushScope,
    PopScope,
    IsNull,
    BinaryAdd,
    BinaryDiv,
    BinaryMul,
    BinarySub,
    BinaryGt,
    BinaryLt,
    BinaryGte,
    BinaryLte,
    BinaryEq,
    BinaryNeq,
    GetIndex,
    SetIndex,
    PushNull,
    PushBoolean(bool),
    PushNumber(f64),
    CreateList,
    InPlacePush,
    PushVariable(String),
    DeclareVariable(String),
    AssignVariable(String),
    JumpIf(usize),
    JumpUnless(usize),
    Jump(usize),
    Echo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BytecodeMarker {
    LoopStart,
    LoopEnd,
    Break,
    Continue,
}

pub struct BytecodeChunk {
    instructions: Vec<BytecodeInstruction>,
    markers: BTreeMap<usize, HashSet<BytecodeMarker>>,
}

impl Debug for BytecodeChunk {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        formatter
            .debug_map()
            .entries(
                self.instructions()
                    .iter()
                    .enumerate()
                    .map(|(line, instruction)| (instruction, self.get_markers(line)))
                    .enumerate(),
            )
            .finish()
    }
}

impl BytecodeChunk {
    pub fn new() -> BytecodeChunk {
        BytecodeChunk {
            instructions: Vec::new(),
            markers: BTreeMap::new(),
        }
    }

    pub fn instructions(&self) -> &Vec<BytecodeInstruction> {
        &self.instructions
    }

    pub fn get(&self, line: usize) -> &BytecodeInstruction {
        &self.instructions[line]
    }

    pub fn set(&mut self, line: usize, instruction: BytecodeInstruction) {
        self.instructions[line] = instruction;
    }

    pub fn add(&mut self, instruction: BytecodeInstruction) {
        self.instructions.push(instruction);
    }

    pub fn last(&self) -> usize {
        self.instructions.len() - 1
    }

    pub fn end(&self) -> usize {
        self.instructions.len()
    }

    pub fn blank(&mut self) -> usize {
        self.add(BytecodeInstruction::Blank);
        self.instructions.len() - 1
    }

    pub fn mark(&mut self, line: usize, marker: BytecodeMarker) {
        if !self.markers.contains_key(&line) {
            self.markers.insert(line, HashSet::new());
        }

        self.markers
            .get_mut(&line)
            .map(|group| group.insert(marker));
    }

    pub fn has_marker(&self, line: usize, marker: BytecodeMarker) -> bool {
        self.markers
            .get(&line)
            .map(|group| group.contains(&marker))
            .unwrap_or(false)
    }

    pub fn get_markers(&self, line: usize) -> HashSet<BytecodeMarker> {
        if let Some(markers) = self.markers.get(&line) {
            return markers.clone();
        }

        HashSet::new()
    }
}

pub fn emit(node: &Box<AstNode>, code: &mut BytecodeChunk) {
    let variant = node.variant();
    match variant {
        AstNodeVariant::Module { statements } => {
            for statement in statements {
                emit(statement, code);
            }
        }
        AstNodeVariant::Null => {
            code.add(BytecodeInstruction::PushNull);
        }
        AstNodeVariant::Boolean { value } => {
            code.add(BytecodeInstruction::PushBoolean(*value));
        }
        AstNodeVariant::Number { value, .. } => {
            code.add(BytecodeInstruction::PushNumber(*value));
        }
        AstNodeVariant::Identifier { name } => {
            code.add(BytecodeInstruction::PushVariable(name.into()));
        }
        AstNodeVariant::List { values } => {
            code.add(BytecodeInstruction::CreateList);
            for value in values {
                emit(value, code);
                code.add(BytecodeInstruction::InPlacePush);
            }
        }
        AstNodeVariant::VariableDeclarationStatement { name, value } => {
            code.add(BytecodeInstruction::DeclareVariable(name.clone()));
            emit(value, code);
            code.add(BytecodeInstruction::AssignVariable(name.into()));
        }
        AstNodeVariant::VariableAssignmentStatement {
            name,
            operator,
            value,
        } => {
            if *operator != AssignmentOperator::Direct {
                code.add(BytecodeInstruction::PushVariable(name.into()));
            }

            match operator {
                AssignmentOperator::Direct => {
                    emit(value, code);
                }
                AssignmentOperator::Mul => {
                    emit(value, code);
                    code.add(BytecodeInstruction::BinaryMul);
                }
                AssignmentOperator::Div => {
                    emit(value, code);
                    code.add(BytecodeInstruction::BinaryDiv);
                }
                AssignmentOperator::Add => {
                    emit(value, code);
                    code.add(BytecodeInstruction::BinaryAdd);
                }
                AssignmentOperator::Sub => {
                    emit(value, code);
                    code.add(BytecodeInstruction::BinarySub);
                }
                AssignmentOperator::And => {
                    emit_and_operation(value, code);
                }
                AssignmentOperator::Or => {
                    emit_or_operation(value, code);
                }
                AssignmentOperator::Ncl => {
                    emit_ncl_operation(value, code);
                }
            }

            code.add(BytecodeInstruction::AssignVariable(name.into()));
        }
        AstNodeVariant::IndexAssignmentStatement {
            index,
            operator,
            value,
        } => {
            let (target, index) = match index.variant() {
                AstNodeVariant::Index { target, index } => (target, index),
                _ => unreachable!(),
            };

            emit(target, code);
            emit(index, code);

            if *operator != AssignmentOperator::Direct {
                emit(target, code);
                emit(index, code);
                code.add(BytecodeInstruction::GetIndex);
            }

            match operator {
                AssignmentOperator::Direct => {
                    emit(value, code);
                }
                AssignmentOperator::Mul => {
                    emit(value, code);
                    code.add(BytecodeInstruction::BinaryMul);
                }
                AssignmentOperator::Div => {
                    emit(value, code);
                    code.add(BytecodeInstruction::BinaryDiv);
                }
                AssignmentOperator::Add => {
                    emit(value, code);
                    code.add(BytecodeInstruction::BinaryAdd);
                }
                AssignmentOperator::Sub => {
                    emit(value, code);
                    code.add(BytecodeInstruction::BinarySub);
                }
                AssignmentOperator::And => {
                    emit_and_operation(value, code);
                }
                AssignmentOperator::Or => {
                    emit_or_operation(value, code);
                }
                AssignmentOperator::Ncl => {
                    emit_ncl_operation(value, code);
                }
            }

            code.add(BytecodeInstruction::SetIndex);
        }
        AstNodeVariant::IfStatement {
            condition,
            block,
            else_statement,
        } => {
            emit(condition, code);
            let jump_else_or_end_if_not_true = code.blank();
            emit(block, code);

            if let Some(else_statement) = else_statement {
                let jump_end = code.blank();
                code.set(
                    jump_else_or_end_if_not_true,
                    BytecodeInstruction::JumpUnless(code.end()),
                );
                emit(else_statement, code);
                code.set(jump_end, BytecodeInstruction::Jump(code.end()));
            } else {
                code.set(
                    jump_else_or_end_if_not_true,
                    BytecodeInstruction::JumpUnless(code.end()),
                );
            }
        }
        AstNodeVariant::ElseStatement { next } => {
            emit(next, code);
        }
        AstNodeVariant::LoopStatement { block } => {
            code.mark(code.end(), BytecodeMarker::LoopStart);
            let start_line = code.end();
            emit(block, code);
            code.add(BytecodeInstruction::Jump(start_line));
            code.mark(code.end(), BytecodeMarker::LoopEnd);
        }
        AstNodeVariant::WhileStatement { condition, block } => {
            code.mark(code.end(), BytecodeMarker::LoopStart);
            let start_line = code.end();
            emit(condition, code);
            code.add(BytecodeInstruction::JumpIf(code.end() + 2));

            code.blank();
            let jump_line = code.last();
            emit(block, code);
            code.add(BytecodeInstruction::Jump(start_line));

            let end_line = code.end();
            code.mark(end_line, BytecodeMarker::LoopEnd);
            code.set(jump_line, BytecodeInstruction::Jump(end_line));
        }
        AstNodeVariant::BreakStatement => {
            code.blank();
            code.mark(code.last(), BytecodeMarker::Break);
        }
        AstNodeVariant::ContinueStatement => {
            code.blank();
            code.mark(code.last(), BytecodeMarker::Continue);
        }
        AstNodeVariant::Block { statements } => {
            code.add(BytecodeInstruction::PushScope);
            for statement in statements {
                emit(statement, code);
            }
            code.add(BytecodeInstruction::PopScope);
        }
        AstNodeVariant::EchoStatement { value } => {
            emit(value, code);
            code.add(BytecodeInstruction::Echo);
        }
        AstNodeVariant::Wrapped { value } => {
            emit(value, code);
        }
        AstNodeVariant::BinaryOperation {
            left,
            operator,
            right,
        } => {
            if let Some(eager) = match operator {
                BinaryOperator::Mul => Some(BytecodeInstruction::BinaryMul),
                BinaryOperator::Div => Some(BytecodeInstruction::BinaryDiv),
                BinaryOperator::Add => Some(BytecodeInstruction::BinaryAdd),
                BinaryOperator::Sub => Some(BytecodeInstruction::BinarySub),
                BinaryOperator::Gt => Some(BytecodeInstruction::BinaryGt),
                BinaryOperator::Lt => Some(BytecodeInstruction::BinaryLt),
                BinaryOperator::Gte => Some(BytecodeInstruction::BinaryGte),
                BinaryOperator::Lte => Some(BytecodeInstruction::BinaryLte),
                BinaryOperator::Eq => Some(BytecodeInstruction::BinaryEq),
                BinaryOperator::Neq => Some(BytecodeInstruction::BinaryNeq),
                BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Ncl => None,
            } {
                emit(left, code);
                emit(right, code);
                code.add(eager);
                return;
            }

            match operator {
                BinaryOperator::And => {
                    emit(left, code);
                    emit_and_operation(right, code);
                }
                BinaryOperator::Or => {
                    emit(left, code);
                    emit_or_operation(right, code);
                }
                BinaryOperator::Ncl => {
                    emit(left, code);
                    emit_ncl_operation(right, code);
                }
                _ => unreachable!(),
            }
        }
        AstNodeVariant::Index { target, index } => {
            emit(target, code);
            emit(index, code);
            code.add(BytecodeInstruction::GetIndex);
        }
    }
}

fn emit_and_operation(value: &Box<AstNode>, code: &mut BytecodeChunk) {
    code.add(BytecodeInstruction::Duplicate);
    let jump_end_if_false = code.blank();
    code.add(BytecodeInstruction::Pop);
    emit(value, code);
    code.set(
        jump_end_if_false,
        BytecodeInstruction::JumpUnless(code.end()),
    );
}

fn emit_or_operation(value: &Box<AstNode>, code: &mut BytecodeChunk) {
    code.add(BytecodeInstruction::Duplicate);
    let jump_end_if_true = code.blank();
    code.add(BytecodeInstruction::Pop);
    emit(value, code);
    code.set(jump_end_if_true, BytecodeInstruction::JumpIf(code.end()));
}

fn emit_ncl_operation(value: &Box<AstNode>, code: &mut BytecodeChunk) {
    code.add(BytecodeInstruction::Duplicate);
    code.add(BytecodeInstruction::IsNull);
    let jump_end_if_not_null = code.blank();
    code.add(BytecodeInstruction::Pop);
    emit(value, code);
    code.set(
        jump_end_if_not_null,
        BytecodeInstruction::JumpUnless(code.end()),
    );
}

fn finalize(code: &mut BytecodeChunk) {
    for i in 0..=code.end() {
        if code.has_marker(i, BytecodeMarker::Break) {
            let mut depth = 0;
            for j in i..=code.end() {
                if code.has_marker(j, BytecodeMarker::LoopStart) {
                    depth += 1;
                } else if code.has_marker(j, BytecodeMarker::LoopEnd) {
                    if depth == 0 {
                        code.set(i, BytecodeInstruction::Jump(j));
                        println!("GOOOOD");
                        break;
                    }

                    depth -= 1;
                }
            }
        } else if code.has_marker(i, BytecodeMarker::Continue) {
            let mut depth = 0;
            for j in (0..=i).rev() {
                if code.has_marker(j, BytecodeMarker::LoopEnd) {
                    depth += 1;
                } else if code.has_marker(j, BytecodeMarker::LoopStart) {
                    if depth == 0 {
                        code.set(i, BytecodeInstruction::Jump(j));
                        break;
                    }

                    depth -= 1
                }
            }
        }
    }
}

pub fn compile(node: &Box<AstNode>) -> BytecodeChunk {
    let mut code = BytecodeChunk::new();
    emit(&node, &mut code);
    finalize(&mut code);

    code
}
