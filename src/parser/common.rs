use crate::ast::{Block, Chunk, Ident};
use crate::lexer::{Symbol, TokenKind};

use super::error::{ParseError, ParseErrorKind};
use super::result::ParseResult;
use super::Parser;

impl<'source> Parser<'source> {
    pub(super) fn eat_chunk(&mut self) -> ParseResult<Chunk> {
        let start = self.start_node();
        let mut stmts = Vec::new();
        while self.peek().is_some() {
            stmts.push(self.eat_stmt()?);
        }
        Ok(Chunk {
            info: self.end_node(start),
            stmts,
        })
    }

    pub(super) fn eat_block(&mut self) -> ParseResult<Block> {
        let start = self.start_node();
        self.eat_symbol(Symbol::OpenBrace)?;
        let mut stmts = Vec::new();
        while self.peek_kind() != TokenKind::Symbol(Symbol::CloseBrace) {
            stmts.push(self.eat_stmt()?);
        }
        self.eat_symbol(Symbol::CloseBrace)?;
        Ok(Block {
            info: self.end_node(start),
            stmts,
        })
    }

    pub(super) fn eat_ident(&mut self) -> ParseResult<Ident> {
        let start = self.start_node();
        let ident = expect!(
            self.next(),
            TokenKind::Ident,
            ParseErrorKind::Expected("identifier"),
            self.index(),
        )?;

        Ok(Ident {
            info: self.end_node(start),
            text: ident.slice().into(),
        })
    }
}
