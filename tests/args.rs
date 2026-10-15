use idpeek::args::{self, Command, DISCORD_EPOCH_MS, ParsedArgs, TWITTER_EPOCH_MS};
use idpeek::calendar::ParsedTime;

fn parse(parts: &[&str]) -> Result<ParsedArgs, String> {
    let args = parts
        .iter()
        .map(|part| (*part).to_owned())
        .collect::<Vec<_>>();
    args::parse(&args)
}

fn assert_error(parts: &[&str], expected: &str) {
    assert_eq!(parse(parts).unwrap_err(), expected);
}

#[test]
fn no_arguments_reports_the_missing_command() {
    assert_error(&[], "missing command (info, span, sort or bound)");
}

#[test]
fn an_unrecognized_first_argument_is_a_quoted_command_error() {
    assert_error(&["inspect"], "unknown command \"inspect\"");
}

#[test]
fn help_is_recognized_only_as_the_sole_argument() {
    assert_eq!(
        parse(&["--help"]),
        Ok(ParsedArgs {
            command: Command::Help,
            epoch_ms: TWITTER_EPOCH_MS,
        })
    );
}

#[test]
fn help_with_an_extra_argument_is_an_unknown_command() {
    assert_error(&["--help", "info"], "unknown command \"--help\"");
}

#[test]
fn info_without_ids_reports_missing_id() {
    assert_error(&["info"], "missing ID");
}

#[test]
fn sort_without_ids_reports_missing_id() {
    assert_error(&["sort", "--desc"], "missing ID");
}

#[test]
fn span_without_ids_reports_missing_id() {
    assert_error(&["span"], "missing ID");
}

#[test]
fn span_with_one_id_reports_missing_id() {
    assert_error(&["span", "first"], "missing ID");
}

#[test]
fn span_accepts_exactly_two_ids_in_order() {
    assert_eq!(
        parse(&["span", "first", "second"]),
        Ok(ParsedArgs {
            command: Command::Span {
                first: "first".to_owned(),
                second: "second".to_owned(),
            },
            epoch_ms: TWITTER_EPOCH_MS,
        })
    );
}

#[test]
fn span_with_a_third_id_reports_too_many_arguments() {
    assert_error(&["span", "first", "second", "third"], "too many arguments");
}

#[test]
fn bound_without_a_kind_reports_missing_kind() {
    assert_error(&["bound"], "missing KIND");
}

#[test]
fn bound_with_a_kind_but_no_time_reports_missing_time() {
    assert_error(&["bound", "ulid"], "missing TIME");
}

#[test]
fn bound_with_a_third_operand_reports_too_many_arguments() {
    assert_error(
        &["bound", "ulid", "2024-03-01", "extra"],
        "too many arguments",
    );
}

#[test]
fn bound_rejects_an_unknown_kind_with_its_original_spelling() {
    assert_error(&["bound", "UUID4", "2024-03-01"], "invalid KIND \"UUID4\"");
}

#[test]
fn bound_rejects_invalid_time_with_its_original_spelling() {
    assert_error(
        &["bound", "ulid", "not-a-time"],
        "invalid TIME \"not-a-time\"",
    );
}

#[test]
fn invalid_kind_precedes_invalid_time() {
    assert_error(&["bound", "uuid4", "not-a-time"], "invalid KIND \"uuid4\"");
}

#[test]
fn operand_count_precedes_invalid_kind_and_time() {
    assert_error(
        &["bound", "uuid4", "not-a-time", "extra"],
        "too many arguments",
    );
}

#[test]
fn invalid_epoch_precedes_a_missing_operand_error() {
    assert_error(&["info", "--epoch", "bad"], "invalid --epoch \"bad\"");
}

#[test]
fn invalid_epoch_precedes_bound_operand_validation() {
    assert_error(&["bound", "--epoch=bad"], "invalid --epoch \"bad\"");
}

#[test]
fn a_trailing_epoch_option_needs_a_value() {
    assert_error(&["info", "id", "--epoch"], "--epoch needs a value");
}

#[test]
fn a_trailing_epoch_option_wins_over_an_earlier_invalid_epoch() {
    assert_error(&["info", "--epoch=bad", "--epoch"], "--epoch needs a value");
}

#[test]
fn unknown_options_are_quoted() {
    assert_error(&["info", "--verbose", "id"], "unknown option \"--verbose\"");
}

#[test]
fn an_unknown_option_is_reported_before_epoch_validation() {
    assert_error(
        &["info", "--epoch=bad", "--verbose"],
        "unknown option \"--verbose\"",
    );
}

#[test]
fn max_is_not_an_option_for_info() {
    assert_error(&["info", "--max", "id"], "unknown option \"--max\"");
}

