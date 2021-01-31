use super::base::{Block, Ident};
use super::expr::{DotExpr, Expr, FunctionExpr, IndexExpr};
use super::node::NodeInfo;
use super::operator::AssignmentOperator;

#[derive(Debug)]
pub enum Stmt {
    If(Box<IfStmt>),
    Loop(Box<LoopStmt>),
    While(Box<WhileStmt>),
    Return(Box<ReturnStmt>),
    Break(Box<BreakStmt>),
    Continue(Box<ContinueStmt>),
    FunctionDeclaration(Box<FunctionDeclarationStmt>),
    VariableDeclaration(Box<VariableDeclarationStmt>),
    VariableAssignment(Box<VariableAssignmentStmt>),
    IndexAssignment(Box<IndexAssignmentStmt>),
    DotAssignment(Box<DotAssignmentStmt>),
    Expr(Box<ExprStmt>),
}

#[derive(Debug)]
pub struct IfStmt {
    pub info: NodeInfo,
    pub condition: Box<Expr>,
    pub block: Box<Block>,
    pub else_clause: Option<Box<ElseClause>>,
}

#[derive(Debug)]
pub struct ElseClause {
    pub info: NodeInfo,
    pub next: ElseClauseNextVariant,
}

#[derive(Debug)]
pub enum ElseClauseNextVariant {
    IfStmt(Box<IfStmt>),
    Block(Box<Block>),
}

#[derive(Debug)]
pub struct LoopStmt {
    pub info: NodeInfo,
    pub block: Box<Block>,
}

#[derive(Debug)]
pub struct WhileStmt {
    pub info: NodeInfo,
    pub condition: Expr,
    pub block: Box<Block>,
}

#[derive(Debug)]
pub struct ReturnStmt {
    pub info: NodeInfo,
    pub value: Option<Expr>,
}

#[derive(Debug)]
pub struct BreakStmt {
    pub info: NodeInfo,
}

#[derive(Debug)]
pub struct ContinueStmt {
    pub info: NodeInfo,
}

#[derive(Debug)]
pub struct FunctionDeclarationStmt {
    pub info: NodeInfo,
    pub is_exported: bool,
    pub function: Box<FunctionExpr>,
}

#[derive(Debug)]
pub struct VariableDeclarationStmt {
    pub info: NodeInfo,
    pub is_exported: bool,
    pub name: Box<Ident>,
    pub value: Expr,
}

#[derive(Debug)]
pub struct VariableAssignmentStmt {
    pub info: NodeInfo,
    pub name: Box<Ident>,
    pub operator: AssignmentOperator,
    pub value: Expr,
}

#[derive(Debug)]
pub struct IndexAssignmentStmt {
    pub info: NodeInfo,
    pub index_expr: Box<IndexExpr>,
    pub operator: AssignmentOperator,
    pub value: Expr,
}

#[derive(Debug)]
pub struct DotAssignmentStmt {
    pub info: NodeInfo,
    pub dot_expr: Box<DotExpr>,
    pub operator: AssignmentOperator,
    pub value: Expr,
}

#[derive(Debug)]
pub struct ExprStmt {
    pub info: NodeInfo,
    pub expr: Expr,
}
