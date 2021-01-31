use super::base::*;
use super::expr::*;
use super::node::*;
use super::stmt::*;

pub type TraverseFilter<'a> = fn(current: &Node<'a>) -> TraverseState;

#[derive(Debug, PartialEq, Eq)]
pub enum TraverseState {
    Continue,
    Stop,
    Exit,
}

pub struct Traverse<'a> {
    stack: Vec<Node<'a>>,
    filter: Option<TraverseFilter<'a>>,
}

impl<'a> Traverse<'a> {
    pub fn new(root: Node<'a>) -> Self {
        Self::with_filter(root, None)
    }

    pub fn with_filter(root: Node<'a>, filter: Option<TraverseFilter<'a>>) -> Self {
        Self {
            stack: vec![root],
            filter,
        }
    }
}

impl<'a> Iterator for Traverse<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = match self.stack.pop() {
            Some(current) => current,
            None => return None,
        };

        let state = if let Some(state_function) = self.filter {
            state_function(&current)
        } else {
            TraverseState::Continue
        };

        if state == TraverseState::Exit {
            return None;
        }

        if state == TraverseState::Stop {
            return Some(current);
        }

        match &current {
            // Base
            Node::Chunk(Chunk { stmts, .. }) => {
                self.stack.extend(stmts.iter().map(Node::from_stmt));
            }
            Node::Block(Block { stmts, .. }) => {
                self.stack.extend(stmts.iter().map(Node::from_stmt));
            }
            Node::Ident(..) => {}
            // Expressions
            Node::NullExpr(..) => {}
            Node::BooleanExpr(..) => {}
            Node::IntExpr(..) => {}
            Node::FloatExpr(..) => {}
            Node::StringExpr(..) => {}
            Node::VariableExpr(VariableExpr { name, .. }) => {
                self.stack.push(Node::Ident(name));
            }
            Node::ListExpr(ListExpr { values, .. }) => {
                self.stack.extend(values.iter().map(Node::from_expr));
            }
            Node::ObjectExpr(ObjectExpr { pairs, .. }) => {
                for ObjectExprPair { key, value, .. } in pairs {
                    match key {
                        ObjectExprKeyVariant::Identifier(identifier) => {
                            self.stack.push(Node::Ident(identifier));
                        }
                        ObjectExprKeyVariant::String(string) => {
                            self.stack.push(Node::StringExpr(string));
                        }
                        ObjectExprKeyVariant::Expr(ObjectExprKeyExpr { value, .. }) => {
                            self.stack.push(Node::from_expr(value));
                        }
                    }

                    self.stack.push(Node::from_expr(value));
                }
            }
            Node::FunctionExpr(FunctionExpr {
                name,
                parameters,
                body,
                ..
            }) => {
                if let Some(name) = name {
                    self.stack.push(Node::Ident(&name));
                }
                self.stack
                    .extend(parameters.iter().map(|parameter| Node::Ident(&parameter)));
                self.stack.push(match body {
                    FunctionExprBody::Block(block) => Node::Block(block),
                    FunctionExprBody::Expr(expr) => Node::from_expr(expr),
                });
            }
            Node::WrappedExpr(WrappedExpr { value, .. }) => {
                self.stack.push(Node::from_expr(value));
            }
            Node::IndexExpr(index) => {
                self.stack.push(Node::IndexExpr(&index));
                self.stack.push(Node::from_expr(&index.target));
                self.stack.push(Node::from_expr(&index.index));
            }
            Node::DotExpr(dot) => {
                self.stack.push(Node::DotExpr(&dot));
                self.stack.push(Node::from_expr(&dot.target));
                self.stack.push(Node::Ident(&dot.property));
            }
            Node::CallExpr(call) => {
                self.stack.push(Node::CallExpr(&call));
                self.stack.push(Node::from_expr(&call.target));
                self.stack.extend(
                    call.arguments
                        .iter()
                        .map(|argument| Node::from_expr(argument)),
                );
            }
            Node::UnaryOperationExpr(UnaryOperationExpr { right, .. }) => {
                self.stack.push(Node::from_expr(right));
            }
            Node::BinaryOperationExpr(BinaryOperationExpr { left, right, .. }) => {
                self.stack.push(Node::from_expr(left));
                self.stack.push(Node::from_expr(right));
            }
            // Statements
            Node::IfStmt(IfStmt {
                condition,
                block,
                else_clause,
                ..
            }) => {
                self.stack.push(Node::from_expr(condition));
                self.stack.push(Node::Block(block));
                if let Some(else_clause) = else_clause {
                    self.stack.push(match &else_clause.next {
                        ElseClauseNextVariant::Block(block) => Node::Block(block),
                        ElseClauseNextVariant::IfStmt(if_stmt) => Node::IfStmt(if_stmt),
                    })
                }
            }
            Node::LoopStmt(LoopStmt { block, .. }) => {
                self.stack.push(Node::Block(block));
            }
            Node::WhileStmt(WhileStmt {
                condition, block, ..
            }) => {
                self.stack.push(Node::from_expr(condition));
                self.stack.push(Node::Block(block));
            }
            Node::ReturnStmt(ReturnStmt { value, .. }) => {
                if let Some(value) = value {
                    self.stack.push(Node::from_expr(value));
                }
            }
            Node::BreakStmt(..) => {}
            Node::ContinueStmt(..) => {}
            Node::FunctionStmt(FunctionDeclarationStmt { function, .. }) => {
                self.stack.push(Node::FunctionExpr(function));
            }
            Node::VariableDeclarationStmt(VariableDeclarationStmt { name, value, .. }) => {
                self.stack.push(Node::Ident(name));
                self.stack.push(Node::from_expr(value));
            }
            Node::VariableAssignmentStmt(VariableAssignmentStmt { name, value, .. }) => {
                self.stack.push(Node::Ident(name));
                self.stack.push(Node::from_expr(value));
            }
            Node::IndexAssignmentStmt(IndexAssignmentStmt {
                index_expr, value, ..
            }) => {
                self.stack.push(Node::from_expr(&index_expr.index));
                self.stack.push(Node::from_expr(&value));
            }
            Node::DotAssignmentStmt(DotAssignmentStmt {
                dot_expr, value, ..
            }) => {
                self.stack.push(Node::Ident(&dot_expr.property));
                self.stack.push(Node::from_expr(&value));
            }
            Node::ExprStmt(ExprStmt { expr, .. }) => {
                self.stack.push(Node::from_expr(expr));
            }
        }

        Some(current)
    }
}
