//! Command-line argument parsing and validation.

use crate::calendar::{self, ParsedTime};
use crate::text;
use std::str::FromStr;

/// Epoch used by Twitter-style Snowflake identifiers.
pub const TWITTER_EPOCH_MS: i64 = 1_288_834_974_657;
/// Epoch used by Discord-style Snowflake identifiers.
pub const DISCORD_EPOCH_MS: i64 = 1_420_070_400_000;

/// A validated command and its operands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Print usage text.
    Help,
    /// Decode each identifier and report its fields.
    Info { ids: Vec<String> },
    /// Compare the creation times of two identifiers.
    Span { first: String, second: String },
    /// Sort identifiers by their creation times.
    Sort { ids: Vec<String>, descending: bool },
    /// Construct a timestamp bound for one identifier family.
    Bound {
        kind: String,
        time: ParsedTime,
        max: bool,
    },
}

/// Parsed command-line arguments with the selected Snowflake epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedArgs {
    /// Command and validated operands.
    pub command: Command,
    /// Epoch in Unix milliseconds used for Snowflake decoding/bounds.
    pub epoch_ms: i64,
}

/// Parse command-line arguments, returning the exact usage diagnostic on failure.
pub fn parse(args: &[String]) -> Result<ParsedArgs, String> {
    if args.is_empty() {
        return Err("missing command (info, span, sort or bound)".to_owned());
    }
    if args[0] == "--help" && args.len() == 1 {
        return Ok(ParsedArgs {
            command: Command::Help,
            epoch_ms: TWITTER_EPOCH_MS,
        });
    }

    let command_name = args[0].as_str();
    if !matches!(command_name, "info" | "span" | "sort" | "bound") {
        return Err(format!("unknown command {}", text::show(command_name)));
    }

    let mut operands = Vec::new();
    let mut epoch_text = None;
    let mut max = false;
    let mut descending = false;
    let mut index = 1;

    while index < args.len() {
        let argument = &args[index];
        if argument == "--epoch" {
            if index + 1 == args.len() {
                return Err("--epoch needs a value".to_owned());
            }
            epoch_text = Some(args[index + 1].clone());
            index += 2;
            continue;
        }
        if let Some(value) = argument.strip_prefix("--epoch=") {
            epoch_text = Some(value.to_owned());
        } else if argument == "--max" && command_name == "bound" {
            max = true;
        } else if argument == "--desc" && command_name == "sort" {
            descending = true;
        } else if argument.starts_with('-') && argument != "-" {
            return Err(format!("unknown option {}", text::show(argument)));
        } else {
            operands.push(argument.clone());
        }
        index += 1;
    }

    let epoch_ms = match epoch_text {
        Some(value) => {
            parse_epoch(&value).ok_or_else(|| format!("invalid --epoch {}", text::show(&value)))?
        }
        None => TWITTER_EPOCH_MS,
    };

    let command = match command_name {
        "info" => {
            if operands.is_empty() {
                return Err("missing ID".to_owned());
            }
            Command::Info { ids: operands }
        }
        "span" => match operands.as_slice() {
            [] | [_] => return Err("missing ID".to_owned()),
            [first, second] => Command::Span {
                first: first.clone(),
                second: second.clone(),
            },
            _ => return Err("too many arguments".to_owned()),
        },
        "sort" => {
            if operands.is_empty() {
                return Err("missing ID".to_owned());
            }
            Command::Sort {
                ids: operands,
                descending,
            }
        }
        "bound" => {
            let kind = match operands.first() {
                Some(kind) => kind,
                None => return Err("missing KIND".to_owned()),
            };
            let time_text = match operands.get(1) {
                Some(time) => time,
                None => return Err("missing TIME".to_owned()),
            };
            if operands.len() > 2 {
                return Err("too many arguments".to_owned());
            }
            if !matches!(
                kind.as_str(),
                "ulid" | "objectid" | "ksuid" | "uuid7" | "snowflake"
            ) {
                return Err(format!("invalid KIND {}", text::show(kind)));
            }
            let time = calendar::parse_iso(time_text)
                .map_err(|_| format!("invalid TIME {}", text::show(time_text)))?;
            Command::Bound {
                kind: kind.clone(),
                time,
                max,
            }
        }
        _ => unreachable!("the command name was validated above"),
    };

    Ok(ParsedArgs { command, epoch_ms })
}

fn parse_epoch(value: &str) -> Option<i64> {
    match value {
        "twitter" => Some(TWITTER_EPOCH_MS),
        "discord" => Some(DISCORD_EPOCH_MS),
        "unix" => Some(0),
        _ => {
            if value.is_empty()
                || value.len() > 13
                || !value.bytes().all(|byte| byte.is_ascii_digit())
            {
                return None;
            }
            i64::from_str(value).ok()
        }
    }
}
