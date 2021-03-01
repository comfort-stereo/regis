use std::collections::{BTreeSet, VecDeque};

use crate::ast::*;
use crate::lexer::{Keyword, Literal, Symbol, TokenKind};
use crate::source::Span;
use crate::unescape::unescape;

use super::{ParseError, ParseErrorKind, ParseResult, Parser};

impl<'source> Parser<'source> {
    pub fn eat_expr(&mut self) -> ParseResult<Expr> {
        // Initially, we're going to try to break the expression down into a list of unary
        // operators, binary operators and operands. We call these the "segments" of the expression.
        let mut segments: Vec<Segment> = Vec::new();

        // Keep reading tokens until we determine we've reach the end of the expression.
        while let Some(token) = self.peek().cloned() {
            // Check to see if the token is an operator.
            let operator = match (
                UnaryOperator::from_token(&token),
                BinaryOperator::from_token(&token),
            ) {
                // Check to see if the token is a unary operator.
                (Some(unary), None) => Some(Segment::UnaryOperator(UnaryOperatorSegment {
                    operator: unary,
                    span: *token.span(),
                })),
                // Check to see if the token is a binary operator.
                (None, Some(binary)) => Some(Segment::BinaryOperator(BinaryOperatorSegment {
                    operator: binary,
                    span: *token.span(),
                })),
                // If the token is an operator that could be interpreted as either unary or binary,
                // check to see if the previous segment is an expression. If the previous segment is
                // an expression, assume it's a binary operator. Otherwise, assume it's unary.
                (Some(unary), Some(binary)) => {
                    if matches!(segments.last(), Some(Segment::Expr(..))) {
                        Some(Segment::BinaryOperator(BinaryOperatorSegment {
                            operator: binary,
                            span: *token.span(),
                        }))
                    } else {
                        Some(Segment::UnaryOperator(UnaryOperatorSegment {
                            operator: unary,
                            span: *token.span(),
                        }))
                    }
                }
                (None, None) => None,
            };

            // If the token was an operator, add it to the segment list, advance to the next token
            // and continue parsing.
            if let Some(operator) = operator {
                segments.push(operator);
                self.next();
                continue;
            }

            // Keep track of whether or not the previous segment we parsed was an expression.
            let previous_is_expr = matches!(segments.last(), Some(Segment::Expr(..)));

            // Parse any expression that could be interpreted as an operand on either side of an
            // operator. For example, if we're parsing the expression "1 + 2 + 3", this will parse
            // the sub-expressions "1", "2" and "3".
            let expr = match token.kind() {
                TokenKind::Whitespace
                | TokenKind::Comment
                | TokenKind::Unknown
                | TokenKind::Eoi => {
                    return Err(ParseError::at_token(
                        ParseErrorKind::UnexpectedToken,
                        &token,
                    ))
                }
                // Parse a keyword-first expression.
                TokenKind::Keyword(keyword) => match keyword {
                    Keyword::True | Keyword::False => {
                        Expr::Boolean(self.eat_boolean_expr()?.into())
                    }
                    Keyword::Null => Expr::Null(self.eat_null_expr()?.into()),
                    Keyword::Fn => Expr::Function(self.eat_function_expr()?.into()),
                    _ => {
                        return Err(ParseError::at_token(
                            ParseErrorKind::UnexpectedToken,
                            &token,
                        ))
                    }
                },
                // Parse a variable expression.
                TokenKind::Ident => Expr::Variable(self.eat_variable_expr()?.into()),
                // Parse a literal expression.
                TokenKind::Literal(literal) => match literal {
                    Literal::String => Expr::String(self.eat_string_expr()?.into()),
                    Literal::Int => Expr::Int(self.eat_int_expr()?.into()),
                    Literal::Float => Expr::Float(self.eat_float_expr()?.into()),
                },
                // Check for certain symbols. What we end up doing here often depends on whether or
                // not the previous segment was an expression.
                TokenKind::Symbol(symbol) => {
                    match symbol {
                        // If the token is "{", check to see if the previous segment was an
                        // expression. If it's an expression, assume the opening brace is the start
                        // of a block and stop eating tokens. Otherwise, try to parse an object
                        // expression.
                        Symbol::OpenBrace => {
                            if previous_is_expr {
                                break;
                            }

                            Expr::Object(self.eat_object_expr()?.into())
                        }
                        // If the token is "[", check to see if the previous segment was an
                        // expression. If it's an expression, attempt to parse an index expression
                        // with the previous segment as the indexed expression. Otherwise, try to
                        // parse a list expression.
                        Symbol::OpenBracket => {
                            if previous_is_expr {
                                if let Segment::Expr(target) = segments.pop().unwrap() {
                                    Expr::Index(self.eat_index_expr(target)?.into())
                                } else {
                                    unreachable!()
                                }
                            } else {
                                Expr::List(self.eat_list_expr()?.into())
                            }
                        }
                        // If the token is ".", attempt to parse a dot expression with the previous
                        // segment as the target expression.
                        Symbol::Dot => {
                            if let Some(Segment::Expr(target)) = segments.pop() {
                                Expr::Dot(self.eat_dot_expr(target)?.into())
                            } else {
                                return Err(ParseError::at_index(
                                    ParseErrorKind::Specific("Invalid dot expression."),
                                    self.index(),
                                ));
                            }
                        }
                        // If the token is "(", check to see if the previous segment was an
                        // expression. If it's an expression, attempt to parse a function call with
                        // the previous segment as the called expression. Otherwise, assume we're
                        // just parsing an expression wrapped in parenthesis.
                        Symbol::OpenParen => {
                            if previous_is_expr {
                                if let Segment::Expr(target) = segments.pop().unwrap() {
                                    Expr::Call(self.eat_call_expr(target)?.into())
                                } else {
                                    unreachable!()
                                }
                            } else {
                                Expr::Wrapped(self.eat_wrapped_expr()?.into())
                            }
                        }
                        // Any other symbol is considered the end of the root expression.
                        _ => break,
                    }
                }
            };

            // Append the parsed expression.
            segments.push(Segment::Expr(expr));
        }

        // If no segments could be parsed, throw an error because no expression was found.
        if segments.is_empty() {
            return Err(ParseError::at_index(
                ParseErrorKind::Expected("expression"),
                self.index(),
            ));
        }

        // Coalesce unary operations (if any) into single expressions.
        segments = self.resolve_unary_operations(segments)?;
        // Coalesce all binary operations (if any) into a single root expression.
        self.resolve_binary_operations(segments)
    }

