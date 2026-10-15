use idpeek::detect::Decoded;
use idpeek::uuid;

fn decode(input: &str) -> Decoded {
    uuid::decode(input).expect("the handwritten UUID should decode")
}

fn field<'a>(decoded: &'a Decoded, name: &str) -> Option<&'a str> {
    decoded
        .fields
        .iter()
        .find_map(|(field_name, value)| (*field_name == name).then_some(value.as_str()))
}

#[test]
fn every_version_nibble_has_its_reference_name() {
    let cases = [
        ("00000000000000008000000000000000", "0 (unassigned)"),
        ("00000000000010008000000000000000", "1 (time and node)"),
        ("00000000000020008000000000000000", "2 (dce security)"),
        ("00000000000030008000000000000000", "3 (md5 name)"),
        ("00000000000040008000000000000000", "4 (random)"),
        ("00000000000050008000000000000000", "5 (sha1 name)"),
        ("00000000000060008000000000000000", "6 (reordered time)"),
        (
            "00000000000070008000000000000000",
            "7 (unix time and random)",
        ),
        ("00000000000080008000000000000000", "8 (custom)"),
        ("00000000000090008000000000000000", "9 (unassigned)"),
        ("000000000000a0008000000000000000", "10 (unassigned)"),
        ("000000000000b0008000000000000000", "11 (unassigned)"),
        ("000000000000c0008000000000000000", "12 (unassigned)"),
        ("000000000000d0008000000000000000", "13 (unassigned)"),
        ("000000000000e0008000000000000000", "14 (unassigned)"),
        ("000000000000f0008000000000000000", "15 (unassigned)"),
    ];

    for (input, expected) in cases {
        assert_eq!(field(&decode(input), "version"), Some(expected));
    }
}

#[test]
fn uuid_kind_is_reported() {
    assert_eq!(decode("00000000-0000-4000-8000-000000000000").kind, "uuid");
}

#[test]
fn zero_uuid_has_version_zero_and_ncs_variant_without_time() {
    let decoded = decode("00000000-0000-0000-0000-000000000000");

    assert_eq!(field(&decoded, "version"), Some("0 (unassigned)"));
    assert_eq!(field(&decoded, "variant"), Some("ncs"));
    assert_eq!(decoded.time_ms, None);
    assert_eq!(decoded.fields.len(), 2);
}

#[test]
fn all_ones_uuid_has_unassigned_version_and_future_variant_without_time() {
    let decoded = decode("ffffffff-ffff-ffff-ffff-ffffffffffff");

    assert_eq!(field(&decoded, "version"), Some("15 (unassigned)"));
    assert_eq!(field(&decoded, "variant"), Some("future"));
    assert_eq!(decoded.time_ms, None);
    assert_eq!(decoded.fields.len(), 2);
}

#[test]
fn variant_zero_prefix_is_ncs() {
    assert_eq!(
        field(&decode("00000000-0000-4000-0000-000000000000"), "variant"),
        Some("ncs")
    );
}

#[test]
fn variant_one_zero_prefix_is_rfc4122() {
    assert_eq!(
        field(&decode("00000000-0000-4000-8000-000000000000"), "variant"),
        Some("rfc4122")
    );
}

#[test]
fn variant_six_prefix_is_microsoft() {
    assert_eq!(
        field(&decode("00000000-0000-4000-c000-000000000000"), "variant"),
        Some("microsoft")
    );
}

#[test]
fn variant_seven_prefix_is_future() {
    assert_eq!(
        field(&decode("00000000-0000-4000-e000-000000000000"), "variant"),
        Some("future")
    );
}

#[test]
fn canonical_uuid_preserves_all_decoded_fields() {
    let decoded = decode("C232AB00-9414-11EC-B3C8-9F6BDECED846");

    assert_eq!(decoded.kind, "uuid");
    assert_eq!(field(&decoded, "version"), Some("1 (time and node)"));
    assert_eq!(field(&decoded, "variant"), Some("rfc4122"));
    assert_eq!(field(&decoded, "time"), Some("2022-02-22T19:22:22.000Z"));
    assert_eq!(field(&decoded, "clock"), Some("13256"));
    assert_eq!(field(&decoded, "node"), Some("9f6bdeced846"));
}

#[test]
fn uuid_field_order_places_time_before_clock_and_node() {
    let decoded = decode("C232AB00-9414-11EC-B3C8-9F6BDECED846");
    let names = decoded
        .fields
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>();

    assert_eq!(names, vec!["version", "variant", "time", "clock", "node"]);
}

#[test]
fn compact_uuid_is_accepted() {
    let decoded = decode("017f22e279b07cc398c4dc0c0c07398f");

    assert_eq!(field(&decoded, "version"), Some("7 (unix time and random)"));
    assert_eq!(field(&decoded, "time"), Some("2022-02-22T19:22:22.000Z"));
}

#[test]
fn uppercase_and_lowercase_uuid_spellings_decode_equally() {
    let uppercase = decode("017F22E2-79B0-7CC3-98C4-DC0C0C07398F");
    let lowercase = decode("017f22e2-79b0-7cc3-98c4-dc0c0c07398f");

    assert_eq!(uppercase.fields, lowercase.fields);
    assert_eq!(uppercase.time_ms, lowercase.time_ms);
}

