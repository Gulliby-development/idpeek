//! MongoDB ObjectId decoding and timestamp-bound construction.

use crate::calendar;
use crate::detect::Decoded;
use crate::text;

/// Decode a 24-hex-digit MongoDB ObjectId.
pub fn decode(input: &str) -> Option<Decoded> {
    let bytes = text::parse_hex(input)?;
    let bytes: [u8; 12] = bytes.try_into().ok()?;
    let seconds = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let milliseconds = i64::from(seconds) * 1_000;
    let random = text::hex(&bytes[4..9]);
    let counter = (u32::from(bytes[9]) << 16) | (u32::from(bytes[10]) << 8) | u32::from(bytes[11]);
    let fields = vec![
        ("time", calendar::format_iso(milliseconds)),
        ("random", random),
        ("counter", counter.to_string()),
    ];

    Some(Decoded {
        kind: "objectid",
        fields,
        time_ms: Some(milliseconds),
    })
}

/// Construct the smallest or largest ObjectId for the timestamp's second.
pub fn bound(milliseconds: i64, max: bool) -> Option<String> {
    const MAX_SECONDS: i64 = u32::MAX as i64;
    let seconds = milliseconds.div_euclid(1_000);
    if !(0..=MAX_SECONDS).contains(&seconds) {
        return None;
    }

    let seconds = u32::try_from(seconds).ok()?;
    let suffix = if max { u64::MAX } else { 0 };
    let mut bytes = [0_u8; 12];
    bytes[..4].copy_from_slice(&seconds.to_be_bytes());
    bytes[4..].copy_from_slice(&suffix.to_be_bytes());
    Some(text::hex(&bytes))
}