#[test]
fn desc_is_not_an_option_for_span() {
    assert_error(
        &["span", "--desc", "first", "second"],
        "unknown option \"--desc\"",
    );
}

#[test]
fn a_single_hyphen_is_preserved_as_an_operand() {
    assert_eq!(
        parse(&["info", "-"]),
        Ok(ParsedArgs {
            command: Command::Info {
                ids: vec!["-".to_owned()],
            },
            epoch_ms: TWITTER_EPOCH_MS,
        })
    );
}

#[test]
fn a_hyphen_prefixed_epoch_value_is_consumed_as_the_value() {
    assert_error(&["info", "--epoch", "-", "id"], "invalid --epoch \"-\"");
}

#[test]
fn an_option_spelling_can_be_consumed_as_the_epoch_value() {
    assert_error(
        &["sort", "--epoch", "--desc", "id"],
        "invalid --epoch \"--desc\"",
    );
}

#[test]
fn an_empty_equals_epoch_value_is_invalid() {
    assert_error(&["info", "--epoch=", "id"], "invalid --epoch \"\"");
}

#[test]
fn numeric_epoch_zero_is_accepted() {
    assert_eq!(
        parse(&["info", "id", "--epoch", "0"]),
        Ok(ParsedArgs {
            command: Command::Info {
                ids: vec!["id".to_owned()],
            },
            epoch_ms: 0,
        })
    );
}

#[test]
fn one_digit_numeric_epoch_is_accepted() {
    assert_eq!(
        parse(&["info", "id", "--epoch=7"]),
        Ok(ParsedArgs {
            command: Command::Info {
                ids: vec!["id".to_owned()],
            },
            epoch_ms: 7,
        })
    );
}

#[test]
fn the_thirteen_digit_epoch_boundary_is_accepted() {
    assert_eq!(
        parse(&["info", "id", "--epoch=9999999999999"]),
        Ok(ParsedArgs {
            command: Command::Info {
                ids: vec!["id".to_owned()],
            },
            epoch_ms: 9_999_999_999_999,
        })
    );
}

#[test]
fn a_fourteen_digit_epoch_is_rejected() {
    assert_error(
        &["info", "id", "--epoch=10000000000000"],
        "invalid --epoch \"10000000000000\"",
    );
}

#[test]
fn numeric_epoch_digits_must_be_ascii() {
    assert_error(&["info", "id", "--epoch=１２"], "invalid --epoch \"??\"");
}

#[test]
fn negative_numeric_epochs_are_rejected() {
    assert_error(&["info", "id", "--epoch=-1"], "invalid --epoch \"-1\"");
}

#[test]
fn leading_zeroes_do_not_change_a_numeric_epoch() {
    assert_eq!(
        parse(&["info", "id", "--epoch=00042"]),
        Ok(ParsedArgs {
            command: Command::Info {
                ids: vec!["id".to_owned()],
            },
            epoch_ms: 42,
        })
    );
}

#[test]
fn the_twitter_epoch_alias_matches_the_default_epoch() {
    assert_eq!(
        parse(&["info", "id", "--epoch=twitter"]),
        Ok(ParsedArgs {
            command: Command::Info {
                ids: vec!["id".to_owned()],
            },
            epoch_ms: TWITTER_EPOCH_MS,
        })
    );
}

#[test]
fn the_discord_epoch_alias_uses_its_fixed_millisecond_value() {
    assert_eq!(
        parse(&["info", "id", "--epoch", "discord"]),
        Ok(ParsedArgs {
            command: Command::Info {
                ids: vec!["id".to_owned()],
            },
            epoch_ms: DISCORD_EPOCH_MS,
        })
    );
}

#[test]
fn the_unix_epoch_alias_uses_zero_milliseconds() {
    assert_eq!(
        parse(&["info", "id", "--epoch=unix"]),
        Ok(ParsedArgs {
            command: Command::Info {
                ids: vec!["id".to_owned()],
            },
            epoch_ms: 0,
        })
    );
}

#[test]
fn the_last_epoch_option_wins() {
    assert_eq!(
        parse(&["info", "id", "--epoch=discord", "--epoch", "unix"]),
        Ok(ParsedArgs {
            command: Command::Info {
                ids: vec!["id".to_owned()],
            },
            epoch_ms: 0,
        })
    );
}

#[test]
fn a_later_valid_epoch_replaces_an_earlier_invalid_one() {
    assert_eq!(
        parse(&["info", "id", "--epoch=bad", "--epoch=discord"]),
        Ok(ParsedArgs {
            command: Command::Info {
                ids: vec!["id".to_owned()],
            },
            epoch_ms: DISCORD_EPOCH_MS,
        })
    );
}