    fn resolve_unary_operations(
        &mut self,
        segments: Vec<Segment>,
    ) -> ParseResult<'source, Vec<Segment>> {
        assert!(!segments.is_empty());

        // Check invariants.
        {
            for pair in segments.windows(2) {
                let left = &pair[0];
                let right = &pair[1];
                // Throw an error if we find a unary operator with a binary operator on the right.
                if let (
                    Segment::UnaryOperator(..),
                    Segment::BinaryOperator(BinaryOperatorSegment { span, .. }),
                ) = (left, right)
                {
                    return Err(ParseError::at_span(
                        ParseErrorKind::Expected("expression"),
                        *span,
                    ));
                }
            }

            // Throw an error if we find a unary operator with nothing on the right.
            if matches!(segments.last(), Some(Segment::UnaryOperator(..))) {
                return Err(ParseError::at_token_or_index(
                    ParseErrorKind::Expected("expression"),
                    self.lookahead(1).cloned().as_ref(),
                    self.index(),
                ));
            }
        }

        // Output another list of segments with all unary operations coalesced into expressions.
        let mut output: Vec<Segment> = Vec::new();

        // Keep track of all unary operators we've come across since the last target expression.
        // Once we find the expression which all of the operators are applied to, merge them into a
        // single expression with all operators applied to the target from right to left. In this
        // language, all unary operators are prefix.
        let mut unaries: Vec<UnaryOperatorSegment> = Vec::new();

