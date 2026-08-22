//! Decoding base64, in the one shape the app needs it: a data URL from a
//! `<canvas>` on its way to a PNG file.
//!
//! Thirty lines of arithmetic instead of a dependency. Standard alphabet
//! only — that is what `canvas.toDataURL` produces — and the `data:` prefix
//! is stripped here rather than on the frontend, so the command takes what
//! the browser gave it and nothing has to agree on a format in between.

use std::fmt;

/// Why a string could not be read as base64.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Base64Error {
    /// A character outside the alphabet.
    BadCharacter(char),
    /// The data ran out mid-group.
    BadLength(usize),
}

impl fmt::Display for Base64Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadCharacter(found) => write!(f, "картинка испорчена: символ {found:?}"),
            Self::BadLength(length) => write!(f, "картинка испорчена: длина {length}"),
        }
    }
}

impl std::error::Error for Base64Error {}

/// The value of one base64 character, or `None` for padding and whitespace.
fn value_of(character: char) -> Option<Result<u8, Base64Error>> {
    let value = match character {
        'A'..='Z' => character as u8 - b'A',
        'a'..='z' => character as u8 - b'a' + 26,
        '0'..='9' => character as u8 - b'0' + 52,
        '+' => 62,
        '/' => 63,
        // Дополнение и переносы строк — не данные, но и не ошибка.
        '=' | '\n' | '\r' | ' ' | '\t' => return None,
        other => return Some(Err(Base64Error::BadCharacter(other))),
    };

    Some(Ok(value))
}

/// Decodes base64, ignoring a `data:…;base64,` prefix if there is one.
pub fn decode(encoded: &str) -> Result<Vec<u8>, Base64Error> {
    let payload = match encoded.split_once("base64,") {
        Some((_, tail)) => tail,
        None => encoded,
    };

    let mut bytes = Vec::with_capacity(payload.len() / 4 * 3);
    let mut buffer: u32 = 0;
    let mut filled = 0;

    for character in payload.chars() {
        let Some(value) = value_of(character) else {
            continue;
        };
        buffer = (buffer << 6) | u32::from(value?);
        filled += 1;

        if filled == 4 {
            bytes.extend_from_slice(&[(buffer >> 16) as u8, (buffer >> 8) as u8, buffer as u8]);
            buffer = 0;
            filled = 0;
        }
    }

    match filled {
        0 => {}
        2 => bytes.push((buffer >> 4) as u8),
        3 => {
            bytes.push((buffer >> 10) as u8);
            bytes.push((buffer >> 2) as u8);
        }
        // Одна оставшаяся шестёрка битов — это обрезанная строка, а не данные.
        _ => return Err(Base64Error::BadLength(payload.len())),
    }

    Ok(bytes)
}
