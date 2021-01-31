use crate::shared::SharedImmutable;

use super::base::{Block, Ident};
use super::node::NodeInfo;
use super::operator::BinaryOperator;
use super::UnaryOperator;

#[derive(Debug)]
pub enum Expr {
    Null(Box<NullExpr>),
    Boolean(Box<BooleanExpr>),
    Int(Box<IntExpr>),
    Float(Box<FloatExpr>),
    String(Box<StringExpr>),
    Variable(Box<VariableExpr>),
    List(Box<ListExpr>),
    Object(Box<ObjectExpr>),
    Function(Box<FunctionExpr>),
    Wrapped(Box<WrappedExpr>),
    Index(Box<IndexExpr>),
    Dot(Box<DotExpr>),
    Call(Box<CallExpr>),
    UnaryOperation(Box<UnaryOperationExpr>),
    BinaryOperation(Box<BinaryOperationExpr>),
}

impl Expr {
    pub fn info(&self) -> &NodeInfo {
        match self {
            Expr::Null(expr) => &expr.info,
            Expr::Boolean(expr) => &expr.info,
            Expr::Int(expr) => &expr.info,
            Expr::Float(expr) => &expr.info,
            Expr::String(expr) => &expr.info,
            Expr::Variable(expr) => &expr.info,
            Expr::List(expr) => &expr.info,
            Expr::Object(expr) => &expr.info,
            Expr::Function(expr) => &expr.info,
            Expr::Wrapped(expr) => &expr.info,
            Expr::Index(expr) => &expr.info,
            Expr::Dot(expr) => &expr.info,
            Expr::Call(expr) => &expr.info,
            Expr::UnaryOperation(expr) => &expr.info,
            Expr::BinaryOperation(expr) => &expr.info,
        }
    }
}

#[derive(Debug)]
pub struct NullExpr {
    pub info: NodeInfo,
}

#[derive(Debug)]
pub struct BooleanExpr {
    pub info: NodeInfo,
    pub value: bool,
}

#[derive(Debug)]
pub struct IntExpr {
    pub info: NodeInfo,
    pub value: i64,
}

#[derive(Debug)]
pub struct FloatExpr {
    pub info: NodeInfo,
    pub value: f64,
}

#[derive(Debug)]
pub struct StringExpr {
    pub info: NodeInfo,
    pub value: SharedImmutable<String>,
}

#[derive(Debug)]
pub struct VariableExpr {
    pub info: NodeInfo,
    pub name: Ident,
}

#[derive(Debug)]
pub struct ListExpr {
    pub info: NodeInfo,
    pub values: Vec<Expr>,
}

#[derive(Debug)]
pub struct ObjectExpr {
    pub info: NodeInfo,
    pub pairs: Vec<ObjectExprPair>,
}

#[derive(Debug)]
pub struct ObjectExprPair {
    pub info: NodeInfo,
    pub key: ObjectExprKeyVariant,
    pub value: Box<Expr>,
}

#[derive(Debug)]
pub enum ObjectExprKeyVariant {
    Identifier(Ident),
    String(StringExpr),
    Expr(ObjectExprKeyExpr),
}

#[derive(Debug)]
pub struct ObjectExprKeyExpr {
    pub info: NodeInfo,
    pub value: Box<Expr>,
}

#[derive(Debug)]
pub struct FunctionExpr {
    pub info: NodeInfo,
    pub name: Option<Box<Ident>>,
    pub parameters: Vec<Ident>,
    pub body: FunctionExprBody,
}

#[derive(Debug)]
pub enum FunctionExprBody {
    Block(Box<Block>),
    Expr(Box<Expr>),
}

#[derive(Debug)]
pub struct WrappedExpr {
    pub info: NodeInfo,
    pub value: Box<Expr>,
}

#[derive(Debug)]
pub struct IndexExpr {
    pub info: NodeInfo,
    pub target: Expr,
    pub index: Expr,
}

#[derive(Debug)]
pub struct DotExpr {
    pub info: NodeInfo,
    pub target: Expr,
    pub property: Ident,
}

#[derive(Debug)]
pub struct CallExpr {
    pub info: NodeInfo,
    pub target: Expr,
    pub arguments: Vec<Expr>,
}

#[derive(Debug)]
pub struct UnaryOperationExpr {
    pub info: NodeInfo,
    pub operator: UnaryOperator,
    pub right: Expr,
}

#[derive(Debug)]
pub struct BinaryOperationExpr {
    pub info: NodeInfo,
    pub operator: BinaryOperator,
    pub left: Expr,
    pub right: Expr,
}