        for segment in segments.into_iter() {
            match segment {
                // If the current segment is an expression, apply all previous unary operators (if
                // any) to the expression from right to left. Append the resulting expression to
                // the output list.
                Segment::Expr(value) => {
                    let start = unaries
                        .first()
                        .map_or_else(|| value.info().span().start(), |unary| unary.span.start());

                    // Drain all unary operators from the buffer and apply them to the target
                    // expression from right to left.
                    let expr = Segment::Expr(unaries.drain(0..unaries.len()).rev().fold(
                        value,
                        |value, unary| {
                            let end = value.info().span().end();
                            Expr::UnaryOperation(
                                UnaryOperationExpr {
                                    info: NodeInfo::new(Span::new(start, end)),
                                    operator: unary.operator,
                                    right: value,
                                }
                                .into(),
                            )
                        },
                    ));

                    // Add the resolved expression to the output list.
                    output.push(expr);
                }
                // If we find a unary operator, add it to the list of unary operators we've found so
                // far.
                Segment::UnaryOperator(unary) => unaries.push(unary),
                // If we find a binary operator, just go ahead and add it to the output list. These
                // will be resolved later.
                Segment::BinaryOperator(binary) => output.push(Segment::BinaryOperator(binary)),
            }
        }

        // Throw an error if there's still unresolved unary operators.
        if !unaries.is_empty() {
            return Err(ParseError::at_token_or_index(
                ParseErrorKind::Expected("expression"),
                self.lookahead(1).cloned().as_ref(),
                self.index(),
            ));
        }