#[test]
fn malformed_uuid_hyphen_positions_are_rejected() {
    assert!(uuid::decode("0000000-0000-1000-8000-000000000000").is_none());
    assert!(uuid::decode("00000000_0000-1000-8000-000000000000").is_none());
    assert!(uuid::decode("00000000-0000-1000_8000-000000000000").is_none());
    assert!(uuid::decode("00000000-0000-1000-8000-00000000000").is_none());
}

#[test]
fn non_hexadecimal_uuid_digits_are_rejected_in_both_shapes() {
    assert!(uuid::decode("0000000000004000800000000000000g").is_none());
    assert!(uuid::decode("00000000-0000-4000-8000-00000000000g").is_none());
}

#[test]
fn uuid_lengths_other_than_compact_or_canonical_are_rejected() {
    assert!(uuid::decode("0000000000004000800000000000000").is_none());
    assert!(uuid::decode("00000000-0000-4000-8000-0000000000000").is_none());
}

#[test]
fn v1_zero_ticks_start_at_the_gregorian_uuid_epoch() {
    let decoded = decode("00000000-0000-1000-8000-000000000000");

    assert_eq!(decoded.time_ms, Some(-12_219_292_800_000));
    assert_eq!(field(&decoded, "time"), Some("1582-10-15T00:00:00.000Z"));
}

#[test]
fn v1_combines_time_high_mid_and_low_in_timestamp_order() {
    let decoded = decode("00000003-0002-1001-8000-000000000000");

    assert_eq!(decoded.time_ms, Some(-12_191_144_443_336));
}

#[test]
fn v1_discards_9999_submillisecond_ticks_by_flooring() {
    let decoded = decode("0000270f-0000-1000-8000-000000000000");

    assert_eq!(decoded.time_ms, Some(-12_219_292_800_000));
}

#[test]
fn v1_floors_19999_ticks_to_one_millisecond() {
    let decoded = decode("00004e1f-0000-1000-8000-000000000000");

    assert_eq!(decoded.time_ms, Some(-12_219_292_799_999));
}

#[test]
fn v1_clock_masks_variant_bits_and_node_uses_last_six_bytes() {
    let decoded = decode("00000000-0000-1000-b456-0123456789ab");

    assert_eq!(field(&decoded, "clock"), Some("13398"));
    assert_eq!(field(&decoded, "node"), Some("0123456789ab"));
}

#[test]
fn v6_zero_reordered_ticks_start_at_the_gregorian_uuid_epoch() {
    let decoded = decode("00000000-0000-6000-8000-000000000000");

    assert_eq!(decoded.time_ms, Some(-12_219_292_800_000));
    assert_eq!(field(&decoded, "time"), Some("1582-10-15T00:00:00.000Z"));
}

#[test]
fn v6_uses_the_first_48_bits_before_the_low_twelve() {
    let decoded = decode("00000000-0010-6002-8000-000000000000");

    assert_eq!(decoded.time_ms, Some(-12_219_292_799_994));
}

#[test]
fn v6_low_twelve_bits_can_cross_a_millisecond_boundary() {
    let decoded = decode("00000000-0002-6710-8000-000000000000");

    assert_eq!(decoded.time_ms, Some(-12_219_292_799_999));
}

#[test]
fn v6_clock_and_node_fields_use_the_same_byte_rules_as_v1() {
    let decoded = decode("00000000-0000-6000-b456-0123456789ab");

    assert_eq!(field(&decoded, "clock"), Some("13398"));
    assert_eq!(field(&decoded, "node"), Some("0123456789ab"));
}

#[test]
fn v7_zero_timestamp_is_the_unix_epoch() {
    let decoded = decode("00000000-0000-7000-8000-000000000000");

    assert_eq!(decoded.time_ms, Some(0));
    assert_eq!(field(&decoded, "time"), Some("1970-01-01T00:00:00.000Z"));
    assert_eq!(field(&decoded, "clock"), None);
    assert_eq!(field(&decoded, "node"), None);
}

#[test]
fn v7_maximum_timestamp_uses_all_first_48_bits() {
    let decoded = decode("ffffffff-ffff-7fff-8000-000000000000");

    assert_eq!(decoded.time_ms, Some(281_474_976_710_655));
}

#[test]
fn v7_timestamp_is_first_48_bits_and_ignores_random_tail() {
    let first = decode("017f22e2-79b0-7000-8000-000000000000");
    let second = decode("017f22e2-79b0-7fff-bfff-ffffffffffff");

    assert_eq!(first.time_ms, Some(1_645_557_742_000));
    assert_eq!(second.time_ms, first.time_ms);
}

#[test]
fn versions_without_time_do_not_emit_a_time_field() {
    let version_two = decode("00000000-0000-2000-8000-000000000000");
    let version_three = decode("00000000-0000-3000-8000-000000000000");
    let version_four = decode("00000000-0000-4000-8000-000000000000");
    let version_five = decode("00000000-0000-5000-8000-000000000000");
    let version_eight = decode("00000000-0000-8000-8000-000000000000");

    assert_eq!(field(&version_two, "time"), None);
    assert_eq!(field(&version_three, "time"), None);
    assert_eq!(field(&version_four, "time"), None);
    assert_eq!(field(&version_five, "time"), None);
    assert_eq!(field(&version_eight, "time"), None);
    assert_eq!(version_two.fields.len(), 2);
    assert_eq!(version_three.fields.len(), 2);
    assert_eq!(version_four.fields.len(), 2);
    assert_eq!(version_five.fields.len(), 2);
    assert_eq!(version_eight.fields.len(), 2);
}
