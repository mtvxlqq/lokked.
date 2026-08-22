//! Tests for the base64 decoder: the canvas data URL in, the PNG bytes out.

use lokked_lib::core::base64::{decode, Base64Error};

#[test]
fn the_classic_example_round_trips() {
    assert_eq!(decode("TWFu").unwrap(), b"Man");
    assert_eq!(decode("cGxlYXN1cmUu").unwrap(), b"pleasure.");
}

#[test]
fn padding_is_handled_at_both_lengths() {
    assert_eq!(decode("TWE=").unwrap(), b"Ma");
    assert_eq!(decode("TQ==").unwrap(), b"M");
}

#[test]
fn a_data_url_prefix_is_stripped() {
    assert_eq!(decode("data:image/png;base64,TWFu").unwrap(), b"Man");
}

#[test]
fn line_breaks_inside_the_payload_are_ignored() {
    assert_eq!(decode("TWFu\ncGxl\r\nYXN1\ncmUu").unwrap(), b"Manpleasure.");
}

#[test]
fn nothing_decodes_to_nothing() {
    assert!(decode("").unwrap().is_empty());
    assert!(decode("data:image/png;base64,").unwrap().is_empty());
}

#[test]
fn a_character_outside_the_alphabet_is_refused() {
    assert_eq!(decode("TW$u"), Err(Base64Error::BadCharacter('$')));
}

#[test]
fn a_payload_cut_mid_group_is_refused() {
    // Пять шестёрок битов — это обрезанная строка: последняя группа неполна.
    assert!(matches!(decode("TWFuY"), Err(Base64Error::BadLength(_))));
}

#[test]
fn every_byte_value_survives_the_trip() {
    // Все 256 значений, закодированные эталонной таблицей.
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes: Vec<u8> = (0..=255).collect();

    let mut encoded = String::new();
    for chunk in bytes.chunks(3) {
        let mut group = [0_u8; 3];
        group[..chunk.len()].copy_from_slice(chunk);
        let packed = u32::from_be_bytes([0, group[0], group[1], group[2]]);

        for step in 0..4 {
            if step <= chunk.len() {
                encoded.push(ALPHABET[((packed >> (18 - step * 6)) & 0x3F) as usize] as char);
            } else {
                encoded.push('=');
            }
        }
    }

    assert_eq!(decode(&encoded).unwrap(), bytes);
}