        Ok(output)
    }

    fn resolve_binary_operations(&self, segments: Vec<Segment>) -> ParseResult<'source, Expr> {
        assert!(!segments.is_empty());

        // Check invariants.
        {
            for pair in segments.windows(2) {
                let left = &pair[0];
                let right = &pair[1];
                match (left, right) {
                    // Throw an error if we find two expressions side by side.
                    (Segment::Expr(..), Segment::Expr(right)) => {
                        return Err(ParseError::at_span(
                            ParseErrorKind::Expected("binary operator"),
                            *right.info().span(),
                        ))
                    }
                    // Throw an error if we find two binary operators side by side.
                    (
                        Segment::BinaryOperator(..),
                        Segment::BinaryOperator(BinaryOperatorSegment { span, .. }),
                    ) => {
                        return Err(ParseError::at_span(
                            ParseErrorKind::Expected("expression"),
                            *span,
                        ))
                    }
                    _ => {}
                }
            }

            // Throw an error if we find a binary operator with no left operand.
            if let Some(Segment::BinaryOperator(BinaryOperatorSegment { span, .. })) =
                segments.first()
            {
                return Err(ParseError::at_index(
                    ParseErrorKind::Expected("left operand"),
                    span.start(),
                ));
            }

            // Throw an error if we find a binary operator with no right operand.
            if let Some(Segment::BinaryOperator(BinaryOperatorSegment { span, .. })) =
                segments.last()
            {
                return Err(ParseError::at_index(
                    ParseErrorKind::Expected("right operand"),
                    span.end(),
                ));
            }
        }

        // Get a list of all operator precedences sorted strongest to weakest. Lower numbers have
        // higher precedence. 1 is the strongest precedence.
        let precedences = segments
            .iter()
            .map(|node| {
                if let Segment::BinaryOperator(BinaryOperatorSegment { operator, .. }) = node {
                    Some(operator.precedence())
                } else {
                    None
                }
            })
            .flatten()
            .collect::<BTreeSet<u8>>();

        fn resolve_precedence(
            precedence: u8,
            input: &mut VecDeque<Segment>,
            output: &mut VecDeque<Segment>,
        ) {
            while let Some(segment) = input.pop_front() {
                // If the segment is a binary operator with the precedence we're currently
                // resolving, store the operator. Otherwise just add the current segment to the
                // output buffer and skip to the next segment.
                let operator = match &segment {
                    Segment::BinaryOperator(BinaryOperatorSegment { operator, .. })
                        if operator.precedence() == precedence =>
                    {
                        *operator
                    }
                    _ => {
                        output.push_back(segment);
                        continue;
                    }
                };

                // Get the left and right operands.
                let (left, right) = {
                    // Get the left operand back from the output buffer.
                    let left = output.pop_back().unwrap();
                    // Get the right operand from the front of input buffer.
                    let right = input.pop_front().unwrap();

                    if let (Segment::Expr(left), Segment::Expr(right)) = (left, right) {
                        (left, right)
                    } else {
                        // Because of the invariant checking done before this, this should never
                        // happen.
                        unreachable!()
                    }
                };

                // Create the binary operation expression.
                let expr = Segment::Expr(Expr::BinaryOperation(
                    BinaryOperationExpr {
                        info: NodeInfo::new(Span::new(
                            left.info().span().start(),
                            right.info().span().end(),
                        )),
                        left,
                        operator,
                        right,
                    }
                    .into(),
                ));

                // Add the binary operation expression to the output buffer.
                output.push_back(expr);
            }
        }

        // Maintain two segment buffers. Segments will be moved back and forth between them as each
        // precedence level is resolved.
        let mut buffer_a = VecDeque::from(segments);
        let mut buffer_b = VecDeque::new();

        // A pointer to the buffer that currently contains segments.
        let segments = &mut buffer_a;
        // A pointer to the buffer segments will be moved to after a precendence level is resolved.
        let output = &mut buffer_b;

        // For each precendence level, convert binary operators with that precedence level into
        // binary operation expressions and add them to the output buffer. For each binary operator
        // with the current precedence, it and its operands will be replaced with a single binary
        // operation in the output buffer.
        for precedence in precedences {
            resolve_precedence(precedence, segments, output);
            std::mem::swap(segments, output);
        }

        // After all precedences are resolved, the only thing remaining in the primary segment
        // buffer should be the root expression.
        if let Some(Segment::Expr(expr)) = segments.pop_back() {
            assert!(segments.is_empty());
            assert!(output.is_empty());
            Ok(expr)
        } else {
            unreachable!()
        }
    }

    fn eat_null_expr(&mut self) -> ParseResult<NullExpr> {
        let start = self.start_node();
        expect!(
            self.next(),
            TokenKind::Keyword(Keyword::Null),
            ParseErrorKind::Expected("null"),
            self.index(),
        )?;

        Ok(NullExpr {
            info: self.end_node(start),
        })
    }

    fn eat_boolean_expr(&mut self) -> ParseResult<BooleanExpr> {
        let start = self.start_node();
        let token = expect!(
            self.next(),
            TokenKind::Keyword(Keyword::True) | TokenKind::Keyword(Keyword::False),
            ParseErrorKind::Expected("boolean"),
            self.index(),
        )?;

        Ok(BooleanExpr {
            info: self.end_node(start),
            value: *token.kind() == TokenKind::Keyword(Keyword::True),
        })
    }

    fn eat_int_expr(&mut self) -> ParseResult<IntExpr> {
        let start = self.start_node();
        let token = expect!(
            self.next(),
            TokenKind::Literal(Literal::Int),
            ParseErrorKind::Expected("int"),
            self.index()
        )?;

        Ok(IntExpr {
            info: self.end_node(start),
            value: token.slice().parse::<i64>().unwrap(),
        })
    }

    fn eat_float_expr(&mut self) -> ParseResult<FloatExpr> {
        let start = self.start_node();
        let token = expect!(
            self.next(),
            TokenKind::Literal(Literal::Float),
            ParseErrorKind::Expected("float"),
            self.index()
        )?;

        Ok(FloatExpr {
            info: self.end_node(start),
            value: token.slice().parse::<f64>().unwrap(),
        })
    }

    fn eat_string_expr(&mut self) -> ParseResult<StringExpr> {
        let start = self.start_node();
        let token = expect!(
            self.next(),
            TokenKind::Literal(Literal::String),
            ParseErrorKind::Expected("string"),
            self.index()
        )?;

        let inner = &token.slice()[1..token.slice().len() - 1];

        Ok(StringExpr {
            info: self.end_node(start),
            value: unescape(inner)
                .ok_or_else(|| {
                    ParseError::at_token(
                        // TODO: Improve this error.
                        ParseErrorKind::Specific("Invalid string literal."),
                        &token,
                    )
                })?
                .into(),
        })
    }

    fn eat_variable_expr(&mut self) -> ParseResult<VariableExpr> {
        let start = self.start_node();
        let name = self.eat_ident()?;
        Ok(VariableExpr {
            info: self.end_node(start),
            name,
        })
    }

    fn eat_list_expr(&mut self) -> ParseResult<ListExpr> {
        let start = self.start_node();
        let mut values = Vec::new();

        self.eat_symbol(Symbol::OpenBracket)?;

        while self.peek_kind() != TokenKind::Symbol(Symbol::CloseBracket) {
            values.push(self.eat_expr()?);
            if self.peek_kind() == TokenKind::Symbol(Symbol::CloseBracket) {
                break;
            }

            if self.lookahead_kind(1) == TokenKind::Symbol(Symbol::CloseBracket) {
                self.attempt(|this| this.eat_symbol(Symbol::Comma)).ok();
            } else {
                self.eat_symbol(Symbol::Comma)?;
            }
        }

        self.eat_symbol(Symbol::CloseBracket)?;

        Ok(ListExpr {
            info: self.end_node(start),
            values,
        })
    }

    fn eat_object_expr_pair(&mut self) -> ParseResult<ObjectExprPair> {
        let start = self.start_node();
        let key = match self.peek_kind() {
            TokenKind::Ident => ObjectExprKeyVariant::Identifier(self.eat_ident()?),
            TokenKind::Literal(Literal::String) => {
                ObjectExprKeyVariant::String(self.eat_string_expr()?)
            }
            TokenKind::Symbol(Symbol::OpenBracket) => {
                let start = self.start_node();
                self.eat_symbol(Symbol::OpenBracket)?;
                let value = self.eat_expr()?.into();
                self.eat_symbol(Symbol::CloseBracket)?;
                ObjectExprKeyVariant::Expr(ObjectExprKeyExpr {
                    info: self.end_node(start),
                    value,
                })
            }
            _ => {
                return Err(ParseError::at_token_or_index(
                    ParseErrorKind::Expected("an indentifier string or '['"),
                    self.next().as_ref(),
                    self.index(),
                ))
            }
        };

        self.eat_symbol(Symbol::Colon)?;

        let value = self.eat_expr()?.into();

        Ok(ObjectExprPair {
            info: self.end_node(start),
            key,
            value,
        })
    }

    fn eat_object_expr(&mut self) -> ParseResult<ObjectExpr> {
        let start = self.start_node();
        let mut pairs = Vec::new();

        self.eat_symbol(Symbol::OpenBrace)?;

        while self.peek_kind() != TokenKind::Symbol(Symbol::CloseBrace) {
            pairs.push(self.eat_object_expr_pair()?);
            if self.peek_kind() == TokenKind::Symbol(Symbol::CloseBrace) {
                break;
            }

            if self.lookahead_kind(1) == TokenKind::Symbol(Symbol::CloseBrace) {
                self.attempt(|this| this.eat_symbol(Symbol::Comma)).ok();
            } else {
                self.eat_symbol(Symbol::Comma)?;
            }
        }

        self.eat_symbol(Symbol::CloseBrace)?;

        Ok(ObjectExpr {
            info: self.end_node(start),
            pairs,
        })
    }

    pub fn eat_function_expr(&mut self) -> ParseResult<FunctionExpr> {
        let start = self.start_node();
        self.eat_keyword(Keyword::Fn)?;
        let name = self.attempt(|this| this.eat_ident()).ok();

        let mut parameters = Vec::new();
        let has_parameters = !matches!(
            self.peek_kind(),
            TokenKind::Symbol(Symbol::OpenBrace) | TokenKind::Symbol(Symbol::Arrow)
        );

        if has_parameters {
            self.eat_symbol(Symbol::OpenParen)?;
            while self.peek_kind() != TokenKind::Symbol(Symbol::CloseParen) {
                parameters.push(self.eat_ident()?);
                if self.peek_kind() != TokenKind::Symbol(Symbol::CloseParen) {
                    if self.lookahead_kind(1) == TokenKind::Symbol(Symbol::CloseParen) {
                        self.attempt(|this| this.eat_symbol(Symbol::Comma))?;
                    } else {
                        self.eat_symbol(Symbol::Comma)?;
                    }
                }
            }
            self.eat_symbol(Symbol::CloseParen)?;
        }

        let body = if self.attempt(|this| this.eat_symbol(Symbol::Arrow)).is_ok() {
            FunctionExprBody::Expr(self.eat_expr()?.into())
        } else {
            FunctionExprBody::Block(self.eat_block()?.into())
        };

        Ok(FunctionExpr {
            info: self.end_node(start),
            name: name.map(Box::new),
            parameters,
            body,
        })
    }

    fn eat_wrapped_expr(&mut self) -> ParseResult<WrappedExpr> {
        let start = self.start_node();
        self.eat_symbol(Symbol::OpenParen)?;
        let value = self.eat_expr()?.into();
        self.eat_symbol(Symbol::CloseParen)?;
        Ok(WrappedExpr {
            info: self.end_node(start),
            value,
        })
    }

    fn eat_index_expr(&mut self, target: Expr) -> ParseResult<IndexExpr> {
        let start = target.info().span().start();
        self.eat_symbol(Symbol::OpenBracket)?;
        let index = self.eat_expr()?;
        self.eat_symbol(Symbol::CloseBracket)?;

        Ok(IndexExpr {
            info: self.end_node(start),
            target,
            index,
        })
    }

    fn eat_dot_expr(&mut self, target: Expr) -> ParseResult<DotExpr> {
        let start = target.info().span().start();
        self.eat_symbol(Symbol::Dot)?;
        let property = self.eat_ident()?;
        Ok(DotExpr {
            info: self.end_node(start),
            target,
            property,
        })
    }

    fn eat_call_expr(&mut self, target: Expr) -> ParseResult<CallExpr> {
        let start = target.info().span().start();
        let mut arguments = Vec::new();

        self.eat_symbol(Symbol::OpenParen)?;

        while self.peek_kind() != TokenKind::Symbol(Symbol::CloseParen) {
            arguments.push(self.eat_expr()?);
            if self.peek_kind() == TokenKind::Symbol(Symbol::CloseParen) {
                break;
            }

            if self.lookahead_kind(1) == TokenKind::Symbol(Symbol::CloseParen) {
                self.attempt(|this| this.eat_symbol(Symbol::Comma)).ok();
            } else {
                self.eat_symbol(Symbol::Comma)?;
            }
        }

        self.eat_symbol(Symbol::CloseParen)?;

        Ok(CallExpr {
            info: self.end_node(start),
            target,
            arguments,
        })
    }
}

