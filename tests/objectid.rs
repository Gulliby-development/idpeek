use idpeek::objectid;

#[test]
fn decodes_the_reference_objectid_fields() {
    let decoded = objectid::decode("507f1f77bcf86cd799439011").unwrap();
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
fn decodes_uppercase_hex_without_changing_the_timestamp() {
    let decoded = objectid::decode("507F1F77BCF86CD799439011").unwrap();
    assert_eq!(
        decoded.fields[0],
        ("time", "2012-10-17T21:13:27.000Z".to_owned())
    );
    assert_eq!(decoded.fields[1], ("random", "bcf86cd799".to_owned()));
    assert_eq!(decoded.fields[2], ("counter", "4427793".to_owned()));
}

#[test]
fn decodes_the_zero_timestamp_and_zero_fields() {
    let decoded = objectid::decode("000000000000000000000000").unwrap();
    assert_eq!(
        decoded.fields,
        vec![
            ("time", "1970-01-01T00:00:00.000Z".to_owned()),
            ("random", "0000000000".to_owned()),
            ("counter", "0".to_owned()),
        ]
    );
    assert_eq!(decoded.time_ms, Some(0));
}

#[test]
fn decodes_the_maximum_unsigned_timestamp_and_fields() {
    let decoded = objectid::decode("ffffffffffffffffffffffff").unwrap();
    assert_eq!(
        decoded.fields,
        vec![
            ("time", "2106-02-07T06:28:15.000Z".to_owned()),
            ("random", "ffffffffff".to_owned()),
            ("counter", "16777215".to_owned()),
        ]
    );
    assert_eq!(decoded.time_ms, Some(4_294_967_295_000));
}

#[test]
fn decodes_a_nonzero_counter_at_its_exact_decimal_value() {
    let decoded = objectid::decode("000000000000000000000001").unwrap();
    assert_eq!(decoded.fields[2], ("counter", "1".to_owned()));
}

#[test]
fn decodes_the_maximum_three_byte_counter() {
    let decoded = objectid::decode("000000000000000000ffffff").unwrap();
    assert_eq!(decoded.fields[2], ("counter", "16777215".to_owned()));
}

#[test]
fn combines_counter_bytes_in_big_endian_order() {
    let decoded = objectid::decode("000000000000000000010203").unwrap();
    assert_eq!(decoded.fields[2], ("counter", "66051".to_owned()));
}

#[test]
fn decodes_all_five_random_bytes_in_lowercase_hex() {
    let decoded = objectid::decode("00000000a1b2c3d4e5000000").unwrap();
    assert_eq!(decoded.fields[1], ("random", "a1b2c3d4e5".to_owned()));
}

#[test]
fn rejects_an_objectid_that_is_one_character_short() {
    assert!(objectid::decode("507f1f77bcf86cd79943901").is_none());
}

#[test]
fn rejects_an_objectid_that_is_one_character_long() {
    assert!(objectid::decode("507f1f77bcf86cd7994390110").is_none());
}

#[test]
fn rejects_non_hexadecimal_objectid_characters() {
    assert!(objectid::decode("507f1f77bcf86cd79943901g").is_none());
}

#[test]
fn rejects_non_ascii_hex_lookalikes() {
    assert!(objectid::decode("507f1f77bcf86cd7994390１").is_none());
}

#[test]
fn smallest_bound_at_epoch_has_zero_suffix() {
    assert_eq!(
        objectid::bound(0, false).as_deref(),
        Some("000000000000000000000000")
    );
}

#[test]
fn largest_bound_at_epoch_has_all_one_suffix_bits() {
    assert_eq!(
        objectid::bound(0, true).as_deref(),
        Some("00000000ffffffffffffffff")
    );
}

#[test]
fn lower_bound_floors_milliseconds_to_the_containing_second() {
    assert_eq!(
        objectid::bound(1_700_000_999, false).as_deref(),
        Some("0019f0a00000000000000000")
    );
}

#[test]
fn upper_bound_floors_milliseconds_to_the_containing_second() {
    assert_eq!(
        objectid::bound(1_700_000_999, true).as_deref(),
        Some("0019f0a0ffffffffffffffff")
    );
}

#[test]
fn bound_accepts_the_last_millisecond_of_the_maximum_second() {
    assert_eq!(
        objectid::bound(4_294_967_295_999, false).as_deref(),
        Some("ffffffff0000000000000000")
    );
}

#[test]
fn bound_rejects_a_negative_timestamp() {
    assert_eq!(objectid::bound(-1, false), None);
}

#[test]
fn bound_rejects_the_first_millisecond_after_objectid_range() {
    assert_eq!(objectid::bound(4_294_967_296_000, false), None);
}
