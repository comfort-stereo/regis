use crate::lexer::{Keyword, Symbol, Token, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    Neg,
    BitNot,
    Not,
}

impl UnaryOperator {
    pub fn from_token(token: &Token<'_>) -> Option<Self> {
        match token.kind() {
            TokenKind::Keyword(keyword) => Self::from_keyword(keyword),
            TokenKind::Symbol(symbol) => Self::from_symbol(symbol),
            _ => None,
        }
    }

    pub fn from_keyword(keyword: &Keyword) -> Option<Self> {
        Some(match keyword {
            Keyword::Not => Self::Not,
            _ => return None,
        })
    }

    pub fn from_symbol(symbol: &Symbol) -> Option<Self> {
        Some(match symbol {
            Symbol::Sub => Self::Neg,
            Symbol::BitNot => Self::BitNot,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    And,
    Or,
    Ncl,
    Lt,
    Gt,
    Lte,
    Gte,
    Eq,
    Neq,
}

impl BinaryOperator {
    pub fn from_token(token: &Token<'_>) -> Option<Self> {
        match token.kind() {
            TokenKind::Keyword(keyword) => Self::from_keyword(keyword),
            TokenKind::Symbol(symbol) => Self::from_symbol(symbol),
            _ => None,
        }
    }

    pub fn from_keyword(keyword: &Keyword) -> Option<Self> {
        Some(match keyword {
            Keyword::And => Self::And,
            Keyword::Or => Self::Or,
            _ => return None,
        })
    }

    pub fn from_symbol(symbol: &Symbol) -> Option<Self> {
        Some(match symbol {
            Symbol::Add => Self::Add,
            Symbol::Sub => Self::Sub,
            Symbol::Mul => Self::Mul,
            Symbol::Div => Self::Div,
            Symbol::Shl => Self::Shl,
            Symbol::Shr => Self::Shr,
            Symbol::BitAnd => Self::BitAnd,
            Symbol::BitOr => Self::BitOr,
            Symbol::Ncl => Self::Ncl,
            Symbol::Lt => Self::Lt,
            Symbol::Gt => Self::Gt,
            Symbol::Lte => Self::Lte,
            Symbol::Gte => Self::Gte,
            Symbol::Eq => Self::Eq,
            Symbol::Neq => Self::Neq,
            _ => return None,
        })
    }

    pub fn precedence(&self) -> u8 {
        match self {
            Self::Ncl => 1,
            Self::Mul | Self::Div => 2,
            Self::Add | Self::Sub => 3,
            Self::BitAnd => 4,
            Self::BitOr => 5,
            Self::Shl | Self::Shr => 6,
            Self::Gt | Self::Lt | Self::Gte | Self::Lte => 7,
            Self::Eq | Self::Neq => 8,
            Self::And => 9,
            Self::Or => 10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssignmentOperator {
    Assign,
    MulAssign,
    DivAssign,
    AddAssign,
    SubAssign,
    NclAssign,
}

impl AssignmentOperator {
    pub fn from_token(token: &Token<'_>) -> Option<Self> {
        match token.kind() {
            TokenKind::Symbol(symbol) => Self::from_symbol(symbol),
            _ => None,
        }
    }

    pub fn from_symbol(symbol: &Symbol) -> Option<Self> {
        Some(match symbol {
            Symbol::Assign => AssignmentOperator::Assign,
            Symbol::AddAssign => AssignmentOperator::AddAssign,
            Symbol::SubAssign => AssignmentOperator::SubAssign,
            Symbol::MulAssign => AssignmentOperator::MulAssign,
            Symbol::DivAssign => AssignmentOperator::DivAssign,
            Symbol::NclAssign => AssignmentOperator::NclAssign,
            _ => return None,
        })
    }
}
