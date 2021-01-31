macro_rules! expect {
    ($token:expr, $($pattern:pat)|+, $error_kind:expr, $index:expr $(,)?) => {{
        if let Some(token) = $token {
            if matches!(token.kind(), $($pattern)|+) {
                Ok(token)
            } else {
                Err(ParseError::at_token($error_kind, &token))
            }
        } else {
            Err(ParseError::at_index($error_kind, $index))
        }
    }};
}

macro_rules! expect_exact {
    ($token:expr, $kind:expr, $error_kind:expr, $index:expr $(,)?) => {{
        if let Some(token) = $token {
            if *token.kind() == $kind {
                Ok(token)
            } else {
                Err(ParseError::at_token($error_kind, &token))
            }
        } else {
            Err(ParseError::at_index($error_kind, $index))
        }
    }};
}
