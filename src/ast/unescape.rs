use std::char;
use std::collections::VecDeque;

pub fn unescape(string: &str) -> Option<String> {
    let mut characters = String::from(string).chars().collect::<VecDeque<_>>();
    let mut result = String::new();

    while let Some(character) = characters.pop_front() {
        if character != '\\' {
            result.push(character);
            continue;
        }

        match characters.pop_front() {
            Some('b') => result.push('\u{0008}'),
            Some('f') => result.push('\u{000C}'),
            Some('n') => result.push('\n'),
            Some('r') => result.push('\r'),
            Some('t') => result.push('\t'),
            Some('\'') => result.push('\''),
            Some('\"') => result.push('\"'),
            Some('\\') => result.push('\\'),
            Some('u') => result.push(unescape_unicode(&mut characters)?),
            _ => return None,
        };
    }

    Some(result)
}

fn unescape_unicode(characters: &mut VecDeque<char>) -> Option<char> {
    let mut result = String::new();

    match characters.pop_front()? {
        '{' => loop {
            match characters.pop_front()? {
                '}' => break,
                character => result.push(character),
            }
        },
        character => {
            result.push(character);
            for _ in 0..3 {
                result.push(characters.pop_front()?);
            }
        }
    }

    u32::from_str_radix(&result, 16).map_or(None, char::from_u32)
}
