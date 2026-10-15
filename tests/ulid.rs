use idpeek::ulid;

#[test]
fn decode_matches_the_reference_ulid_fields() {
    let decoded = ulid::decode("01ARZ3NDEKTSV4RRFFQ69G5FAV").expect("valid ULID");

    assert_eq!(decoded.kind, "ulid");
    assert_eq!(
        decoded.fields,
        vec![
            ("time", "2016-07-30T23:54:10.259Z".to_string()),
            ("random", "d6764c61efb99302bd5b".to_string()),
        ]
    );
    assert_eq!(decoded.time_ms, Some(1_469_922_850_259));
}

#[test]
fn decode_accepts_lowercase_and_decodes_the_same_fields() {
    let decoded = ulid::decode("01arz3ndektsv4rrffq69g5fav").expect("lowercase ULID");
    let uppercase = ulid::decode("01ARZ3NDEKTSV4RRFFQ69G5FAV").expect("uppercase ULID");

    assert_eq!(decoded.kind, "ulid");
    assert_eq!(decoded.fields, uppercase.fields);
    assert_eq!(decoded.time_ms, uppercase.time_ms);
}

#[test]
fn decode_preserves_the_maximum_timestamp_and_random_bits() {
    let decoded = ulid::decode("7ZZZZZZZZZZZZZZZZZZZZZZZZZ").expect("maximum ULID");

    assert_eq!(
        decoded.fields,
        vec![
            ("time", "10889-08-02T05:31:50.655Z".to_string()),
            ("random", "ffffffffffffffffffff".to_string()),
        ]
    );
    assert_eq!(decoded.time_ms, Some(281_474_976_710_655));
}

#[test]
fn decode_zero_has_unix_epoch_and_zero_randomness() {
    let decoded = ulid::decode("00000000000000000000000000").expect("zero ULID");

    assert_eq!(
        decoded.fields,
        vec![
            ("time", "1970-01-01T00:00:00.000Z".to_string()),
            ("random", "00000000000000000000".to_string()),
        ]
    );
    assert_eq!(decoded.time_ms, Some(0));
}

#[test]
fn decode_rejects_the_first_timestamp_value_above_the_u128_range() {
    assert_eq!(ulid::decode("80000000000000000000000000"), None);
}

#[test]
fn decode_rejects_a_first_character_above_the_ulid_timestamp_limit() {
    assert_eq!(ulid::decode("90000000000000000000000000"), None);
}

#[test]
fn decode_rejects_each_forbidden_crockford_letter() {
    assert_eq!(ulid::decode("I1ARZ3NDEKTSV4RRFFQ69G5FAV"), None);
    assert_eq!(ulid::decode("L1ARZ3NDEKTSV4RRFFQ69G5FAV"), None);
    assert_eq!(ulid::decode("O1ARZ3NDEKTSV4RRFFQ69G5FAV"), None);
    assert_eq!(ulid::decode("U1ARZ3NDEKTSV4RRFFQ69G5FAV"), None);
    assert_eq!(ulid::decode("i1arz3ndektsv4rrffq69g5fav"), None);
    assert_eq!(ulid::decode("l1arz3ndektsv4rrffq69g5fav"), None);
    assert_eq!(ulid::decode("o1arz3ndektsv4rrffq69g5fav"), None);
    assert_eq!(ulid::decode("u1arz3ndektsv4rrffq69g5fav"), None);
}

#[test]
fn decode_rejects_short_and_long_spellings() {
    assert_eq!(ulid::decode("01ARZ3NDEKTSV4RRFFQ69G5FA"), None);
    assert_eq!(ulid::decode("01ARZ3NDEKTSV4RRFFQ69G5FAV0"), None);
}

#[test]
fn decode_rejects_non_ascii_characters() {
    assert_eq!(ulid::decode("０1ARZ3NDEKTSV4RRFFQ69G5FAV"), None);
}

#[test]
fn bound_zero_minimum_has_zero_random_bits() {
    assert_eq!(
        ulid::bound(0, false),
        Some("00000000000000000000000000".to_string())
    );
}

#[test]
fn bound_zero_maximum_has_all_eighty_random_bits_set() {
    assert_eq!(
        ulid::bound(0, true),
        Some("0000000000ZZZZZZZZZZZZZZZZ".to_string())
    );
}

#[test]
fn bound_one_millisecond_minimum_changes_only_timestamp_bits() {
    assert_eq!(
        ulid::bound(1, false),
        Some("00000000010000000000000000".to_string())
    );
}

#[test]
fn bound_maximum_timestamp_minimum_zeros_random_bits() {
    assert_eq!(
        ulid::bound(281_474_976_710_655, false),
        Some("7ZZZZZZZZZ0000000000000000".to_string())
    );
}

#[test]
fn bound_maximum_timestamp_maximum_sets_all_random_bits() {
    assert_eq!(
        ulid::bound(281_474_976_710_655, true),
        Some("7ZZZZZZZZZZZZZZZZZZZZZZZZZ".to_string())
    );
}

#[test]
fn bound_rejects_the_millisecond_before_the_timestamp_range() {
    assert_eq!(ulid::bound(-1, false), None);
    assert_eq!(ulid::bound(-1, true), None);
}

#[test]
fn bound_rejects_the_millisecond_after_the_timestamp_range() {
    assert_eq!(ulid::bound(281_474_976_710_656, false), None);
    assert_eq!(ulid::bound(281_474_976_710_656, true), None);
}

#[test]
fn bound_matches_the_reference_2024_midnight_ulid() {
    assert_eq!(
        ulid::bound(1_709_251_200_000, false),
        Some("01HQVMZ1000000000000000000".to_string())
    );
}

#[test]
fn bound_matches_the_reference_fractional_second_maximum_ulid() {
    assert_eq!(
        ulid::bound(1_709_296_200_500, true),
        Some("01HQWZWASMZZZZZZZZZZZZZZZZ".to_string())
    );
}
