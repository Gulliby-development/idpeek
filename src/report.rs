//! Rendering helpers for command output and diagnostics.

use crate::{calendar, detect::Decoded};
use std::io::{self, Write};

/// Usage text shared by help and command-line documentation checks.
pub const USAGE: &str = "usage: idpeek info [--epoch EPOCH] ID...\n       idpeek span [--epoch EPOCH] ID1 ID2\n       idpeek sort [--epoch EPOCH] [--desc] ID...\n       idpeek bound [--epoch EPOCH] [--max] KIND TIME\n";

/// Write the four-line usage message.
pub fn write_usage<W: Write>(writer: &mut W) -> io::Result<()> {
    write!(writer, "{USAGE}")
}

/// Prefix a diagnostic with the executable name and terminate it with LF.
pub fn write_error<W: Write>(writer: &mut W, message: &str) -> io::Result<()> {
    write!(writer, "idpeek: {message}")?;
    writer.write_all(b"\n")
}

/// Write an identifier and its decoded fields in report order.
pub fn write_info<W: Write>(writer: &mut W, id: &str, decoded: &Decoded) -> io::Result<()> {
    write!(writer, "{id}")?;
    writer.write_all(b"\n")?;
    write_field(writer, "kind", decoded.kind)?;
    for (name, value) in &decoded.fields {
        write_field(writer, name, value)?;
    }
    Ok(())
}

/// Write an identifier as one LF-terminated line.
pub fn write_id<W: Write>(writer: &mut W, id: &str) -> io::Result<()> {
    write!(writer, "{id}")?;
    writer.write_all(b"\n")
}

/// Write the two timestamped identifiers and their relative-time sentence.
pub fn write_span<W: Write>(
    writer: &mut W,
    first: &str,
    first_ms: i64,
    second: &str,
    second_ms: i64,
) -> io::Result<()> {
    let width = first.chars().count().max(second.chars().count());
    let first_time = calendar::format_iso(first_ms);
    let second_time = calendar::format_iso(second_ms);
    write!(writer, "{first:<width$}  {first_time}")?;
    writer.write_all(b"\n")?;
    write!(writer, "{second:<width$}  {second_time}")?;
    writer.write_all(b"\n")?;

    if first_ms == second_ms {
        write!(writer, "the second is at the same time as the first")?;
        writer.write_all(b"\n")
    } else {
        let difference_ms = i64::try_from(first_ms.abs_diff(second_ms)).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "timestamp difference exceeds supported range",
            )
        })?;
        let duration = calendar::format_duration(difference_ms);
        let relation = if second_ms < first_ms {
            "earlier"
        } else {
            "later"
        };
        write!(writer, "the second is {duration} {relation} than the first")?;
        writer.write_all(b"\n")
    }
}

/// Write a list of identifiers, one per LF-terminated line.
pub fn write_sorted<W: Write>(writer: &mut W, ids: &[String]) -> io::Result<()> {
    for id in ids {
        write_id(writer, id)?;
    }
    Ok(())
}

/// Write one generated identifier bound.
pub fn write_bound<W: Write>(writer: &mut W, id: &str) -> io::Result<()> {
    write_id(writer, id)
}

fn write_field<W: Write>(writer: &mut W, name: &str, value: &str) -> io::Result<()> {
    write!(writer, "  {name:<8}  {value}")?;
    writer.write_all(b"\n")
}
