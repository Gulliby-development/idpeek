use idpeek::text::{hex, parse_hex, show};

#[test]
fn show_quotes_empty_text() {
    assert_eq!(show(""), r#""""#);
}

#[test]
fn show_preserves_printable_ascii_boundaries() {
    assert_eq!(show("!~"), r#""!~""#);
}

#[test]
fn show_preserves_quotes_and_backslashes() {
    assert_eq!(show(r#"a"b\c"#), r#""a"b\c""#);
}

#[test]
fn show_replaces_spaces_and_ascii_controls() {
    assert_eq!(show("\0 \t\n\r\x7f"), r#""??????""#);
}

#[test]
fn show_replaces_each_non_ascii_code_point_once() {
    assert_eq!(show("é中🦀"), r#""???""#);
}

#[test]
fn show_mixes_preserved_ascii_and_replaced_characters() {
    assert_eq!(show("Aé!中~"), r#""A?!?~""#);
}

#[test]
fn hex_encodes_empty_input() {
    assert_eq!(hex(&[]), "");
}

#[test]
fn hex_encodes_single_byte_with_a_leading_zero() {
    assert_eq!(hex(&[0x0a]), "0a");
}

#[test]
fn hex_encodes_all_byte_nibble_boundaries_in_lowercase() {
    assert_eq!(
        hex(&[0x00, 0x01, 0x0f, 0x10, 0x7f, 0x80, 0xfe, 0xff]),
        "00010f107f80feff"
    );
}

#[test]
fn parse_hex_accepts_empty_input() {
    assert_eq!(parse_hex(""), Some(Vec::new()));
}

#[test]
fn parse_hex_accepts_lowercase_digits() {
    assert_eq!(parse_hex("00af19"), Some(vec![0x00, 0xaf, 0x19]));
}

#[test]
fn parse_hex_accepts_uppercase_digits() {
    assert_eq!(parse_hex("00AF19"), Some(vec![0x00, 0xaf, 0x19]));
}

#[test]
fn parse_hex_accepts_mixed_case_digits() {
    assert_eq!(parse_hex("aB09cD"), Some(vec![0xab, 0x09, 0xcd]));
}

#[test]
fn parse_hex_rejects_odd_length_input() {
    assert_eq!(parse_hex("0"), None);
    assert_eq!(parse_hex("abc"), None);
}

#[test]
fn parse_hex_rejects_non_hex_ascii_digits() {
    assert_eq!(parse_hex("g0"), None);
    assert_eq!(parse_hex("0g"), None);
}

#[test]
fn parse_hex_rejects_non_ascii_input() {
    assert_eq!(parse_hex("é"), None);
}

#[test]
fn parse_hex_parses_each_byte_pair_in_order() {
    assert_eq!(
        parse_hex("1234567890abcdef"),
        Some(vec![0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef])
    );
}
