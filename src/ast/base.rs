use crate::shared::SharedImmutable;

use super::node::NodeInfo;
use super::stmt::Stmt;

#[derive(Debug)]
pub struct Chunk {
    pub info: NodeInfo,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub struct Block {
    pub info: NodeInfo,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub struct Ident {
    pub info: NodeInfo,
    pub text: SharedImmutable<String>,
}