enum Segment {
    Expr(Expr),
    UnaryOperator(UnaryOperatorSegment),
    BinaryOperator(BinaryOperatorSegment),
}

struct UnaryOperatorSegment {
    operator: UnaryOperator,
    span: Span,
}

struct BinaryOperatorSegment {
    operator: BinaryOperator,
    span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_expr() {
        assert!(matches!(Parser::new("null").eat_expr(), Ok(Expr::Null(..))));
    }

    #[test]
    fn boolean_expr() {
        assert!(matches!(
            Parser::new("true").eat_expr(),
            Ok(Expr::Boolean(..))
        ));
        assert!(matches!(
            Parser::new("false").eat_expr(),
            Ok(Expr::Boolean(..))
        ));
    }

    #[test]
    fn int_expr() {
        assert!(matches!(Parser::new("100").eat_expr(), Ok(Expr::Int(..))));
    }

    #[test]
    fn float_expr() {
        assert!(matches!(
            Parser::new("100.0").eat_expr(),
            Ok(Expr::Float(..))
        ));
    }

    #[test]
    fn string_expr() {
        assert!(matches!(
            Parser::new("\"hello\"").eat_expr(),
            Ok(Expr::String(..))
        ));
    }

    #[test]
    fn list_expr() {
        assert!(matches!(Parser::new("[]").eat_expr(), Ok(Expr::List(..))));
        assert!(matches!(Parser::new("[1]").eat_expr(), Ok(Expr::List(..))));
        assert!(matches!(Parser::new("[1,]").eat_expr(), Ok(Expr::List(..))));
        assert!(matches!(
            Parser::new("[1,2,3]").eat_expr(),
            Ok(Expr::List(..))
        ));
        assert!(matches!(
            Parser::new("[1, 2, 3]").eat_expr(),
            Ok(Expr::List(..))
        ));
        assert!(matches!(
            Parser::new("[true, 1, \"string\", [], {}]").eat_expr(),
            Ok(Expr::List(..))
        ));
    }

