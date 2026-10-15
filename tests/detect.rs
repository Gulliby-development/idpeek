use idpeek::detect;

#[test]
fn decode_routes_canonical_uuid_and_preserves_field_order() {
    let decoded = detect::decode("017F22E2-79B0-7CC3-98C4-DC0C0C07398F", 0).unwrap();

    assert_eq!(decoded.kind, "uuid");
    assert_eq!(
        decoded.fields,
        vec![
            ("version", "7 (unix time and random)".to_owned()),
            ("variant", "rfc4122".to_owned()),
            ("time", "2022-02-22T19:22:22.000Z".to_owned()),
        ]
    );
    assert_eq!(decoded.time_ms, Some(1_645_557_742_000));
}

#[test]
fn decode_routes_compact_uuid_at_unix_epoch() {
    let decoded = detect::decode("00000000000070008000000000000000", 0).unwrap();

    assert_eq!(decoded.kind, "uuid");
    assert_eq!(
        decoded.fields,
        vec![
            ("version", "7 (unix time and random)".to_owned()),
            ("variant", "rfc4122".to_owned()),
            ("time", "1970-01-01T00:00:00.000Z".to_owned()),
        ]
    );
    assert_eq!(decoded.time_ms, Some(0));
}

#[test]
fn decode_routes_objectid_and_decodes_each_field() {
    let decoded = detect::decode("507f1f77bcf86cd799439011", 0).unwrap();

    assert_eq!(decoded.kind, "objectid");
    assert_eq!(
        decoded.fields,
        vec![
            ("time", "2012-10-17T21:13:27.000Z".to_owned()),
            ("random", "bcf86cd799".to_owned()),
            ("counter", "4427793".to_owned()),
        ]
    );
    assert_eq!(decoded.time_ms, Some(1_350_508_407_000));
}

#[test]
fn decode_routes_lowercase_ulid_and_preserves_its_timestamp_and_randomness() {
    let decoded = detect::decode("01arz3ndektsv4rrffq69g5fav", 0).unwrap();

    assert_eq!(decoded.kind, "ulid");
    assert_eq!(
        decoded.fields,
        vec![
            ("time", "2016-07-30T23:54:10.259Z".to_owned()),
            ("random", "d6764c61efb99302bd5b".to_owned()),
        ]
    );
    assert_eq!(decoded.time_ms, Some(1_469_922_850_259));
}

#[test]
fn decode_routes_ksuid_to_its_timestamp_and_payload() {
    let decoded = detect::decode("0ujtsYcgvSTl8PAuAdqWYSMnLOv", 0).unwrap();

    assert_eq!(decoded.kind, "ksuid");
    assert_eq!(
        decoded.fields,
        vec![
            ("time", "2017-10-10T04:00:47.000Z".to_owned()),
            ("payload", "b5a1cd34b5f99d1154fb6853345c9735".to_owned()),
        ]
    );
    assert_eq!(decoded.time_ms, Some(1_507_608_047_000));
}

#[test]
fn decode_routes_snowflake_using_the_selected_epoch() {
    let decoded = detect::decode("175928847299117063", 1_420_070_400_000).unwrap();

    assert_eq!(decoded.kind, "snowflake");
    assert_eq!(
        decoded.fields,
        vec![
            ("epoch", "1420070400000".to_owned()),
            ("time", "2016-04-30T11:18:25.796Z".to_owned()),
            ("worker", "32".to_owned()),
            ("sequence", "7".to_owned()),
        ]
    );
    assert_eq!(decoded.time_ms, Some(1_462_015_105_796));
}

#[test]
fn decode_accepts_a_fifteen_digit_zero_snowflake() {
    let decoded = detect::decode("000000000000000", 0).unwrap();

    assert_eq!(decoded.kind, "snowflake");
    assert_eq!(
        decoded.fields,
        vec![
            ("epoch", "0".to_owned()),
            ("time", "1970-01-01T00:00:00.000Z".to_owned()),
            ("worker", "0".to_owned()),
            ("sequence", "0".to_owned()),
        ]
    );
    assert_eq!(decoded.time_ms, Some(0));
}

