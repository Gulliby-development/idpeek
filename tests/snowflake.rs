use idpeek::snowflake::{self, Decoded, MAX_TIMESTAMP_DELTA_MS};

#[test]
fn decodes_the_discord_reference_snowflake_fields() {
    assert_eq!(
        snowflake::decode("175928847299117063", 1_420_070_400_000),
        Some(Decoded {
            epoch_ms: 1_420_070_400_000,
            time_ms: 1_462_015_105_796,
            worker: 32,
            sequence: 7,
        })
    );
}

#[test]
fn decodes_the_reference_snowflake_with_the_twitter_epoch() {
    assert_eq!(
        snowflake::decode("175928847299117063", 1_288_834_974_657),
        Some(Decoded {
            epoch_ms: 1_288_834_974_657,
            time_ms: 1_330_779_680_453,
            worker: 32,
            sequence: 7,
        })
    );
}

#[test]
fn zero_identifier_uses_the_selected_epoch_and_zero_fields() {
    assert_eq!(
        snowflake::decode("0", 1_288_834_974_657),
        Some(Decoded {
            epoch_ms: 1_288_834_974_657,
            time_ms: 1_288_834_974_657,
            worker: 0,
            sequence: 0,
        })
    );
}

#[test]
fn zero_identifier_with_unix_epoch_has_unix_time() {
    assert_eq!(
        snowflake::decode("0", 0),
        Some(Decoded {
            epoch_ms: 0,
            time_ms: 0,
            worker: 0,
            sequence: 0,
        })
    );
}

#[test]
fn decodes_the_first_timestamp_millisecond() {
    assert_eq!(
        snowflake::decode("4194304", 0),
        Some(Decoded {
            epoch_ms: 0,
            time_ms: 1,
            worker: 0,
            sequence: 0,
        })
    );
}

#[test]
fn decodes_the_first_timestamp_millisecond_with_all_sequence_bits() {
    assert_eq!(
        snowflake::decode("8388607", 0),
        Some(Decoded {
            epoch_ms: 0,
            time_ms: 1,
            worker: 1023,
            sequence: 4095,
        })
    );
}

#[test]
fn decodes_the_lowest_nonzero_worker_bit() {
    assert_eq!(
        snowflake::decode("4096", 0),
        Some(Decoded {
            epoch_ms: 0,
            time_ms: 0,
            worker: 1,
            sequence: 0,
        })
    );
}

#[test]
fn decodes_the_maximum_worker_value() {
    assert_eq!(
        snowflake::decode("4190208", 0),
        Some(Decoded {
            epoch_ms: 0,
            time_ms: 0,
            worker: 1023,
            sequence: 0,
        })
    );
}

#[test]
fn decodes_maximum_worker_and_sequence_at_the_epoch() {
    assert_eq!(
        snowflake::decode("4194303", 0),
        Some(Decoded {
            epoch_ms: 0,
            time_ms: 0,
            worker: 1023,
            sequence: 4095,
        })
    );
}

#[test]
fn preserves_leading_zeroes_as_a_valid_decimal_spelling() {
    assert_eq!(
        snowflake::decode("00000042", 0),
        Some(Decoded {
            epoch_ms: 0,
            time_ms: 0,
            worker: 0,
            sequence: 42,
        })
    );
}

#[test]
fn accepts_twenty_leading_zeroes_with_a_nonzero_epoch() {
    assert_eq!(
        snowflake::decode("00000000000000000000", 5),
        Some(Decoded {
            epoch_ms: 5,
            time_ms: 5,
            worker: 0,
            sequence: 0,
        })
    );
}

#[test]
fn accepts_u64_max_even_though_it_exceeds_the_bound_timestamp_range() {
    assert_eq!(
        snowflake::decode("18446744073709551615", 0),
        Some(Decoded {
            epoch_ms: 0,
            time_ms: 4_398_046_511_103,
            worker: 1023,
            sequence: 4095,
        })
    );
}

#[test]
fn rejects_empty_input() {
    assert_eq!(snowflake::decode("", 0), None);
}

#[test]
fn rejects_negative_decimal_input() {
    assert_eq!(snowflake::decode("-1", 0), None);
}

#[test]
fn rejects_a_plus_sign_before_decimal_digits() {
    assert_eq!(snowflake::decode("+42", 0), None);
}

#[test]
fn rejects_non_decimal_characters() {
    assert_eq!(snowflake::decode("12x4", 0), None);
}

#[test]
fn rejects_non_ascii_decimal_lookalikes() {
    assert_eq!(snowflake::decode("１２", 0), None);
}

#[test]
fn rejects_whitespace_around_decimal_digits() {
    assert_eq!(snowflake::decode(" 42", 0), None);
}

#[test]
fn rejects_a_decimal_value_above_u64_max() {
    assert_eq!(snowflake::decode("18446744073709551616", 0), None);
}

#[test]
fn rejects_epoch_addition_overflow() {
    assert_eq!(snowflake::decode("4194304", i64::MAX), None);
}

#[test]
fn supports_a_negative_epoch_when_the_result_is_representable() {
    assert_eq!(
        snowflake::decode("4194304", -1),
        Some(Decoded {
            epoch_ms: -1,
            time_ms: 0,
            worker: 0,
            sequence: 0,
        })
    );
}

#[test]
fn zero_bound_is_the_smallest_identifier_at_unix_epoch() {
    assert_eq!(snowflake::bound(0, false, 0).as_deref(), Some("0"));
}

#[test]
fn maximum_bound_at_unix_epoch_sets_all_twenty_two_low_bits() {
    assert_eq!(snowflake::bound(0, true, 0).as_deref(), Some("4194303"));
}

#[test]
fn one_millisecond_bounds_set_timestamp_and_low_bits_exactly() {
    assert_eq!(snowflake::bound(1, false, 0).as_deref(), Some("4194304"));
    assert_eq!(snowflake::bound(1, true, 0).as_deref(), Some("8388607"));
}

#[test]
fn bounds_use_the_selected_epoch() {
    assert_eq!(
        snowflake::bound(1_420_070_400_000, false, 1_420_070_400_000).as_deref(),
        Some("0")
    );
    assert_eq!(
        snowflake::bound(1_420_070_400_000, true, 1_420_070_400_000).as_deref(),
        Some("4194303")
    );
}

#[test]
fn bound_accepts_the_exact_41_bit_timestamp_maximum() {
    assert_eq!(MAX_TIMESTAMP_DELTA_MS, 2_199_023_255_551);
    assert_eq!(
        snowflake::bound(MAX_TIMESTAMP_DELTA_MS, false, 0).as_deref(),
        Some("9223372036850581504")
    );
    assert_eq!(
        snowflake::bound(MAX_TIMESTAMP_DELTA_MS, true, 0).as_deref(),
        Some("9223372036854775807")
    );
}

#[test]
fn bound_rejects_the_millisecond_after_the_41_bit_maximum() {
    assert_eq!(snowflake::bound(2_199_023_255_552, false, 0), None);
}

#[test]
fn bound_rejects_a_timestamp_before_the_selected_epoch() {
    assert_eq!(
        snowflake::bound(1_420_070_399_999, false, 1_420_070_400_000),
        None
    );
}

#[test]
fn bound_rejects_subtraction_overflow() {
    assert_eq!(snowflake::bound(i64::MIN, false, i64::MAX), None);
}