    #[test]
    fn object_expr() {
        assert!(matches!(Parser::new("{}").eat_expr(), Ok(Expr::Object(..))));
        assert!(matches!(
            Parser::new(
                "{
                    name: \"Steve\",
                    points: 100
                }"
            )
            .eat_expr(),
            Ok(Expr::Object(..))
        ));
        assert!(matches!(
            Parser::new(
                "{
                    name: \"Steve\",
                    points: 100,
                }"
            )
            .eat_expr(),
            Ok(Expr::Object(..))
        ));
        assert!(matches!(
            Parser::new(
                "{
                    \"name\": \"Steve\",
                    \"points\": 100
                }"
            )
            .eat_expr(),
            Ok(Expr::Object(..))
        ));
        assert!(matches!(
            Parser::new(
                "{
                    [\"name\"]: \"Steve\",
                    [\"points\"]: 100
                }"
            )
            .eat_expr(),
            Ok(Expr::Object(..))
        ));
        assert!(matches!(
            Parser::new(
                "{
                    [null]: null,
                    [true]: true,
                    [false]: false,
                    [100]: 100,
                    [100.0]: 100.0,
                    [[]]: [1, 2, 3],
                    [{}]: { a: 1, b: 2, c: 3 },
                }"
            )
            .eat_expr(),
            Ok(Expr::Object(..))
        ));
    }

    #[test]
    fn function_expr() {
        assert!(matches!(
            Parser::new("fn run() {}").eat_expr(),
            Ok(Expr::Function(..))
        ));
        assert!(matches!(
            Parser::new("fn run(a, b, c) {}").eat_expr(),
            Ok(Expr::Function(..))
        ));
        assert!(matches!(
            Parser::new("fn run(a, b, c,) {}").eat_expr(),
            Ok(Expr::Function(..))
        ));
        assert!(matches!(
            Parser::new("fn () {}").eat_expr(),
            Ok(Expr::Function(..))
        ));
        assert!(matches!(
            Parser::new("fn (a, b, c) {}").eat_expr(),
            Ok(Expr::Function(..))
        ));
        assert!(matches!(
            Parser::new("fn (a, b, c,) {}").eat_expr(),
            Ok(Expr::Function(..))
        ));
        assert!(matches!(
            Parser::new("fn {}").eat_expr(),
            Ok(Expr::Function(..))
        ));

        assert!(matches!(
            Parser::new("fn run() => null").eat_expr(),
            Ok(Expr::Function(..))
        ));
        assert!(matches!(
            Parser::new("fn run(a, b, c) => null").eat_expr(),
            Ok(Expr::Function(..))
        ));
        assert!(matches!(
            Parser::new("fn run(a, b, c,) => null").eat_expr(),
            Ok(Expr::Function(..))
        ));
        assert!(matches!(
            Parser::new("fn () => null").eat_expr(),
            Ok(Expr::Function(..))
        ));
        assert!(matches!(
            Parser::new("fn (a, b, c) => null").eat_expr(),
            Ok(Expr::Function(..))
        ));
        assert!(matches!(
            Parser::new("fn (a, b, c,) => null").eat_expr(),
            Ok(Expr::Function(..))
        ));
        assert!(matches!(
            Parser::new("fn => null").eat_expr(),
            Ok(Expr::Function(..))
        ));
    }

    #[test]
    fn wrapped_expr() {
        assert!(matches!(
            Parser::new("(1)").eat_expr(),
            Ok(Expr::Wrapped(..))
        ));
        assert!(matches!(
            Parser::new("([1, 2, 3])").eat_expr(),
            Ok(Expr::Wrapped(..))
        ));
        assert!(matches!(
            Parser::new("({ a: 1, b: 2, c: 3 })").eat_expr(),
            Ok(Expr::Wrapped(..))
        ));
    }

    #[test]
    fn index_expr() {
        assert!(matches!(
            Parser::new("container[0]").eat_expr(),
            Ok(Expr::Index(..))
        ));
        assert!(matches!(
            Parser::new("container[0][0]").eat_expr(),
            Ok(Expr::Index(..))
        ));
        assert!(matches!(
            Parser::new("[][0]").eat_expr(),
            Ok(Expr::Index(..))
        ));
        assert!(matches!(
            Parser::new("\"string\"[0]").eat_expr(),
            Ok(Expr::Index(..))
        ));
    }

    #[test]
    fn dot_expr() {
        assert!(matches!(
            Parser::new("object.property").eat_expr(),
            Ok(Expr::Dot(..))
        ));
        assert!(matches!(
            Parser::new("object.property.property").eat_expr(),
            Ok(Expr::Dot(..))
        ));
        assert!(matches!(
            Parser::new("{}.property").eat_expr(),
            Ok(Expr::Dot(..))
        ));
    }

    #[test]
    fn call_expr() {
        assert!(matches!(
            Parser::new("function()").eat_expr(),
            Ok(Expr::Call(..))
        ));
        assert!(matches!(
            Parser::new("function()()").eat_expr(),
            Ok(Expr::Call(..))
        ));
        assert!(matches!(
            Parser::new("function(a, b, c)").eat_expr(),
            Ok(Expr::Call(..))
        ));
        assert!(matches!(
            Parser::new("function(a, b, c,)").eat_expr(),
            Ok(Expr::Call(..))
        ));
        assert!(matches!(
            Parser::new("fn run() {}()").eat_expr(),
            Ok(Expr::Call(..))
        ));
        assert!(matches!(
            Parser::new("fn run() {}(a, b, c)(a, b, c)").eat_expr(),
            Ok(Expr::Call(..))
        ));
    }

    #[test]
    fn unary_operation_expr() {
        assert!(matches!(
            Parser::new("-1").eat_expr(),
            Ok(Expr::UnaryOperation(..))
        ));
        assert!(matches!(
            Parser::new("~1").eat_expr(),
            Ok(Expr::UnaryOperation(..))
        ));
        assert!(matches!(
            Parser::new("not true").eat_expr(),
            Ok(Expr::UnaryOperation(..))
        ));
        assert!(matches!(
            Parser::new("not -~1").eat_expr(),
            Ok(Expr::UnaryOperation(..))
        ));
    }

    #[test]
    fn binary_operation_expr() {
        assert!(matches!(
            Parser::new("1 + 1").eat_expr(),
            Ok(Expr::BinaryOperation(..))
        ));

        assert!(matches!(
            Parser::new("1 + 2 - 3 * 10 / null ?? 5 > 1000 == true").eat_expr(),
            Ok(Expr::BinaryOperation(..))
        ));
    }

    #[test]
    fn mixed_operation_expr() {
        assert!(matches!(
            Parser::new("not false and 1 + -2 - ~3 or not true").eat_expr(),
            Ok(Expr::BinaryOperation(..))
        ));
    }
}