#[test]
fn decode_accepts_u64_max_as_a_twenty_digit_snowflake() {
    let decoded = detect::decode("18446744073709551615", 0).unwrap();

    assert_eq!(decoded.kind, "snowflake");
    assert_eq!(decoded.time_ms, Some(4_398_046_511_103));
    assert_eq!(decoded.fields[2], ("worker", "1023".to_owned()));
    assert_eq!(decoded.fields[3], ("sequence", "4095".to_owned()));
}

#[test]
fn decode_dispatch_counts_unicode_characters_for_uuid_length() {
    let input = "é0000000-0000-7000-8000-000000000000";

    assert_eq!(input.chars().count(), 36);
    assert_eq!(input.len(), 37);
    assert_eq!(
        detect::decode(input, 0),
        Err("invalid UUID \"?0000000-0000-7000-8000-000000000000\"".to_owned())
    );
}

#[test]
fn decode_dispatch_counts_unicode_characters_before_rejecting_non_ascii_digits() {
    let input = "１２３456789012345";

    assert_eq!(input.chars().count(), 15);
    assert_eq!(input.len(), 21);
    assert_eq!(
        detect::decode(input, 0),
        Err("unknown ID format \"???456789012345\"".to_owned())
    );
}

#[test]
fn decode_reports_unknown_format_for_non_ascii_decimal_characters() {
    assert_eq!(
        detect::decode("１２３４５６７８９０１２３４５", 0),
        Err("unknown ID format \"???????????????\"".to_owned())
    );
}

#[test]
fn decode_rejects_snowflake_decimal_overflow_at_twenty_characters() {
    assert_eq!(
        detect::decode("18446744073709551616", 0),
        Err("invalid snowflake \"18446744073709551616\"".to_owned())
    );
}

#[test]
fn decode_reports_unknown_format_for_unsupported_character_lengths() {
    for (input, expected) in [
        ("abc", "unknown ID format \"abc\""),
        ("12345678901234", "unknown ID format \"12345678901234\""),
        (
            "123456789012345678901",
            "unknown ID format \"123456789012345678901\"",
        ),
    ] {
        assert_eq!(detect::decode(input, 0), Err(expected.to_owned()));
    }
}

#[test]
fn decode_reports_unknown_format_for_fifteen_character_non_digit_text() {
    assert_eq!(
        detect::decode("abcdefghijklmno", 0),
        Err("unknown ID format \"abcdefghijklmno\"".to_owned())
    );
}

#[test]
fn decode_reports_the_family_specific_error_for_each_malformed_recognized_shape() {
    for (input, expected) in [
        (
            "z0000000-0000-7000-8000-000000000000",
            "invalid UUID \"z0000000-0000-7000-8000-000000000000\"",
        ),
        (
            "507f1f77bcf86cd79943901g",
            "invalid ObjectId \"507f1f77bcf86cd79943901g\"",
        ),
        (
            "80000000000000000000000000",
            "invalid ULID \"80000000000000000000000000\"",
        ),
        (
            "aWgEPTl1tmebfsQzFP4bxwgy80W",
            "invalid KSUID \"aWgEPTl1tmebfsQzFP4bxwgy80W\"",
        ),
        (
            "18446744073709551616",
            "invalid snowflake \"18446744073709551616\"",
        ),
    ] {
        assert_eq!(detect::decode(input, 0), Err(expected.to_owned()));
    }
}

#[test]
fn bound_rejects_an_unknown_kind_with_its_original_spelling() {
    assert_eq!(
        detect::bound("UUID7", 0, false, 0),
        Err("invalid KIND \"UUID7\"".to_owned())
    );
}

#[test]
fn bound_uuid7_uses_the_minimum_version_variant_and_random_tail() {
    assert_eq!(
        detect::bound("uuid7", 0, false, 0),
        Ok("00000000-0000-7000-8000-000000000000".to_owned())
    );
}

#[test]
fn bound_uuid7_uses_the_maximum_tail_without_changing_timestamp_bits() {
    assert_eq!(
        detect::bound("uuid7", 0x0123_4567_89ab, true, 0),
        Ok("01234567-89ab-7fff-bfff-ffffffffffff".to_owned())
    );
}

