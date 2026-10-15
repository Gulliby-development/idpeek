//! KSUID decoding and construction using a fixed-width base62 codec.

use crate::text::hex;

/// The KSUID epoch as Unix seconds (2014-05-13T16:53:20Z).
pub const EPOCH_UNIX_SECONDS: u64 = 1_400_000_000;

const ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// The fields decoded from a KSUID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decoded {
    /// The embedded creation time in Unix milliseconds.
    pub time_ms: i64,
    /// The 128-bit payload.
    pub payload: [u8; 16],
}

/// Decode a 27-character KSUID into its time and payload.
pub fn decode(text: &str) -> Option<Decoded> {
    let bytes = decode_bytes(text)?;
    let relative_seconds = u32::from_be_bytes(bytes[..4].try_into().ok()?);
    let seconds = EPOCH_UNIX_SECONDS.checked_add(u64::from(relative_seconds))?;
    let time_ms = i64::try_from(seconds.checked_mul(1_000)?).ok()?;
    let payload = bytes[4..].try_into().ok()?;

    Some(Decoded { time_ms, payload })
}

/// Decode exactly 27 base62 characters into a big-endian 160-bit value.
///
/// Returns None for an invalid alphabet, a wrong length, or a value that
/// does not fit in 20 bytes.
pub fn decode_bytes(text: &str) -> Option<[u8; 20]> {
    if text.len() != 27 || !text.is_ascii() {
        return None;
    }

    let mut value = [0_u8; 20];
    for byte in text.bytes() {
        let digit = u16::from(base62_digit(byte)?);
        let mut carry = digit;
        for slot in value.iter_mut().rev() {
            let expanded = u16::from(*slot) * 62 + carry;
            *slot = expanded as u8;
            carry = expanded >> 8;
        }
        if carry != 0 {
            return None;
        }
    }
    Some(value)
}

/// Encode a big-endian 160-bit value as 27 base62 characters.
pub fn encode(bytes: &[u8; 20]) -> String {
    let mut quotient = *bytes;
    let mut digits = [b'0'; 27];

    for output in digits.iter_mut().rev() {
        let mut remainder = 0_u16;
        for byte in &mut quotient {
            let expanded = remainder * 256 + u16::from(*byte);
            *byte = (expanded / 62) as u8;
            remainder = expanded % 62;
        }
        *output = ALPHABET[usize::from(remainder)];
    }

    digits.iter().map(|byte| char::from(*byte)).collect()
}

/// Build the smallest or largest KSUID for a timestamp.
///
/// KSUID timestamps have whole-second precision, so milliseconds are floored
/// before the supported range is checked.
pub fn bound(time_ms: i64, max: bool) -> Option<String> {
    let seconds = time_ms.div_euclid(1_000);
    let minimum = i64::try_from(EPOCH_UNIX_SECONDS).ok()?;
    let maximum = minimum.checked_add(i64::from(u32::MAX))?;
    if !(minimum..=maximum).contains(&seconds) {
        return None;
    }

    let relative_seconds = u32::try_from(seconds - minimum).ok()?;
    let mut bytes = [0_u8; 20];
    bytes[..4].copy_from_slice(&relative_seconds.to_be_bytes());
    if max {
        bytes[4..].fill(u8::MAX);
    }
    Some(encode(&bytes))
}

/// Render the 128-bit KSUID payload as lowercase hexadecimal.
pub fn payload_hex(decoded: &Decoded) -> String {
    hex(&decoded.payload)
}

fn base62_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'Z' => Some(byte - b'A' + 10),
        b'a'..=b'z' => Some(byte - b'a' + 36),
        _ => None,
    }
}