#[test]
fn the_last_invalid_epoch_is_the_reported_value() {
    assert_error(
        &["info", "id", "--epoch=discord", "--epoch=bad"],
        "invalid --epoch \"bad\"",
    );
}

#[test]
fn epoch_option_is_valid_after_operands() {
    assert_eq!(
        parse(&["sort", "first", "second", "--epoch=unix"]),
        Ok(ParsedArgs {
            command: Command::Sort {
                ids: vec!["first".to_owned(), "second".to_owned()],
                descending: false,
            },
            epoch_ms: 0,
        })
    );
}

#[test]
fn sort_records_descending_flag_after_operands() {
    assert_eq!(
        parse(&["sort", "first", "--desc", "second"]),
        Ok(ParsedArgs {
            command: Command::Sort {
                ids: vec!["first".to_owned(), "second".to_owned()],
                descending: true,
            },
            epoch_ms: TWITTER_EPOCH_MS,
        })
    );
}

#[test]
fn bound_records_max_and_epoch_flags() {
    assert_eq!(
        parse(&["bound", "--max", "--epoch=unix", "ulid", "1970-01-01"]),
        Ok(ParsedArgs {
            command: Command::Bound {
                kind: "ulid".to_owned(),
                time: ParsedTime::Milliseconds(0),
                max: true,
            },
            epoch_ms: 0,
        })
    );
}

#[test]
fn bound_defaults_to_the_minimum_encoding() {
    assert_eq!(
        parse(&["bound", "objectid", "2024-03-01"]),
        Ok(ParsedArgs {
            command: Command::Bound {
                kind: "objectid".to_owned(),
                time: ParsedTime::Milliseconds(1_709_251_200_000),
                max: false,
            },
            epoch_ms: TWITTER_EPOCH_MS,
        })
    );
}

#[test]
fn bound_preserves_fractional_milliseconds_from_time() {
    assert_eq!(
        parse(&["bound", "ulid", "2024-03-01T12:30:00.5Z"]),
        Ok(ParsedArgs {
            command: Command::Bound {
                kind: "ulid".to_owned(),
                time: ParsedTime::Milliseconds(1_709_296_200_500),
                max: false,
            },
            epoch_ms: TWITTER_EPOCH_MS,
        })
    );
}

#[test]
fn bound_rejects_a_non_leap_day_as_invalid_time() {
    assert_error(
        &["bound", "ulid", "2024-02-30"],
        "invalid TIME \"2024-02-30\"",
    );
}

#[test]
fn bound_rejects_year_zero_as_invalid_time() {
    assert_error(
        &["bound", "ulid", "0000-01-01"],
        "invalid TIME \"0000-01-01\"",
    );
}

#[test]
fn bound_rejects_an_omitted_utc_suffix() {
    assert_error(
        &["bound", "ulid", "2024-03-01T12:00:00"],
        "invalid TIME \"2024-03-01T12:00:00\"",
    );
}

#[test]
fn bound_rejects_a_timezone_offset() {
    assert_error(
        &["bound", "ulid", "2024-03-01T12:00:00+00:00"],
        "invalid TIME \"2024-03-01T12:00:00+00:00\"",
    );
}

#[test]
fn bound_rejects_seconds_outside_the_allowed_range() {
    assert_error(
        &["bound", "ulid", "2024-03-01T12:00:60Z"],
        "invalid TIME \"2024-03-01T12:00:60Z\"",
    );
}

#[test]
fn bound_rejects_four_fraction_digits() {
    assert_error(
        &["bound", "ulid", "2024-03-01T12:00:00.1234Z"],
        "invalid TIME \"2024-03-01T12:00:00.1234Z\"",
    );
}

#[test]
fn all_five_bound_kinds_are_accepted() {
    assert!(matches!(
        parse(&["bound", "ulid", "2024-03-01"]),
        Ok(ParsedArgs {
            command: Command::Bound { kind, .. },
            ..
        }) if kind == "ulid"
    ));
    assert!(matches!(
        parse(&["bound", "objectid", "2024-03-01"]),
        Ok(ParsedArgs {
            command: Command::Bound { kind, .. },
            ..
        }) if kind == "objectid"
    ));
    assert!(matches!(
        parse(&["bound", "ksuid", "2024-03-01"]),
        Ok(ParsedArgs {
            command: Command::Bound { kind, .. },
            ..
        }) if kind == "ksuid"
    ));
    assert!(matches!(
        parse(&["bound", "uuid7", "2024-03-01"]),
        Ok(ParsedArgs {
            command: Command::Bound { kind, .. },
            ..
        }) if kind == "uuid7"
    ));
    assert!(matches!(
        parse(&["bound", "snowflake", "2024-03-01"]),
        Ok(ParsedArgs {
            command: Command::Bound { kind, .. },
            ..
        }) if kind == "snowflake"
    ));
}
