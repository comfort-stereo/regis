use super::error::ParseError;

pub type ParseResult<'source, T> = Result<T, ParseError>;
