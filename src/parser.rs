mod error;
#[macro_use]
mod expect;
mod common;
mod expr;
mod result;
mod stmt;

pub use self::error::*;
pub use self::result::*;

use std::collections::VecDeque;
use std::iter::Filter;

use crate::ast::{Chunk, NodeInfo};
use crate::lexer::{Keyword, Lexer, Symbol, Token, TokenKind};
use crate::source::Span;

pub struct Parser<'source> {
    tokens: Filter<Lexer<'source>, fn(&Token<'source>) -> bool>,
    index: usize,
    buffer: VecDeque<Token<'source>>,
    buffer_index: usize,
    attempt_depth: usize,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            tokens: Lexer::new(source).filter(|current| {
                !matches!(current.kind(), TokenKind::Comment | TokenKind::Whitespace)
            }),
            index: 0,
            buffer: VecDeque::new(),
            buffer_index: 0,
            attempt_depth: 0,
        }
    }

    pub fn parse(mut self) -> ParseResult<'source, Chunk> {
        self.eat_chunk()
    }

    fn index(&self) -> usize {
        self.index
    }

    fn start_node(&self) -> usize {
        self.index()
    }

    fn end_node(&self, start: usize) -> NodeInfo {
        NodeInfo::new(Span::new(start, self.index()))
    }

    fn next(&mut self) -> Option<Token<'source>> {
        let next = if self.attempt_depth == 0 {
            self.buffer.pop_front().or_else(|| self.tokens.next())
        } else if self.buffer_index < self.buffer.len() {
            let next = self.buffer[self.buffer_index];
            self.buffer_index += 1;
            Some(next)
        } else if let Some(next) = self.tokens.next() {
            self.buffer.push_back(next);
            self.buffer_index += 1;
            Some(next)
        } else {
            None
        };

        if let Some(next) = next {
            self.index = next.end();
        }

        next
    }

    fn attempt<R, B: Fn(&mut Self) -> Result<R, ParseError>>(
        &mut self,
        block: B,
    ) -> Result<R, ParseError> {
        let start_index = self.index;
        let start_buffer_index = self.buffer_index;

        self.attempt_depth += 1;
        let result = block(self);
        self.attempt_depth -= 1;

        if result.is_err() {
            // If the parse attempt failed, reset to the initial state.
            self.index = start_index;
            self.buffer_index = start_buffer_index;
        } else if self.attempt_depth == 0 {
            // If the parse attempt was successful and we're back at the root, remove extra tokens
            // from the buffer.
            self.buffer.drain(start_buffer_index..self.buffer_index);
            self.buffer_index = start_buffer_index;
        }

        result
    }

    fn peek(&mut self) -> Option<&Token<'source>> {
        self.lookahead(0)
    }

    fn peek_kind(&mut self) -> TokenKind {
        self.lookahead_kind(0)
    }

    fn lookahead(&mut self, by: usize) -> Option<&Token<'source>> {
        while self.buffer.len() <= by + self.buffer_index {
            if let Some(current) = self.tokens.next() {
                self.buffer.push_back(current);
            } else {
                return None;
            }
        }

        self.buffer.get(by + self.buffer_index)
    }

    fn lookahead_kind(&mut self, by: usize) -> TokenKind {
        self.lookahead(by)
            .map_or(TokenKind::Eoi, |token| *token.kind())
    }

    fn eat_symbol(&mut self, symbol: Symbol) -> ParseResult<()> {
        expect_exact!(
            self.next(),
            TokenKind::Symbol(symbol),
            ParseErrorKind::ExpectedQuoted(symbol.text()),
            self.index(),
        )
        .map(|_| ())
    }

    fn eat_keyword(&mut self, keyword: Keyword) -> ParseResult<()> {
        expect_exact!(
            self.next(),
            TokenKind::Keyword(keyword),
            ParseErrorKind::ExpectedQuoted(keyword.text()),
            self.index(),
        )
        .map(|_| ())
    }
}
