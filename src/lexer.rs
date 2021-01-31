mod token;

pub use self::token::*;

use std::collections::VecDeque;
use std::str::Chars;

pub struct Lexer<'source> {
    source: &'source str,
    chars: Chars<'source>,
    buffer: VecDeque<char>,
    index: usize,
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Token<'source>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.source.len() {
            return None;
        }

        if let Some(token) = self.try_get_valid_token() {
            self.advance(token.slice().len());
            Some(token)
        } else {
            Some(self.advance_while_unknown())
        }
    }
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            source,
            chars: source.chars(),
            buffer: VecDeque::new(),
            index: 0,
        }
    }

    fn try_get_valid_token(&mut self) -> Option<Token<'source>> {
        self.whitespace()
            .or_else(|| self.keyword())
            .or_else(|| self.ident())
            .or_else(|| self.symbol())
            .or_else(|| self.literal())
            .or_else(|| self.comment())
    }

    fn advance(&mut self, by: usize) {
        for _ in 0..by {
            if let Some(character) = self.buffer.pop_front().or_else(|| self.chars.next()) {
                self.index += character.len_utf8();
            } else {
                break;
            }
        }
    }

    fn advance_while_unknown(&mut self) -> Token<'source> {
        let start = self.index;
        while self.peek().is_some() && self.try_get_valid_token().is_none() {
            self.advance(1);
        }
        let end = self.index;

        Token::new(TokenKind::Unknown, start, &self.source[start..end])
    }

    fn peek(&mut self) -> Option<char> {
        self.lookahead(0)
    }

    fn lookahead(&mut self, by: usize) -> Option<char> {
        while self.buffer.len() <= by {
            self.buffer.push_back(self.chars.next()?);
        }

        Some(self.buffer[by])
    }

    fn slice(&self, length: usize) -> &'source str {
        &self.source[self.index..self.index + length]
    }

    fn token(&self, kind: TokenKind, slice: &'source str) -> Token<'source> {
        Token::new(kind, self.index, slice)
    }

    fn read_slice_while(
        &mut self,
        skip: usize,
        predicate: fn(character: char) -> bool,
    ) -> &'source str {
        let length = self.read_while(skip, predicate);
        self.slice(length)
    }

    fn read_while(&mut self, skip: usize, predicate: fn(character: char) -> bool) -> usize {
        let mut length = skip;
        while let Some(character) = self.lookahead(length) {
            if predicate(character) {
                length += 1;
            } else {
                break;
            }
        }

        length
    }

    fn whitespace(&mut self) -> Option<Token<'source>> {
        let slice = self.read_slice_while(0, is_whitespace);
        if slice.is_empty() {
            return None;
        }

        Some(self.token(TokenKind::Whitespace, slice))
    }

    fn keyword(&mut self) -> Option<Token<'source>> {
        let slice = self.read_slice_while(0, is_alpha);
        if slice.is_empty() {
            return None;
        }

        let keyword = match slice {
            "let" => Keyword::Let,
            "fn" => Keyword::Fn,
            "export" => Keyword::Export,
            "if" => Keyword::If,
            "else" => Keyword::Else,
            "while" => Keyword::While,
            "loop" => Keyword::Loop,
            "return" => Keyword::Return,
            "break" => Keyword::Break,
            "continue" => Keyword::Continue,
            "and" => Keyword::And,
            "or" => Keyword::Or,
            "not" => Keyword::Not,
            "null" => Keyword::Null,
            "true" => Keyword::True,
            "false" => Keyword::False,
            _ => return None,
        };

        Some(self.token(TokenKind::Keyword(keyword), slice))
    }

    fn ident(&mut self) -> Option<Token<'source>> {
        if !is_ident_first(self.peek()?) {
            return None;
        }

        let length = self.read_while(1, is_ident_continue);
        Some(self.token(TokenKind::Ident, self.slice(length)))
    }

    fn symbol(&mut self) -> Option<Token<'source>> {
        let first = self.peek();
        let second = self.lookahead(1);
        let third = self.lookahead(2);

        let symbol = match first? {
            ',' => Symbol::Comma,
            ':' => Symbol::Colon,
            ';' => Symbol::Semicolon,
            '.' => Symbol::Dot,
            '(' => Symbol::OpenParen,
            ')' => Symbol::CloseParen,
            '{' => Symbol::OpenBrace,
            '}' => Symbol::CloseBrace,
            '[' => Symbol::OpenBracket,
            ']' => Symbol::CloseBracket,
            '+' => match second {
                Some('=') => Symbol::AddAssign,
                _ => Symbol::Add,
            },
            '-' => match second {
                Some('=') => Symbol::SubAssign,
                _ => Symbol::Sub,
            },
            '*' => match second {
                Some('=') => Symbol::MulAssign,
                _ => Symbol::Mul,
            },
            '/' => match second {
                Some('=') => Symbol::DivAssign,
                _ => Symbol::Div,
            },
            '<' => match second {
                Some('=') => Symbol::Lte,
                Some('<') => match third {
                    Some('=') => Symbol::ShlAssign,
                    _ => Symbol::Shl,
                },
                _ => Symbol::Lt,
            },
            '>' => match second {
                Some('=') => Symbol::Gte,
                Some('>') => match third {
                    Some('=') => Symbol::ShrAssign,
                    _ => Symbol::Shr,
                },
                _ => Symbol::Gt,
            },
            '&' => match second {
                Some('=') => Symbol::BitAndAssign,
                _ => Symbol::BitAnd,
            },
            '|' => match second {
                Some('=') => Symbol::BitOrAssign,
                _ => Symbol::BitOr,
            },
            '~' => Symbol::BitNot,
            '?' => match second {
                Some('?') => match third {
                    Some('=') => Symbol::NclAssign,
                    _ => Symbol::Ncl,
                },
                _ => return None,
            },
            '=' => match second {
                Some('=') => Symbol::Eq,
                Some('>') => Symbol::Arrow,
                _ => Symbol::Assign,
            },
            '!' => match second {
                Some('=') => Symbol::Neq,
                _ => return None,
            },
            _ => return None,
        };

        Some(self.token(TokenKind::Symbol(symbol), self.slice(symbol.text().len())))
    }

    fn literal(&mut self) -> Option<Token<'source>> {
        self.number().or_else(|| self.string())
    }

    fn number(&mut self) -> Option<Token<'source>> {
        let int = self.read_slice_while(0, is_digit);
        if int.is_empty() {
            return None;
        }

        if !matches!(self.lookahead(int.len()), Some('.')) {
            return Some(self.token(TokenKind::Literal(Literal::Int), int));
        }

        let float = self.read_slice_while(int.len() + 1, is_digit);
        if float.ends_with('.') {
            return None;
        }

        Some(self.token(TokenKind::Literal(Literal::Float), float))
    }

    fn string(&mut self) -> Option<Token<'source>> {
        if self.peek()? != '"' {
            return None;
        }

        let mut length = 1;
        while let Some(character) = self.lookahead(length) {
            length += 1;
            if character == '"' {
                break;
            }

            if character == '\\' {
                length += 1;
            }
        }

        Some(self.token(TokenKind::Literal(Literal::String), self.slice(length)))
    }

    fn comment(&mut self) -> Option<Token<'source>> {
        if self.peek()? != '#' {
            return None;
        }

        let length = self.read_while(1, |current| current != '\n');
        Some(self.token(TokenKind::Comment, self.slice(length)))
    }
}

