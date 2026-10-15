//! Gregorian calendar conversion and ISO-8601-shaped timestamps.
//!
//! This module intentionally uses no date/time library. Parsing keeps syntax
//! validity separate from the signed millisecond range used by identifier
//! codecs: a valid date outside that range is represented by `OutOfRange`.

const MILLIS_PER_SECOND: i128 = 1_000;
const MILLIS_PER_MINUTE: i128 = 60 * MILLIS_PER_SECOND;
const MILLIS_PER_HOUR: i128 = 60 * MILLIS_PER_MINUTE;
const MILLIS_PER_DAY: i128 = 24 * MILLIS_PER_HOUR;
const MILLIS_PER_DAY_U64: u64 = 86_400_000;

/// A syntactically valid parsed time, possibly outside the signed-ms range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedTime {
    /// Unix milliseconds representable by the identifier APIs.
    Milliseconds(i64),
    /// The date is valid but its Unix-millisecond value does not fit in i64.
    OutOfRange,
}

/// Error returned when an ISO date or timestamp has invalid syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseError;

impl From<()> for ParseError {
    fn from(_: ()) -> Self {
        Self
    }
}

impl PartialEq<()> for ParseError {
    fn eq(&self, _: &()) -> bool {
        true
    }
}

/// Result wrapper that keeps invalid syntax distinct while remaining comparable
/// with the historical unit-error result shape used by callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IsoResult(Result<ParsedTime, ParseError>);

impl IsoResult {
    /// Map a syntax error into a caller-specific diagnostic.
    pub fn map_err<F>(self, operation: impl FnOnce(ParseError) -> F) -> Result<ParsedTime, F> {
        self.0.map_err(operation)
    }
}

impl PartialEq<Result<ParsedTime, ()>> for IsoResult {
    fn eq(&self, other: &Result<ParsedTime, ()>) -> bool {
        match (&self.0, other) {
            (Ok(left), Ok(right)) => left == right,
            (Err(_), Err(())) => true,
            _ => false,
        }
    }
}

impl ParsedTime {
    /// Returns the millisecond value when it fits in the supported range.
    pub const fn as_milliseconds(self) -> Option<i64> {
        match self {
            Self::Milliseconds(value) => Some(value),
            Self::OutOfRange => None,
        }
    }
}

/// Formats Unix milliseconds as UTC with a four-digit minimum year.
pub fn format_iso(milliseconds: i64) -> String {
    let days = milliseconds.div_euclid(MILLIS_PER_DAY as i64);
    let day_millis = milliseconds.rem_euclid(MILLIS_PER_DAY as i64);
    let (year, month, day) = civil_from_days(days);

    let hour = day_millis / 3_600_000;
    let minute = day_millis % 3_600_000 / 60_000;
    let second = day_millis % 60_000 / 1_000;
    let millisecond = day_millis % 1_000;
    let year_text = if year < 0 {
        format!("-{:04}", year.unsigned_abs())
    } else {
        format!("{year:04}")
    };

    format!("{year_text}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millisecond:03}Z")
}

/// Parses `YYYY-MM-DD` or `YYYY-MM-DDTHH:MM:SS[.f]Z`.
///
/// Years have at least four ASCII digits and must be at least 0001. A valid
/// date too large to represent as signed Unix milliseconds returns
/// `Ok(ParsedTime::OutOfRange)`; malformed dates return `Err(ParseError)`.
pub fn parse_iso(input: &str) -> IsoResult {
    let result = (|| -> Result<ParsedTime, ParseError> {
        let (date, time) = match input.split_once('T') {
            Some((date, time)) => (date, Some(time)),
            None => (input, None),
        };

        let mut date_parts = date.split('-');
        let year_text = date_parts.next().ok_or(())?;
        let month_text = date_parts.next().ok_or(())?;
        let day_text = date_parts.next().ok_or(())?;
        if date_parts.next().is_some()
            || year_text.len() < 4
            || !is_ascii_digits(year_text)
            || month_text.len() != 2
            || day_text.len() != 2
            || !is_ascii_digits(month_text)
            || !is_ascii_digits(day_text)
        {
            return Err(ParseError);
        }

        let (year_mod_400, year_value) = parse_year(year_text)?;
        if year_value == Some(0) {
            return Err(ParseError);
        }
        let month = parse_two_digits(month_text)?;
        let day = parse_two_digits(day_text)?;
        if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year_mod_400, month) {
            return Err(ParseError);
        }

        let (hour, minute, second, millisecond) = match time {
            Some(value) => parse_clock(value)?,
            None => (0, 0, 0, 0),
        };

        let Some(year) = year_value else {
            return Ok(ParsedTime::OutOfRange);
        };
        let Some(days) = days_from_civil(year, month, day) else {
            return Ok(ParsedTime::OutOfRange);
        };
        let clock_millis = i128::from(hour) * MILLIS_PER_HOUR
            + i128::from(minute) * MILLIS_PER_MINUTE
            + i128::from(second) * MILLIS_PER_SECOND
            + i128::from(millisecond);
        let Some(total_millis) = days
            .checked_mul(MILLIS_PER_DAY)
            .and_then(|value| value.checked_add(clock_millis))
        else {
            return Ok(ParsedTime::OutOfRange);
        };

        match i64::try_from(total_millis) {
            Ok(value) => Ok(ParsedTime::Milliseconds(value)),
            Err(_) => Ok(ParsedTime::OutOfRange),
        }
    })();
    IsoResult(result)
}

