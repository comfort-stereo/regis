use crate::ast::*;
use crate::lexer::{Keyword, Symbol, TokenKind};

use super::error::{ParseError, ParseErrorKind};
use super::result::ParseResult;
use super::Parser;

impl<'source> Parser<'source> {
    pub fn eat_stmt(&mut self) -> ParseResult<Stmt> {
        let first = self.peek().cloned();
        let second = self.lookahead(1).cloned();

        let stmt = self.attempt(|this| this.eat_expr_first_stmt());
        if let Ok(stmt) = stmt {
            return Ok(stmt);
        }

        Ok(match first.map(|first| *first.kind()) {
            Some(TokenKind::Keyword(Keyword::If)) => Stmt::If(self.eat_if_stmt()?.into()),
            Some(TokenKind::Keyword(Keyword::While)) => Stmt::While(self.eat_while_stmt()?.into()),
            Some(TokenKind::Keyword(Keyword::Loop)) => Stmt::Loop(self.eat_loop_stmt()?.into()),
            Some(TokenKind::Keyword(Keyword::Return)) => {
                Stmt::Return(self.eat_return_stmt()?.into())
            }
            Some(TokenKind::Keyword(Keyword::Break)) => Stmt::Break(self.eat_break_stmt()?.into()),
            Some(TokenKind::Keyword(Keyword::Continue)) => {
                Stmt::Continue(self.eat_continue_stmt()?.into())
            }
            Some(TokenKind::Keyword(Keyword::Fn)) => {
                Stmt::FunctionDeclaration(self.eat_function_declaration_stmt()?.into())
            }
            Some(TokenKind::Keyword(Keyword::Let)) => {
                Stmt::VariableDeclaration(self.eat_variable_declaration_stmt()?.into())
            }
            Some(TokenKind::Keyword(Keyword::Export)) => {
                match second.map(|second| *second.kind()) {
                    Some(TokenKind::Keyword(Keyword::Fn)) => {
                        Stmt::FunctionDeclaration(self.eat_function_declaration_stmt()?.into())
                    }
                    Some(TokenKind::Keyword(Keyword::Let)) => {
                        Stmt::VariableDeclaration(self.eat_variable_declaration_stmt()?.into())
                    }
                    _ => {
                        return Err(ParseError::at_token_or_index(
                            ParseErrorKind::Expected("function or variable declaration"),
                            second.as_ref(),
                            self.index(),
                        ))
                    }
                }
            }
            _ => {
                return stmt;
            }
        })
    }

    fn eat_expr_first_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.start_node();
        let first = self.eat_expr()?;

        if let Expr::Function(function) = first {
            if self.peek_kind() == TokenKind::Symbol(Symbol::Semicolon)
                || matches!(function.body, FunctionExprBody::Expr(..))
            {
                self.eat_symbol(Symbol::Semicolon)?;
            }

            return Ok(Stmt::FunctionDeclaration(
                FunctionDeclarationStmt {
                    info: self.end_node(start),
                    is_exported: false,
                    function,
                }
                .into(),
            ));
        }

