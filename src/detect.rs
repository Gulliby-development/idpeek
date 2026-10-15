//! Identifier dispatch and shared decoded representation.

use crate::calendar;
use crate::{ksuid, objectid, snowflake, text, ulid, uuid};

/// A decoded identifier in report-ready field order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decoded {
    /// Canonical display name for the identifier family.
    pub kind: &'static str,
    /// Ordered name/value fields, excluding the leading kind field.
    pub fields: Vec<(&'static str, String)>,
    /// Creation time in Unix milliseconds when this format/version carries one.
    pub time_ms: Option<i64>,
}

/// Decode an identifier according to its character count and selected epoch.
///
/// Length selects the family before format validation. This means malformed
/// values of a recognized length receive a family-specific error.
pub fn decode(input: &str, epoch_ms: i64) -> Result<Decoded, String> {
    match input.chars().count() {
        32 | 36 => uuid::decode(input).ok_or_else(|| invalid("UUID", input)),
        24 => objectid::decode(input).ok_or_else(|| invalid("ObjectId", input)),
        26 => ulid::decode(input).ok_or_else(|| invalid("ULID", input)),
        27 => ksuid::decode(input)
            .map(|decoded| Decoded {
                kind: "ksuid",
                fields: vec![
                    ("time", calendar::format_iso(decoded.time_ms)),
                    ("payload", ksuid::payload_hex(&decoded)),
                ],
                time_ms: Some(decoded.time_ms),
            })
            .ok_or_else(|| invalid("KSUID", input)),
        15..=20 if input.bytes().all(|byte| byte.is_ascii_digit()) => {
            snowflake::decode(input, epoch_ms)
                .map(|decoded| Decoded {
                    kind: "snowflake",
                    fields: vec![
                        ("epoch", decoded.epoch_ms.to_string()),
                        ("time", calendar::format_iso(decoded.time_ms)),
                        ("worker", decoded.worker.to_string()),
                        ("sequence", decoded.sequence.to_string()),
                    ],
                    time_ms: Some(decoded.time_ms),
                })
                .ok_or_else(|| invalid("snowflake", input))
        }
        _ => Err(format!("unknown ID format {}", text::show(input))),
    }
}

/// Construct the minimum or maximum identifier bound for a supported kind.
///
/// The supplied time is Unix milliseconds. Formats with second precision
/// perform their own floor-to-second conversion.
pub fn bound(kind: &str, milliseconds: i64, max: bool, epoch_ms: i64) -> Result<String, String> {
    let result = match kind {
        "ulid" => ulid::bound(milliseconds, max),
        "objectid" => objectid::bound(milliseconds, max),
        "ksuid" => ksuid::bound(milliseconds, max),
        "uuid7" => uuid7_bound(milliseconds, max),
        "snowflake" => snowflake::bound(milliseconds, max, epoch_ms),
        _ => return Err(format!("invalid KIND {}", text::show(kind))),
    };

    result.ok_or_else(|| format!("time is out of range for {kind}"))
}

fn invalid(kind: &str, input: &str) -> String {
    format!("invalid {kind} {}", text::show(input))
}

fn uuid7_bound(milliseconds: i64, max: bool) -> Option<String> {
    const MAX_TIMESTAMP: i64 = (1_i64 << 48) - 1;
    if !(0..=MAX_TIMESTAMP).contains(&milliseconds) {
        return None;
    }

    let timestamp = u64::try_from(milliseconds).ok()?;
    let timestamp_hex = format!("{timestamp:012x}");
    let tail = if max {
        "7fff-bfff-ffffffffffff"
    } else {
        "7000-8000-000000000000"
    };
    Some(format!(
        "{}-{}-{tail}",
        &timestamp_hex[..8],
        &timestamp_hex[8..]
    ))
}
