use crate::source::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'source> {
    kind: TokenKind,
    span: Span,
    slice: &'source str,
}

impl<'source> Token<'source> {
    pub fn new(kind: TokenKind, start: usize, slice: &'source str) -> Self {
        Self {
            kind,
            span: Span::new(start, start + slice.len()),
            slice,
        }
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn slice(&self) -> &str {
        self.slice
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    Whitespace,
    Keyword(Keyword),
    Ident,
    Literal(Literal),
    Symbol(Symbol),
    Comment,
    Unknown,
    Eoi,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Keyword {
    Let,
    Fn,
    Export,
    If,
    Else,
    While,
    Loop,
    Return,
    Break,
    Continue,
    And,
    Or,
    Not,
    Null,
    True,
    False,
}

impl Keyword {
    pub fn text(&self) -> &'static str {
        match self {
            Keyword::Let => "let",
            Keyword::Fn => "fn",
            Keyword::Export => "export",
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::While => "while",
            Keyword::Loop => "loop",
            Keyword::Return => "return",
            Keyword::Break => "break",
            Keyword::Continue => "continue",
            Keyword::And => "and",
            Keyword::Or => "or",
            Keyword::Not => "not",
            Keyword::Null => "null",
            Keyword::True => "true",
            Keyword::False => "false",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Symbol {
    Comma,
    Colon,
    Semicolon,
    Dot,
    Arrow,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Add,
    Sub,
    Mul,
    Div,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    BitNot,
    Ncl,
    Lt,
    Gt,
    Lte,
    Gte,
    Eq,
    Neq,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ShlAssign,
    ShrAssign,
    BitAndAssign,
    BitOrAssign,
    NclAssign,
}

impl Symbol {
    pub fn text(&self) -> &'static str {
        match self {
            Symbol::Comma => ",",
            Symbol::Colon => ":",
            Symbol::Semicolon => ";",
            Symbol::Dot => ".",
            Symbol::Arrow => "=>",
            Symbol::OpenParen => "(",
            Symbol::CloseParen => ")",
            Symbol::OpenBrace => "{",
            Symbol::CloseBrace => "}",
            Symbol::OpenBracket => "[",
            Symbol::CloseBracket => "]",
            Symbol::Add => "+",
            Symbol::Sub => "-",
            Symbol::Mul => "*",
            Symbol::Div => "/",
            Symbol::Shl => "<<",
            Symbol::Shr => ">>",
            Symbol::BitAnd => "&",
            Symbol::BitOr => "|",
            Symbol::BitNot => "~",
            Symbol::Ncl => "??",
            Symbol::Lt => "<",
            Symbol::Gt => ">",
            Symbol::Lte => "<=",
            Symbol::Gte => ">=",
            Symbol::Eq => "==",
            Symbol::Neq => "!=",
            Symbol::Assign => "=",
            Symbol::AddAssign => "+=",
            Symbol::SubAssign => "-=",
            Symbol::MulAssign => "*=",
            Symbol::DivAssign => "/=",
            Symbol::ShlAssign => "<<=",
            Symbol::ShrAssign => ">>=",
            Symbol::BitAndAssign => "&=",
            Symbol::BitOrAssign => "|=",
            Symbol::NclAssign => "??=",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Literal {
    String,
    Int,
    Float,
}