        Ok(match self.peek_kind() {
            TokenKind::Symbol(Symbol::Semicolon) => {
                self.eat_symbol(Symbol::Semicolon)?;
                Stmt::Expr(
                    ExprStmt {
                        info: self.end_node(start),
                        expr: first,
                    }
                    .into(),
                )
            }
            TokenKind::Symbol(symbol) => {
                if let Some(operator) = AssignmentOperator::from_symbol(&symbol) {
                    self.eat_symbol(symbol)?;
                    let value = self.eat_expr()?;
                    self.eat_symbol(Symbol::Semicolon)?;
                    match first {
                        Expr::Variable(variable) => Stmt::VariableAssignment(
                            VariableAssignmentStmt {
                                info: self.end_node(start),
                                name: variable.name.into(),
                                operator,
                                value,
                            }
                            .into(),
                        ),
                        Expr::Index(index) => Stmt::IndexAssignment(
                            IndexAssignmentStmt {
                                info: self.end_node(start),
                                index_expr: index,
                                operator,
                                value,
                            }
                            .into(),
                        ),
                        Expr::Dot(dot) => Stmt::DotAssignment(
                            DotAssignmentStmt {
                                info: self.end_node(start),
                                dot_expr: dot,
                                operator,
                                value,
                            }
                            .into(),
                        ),
                        _ => {
                            return Err(ParseError::at_index(
                                ParseErrorKind::Specific(
                                    "Invalid left hand side of assignment statement.",
                                ),
                                self.index(),
                            ));
                        }
                    }
                } else {
                    return Err(ParseError::at_index(
                        ParseErrorKind::Expected("assignment operator"),
                        self.index(),
                    ));
                }
            }
            _ => {
                return Err(ParseError::at_index(
                    ParseErrorKind::Expected("';'"),
                    self.index(),
                ))
            }
        })
    }

    fn eat_if_stmt(&mut self) -> ParseResult<IfStmt> {
        let start = self.start_node();
        self.eat_keyword(Keyword::If)?;
        let condition = self.eat_expr()?.into();
        let block = self.eat_block()?.into();
        let else_clause = if self.peek_kind() != TokenKind::Keyword(Keyword::Else) {
            None
        } else {
            let start = self.start_node();
            self.eat_keyword(Keyword::Else)?;
            let next = if self.peek_kind() == TokenKind::Keyword(Keyword::If) {
                ElseClauseNextVariant::IfStmt(self.eat_if_stmt()?.into())
            } else {
                ElseClauseNextVariant::Block(self.eat_block()?.into())
            };
            Some(
                ElseClause {
                    info: self.end_node(start),
                    next,
                }
                .into(),
            )
        };

        Ok(IfStmt {
            info: self.end_node(start),
            condition,
            block,
            else_clause,
        })
    }

    fn eat_while_stmt(&mut self) -> ParseResult<WhileStmt> {
        let start = self.start_node();
        self.eat_keyword(Keyword::While)?;
        let condition = self.eat_expr()?;
        let block = self.eat_block()?.into();
        Ok(WhileStmt {
            info: self.end_node(start),
            condition,
            block,
        })
    }

    fn eat_loop_stmt(&mut self) -> ParseResult<LoopStmt> {
        let start = self.start_node();
        self.eat_keyword(Keyword::Loop)?;
        let block = self.eat_block()?.into();
        Ok(LoopStmt {
            info: self.end_node(start),
            block,
        })
    }

    fn eat_return_stmt(&mut self) -> ParseResult<ReturnStmt> {
        let start = self.start_node();
        self.eat_keyword(Keyword::Return)?;
        let value = self.attempt(|this| this.eat_expr()).ok();
        let ok = self.eat_symbol(Symbol::Semicolon);
        ok?;

        Ok(ReturnStmt {
            info: self.end_node(start),
            value,
        })
    }

    fn eat_break_stmt(&mut self) -> ParseResult<BreakStmt> {
        let start = self.start_node();
        self.eat_keyword(Keyword::Break)?;
        self.eat_symbol(Symbol::Semicolon)?;
        Ok(BreakStmt {
            info: self.end_node(start),
        })
    }

    fn eat_continue_stmt(&mut self) -> ParseResult<ContinueStmt> {
        let start = self.start_node();
        self.eat_keyword(Keyword::Continue)?;
        self.eat_symbol(Symbol::Semicolon)?;
        Ok(ContinueStmt {
            info: self.end_node(start),
        })
    }

    fn eat_function_declaration_stmt(&mut self) -> ParseResult<FunctionDeclarationStmt> {
        let start = self.start_node();
        let is_exported = self
            .attempt(|this| this.eat_keyword(Keyword::Export))
            .is_ok();
        let function = self.eat_function_expr()?.into();

        Ok(FunctionDeclarationStmt {
            info: self.end_node(start),
            is_exported,
            function,
        })
    }

    fn eat_variable_declaration_stmt(&mut self) -> ParseResult<VariableDeclarationStmt> {
        let start = self.start_node();
        let is_exported = self
            .attempt(|this| this.eat_keyword(Keyword::Export))
            .is_ok();
        self.eat_keyword(Keyword::Let)?;
        let name = self.eat_ident()?;
        self.eat_symbol(Symbol::Assign)?;
        let value = self.eat_expr()?;
        self.eat_symbol(Symbol::Semicolon)?;

        Ok(VariableDeclarationStmt {
            info: self.end_node(start),
            is_exported,
            name: name.into(),
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::*;

    use super::Parser;

    #[test]
    fn if_stmt() {
        assert!(matches!(
            Parser::new("if true {} else {}").eat_stmt(),
            Ok(Stmt::If(..))
        ));
        assert!(matches!(
            Parser::new("if true {} else if false {} else {}").eat_stmt(),
            Ok(Stmt::If(..))
        ));
    }

    #[test]
    fn while_stmt() {
        assert!(matches!(
            Parser::new("while true {}").eat_stmt(),
            Ok(Stmt::While(..))
        ));
    }

    #[test]
    fn loop_stmt() {
        assert!(matches!(
            Parser::new("loop {}").eat_stmt(),
            Ok(Stmt::Loop(..))
        ));
    }

    #[test]
    fn return_stmt() {
        assert!(matches!(
            Parser::new("return;").eat_stmt(),
            Ok(Stmt::Return(..))
        ));
        assert!(matches!(
            Parser::new("return 100;").eat_stmt(),
            Ok(Stmt::Return(..))
        ));
    }

    #[test]
    fn break_stmt() {
        assert!(matches!(
            Parser::new("break;").eat_stmt(),
            Ok(Stmt::Break(..))
        ));
    }

    #[test]
    fn continue_stmt() {
        assert!(matches!(
            Parser::new("continue;").eat_stmt(),
            Ok(Stmt::Continue(..))
        ));
    }

    #[test]
    fn function_declaration_stmt() {
        assert!(matches!(
            Parser::new("fn run() {}").eat_stmt(),
            Ok(Stmt::FunctionDeclaration(..))
        ));
        assert!(matches!(
            Parser::new("fn run() {};").eat_stmt(),
            Ok(Stmt::FunctionDeclaration(..))
        ));
        assert!(matches!(
            Parser::new("fn () {}").eat_stmt(),
            Ok(Stmt::FunctionDeclaration(..))
        ));
        assert!(matches!(
            Parser::new("fn () {};").eat_stmt(),
            Ok(Stmt::FunctionDeclaration(..))
        ));
        assert!(matches!(
            Parser::new("fn run() => null;").eat_stmt(),
            Ok(Stmt::FunctionDeclaration(..))
        ));
        assert!(matches!(
            Parser::new("fn () => null;").eat_stmt(),
            Ok(Stmt::FunctionDeclaration(..))
        ));
    }

    #[test]
    fn variable_declaration_stmt() {
        assert!(matches!(
            Parser::new("let name = \"value\";").eat_stmt(),
            Ok(Stmt::VariableDeclaration(..))
        ));
    }
}
