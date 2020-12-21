/*
pub fn emit(node: &AstNodeVariant, code: &mut BytecodeChunk) {
    match variant {
        AstNodeVariant::Module { statements } => {
            for statement in statements {
                emit(statement, code);
            }
        }
        AstNodeVariant::Null => {
            code.add(Instruction::PushNull);
        }
        AstNodeVariant::Boolean { value } => {
            code.add(Instruction::PushBoolean(*value));
        }
        AstNodeVariant::Number { value, .. } => {
            code.add(Instruction::PushNumber(*value));
        }
        AstNodeVariant::String { value, .. } => {
            code.add(Instruction::PushString(value.clone()));
        }
        AstNodeVariant::Identifier { name } => {
            code.add(Instruction::PushVariable(name.clone()));
        }
        AstNodeVariant::List { values } => {
            for value in values.iter().rev() {
                emit(value, code);
            }
            code.add(Instruction::CreateList(values.len()));
        }
        AstNodeVariant::Dict { pairs } => {
            for pair in pairs.iter().rev() {
                emit(pair, code);
            }
            code.add(Instruction::CreateDict(pairs.len()));
        }
        AstNodeVariant::Pair { key, value } => {
            match key.variant() {
                AstNodeVariant::Identifier { name } => {
                    code.add(Instruction::PushString(name.clone()));
                }
                _ => emit(key, code),
            };
            emit(value, code);
        }
        AstNodeVariant::Function {
            name,
            parameters,
            block,
        } => {
            let mut function_code = BytecodeChunk::new();
            emit(&block, &mut function_code);
            let function = SharedImmutable::new(Function::new(
                Some(name.clone()),
                parameters.clone(),
                SharedImmutable::new(function_code),
            ));

            code.add(Instruction::CreateFunction(function));
        }
        AstNodeVariant::Lambda { parameters, body } => {
            let mut function_code = BytecodeChunk::new();
            emit(&body, &mut function_code);
            let function = SharedImmutable::new(Function::new(
                None,
                parameters.clone(),
                SharedImmutable::new(function_code),
            ));

            code.add(Instruction::CreateFunction(function));
        }
        AstNodeVariant::FunctionStatement { function } => {
            let name = match function.variant() {
                AstNodeVariant::Function { name, .. } => name,
                _ => unreachable!(),
            };

            code.add(Instruction::DeclareVariable(name.clone()));
            emit(&function, code);
            code.add(Instruction::AssignVariable(name.clone()));
        }
        AstNodeVariant::VariableDeclarationStatement { name, value } => {
            code.add(Instruction::DeclareVariable(name.clone()));
            emit(value, code);
            code.add(Instruction::AssignVariable(name.clone()));
        }
        AstNodeVariant::VariableAssignmentStatement {
            name,
            operator,
            value,
        } => {
            if *operator != AssignmentOperator::Direct {
                code.add(Instruction::PushVariable(name.clone()));
            }

            match operator {
                AssignmentOperator::Direct => {
                    emit(value, code);
                }
                AssignmentOperator::Mul => {
                    emit(value, code);
                    code.add(Instruction::BinaryMul);
                }
                AssignmentOperator::Div => {
                    emit(value, code);
                    code.add(Instruction::BinaryDiv);
                }
                AssignmentOperator::Add => {
                    emit(value, code);
                    code.add(Instruction::BinaryAdd);
                }
                AssignmentOperator::Sub => {
                    emit(value, code);
                    code.add(Instruction::BinarySub);
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

            code.add(Instruction::AssignVariable(name.clone()));
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
                code.add(Instruction::GetIndex);
            }

            emit_set_index_value(operator, value, code);
            code.add(Instruction::SetIndex);
        }
        AstNodeVariant::DotAssignmentStatement {
            dot,
            operator,
            value,
        } => {
            let (target, property) = match dot.variant() {
                AstNodeVariant::Dot { target, property } => (target, property),
                _ => unreachable!(),
            };

            emit(target, code);
            code.add(Instruction::PushString(property.clone()));

            if *operator != AssignmentOperator::Direct {
                emit(target, code);
                code.add(Instruction::PushString(property.clone()));
                code.add(Instruction::GetIndex);
            }

            emit_set_index_value(operator, value, code);
            code.add(Instruction::SetIndex);
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
                    Instruction::JumpUnless(code.end()),
                );
                emit(else_statement, code);
                code.set(jump_end, Instruction::Jump(code.end()));
            } else {
                code.set(
                    jump_else_or_end_if_not_true,
                    Instruction::JumpUnless(code.end()),
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
            code.add(Instruction::Jump(start_line));
            code.mark(code.end(), BytecodeMarker::LoopEnd);
        }
        AstNodeVariant::WhileStatement { condition, block } => {
            code.mark(code.end(), BytecodeMarker::LoopStart);
            let start_line = code.end();
            emit(condition, code);
            code.add(Instruction::JumpIf(code.end() + 2));

            code.blank();
            let jump_line = code.last();
            emit(block, code);
            code.add(Instruction::Jump(start_line));

            let end_line = code.end();
            code.mark(end_line, BytecodeMarker::LoopEnd);
            code.set(jump_line, Instruction::Jump(end_line));
        }
        AstNodeVariant::Block { statements } => {
            code.add(Instruction::PushScope);
            for statement in statements {
                emit(statement, code);
            }
            code.add(Instruction::PopScope);
        }
        AstNodeVariant::FunctionBlock { statements } => {
            for statement in statements {
                emit(statement, code);
            }
        }
        AstNodeVariant::ReturnStatement { value } => {
            if let Some(value) = value {
                emit(value, code);
            } else {
                code.add(Instruction::PushNull);
            }

            code.add(Instruction::Return);
        }
        AstNodeVariant::BreakStatement => {
            code.blank();
            code.mark(code.last(), BytecodeMarker::Break);
        }
        AstNodeVariant::ContinueStatement => {
            code.blank();
            code.mark(code.last(), BytecodeMarker::Continue);
        }
        AstNodeVariant::EchoStatement { value } => {
            emit(value, code);
            code.add(Instruction::Echo);
        }
        AstNodeVariant::ExpressionStatement { expression } => {
            emit(expression, code);
            code.add(Instruction::Pop);
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
                BinaryOperator::Mul => Some(Instruction::BinaryMul),
                BinaryOperator::Div => Some(Instruction::BinaryDiv),
                BinaryOperator::Add => Some(Instruction::BinaryAdd),
                BinaryOperator::Sub => Some(Instruction::BinarySub),
                BinaryOperator::Gt => Some(Instruction::BinaryGt),
                BinaryOperator::Lt => Some(Instruction::BinaryLt),
                BinaryOperator::Gte => Some(Instruction::BinaryGte),
                BinaryOperator::Lte => Some(Instruction::BinaryLte),
                BinaryOperator::Eq => Some(Instruction::BinaryEq),
                BinaryOperator::Neq => Some(Instruction::BinaryNeq),
                BinaryOperator::Push => Some(Instruction::BinaryPush),
                BinaryOperator::Ncl | BinaryOperator::And | BinaryOperator::Or => None,
            } {
                emit(left, code);
                emit(right, code);
                code.add(eager);
                return;
            }

            match operator {
                BinaryOperator::Ncl => {
                    emit(left, code);
                    emit_ncl_operation(right, code);
                }
                BinaryOperator::And => {
                    emit(left, code);
                    emit_and_operation(right, code);
                }
                BinaryOperator::Or => {
                    emit(left, code);
                    emit_or_operation(right, code);
                }
                _ => unreachable!(),
            }
        }
        AstNodeVariant::Index { target, index } => {
            emit(target, code);
            emit(index, code);
            code.add(Instruction::GetIndex);
        }
        AstNodeVariant::Dot { target, property } => {
            emit(target, code);
            code.add(Instruction::PushString(property.clone()));
            code.add(Instruction::GetIndex);
        }
        AstNodeVariant::Call { target, arguments } => {
            for argument in arguments.iter().rev() {
                emit(argument, code);
            }
            emit(target, code);
            code.add(Instruction::Call(arguments.len()));
        }
        AstNodeVariant::Unknown => {}
    }
}

fn emit_ncl_operation(value: &Box<AstNode>, code: &mut BytecodeChunk) {
    code.add(Instruction::Duplicate);
    code.add(Instruction::IsNull);
    let jump_end_if_not_null = code.blank();
    code.add(Instruction::Pop);
    emit(value, code);
    code.set(
        jump_end_if_not_null,
        Instruction::JumpUnless(code.end()),
    );
}

fn emit_and_operation(value: &Box<AstNode>, code: &mut BytecodeChunk) {
    code.add(Instruction::Duplicate);
    let jump_end_if_false = code.blank();
    code.add(Instruction::Pop);
    emit(value, code);
    code.set(
        jump_end_if_false,
        Instruction::JumpUnless(code.end()),
    );
}

fn emit_or_operation(value: &Box<AstNode>, code: &mut BytecodeChunk) {
    code.add(Instruction::Duplicate);
    let jump_end_if_true = code.blank();
    code.add(Instruction::Pop);
    emit(value, code);
    code.set(jump_end_if_true, Instruction::JumpIf(code.end()));
}

fn emit_set_index_value(
    operator: &AssignmentOperator,
    value: &Box<AstNode>,
    code: &mut BytecodeChunk,
) {
    match operator {
        AssignmentOperator::Direct => {
            emit(value, code);
        }
        AssignmentOperator::Mul => {
            emit(value, code);
            code.add(Instruction::BinaryMul);
        }
        AssignmentOperator::Div => {
            emit(value, code);
            code.add(Instruction::BinaryDiv);
        }
        AssignmentOperator::Add => {
            emit(value, code);
            code.add(Instruction::BinaryAdd);
        }
        AssignmentOperator::Sub => {
            emit(value, code);
            code.add(Instruction::BinarySub);
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
}
*/
