use itertools::Itertools;

use pest::error::{InputLocation, LineColLocation};

use super::grammar::{GrammarError, GrammarErrorVariant, GrammarRule};
use super::location::{Location, Position};

#[derive(Debug)]
pub struct ParseError {
    pub location: Location,
    pub expected: Vec<String>,
}

impl ParseError {
    fn new(location: Location, expected: Vec<String>) -> Self {
        Self { location, expected }
    }

    pub(super) fn from_grammar_error(error: GrammarError) -> Self {
        let expected = match error.variant {
            GrammarErrorVariant::ParsingError { positives, .. } => positives
                .iter()
                .filter_map(|rule| Self::display_grammar_rule(rule))
                .unique()
                .collect(),
            GrammarErrorVariant::CustomError { .. } => Vec::new(),
        };

        let (start_index, end_index) = match error.location {
            InputLocation::Pos(start) => (start, start),
            InputLocation::Span((start, end)) => (start, end),
        };

        let (start, end) = match error.line_col {
            LineColLocation::Pos((line, column)) => {
                let start = Position {
                    index: start_index,
                    line,
                    column,
                };
                let end = Position {
                    index: end_index,
                    ..start
                };

                (start, end)
            }
            LineColLocation::Span((start_line, start_column), (end_line, end_column)) => {
                let start = Position {
                    index: start_index,
                    line: start_line,
                    column: start_column,
                };
                let end = Position {
                    index: end_index,
                    line: end_line,
                    column: end_column,
                };

                (start, end)
            }
        };

        ParseError::new(
            Location {
                path: None,
                start,
                end,
            },
            expected,
        )
    }

    fn display_grammar_rule(error: &GrammarRule) -> Option<String> {
        match error {
            GrammarRule::wrapped => None,
            GrammarRule::binary_operations => None,
            GrammarRule::chain => None,
            GrammarRule::operator_binary_ncl
            | GrammarRule::operator_binary_mul
            | GrammarRule::operator_binary_div
            | GrammarRule::operator_binary_add
            | GrammarRule::operator_binary_sub
            | GrammarRule::operator_binary_gt
            | GrammarRule::operator_binary_lt
            | GrammarRule::operator_binary_gte
            | GrammarRule::operator_binary_lte
            | GrammarRule::operator_binary_eq
            | GrammarRule::operator_binary_neq
            | GrammarRule::operator_binary_and
            | GrammarRule::operator_binary_or
            | GrammarRule::operator_binary_push => Some("binary-operator".into()),
            GrammarRule::operator_assign_direct
            | GrammarRule::operator_assign_ncl
            | GrammarRule::operator_assign_mul
            | GrammarRule::operator_assign_div
            | GrammarRule::operator_assign_add
            | GrammarRule::operator_assign_sub
            | GrammarRule::operator_assign_and
            | GrammarRule::operator_assign_or => Some("assignment-operator".into()),
            GrammarRule::index => Some("index".into()),
            GrammarRule::dot => Some("dot".into()),
            GrammarRule::call => Some("call".into()),
            error => Some(format!("{:?}", error)),
        }
    }
}