/// Formats a nonnegative duration as `HH:MM:SS.mmm`, adding `Nd ` for days.
///
/// The sign is intentionally omitted; callers describe whether the compared
/// timestamp is earlier or later separately.
pub fn format_duration(milliseconds: i64) -> String {
    let total = milliseconds.unsigned_abs();
    let days = total / MILLIS_PER_DAY_U64;
    let within_day = total % MILLIS_PER_DAY_U64;
    let hours = within_day / 3_600_000;
    let minutes = within_day % 3_600_000 / 60_000;
    let seconds = within_day % 60_000 / 1_000;
    let millis = within_day % 1_000;

    if days == 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}.{millis:03}")
    } else {
        format!("{days}d {hours:02}:{minutes:02}:{seconds:02}.{millis:03}")
    }
}

fn parse_year(text: &str) -> Result<(u32, Option<i128>), ()> {
    let mut modulo = 0_u32;
    let mut value = Some(0_i128);
    let mut nonzero = false;
    for byte in text.bytes() {
        if !byte.is_ascii_digit() {
            return Err(());
        }
        let digit = u32::from(byte - b'0');
        modulo = (modulo * 10 + digit) % 400;
        nonzero |= digit != 0;
        value = value.and_then(|current| {
            current
                .checked_mul(10)
                .and_then(|next| next.checked_add(i128::from(digit)))
        });
    }
    if !nonzero {
        return Err(());
    }
    Ok((modulo, value))
}

fn parse_clock(text: &str) -> Result<(u32, u32, u32, u32), ()> {
    let clock = text.strip_suffix('Z').ok_or(())?;
    let (whole, fraction) = match clock.split_once('.') {
        Some((whole, fraction)) => {
            if fraction.is_empty() || fraction.len() > 3 || !is_ascii_digits(fraction) {
                return Err(());
            }
            (whole, Some(fraction))
        }
        None => (clock, None),
    };

    let mut parts = whole.split(':');
    let hour_text = parts.next().ok_or(())?;
    let minute_text = parts.next().ok_or(())?;
    let second_text = parts.next().ok_or(())?;
    if parts.next().is_some()
        || hour_text.len() != 2
        || minute_text.len() != 2
        || second_text.len() != 2
        || !is_ascii_digits(hour_text)
        || !is_ascii_digits(minute_text)
        || !is_ascii_digits(second_text)
    {
        return Err(());
    }
    let hour = parse_two_digits(hour_text)?;
    let minute = parse_two_digits(minute_text)?;
    let second = parse_two_digits(second_text)?;
    if hour > 23 || minute > 59 || second > 59 {
        return Err(());
    }

    let millisecond = match fraction {
        Some(value) => {
            let number = parse_small_decimal(value)?;
            match value.len() {
                1 => number * 100,
                2 => number * 10,
                _ => number,
            }
        }
        None => 0,
    };
    Ok((hour, minute, second, millisecond))
}

fn is_ascii_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

fn parse_two_digits(text: &str) -> Result<u32, ()> {
    if text.len() != 2 || !is_ascii_digits(text) {
        return Err(());
    }
    parse_small_decimal(text)
}

fn parse_small_decimal(text: &str) -> Result<u32, ()> {
    text.bytes().try_fold(0_u32, |value, byte| {
        if byte.is_ascii_digit() {
            Ok(value * 10 + u32::from(byte - b'0'))
        } else {
            Err(())
        }
    })
}

fn days_in_month(year_mod_400: u32, month: u32) -> u32 {
    match month {
        2 if year_mod_400.is_multiple_of(4)
            && (!year_mod_400.is_multiple_of(100) || year_mod_400 == 0) =>
        {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn days_from_civil(year: i128, month: u32, day: u32) -> Option<i128> {
    let adjusted_year = if month <= 2 {
        year.checked_sub(1)?
    } else {
        year
    };
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let shifted_month = i128::from(month) + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + i128::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era.checked_mul(146_097)?
        .checked_add(day_of_era)?
        .checked_sub(719_468)
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted_days = days + 719_468;
    let era = shifted_days.div_euclid(146_097);
    let day_of_era = shifted_days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month as u32, day as u32)
}
