use std::char;
use std::collections::VecDeque;

pub fn unescape(string: &str) -> Option<String> {
    let mut characters: VecDeque<_> = String::from(string).chars().collect();
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
            Some('u') => result.push(match unescape_unicode(&mut characters) {
                Some(unicode) => unicode,
                None => continue,
            }),
            _ => return None,
        };
    }

    Some(result)
}

fn unescape_unicode(characters: &mut VecDeque<char>) -> Option<char> {
    let mut result = String::new();

    match characters.pop_front() {
        Some('{') => loop {
            match characters.pop_front() {
                Some('}') => break,
                Some(character) => result.push(character),
                None => return None,
            }
        },
        Some(character) => {
            result.push(character);
            for _ in 0..3 {
                match characters.pop_front() {
                    Some(character) => result.push(character),
                    None => return None,
                }
            }
        }
        None => return None,
    }

    u32::from_str_radix(&result, 16).map_or(None, |value| char::from_u32(value))
}
