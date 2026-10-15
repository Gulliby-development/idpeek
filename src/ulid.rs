//! ULID decoding and timestamp-bound construction.

use crate::base32;
use crate::calendar;
use crate::detect::Decoded;

/// Decode a 26-character Crockford base32 ULID.
pub fn decode(input: &str) -> Option<Decoded> {
    let value = base32::decode(input)?;
    let timestamp = i64::try_from(value >> 80).ok()?;
    let random = value & ((1_u128 << 80) - 1);
    let fields = vec![
        ("time", calendar::format_iso(timestamp)),
        ("random", format!("{random:020x}")),
    ];

    Some(Decoded {
        kind: "ulid",
        fields,
        time_ms: Some(timestamp),
    })
}

/// Construct the lexicographically smallest or largest ULID for a timestamp.
pub fn bound(milliseconds: i64, max: bool) -> Option<String> {
    const MAX_TIMESTAMP: i64 = (1_i64 << 48) - 1;
    if !(0..=MAX_TIMESTAMP).contains(&milliseconds) {
        return None;
    }

    let timestamp = u128::try_from(milliseconds).ok()?;
    let random = if max { (1_u128 << 80) - 1 } else { 0 };
    Some(base32::encode((timestamp << 80) | random))
}
