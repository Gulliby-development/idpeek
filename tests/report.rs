use idpeek::detect::{self, Decoded};
use idpeek::report;

#[test]
fn usage_is_the_exact_four_line_lf_terminated_message() {
    let mut output = Vec::new();

    report::write_usage(&mut output).unwrap();

    assert_eq!(
        output,
        b"usage: idpeek info [--epoch EPOCH] ID...\n       idpeek span [--epoch EPOCH] ID1 ID2\n       idpeek sort [--epoch EPOCH] [--desc] ID...\n       idpeek bound [--epoch EPOCH] [--max] KIND TIME\n"
    );
}

#[test]
fn error_lines_have_the_prefix_and_lf_terminator() {
    let mut output = Vec::new();

    report::write_error(&mut output, "invalid ULID \"bad\"").unwrap();

    assert_eq!(output, b"idpeek: invalid ULID \"bad\"\n");
}

#[test]
fn info_writes_kind_before_ordered_and_padded_fields() {
    let decoded = Decoded {
        kind: "snowflake",
        fields: vec![
            ("epoch", "1420070400000".to_owned()),
            ("time", "2016-04-30T11:18:25.796Z".to_owned()),
            ("worker", "32".to_owned()),
            ("sequence", "7".to_owned()),
        ],
        time_ms: Some(1_461_999_505_796),
    };
    let mut output = Vec::new();

    report::write_info(&mut output, "175928847299117063", &decoded).unwrap();

    assert_eq!(
        output,
        b"175928847299117063\n  kind      snowflake\n  epoch     1420070400000\n  time      2016-04-30T11:18:25.796Z\n  worker    32\n  sequence  7\n"
    );
}

#[test]
fn info_preserves_the_identifier_spelling_and_field_value_bytes() {
    let decoded = Decoded {
        kind: "ulid",
        fields: vec![("random", "abcdef0123456789abcd".to_owned())],
        time_ms: Some(0),
    };
    let mut output = Vec::new();

    report::write_info(&mut output, "Mixed-Case-λ", &decoded).unwrap();

    assert_eq!(
        output,
        "Mixed-Case-λ\n  kind      ulid\n  random    abcdef0123456789abcd\n".as_bytes()
    );
}

#[test]
fn info_does_not_add_unavailable_uuid_time_or_other_fields() {
    let decoded = detect::decode("5df41881-3aed-3515-88a7-2f4a814cf09e", 0).unwrap();
    let mut output = Vec::new();

    report::write_info(
        &mut output,
        "5df41881-3aed-3515-88a7-2f4a814cf09e",
        &decoded,
    )
    .unwrap();

    assert_eq!(
        output,
        b"5df41881-3aed-3515-88a7-2f4a814cf09e\n  kind      uuid\n  version   3 (md5 name)\n  variant   rfc4122\n"
    );
    assert_eq!(decoded.time_ms, None);
}

#[test]
fn write_id_emits_one_exact_lf_terminated_line() {
    let mut output = Vec::new();

    report::write_id(&mut output, "01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();

    assert_eq!(output, b"01ARZ3NDEKTSV4RRFFQ69G5FAV\n");
}

#[test]
fn sorted_writer_emits_each_supplied_id_in_order() {
    let ids = vec!["first".to_owned(), "second".to_owned(), "third".to_owned()];
    let mut output = Vec::new();

    report::write_sorted(&mut output, &ids).unwrap();

    assert_eq!(output, b"first\nsecond\nthird\n");
}

#[test]
fn sorted_writer_emits_nothing_for_an_empty_list() {
    let ids: Vec<String> = Vec::new();
    let mut output = Vec::new();

    report::write_sorted(&mut output, &ids).unwrap();

    assert_eq!(output, b"");
}

#[test]
fn bound_writer_emits_the_identifier_and_one_lf() {
    let mut output = Vec::new();

    report::write_bound(&mut output, "018df74f-8400-7000-8000-000000000000").unwrap();

    assert_eq!(output, b"018df74f-8400-7000-8000-000000000000\n");
}

#[test]
fn span_matches_the_reference_earlier_duration_and_padding() {
    let mut output = Vec::new();

    report::write_span(
        &mut output,
        "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        1_469_922_850_259,
        "507f1f77bcf86cd799439011",
        1_350_508_407_000,
    )
    .unwrap();

    assert_eq!(
        output,
        b"01ARZ3NDEKTSV4RRFFQ69G5FAV  2016-07-30T23:54:10.259Z\n507f1f77bcf86cd799439011    2012-10-17T21:13:27.000Z\nthe second is 1382d 02:40:43.259 earlier than the first\n"
    );
}

#[test]
fn span_describes_a_later_second_identifier() {
    let mut output = Vec::new();

    report::write_span(&mut output, "early", 0, "later", 1_000).unwrap();

    assert_eq!(
        output,
        b"early  1970-01-01T00:00:00.000Z\nlater  1970-01-01T00:00:01.000Z\nthe second is 00:00:01.000 later than the first\n"
    );
}

#[test]
fn span_describes_equal_timestamps_without_a_duration() {
    let mut output = Vec::new();

    report::write_span(&mut output, "a", 0, "a longer id", 0).unwrap();

    assert_eq!(
        output,
        b"a            1970-01-01T00:00:00.000Z\na longer id  1970-01-01T00:00:00.000Z\nthe second is at the same time as the first\n"
    );
}

#[test]
fn span_formats_the_day_boundary_and_fractional_milliseconds() {
    let mut output = Vec::new();

    report::write_span(&mut output, "A", 86_400_001, "LONG", 0).unwrap();

    assert_eq!(
        output,
        b"A     1970-01-02T00:00:00.001Z\nLONG  1970-01-01T00:00:00.000Z\nthe second is 1d 00:00:00.001 earlier than the first\n"
    );
}

#[test]
fn span_width_counts_unicode_scalar_values_not_utf8_bytes() {
    let mut output = Vec::new();

    report::write_span(&mut output, "猫", 0, "id", 0).unwrap();

    assert_eq!(
        output,
        "猫   1970-01-01T00:00:00.000Z\nid  1970-01-01T00:00:00.000Z\nthe second is at the same time as the first\n".as_bytes()
    );
}
