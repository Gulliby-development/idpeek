use crate::{args, calendar, detect, report, text};
use std::io::{Read, Write};

/// Runs idpeek with caller-supplied arguments and streams, returning its exit code.
pub fn run(
    arguments: &[String],
    stdin: &mut dyn Read,
    mut stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let parsed = match args::parse(arguments) {
        Ok(parsed) => parsed,
        Err(message) => {
            write_diagnostic(stderr, &message);
            return 2;
        }
    };

    match parsed.command {
        args::Command::Help => match report::write_usage(&mut stdout) {
            Ok(()) => 0,
            Err(_) => {
                write_diagnostic(stderr, "failed to write standard output");
                1
            }
        },
        args::Command::Info { ids } => run_info(&ids, parsed.epoch_ms, stdin, stdout, stderr),
        args::Command::Span { first, second } => {
            run_span(&first, &second, parsed.epoch_ms, stdout, stderr)
        }
        args::Command::Sort { ids, descending } => {
            run_sort(&ids, descending, parsed.epoch_ms, stdin, stdout, stderr)
        }
        args::Command::Bound { kind, time, max } => {
            run_bound(&kind, time, max, parsed.epoch_ms, stdout, stderr)
        }
    }
}

fn run_info(
    ids: &[String],
    epoch_ms: i64,
    stdin: &mut dyn Read,
    mut stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let expanded = match expand_stdin(ids, stdin) {
        Ok(ids) => ids,
        Err(message) => {
            write_diagnostic(stderr, message);
            return 1;
        }
    };

    let mut exit_code = 0;
    for id in expanded {
        match detect::decode(&id, epoch_ms) {
            Ok(decoded) => {
                if report::write_info(&mut stdout, &id, &decoded).is_err() {
                    write_diagnostic(stderr, "failed to write standard output");
                    return 1;
                }
            }
            Err(message) => {
                write_diagnostic(stderr, &message);
                exit_code = 1;
            }
        }
    }
    exit_code
}

fn run_span(
    first: &str,
    second: &str,
    epoch_ms: i64,
    mut stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let first_ms = match time_for_id(first, epoch_ms) {
        Ok(milliseconds) => milliseconds,
        Err(message) => {
            write_diagnostic(stderr, &message);
            return 1;
        }
    };
    let second_ms = match time_for_id(second, epoch_ms) {
        Ok(milliseconds) => milliseconds,
        Err(message) => {
            write_diagnostic(stderr, &message);
            return 1;
        }
    };

    match report::write_span(&mut stdout, first, first_ms, second, second_ms) {
        Ok(()) => 0,
        Err(_) => {
            write_diagnostic(stderr, "failed to write standard output");
            1
        }
    }
}

fn run_sort(
    ids: &[String],
    descending: bool,
    epoch_ms: i64,
    stdin: &mut dyn Read,
    mut stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let expanded = match expand_stdin(ids, stdin) {
        Ok(ids) => ids,
        Err(message) => {
            write_diagnostic(stderr, message);
            return 1;
        }
    };

    let mut timed_ids = Vec::with_capacity(expanded.len());
    for id in expanded {
        match time_for_id(&id, epoch_ms) {
            Ok(milliseconds) => timed_ids.push((milliseconds, id)),
            Err(message) => {
                write_diagnostic(stderr, &message);
                return 1;
            }
        }
    }

    timed_ids.sort_by(|left, right| {
        if descending {
            right.0.cmp(&left.0)
        } else {
            left.0.cmp(&right.0)
        }
    });
    let sorted_ids: Vec<String> = timed_ids.into_iter().map(|(_, id)| id).collect();
    match report::write_sorted(&mut stdout, &sorted_ids) {
        Ok(()) => 0,
        Err(_) => {
            write_diagnostic(stderr, "failed to write standard output");
            1
        }
    }
}

fn run_bound(
    kind: &str,
    time: calendar::ParsedTime,
    max: bool,
    epoch_ms: i64,
    mut stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let result = match time.as_milliseconds() {
        Some(milliseconds) => detect::bound(kind, milliseconds, max, epoch_ms),
        None => Err(format!("time is out of range for {kind}")),
    };
    match result {
        Ok(id) => match report::write_bound(&mut stdout, &id) {
            Ok(()) => 0,
            Err(_) => {
                write_diagnostic(stderr, "failed to write standard output");
                1
            }
        },
        Err(message) => {
            write_diagnostic(stderr, &message);
            1
        }
    }
}

fn time_for_id(id: &str, epoch_ms: i64) -> Result<i64, String> {
    let decoded = detect::decode(id, epoch_ms)?;
    decoded
        .time_ms
        .ok_or_else(|| format!("{} has no time", text::show(id)))
}

fn expand_stdin(ids: &[String], stdin: &mut dyn Read) -> Result<Vec<String>, &'static str> {
    let mut expanded = Vec::with_capacity(ids.len());
    let mut consumed = false;
    let mut input_lines = Vec::new();

    for id in ids {
        if id != "-" {
            expanded.push(id.clone());
            continue;
        }
        if consumed {
            continue;
        }
        consumed = true;

        let mut bytes = Vec::new();
        stdin
            .read_to_end(&mut bytes)
            .map_err(|_| "failed to read standard input")?;
        let input = std::str::from_utf8(&bytes).map_err(|_| "standard input is not valid UTF-8")?;
        append_input_lines(input, &mut input_lines);
        expanded.append(&mut input_lines);
    }

    Ok(expanded)
}

fn append_input_lines(input: &str, output: &mut Vec<String>) {
    for raw_line in input.split_inclusive('\n') {
        let line = match raw_line.strip_suffix('\n') {
            Some(line) => line.strip_suffix('\r').unwrap_or(line),
            None => raw_line,
        };
        let trimmed = line.trim_matches(|character| character == ' ' || character == '\t');
        if !trimmed.is_empty() {
            output.push(trimmed.to_owned());
        }
    }
}

fn write_diagnostic(mut stderr: &mut dyn Write, message: &str) {
    let _ = report::write_error(&mut stderr, message);
}
