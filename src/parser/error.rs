use std::fmt::{Display, Formatter, Result as FormatResult};

use crate::lexer::Token;
use crate::source::Span;

pub enum ParseErrorKind {
    UnexpectedToken,
    Expected(&'static str),
    ExpectedQuoted(&'static str),
    Specific(&'static str),
}

pub struct ParseError {
    kind: ParseErrorKind,
    span: Span,
}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        match self.kind() {
            ParseErrorKind::UnexpectedToken => write!(formatter, "Unexpected token."),
            ParseErrorKind::Expected(expected) => write!(formatter, "Expected: {}", expected),
            ParseErrorKind::ExpectedQuoted(expected) => {
                write!(formatter, "Expected: '{}'", expected)
            }
            ParseErrorKind::Specific(specific) => write!(formatter, "{}", specific),
        }
    }
}

impl ParseError {
    pub fn at_index(kind: ParseErrorKind, index: usize) -> Self {
        Self::at_span(kind, Span::at(index))
    }

    pub fn at_span(kind: ParseErrorKind, span: Span) -> Self {
        ParseError { kind, span }
    }

    pub fn at_token(kind: ParseErrorKind, token: &Token<'_>) -> Self {
        ParseError {
            kind,
            span: token.span(),
        }
    }

    pub fn at_token_or_index(
        kind: ParseErrorKind,
        token: Option<&Token<'_>>,
        index: usize,
    ) -> Self {
        if let Some(token) = token {
            Self::at_token(kind, token)
        } else {
            Self::at_index(kind, index)
        }
    }

    pub fn kind(&self) -> &ParseErrorKind {
        &self.kind
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}
