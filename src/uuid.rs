//! UUID decoding for the time-bearing UUID versions and their descriptive fields.

use crate::calendar;
use crate::detect::Decoded;
use crate::text;

/// Decode a hyphenated or compact UUID into the shared report representation.
pub fn decode(input: &str) -> Option<Decoded> {
    let bytes = parse_uuid(input)?;
    let version = bytes[6] >> 4;
    let variant = match bytes[8] >> 5 {
        0b000..=0b011 => "ncs",
        0b100..=0b101 => "rfc4122",
        0b110 => "microsoft",
        _ => "future",
    };

    let version_name = match version {
        1 => "time and node",
        2 => "dce security",
        3 => "md5 name",
        4 => "random",
        5 => "sha1 name",
        6 => "reordered time",
        7 => "unix time and random",
        8 => "custom",
        _ => "unassigned",
    };

    let mut fields = vec![
        ("version", format!("{version} ({version_name})")),
        ("variant", variant.to_owned()),
    ];
    let time_ms = match version {
        1 => {
            let time_low = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            let time_mid = u16::from_be_bytes([bytes[4], bytes[5]]);
            let time_hi = u16::from_be_bytes([bytes[6] & 0x0f, bytes[7]]);
            let ticks =
                (u64::from(time_hi) << 48) | (u64::from(time_mid) << 32) | u64::from(time_low);
            Some(uuid_ticks_to_unix_ms(ticks))
        }
        6 => {
            let first_48 = u64::from_be_bytes([
                0, 0, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
            ]);
            let time_hi = u16::from_be_bytes([bytes[6] & 0x0f, bytes[7]]);
            let ticks = (first_48 << 12) | u64::from(time_hi);
            Some(uuid_ticks_to_unix_ms(ticks))
        }
        7 => {
            let timestamp = u64::from_be_bytes([
                0, 0, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
            ]);
            Some(i64::try_from(timestamp).ok()?)
        }
        _ => None,
    };

    if matches!(version, 1 | 6) {
        let clock = (u16::from(bytes[8] & 0x3f) << 8) | u16::from(bytes[9]);
        fields.push(("clock", clock.to_string()));
        fields.push(("node", text::hex(&bytes[10..16])));
    }
    if let Some(milliseconds) = time_ms {
        fields.insert(2, ("time", calendar::format_iso(milliseconds)));
    }

    Some(Decoded {
        kind: "uuid",
        fields,
        time_ms,
    })
}

fn parse_uuid(input: &str) -> Option<[u8; 16]> {
    let compact = match input.len() {
        32 if input.bytes().all(|byte| byte.is_ascii_hexdigit()) => input.to_owned(),
        36 => {
            let bytes = input.as_bytes();
            if bytes[8] != b'-'
                || bytes[13] != b'-'
                || bytes[18] != b'-'
                || bytes[23] != b'-'
                || bytes.iter().enumerate().any(|(index, byte)| {
                    !matches!(index, 8 | 13 | 18 | 23) && !byte.is_ascii_hexdigit()
                })
            {
                return None;
            }
            input
                .chars()
                .filter(|&character| character != '-')
                .collect()
        }
        _ => return None,
    };
    let parsed = text::parse_hex(&compact)?;
    parsed.try_into().ok()
}

fn uuid_ticks_to_unix_ms(ticks: u64) -> i64 {
    let milliseconds = ticks / 10_000;
    i64::try_from(milliseconds).expect("UUID timestamp fits i64") - 12_219_292_800_000
}