fn is_whitespace(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\n' | '\r')
}

fn is_ident_first(character: char) -> bool {
    is_alpha(character) || character == '@' || character == '_'
}

fn is_ident_continue(character: char) -> bool {
    is_alpha(character) || is_digit(character) || character == '_'
}

fn is_digit(character: char) -> bool {
    ('0'..='9').contains(&character)
}

fn is_alpha(character: char) -> bool {
    is_alpha_lower(character) || is_alpha_upper(character)
}

fn is_alpha_lower(character: char) -> bool {
    ('a'..='z').contains(&character)
}

fn is_alpha_upper(character: char) -> bool {
    ('A'..='Z').contains(&character)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Check<'source> {
        source: &'source str,
        tokens: Vec<Token<'source>>,
    }

    impl<'source> Check<'source> {
        pub fn new(source: &'source str) -> Self {
            Self {
                source,
                tokens: Lexer::new(source).collect(),
            }
        }

        pub fn count(self, count: usize) -> Self {
            assert_eq!(self.tokens.len(), count);
            self
        }

        pub fn token(self, index: usize, kind: TokenKind, slice: &str) -> Self {
            if let Some(start) = self.source.find(slice) {
                if let Some(token) = self.tokens.get(index) {
                    assert_eq!(*token, Token::new(kind, start, slice));
                } else {
                    panic!("There's no token at index {}.", index);
                }
            } else {
                panic!(
                    "Failed to find index of '{}' in source text '{}'.",
                    slice, self.source
                );
            }

            self
        }
    }

    #[test]
    fn empty() {
        let source = "";
        assert_eq!(Lexer::new(&source).next(), None);
    }

    #[test]
    fn whitespace() {
        let source = " ";
        Check::new(&source)
            .token(0, TokenKind::Whitespace, source)
            .count(1);

        let source = " \r\n\t\r\r\n\n    \t\t";
        Check::new(&source)
            .token(0, TokenKind::Whitespace, source)
            .count(1);
    }

    #[test]
    fn ident() {
        Check::new("Hello _these\nare\tIDENTS")
            .token(0, TokenKind::Ident, "Hello")
            .token(1, TokenKind::Whitespace, " ")
            .token(2, TokenKind::Ident, "_these")
            .token(3, TokenKind::Whitespace, "\n")
            .token(4, TokenKind::Ident, "are")
            .token(5, TokenKind::Whitespace, "\t")
            .token(6, TokenKind::Ident, "IDENTS")
            .count(7);

        Check::new(" _\r@")
            .token(1, TokenKind::Ident, "_")
            .token(3, TokenKind::Ident, "@")
            .count(4);
    }

    #[test]
    fn keyword() {
        use Keyword::*;

        Check::new("fn let export")
            .token(0, TokenKind::Keyword(Fn), Fn.text())
            .token(2, TokenKind::Keyword(Let), Let.text())
            .token(4, TokenKind::Keyword(Export), Export.text())
            .count(5);

        Check::new("if else while loop")
            .token(0, TokenKind::Keyword(If), If.text())
            .token(2, TokenKind::Keyword(Else), Else.text())
            .token(4, TokenKind::Keyword(While), While.text())
            .token(6, TokenKind::Keyword(Loop), Loop.text())
            .count(7);

        Check::new("return break continue")
            .token(0, TokenKind::Keyword(Return), Return.text())
            .token(2, TokenKind::Keyword(Break), Break.text())
            .token(4, TokenKind::Keyword(Continue), Continue.text())
            .count(5);

        Check::new("and or not")
            .token(0, TokenKind::Keyword(And), And.text())
            .token(2, TokenKind::Keyword(Or), Or.text())
            .token(4, TokenKind::Keyword(Not), Not.text())
            .count(5);

        Check::new("null true false")
            .token(0, TokenKind::Keyword(Null), Null.text())
            .token(2, TokenKind::Keyword(True), True.text())
            .token(4, TokenKind::Keyword(False), False.text())
            .count(5);
    }

    #[test]
    fn symbol() {
        use Symbol::*;

        Check::new(",:;.=>")
            .token(0, TokenKind::Symbol(Comma), Comma.text())
            .token(1, TokenKind::Symbol(Colon), Colon.text())
            .token(2, TokenKind::Symbol(Semicolon), Semicolon.text())
            .token(3, TokenKind::Symbol(Dot), Dot.text())
            .token(4, TokenKind::Symbol(Arrow), Arrow.text())
            .count(5);

        Check::new("(){}[]")
            .token(0, TokenKind::Symbol(OpenParen), OpenParen.text())
            .token(1, TokenKind::Symbol(CloseParen), CloseParen.text())
            .token(2, TokenKind::Symbol(OpenBrace), OpenBrace.text())
            .token(3, TokenKind::Symbol(CloseBrace), CloseBrace.text())
            .token(4, TokenKind::Symbol(OpenBracket), OpenBracket.text())
            .token(5, TokenKind::Symbol(CloseBracket), CloseBracket.text())
            .count(6);

        Check::new("+ - * /")
            .token(0, TokenKind::Symbol(Add), Add.text())
            .token(2, TokenKind::Symbol(Sub), Sub.text())
            .token(4, TokenKind::Symbol(Mul), Mul.text())
            .token(6, TokenKind::Symbol(Div), Div.text())
            .count(7);

        Check::new("<< >> & |")
            .token(0, TokenKind::Symbol(Shl), Shl.text())
            .token(2, TokenKind::Symbol(Shr), Shr.text())
            .token(4, TokenKind::Symbol(BitAnd), BitAnd.text())
            .token(6, TokenKind::Symbol(BitOr), BitOr.text())
            .count(7);

        Check::new("~")
            .token(0, TokenKind::Symbol(BitNot), BitNot.text())
            .count(1);

        Check::new("??")
            .token(0, TokenKind::Symbol(Ncl), Ncl.text())
            .count(1);

        Check::new("< > <= >=")
            .token(0, TokenKind::Symbol(Lt), Lt.text())
            .token(2, TokenKind::Symbol(Gt), Gt.text())
            .token(4, TokenKind::Symbol(Lte), Lte.text())
            .token(6, TokenKind::Symbol(Gte), Gte.text())
            .count(7);

        Check::new("== !=")
            .token(0, TokenKind::Symbol(Eq), Eq.text())
            .token(2, TokenKind::Symbol(Neq), Neq.text())
            .count(3);

        Check::new("=")
            .token(0, TokenKind::Symbol(Assign), Assign.text())
            .count(1);

        Check::new("+= -= *= /=")
            .token(0, TokenKind::Symbol(AddAssign), AddAssign.text())
            .token(2, TokenKind::Symbol(SubAssign), SubAssign.text())
            .token(4, TokenKind::Symbol(MulAssign), MulAssign.text())
            .token(6, TokenKind::Symbol(DivAssign), DivAssign.text())
            .count(7);

        Check::new("<<= >>= &= |=")
            .token(0, TokenKind::Symbol(ShlAssign), ShlAssign.text())
            .token(2, TokenKind::Symbol(ShrAssign), ShrAssign.text())
            .token(4, TokenKind::Symbol(BitAndAssign), BitAndAssign.text())
            .token(6, TokenKind::Symbol(BitOrAssign), BitOrAssign.text())
            .count(7);

        Check::new("??=")
            .token(0, TokenKind::Symbol(NclAssign), NclAssign.text())
            .count(1);
    }

    #[test]
    fn string() {
        use Literal::*;
        {
            let source = r#""A long, long time ago.""#;
            Check::new(source)
                .token(0, TokenKind::Literal(String), source)
                .count(1);
        }

        {
            let source = r#""A" "B" "C" "1" "2" "3""#;
            Check::new(source)
                .token(0, TokenKind::Literal(String), r#""A""#)
                .token(2, TokenKind::Literal(String), r#""B""#)
                .token(4, TokenKind::Literal(String), r#""C""#)
                .token(6, TokenKind::Literal(String), r#""1""#)
                .token(8, TokenKind::Literal(String), r#""2""#)
                .token(10, TokenKind::Literal(String), r#""3""#)
                .count(11);
        }

        {
            let source = r#""`~!@#$%^&*()-=_+[]{};:'?/<>,.""#;
            Check::new(source)
                .token(0, TokenKind::Literal(String), source)
                .count(1);
        }

        {
            let source = r#""\"\" \\\\ \u{0123} \u0123""#;
            Check::new(source)
                .token(0, TokenKind::Literal(String), source)
                .count(1);
        }
    }

    #[test]
    fn int() {
        use Literal::*;

        Check::new("0 1 2 3 4 5 6 7 8 9")
            .token(0, TokenKind::Literal(Int), "0")
            .token(2, TokenKind::Literal(Int), "1")
            .token(4, TokenKind::Literal(Int), "2")
            .token(6, TokenKind::Literal(Int), "3")
            .token(8, TokenKind::Literal(Int), "4")
            .token(10, TokenKind::Literal(Int), "5")
            .token(12, TokenKind::Literal(Int), "6")
            .token(14, TokenKind::Literal(Int), "7")
            .token(16, TokenKind::Literal(Int), "8")
            .token(18, TokenKind::Literal(Int), "9")
            .count(19);

        Check::new("0123456789")
            .token(0, TokenKind::Literal(Int), "0123456789")
            .count(1);

        Check::new("000123")
            .token(0, TokenKind::Literal(Int), "000123")
            .count(1);
    }

    #[test]
    fn float() {
        use Literal::*;

        Check::new("0.0 1.1 2.2 3.3 4.4 5.5 6.6 7.7 8.8 9.9")
            .token(0, TokenKind::Literal(Float), "0.0")
            .token(2, TokenKind::Literal(Float), "1.1")
            .token(4, TokenKind::Literal(Float), "2.2")
            .token(6, TokenKind::Literal(Float), "3.3")
            .token(8, TokenKind::Literal(Float), "4.4")
            .token(10, TokenKind::Literal(Float), "5.5")
            .token(12, TokenKind::Literal(Float), "6.6")
            .token(14, TokenKind::Literal(Float), "7.7")
            .token(16, TokenKind::Literal(Float), "8.8")
            .token(18, TokenKind::Literal(Float), "9.9")
            .count(19);

        Check::new("0123456789.0123456789")
            .token(0, TokenKind::Literal(Float), "0123456789.0123456789")
            .count(1);

        Check::new("000.000")
            .token(0, TokenKind::Literal(Float), "000.000")
            .count(1);
    }

    #[test]
    fn comment() {
        {
            let source = "# This is a comment.";
            Check::new(source).token(0, TokenKind::Comment, source);
        }

        {
            let source = "
#First comment.
### Second comment.
# Third comment. # Neat. #
                         "
            .trim();
            Check::new(source)
                .token(0, TokenKind::Comment, "#First comment.")
                .token(2, TokenKind::Comment, "### Second comment.")
                .token(4, TokenKind::Comment, "# Third comment. # Neat. #");
        }
    }

    #[test]
    fn unknown() {
        Check::new("$").token(0, TokenKind::Unknown, "$");

        Check::new("start $\\?\nend")
            .token(0, TokenKind::Ident, "start")
            .token(1, TokenKind::Whitespace, " ")
            .token(2, TokenKind::Unknown, "$\\?")
            .token(3, TokenKind::Whitespace, "\n")
            .token(4, TokenKind::Ident, "end");
    }
}
