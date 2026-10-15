//! Snowflake decoding and time-bound construction.

/// The number of timestamp milliseconds represented by Snowflake's 41 bits.
pub const MAX_TIMESTAMP_DELTA_MS: i64 = (1_i64 << 41) - 1;
const LOW_BITS_MASK: u64 = (1_u64 << 22) - 1;

/// The fields decoded from a Snowflake identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decoded {
    /// The configured epoch in Unix milliseconds.
    pub epoch_ms: i64,
    /// The embedded time in Unix milliseconds.
    pub time_ms: i64,
    /// The 10-bit worker identifier.
    pub worker: u16,
    /// The 12-bit sequence number.
    pub sequence: u16,
}

/// Decode an unsigned decimal Snowflake identifier with the selected epoch.
pub fn decode(text: &str, epoch_ms: i64) -> Option<Decoded> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let value = text.parse::<u64>().ok()?;
    let relative_ms = i64::try_from(value >> 22).ok()?;
    let time_ms = epoch_ms.checked_add(relative_ms)?;

    Some(Decoded {
        epoch_ms,
        time_ms,
        worker: ((value >> 12) & 1_023) as u16,
        sequence: (value & 4_095) as u16,
    })
}

/// Construct the smallest or largest Snowflake identifier for a timestamp.
///
/// The timestamp must be from the selected epoch through its 41-bit maximum.
pub fn bound(time_ms: i64, max: bool, epoch_ms: i64) -> Option<String> {
    let relative_ms = time_ms.checked_sub(epoch_ms)?;
    if !(0..=MAX_TIMESTAMP_DELTA_MS).contains(&relative_ms) {
        return None;
    }

    let low_bits = if max { LOW_BITS_MASK } else { 0 };
    let value = (u64::try_from(relative_ms).ok()? << 22) | low_bits;
    Some(value.to_string())
}
