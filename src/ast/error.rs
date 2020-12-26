use pest::error::{InputLocation, LineColLocation};

use super::grammar::{GrammarError, GrammarErrorVariant, GrammarRule};
use super::location::{Location, Position};

#[derive(Debug)]
pub struct ParseError {
    pub location: Location,
    pub positives: Vec<String>,
    pub negatives: Vec<String>,
}

impl ParseError {
    fn new(location: Location, positives: Vec<String>, negatives: Vec<String>) -> Self {
        Self {
            location,
            positives,
            negatives,
        }
    }

    pub(super) fn from_grammar_error(error: GrammarError) -> Self {
        let (positives, negatives) = match error.variant {
            GrammarErrorVariant::ParsingError {
                positives,
                negatives,
            } => (
                positives
                    .iter()
                    .map(|rule| Self::display_grammar_rule(rule))
                    .collect(),
                negatives
                    .iter()
                    .map(|rule| Self::display_grammar_rule(rule))
                    .collect(),
            ),
            GrammarErrorVariant::CustomError { .. } => (Vec::new(), Vec::new()),
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

        ParseError::new(Location { start, end }, positives, negatives)
    }

    fn display_grammar_rule(error: &GrammarRule) -> String {
        format!("{:?}", error)
    }
}
