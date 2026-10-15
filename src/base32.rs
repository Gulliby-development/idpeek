//! Crockford base32 conversion for 128-bit identifier values.

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Decode a 26-character Crockford base32 value.
///
/// The alphabet is case-insensitive, but the ambiguous letters I, L, O and U
/// are not aliases. Values that do not fit in 128 bits are rejected.
pub fn decode(value: &str) -> Option<u128> {
    if value.len() != 26 {
        return None;
    }

    let mut decoded = 0_u128;
    for digit in value.bytes() {
        let digit_value = alphabet_value(digit)?;
        decoded = decoded
            .checked_mul(32)?
            .checked_add(u128::from(digit_value))?;
    }
    Some(decoded)
}

/// Encode a 128-bit value as 26 uppercase Crockford base32 characters.
pub fn encode(mut value: u128) -> String {
    let mut encoded = [b'0'; 26];
    for index in (0..encoded.len()).rev() {
        let digit = (value & 0x1f) as usize;
        encoded[index] = ALPHABET[digit];
        value >>= 5;
    }
    String::from_utf8(encoded.to_vec()).expect("Crockford alphabet is ASCII")
}

fn alphabet_value(character: u8) -> Option<u8> {
    match character {
        b'0'..=b'9' => Some(character - b'0'),
        b'A'..=b'H' => Some(character - b'A' + 10),
        b'J'..=b'K' => Some(character - b'J' + 18),
        b'M'..=b'N' => Some(character - b'M' + 20),
        b'P'..=b'T' => Some(character - b'P' + 22),
        b'V'..=b'Z' => Some(character - b'V' + 27),
        b'a'..=b'h' => Some(character - b'a' + 10),
        b'j'..=b'k' => Some(character - b'j' + 18),
        b'm'..=b'n' => Some(character - b'm' + 20),
        b'p'..=b't' => Some(character - b'p' + 22),
        b'v'..=b'z' => Some(character - b'v' + 27),
        _ => None,
    }
}
