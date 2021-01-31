use uuid::Uuid;

use crate::source::Span;

use super::base::*;
use super::expr::*;
use super::stmt::*;

#[derive(Debug, Clone)]
pub struct NodeInfo {
    id: Uuid,
    span: Span,
}

impl NodeInfo {
    pub fn new(span: Span) -> Self {
        Self {
            id: Uuid::new_v4(),
            span,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}

#[derive(Debug)]
pub enum Node<'a> {
    // Base
    Chunk(&'a Chunk),
    Block(&'a Block),
    Ident(&'a Ident),
    // Exprs
    NullExpr(&'a NullExpr),
    BooleanExpr(&'a BooleanExpr),
    IntExpr(&'a IntExpr),
    FloatExpr(&'a FloatExpr),
    StringExpr(&'a StringExpr),
    VariableExpr(&'a VariableExpr),
    ListExpr(&'a ListExpr),
    ObjectExpr(&'a ObjectExpr),
    FunctionExpr(&'a FunctionExpr),
    WrappedExpr(&'a WrappedExpr),
    IndexExpr(&'a IndexExpr),
    DotExpr(&'a DotExpr),
    CallExpr(&'a CallExpr),
    UnaryOperationExpr(&'a UnaryOperationExpr),
    BinaryOperationExpr(&'a BinaryOperationExpr),
    // Stmts
    IfStmt(&'a IfStmt),
    LoopStmt(&'a LoopStmt),
    WhileStmt(&'a WhileStmt),
    ReturnStmt(&'a ReturnStmt),
    BreakStmt(&'a BreakStmt),
    ContinueStmt(&'a ContinueStmt),
    FunctionStmt(&'a FunctionDeclarationStmt),
    VariableDeclarationStmt(&'a VariableDeclarationStmt),
    VariableAssignmentStmt(&'a VariableAssignmentStmt),
    IndexAssignmentStmt(&'a IndexAssignmentStmt),
    DotAssignmentStmt(&'a DotAssignmentStmt),
    ExprStmt(&'a ExprStmt),
}

impl<'a> Node<'a> {
    pub fn from_expr(expr: &'a Expr) -> Self {
        match expr {
            Expr::Null(expr) => Self::NullExpr(expr),
            Expr::Boolean(expr) => Self::BooleanExpr(expr),
            Expr::Int(expr) => Self::IntExpr(expr),
            Expr::Float(expr) => Self::FloatExpr(expr),
            Expr::String(expr) => Self::StringExpr(expr),
            Expr::Variable(expr) => Self::VariableExpr(expr),
            Expr::List(expr) => Self::ListExpr(expr),
            Expr::Object(expr) => Self::ObjectExpr(expr),
            Expr::Function(expr) => Self::FunctionExpr(expr),
            Expr::Wrapped(expr) => Self::WrappedExpr(expr),
            Expr::Index(expr) => Self::IndexExpr(expr),
            Expr::Dot(expr) => Self::DotExpr(expr),
            Expr::Call(expr) => Self::CallExpr(expr),
            Expr::UnaryOperation(expr) => Self::UnaryOperationExpr(expr),
            Expr::BinaryOperation(expr) => Self::BinaryOperationExpr(expr),
        }
    }

    pub fn from_stmt(stmt: &'a Stmt) -> Self {
        match stmt {
            Stmt::If(stmt) => Self::IfStmt(stmt),
            Stmt::Loop(stmt) => Self::LoopStmt(stmt),
            Stmt::While(stmt) => Self::WhileStmt(stmt),
            Stmt::Return(stmt) => Self::ReturnStmt(stmt),
            Stmt::Break(stmt) => Self::BreakStmt(stmt),
            Stmt::Continue(stmt) => Self::ContinueStmt(stmt),
            Stmt::FunctionDeclaration(stmt) => Self::FunctionStmt(stmt),
            Stmt::VariableDeclaration(stmt) => Self::VariableDeclarationStmt(stmt),
            Stmt::VariableAssignment(stmt) => Self::VariableAssignmentStmt(stmt),
            Stmt::IndexAssignment(stmt) => Self::IndexAssignmentStmt(stmt),
            Stmt::DotAssignment(stmt) => Self::DotAssignmentStmt(stmt),
            Stmt::Expr(stmt) => Self::ExprStmt(stmt),
        }
    }
}