#[test]
fn bound_uuid7_accepts_both_tails_at_the_maximum_48_bit_timestamp() {
    let maximum = 281_474_976_710_655_i64;

    assert_eq!(
        detect::bound("uuid7", maximum, false, 0),
        Ok("ffffffff-ffff-7000-8000-000000000000".to_owned())
    );
    assert_eq!(
        detect::bound("uuid7", maximum, true, 0),
        Ok("ffffffff-ffff-7fff-bfff-ffffffffffff".to_owned())
    );
}

#[test]
fn bound_ulid_encodes_zero_time_with_minimum_and_maximum_random_suffixes() {
    assert_eq!(
        detect::bound("ulid", 0, false, 0),
        Ok("00000000000000000000000000".to_owned())
    );
    assert_eq!(
        detect::bound("ulid", 0, true, 0),
        Ok("0000000000ZZZZZZZZZZZZZZZZ".to_owned())
    );
}

#[test]
fn bound_ulid_encodes_the_maximum_timestamp_and_random_suffix() {
    assert_eq!(
        detect::bound("ulid", 281_474_976_710_655, true, 0),
        Ok("7ZZZZZZZZZZZZZZZZZZZZZZZZZ".to_owned())
    );
}

#[test]
fn bound_objectid_floors_to_seconds_and_sets_all_suffix_bits() {
    assert_eq!(
        detect::bound("objectid", 1_999, false, 0),
        Ok("000000010000000000000000".to_owned())
    );
    assert_eq!(
        detect::bound("objectid", 1_999, true, 0),
        Ok("00000001ffffffffffffffff".to_owned())
    );
    assert_eq!(
        detect::bound("objectid", 4_294_967_295_999, false, 0),
        Ok("ffffffff0000000000000000".to_owned())
    );
    assert_eq!(
        detect::bound("objectid", 4_294_967_295_999, true, 0),
        Ok("ffffffffffffffffffffffff".to_owned())
    );
}

#[test]
fn bound_ksuid_handles_the_epoch_reference_and_last_representable_second() {
    assert_eq!(
        detect::bound("ksuid", 1_400_000_000_999, false, 0),
        Ok("000000000000000000000000000".to_owned())
    );
    assert_eq!(
        detect::bound("ksuid", 1_709_251_200_000, false, 0),
        Ok("2d3ykafzYwGnkMqnex3Bf5k5Uye".to_owned())
    );
    assert_eq!(
        detect::bound("ksuid", 5_694_967_295_999, true, 0),
        Ok("aWgEPTl1tmebfsQzFP4bxwgy80V".to_owned())
    );
}

#[test]
fn bound_snowflake_uses_the_selected_epoch_and_all_22_low_bits() {
    let epoch = 1_288_834_974_657_i64;
    let maximum_time = epoch + ((1_i64 << 41) - 1);

    assert_eq!(
        detect::bound("snowflake", epoch, false, epoch),
        Ok("0".to_owned())
    );
    assert_eq!(
        detect::bound("snowflake", epoch, true, epoch),
        Ok("4194303".to_owned())
    );
    assert_eq!(
        detect::bound("snowflake", epoch + 1, false, epoch),
        Ok("4194304".to_owned())
    );
    assert_eq!(
        detect::bound("snowflake", maximum_time, true, epoch),
        Ok("9223372036854775807".to_owned())
    );
}

#[test]
fn bound_reports_range_errors_at_both_sides_of_every_kind() {
    for (kind, milliseconds, epoch) in [
        ("ulid", -1, 0),
        ("ulid", 281_474_976_710_656, 0),
        ("uuid7", -1, 0),
        ("uuid7", 281_474_976_710_656, 0),
        ("objectid", -1, 0),
        ("objectid", 4_294_967_296_000, 0),
        ("ksuid", 1_399_999_999_000, 0),
        ("ksuid", 5_694_967_296_000, 0),
        ("snowflake", 1_288_834_974_656, 1_288_834_974_657),
        ("snowflake", 3_487_858_230_209, 1_288_834_974_657),
    ] {
        assert_eq!(
            detect::bound(kind, milliseconds, false, epoch),
            Err(format!("time is out of range for {kind}"))
        );
    }
}
