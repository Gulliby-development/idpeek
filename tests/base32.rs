use idpeek::base32;

#[test]
fn encode_zero_as_twenty_six_zeroes() {
    assert_eq!(base32::encode(0), "00000000000000000000000000");
}

#[test]
fn decode_twenty_six_zeroes_as_zero() {
    assert_eq!(base32::decode("00000000000000000000000000"), Some(0));
}

#[test]
fn encode_one_with_a_leading_zero_padded_value() {
    assert_eq!(base32::encode(1), "00000000000000000000000001");
}

#[test]
fn decode_one_from_the_last_digit() {
    assert_eq!(base32::decode("00000000000000000000000001"), Some(1));
}

#[test]
fn encode_the_last_digit_boundary() {
    assert_eq!(base32::encode(31), "0000000000000000000000000Z");
}

#[test]
fn encode_the_next_digit_boundary() {
    assert_eq!(base32::encode(32), "00000000000000000000000010");
}

#[test]
fn encode_each_crockford_digit_with_its_exact_alphabet_symbol() {
    let cases = [
        (0_u128, b'0'),
        (1, b'1'),
        (2, b'2'),
        (3, b'3'),
        (4, b'4'),
        (5, b'5'),
        (6, b'6'),
        (7, b'7'),
        (8, b'8'),
        (9, b'9'),
        (10, b'A'),
        (11, b'B'),
        (12, b'C'),
        (13, b'D'),
        (14, b'E'),
        (15, b'F'),
        (16, b'G'),
        (17, b'H'),
        (18, b'J'),
        (19, b'K'),
        (20, b'M'),
        (21, b'N'),
        (22, b'P'),
        (23, b'Q'),
        (24, b'R'),
        (25, b'S'),
        (26, b'T'),
        (27, b'V'),
        (28, b'W'),
        (29, b'X'),
        (30, b'Y'),
        (31, b'Z'),
    ];

    for (value, expected_digit) in cases {
        let encoded = base32::encode(value);
        assert_eq!(encoded.len(), 26);
        assert_eq!(
            &encoded[..25],
            concat!("00000", "00000", "00000", "00000", "00000")
        );
        assert_eq!(encoded.as_bytes()[25], expected_digit);
    }
}

#[test]
fn decode_each_crockford_digit_with_its_exact_value() {
    let cases = [
        (concat!("00000", "00000", "00000", "00000", "00000", "0"), 0),
        (concat!("00000", "00000", "00000", "00000", "00000", "1"), 1),
        (concat!("00000", "00000", "00000", "00000", "00000", "2"), 2),
        (concat!("00000", "00000", "00000", "00000", "00000", "3"), 3),
        (concat!("00000", "00000", "00000", "00000", "00000", "4"), 4),
        (concat!("00000", "00000", "00000", "00000", "00000", "5"), 5),
        (concat!("00000", "00000", "00000", "00000", "00000", "6"), 6),
        (concat!("00000", "00000", "00000", "00000", "00000", "7"), 7),
        (concat!("00000", "00000", "00000", "00000", "00000", "8"), 8),
        (concat!("00000", "00000", "00000", "00000", "00000", "9"), 9),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "A"),
            10,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "B"),
            11,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "C"),
            12,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "D"),
            13,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "E"),
            14,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "F"),
            15,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "G"),
            16,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "H"),
            17,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "J"),
            18,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "K"),
            19,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "M"),
            20,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "N"),
            21,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "P"),
            22,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "Q"),
            23,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "R"),
            24,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "S"),
            25,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "T"),
            26,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "V"),
            27,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "W"),
            28,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "X"),
            29,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "Y"),
            30,
        ),
        (
            concat!("00000", "00000", "00000", "00000", "00000", "Z"),
            31,
        ),
    ];

    for (encoded, expected) in cases {
        assert_eq!(encoded.len(), 26);
        assert_eq!(base32::decode(encoded), Some(expected));
    }
}

#[test]
fn decode_accepts_lowercase_letters() {
    assert_eq!(base32::decode("0000000000000000000000000a"), Some(10));
    assert_eq!(base32::decode("0000000000000000000000000z"), Some(31));
}

#[test]
fn decode_accepts_mixed_case_ulid_text() {
    assert_eq!(
        base32::decode("01arz3ndektsv4rrffq69g5fav"),
        base32::decode("01ARZ3NDEKTSV4RRFFQ69G5FAV")
    );
}

#[test]
fn decode_rejects_i_instead_of_treating_it_as_an_alias() {
    assert_eq!(base32::decode("0000000000000000000000000I"), None);
}

#[test]
fn decode_rejects_l_instead_of_treating_it_as_an_alias() {
    assert_eq!(base32::decode("0000000000000000000000000L"), None);
}

#[test]
fn decode_rejects_o_instead_of_treating_it_as_an_alias() {
    assert_eq!(base32::decode("0000000000000000000000000O"), None);
}

#[test]
fn decode_rejects_u_instead_of_treating_it_as_an_alias() {
    assert_eq!(base32::decode("0000000000000000000000000U"), None);
}

#[test]
fn decode_rejects_forbidden_lowercase_letters() {
    assert_eq!(base32::decode("0000000000000000000000000i"), None);
    assert_eq!(base32::decode("0000000000000000000000000l"), None);
    assert_eq!(base32::decode("0000000000000000000000000o"), None);
    assert_eq!(base32::decode("0000000000000000000000000u"), None);
}

#[test]
fn decode_rejects_values_that_overflow_u128_at_the_first_digit() {
    assert_eq!(base32::decode("80000000000000000000000000"), None);
}

#[test]
fn decode_rejects_the_largest_first_digit_as_u128_overflow() {
    assert_eq!(base32::decode("Z0000000000000000000000000"), None);
}

#[test]
fn encode_and_decode_the_highest_representable_top_bit() {
    let value = 1_u128 << 127;
    assert_eq!(base32::encode(value), "40000000000000000000000000");
    assert_eq!(base32::decode("40000000000000000000000000"), Some(value));
}

#[test]
fn encode_the_maximum_u128_as_the_ulid_maximum() {
    assert_eq!(base32::encode(u128::MAX), "7ZZZZZZZZZZZZZZZZZZZZZZZZZ");
}

#[test]
fn decode_the_maximum_u128_as_the_ulid_maximum() {
    assert_eq!(
        base32::decode("7ZZZZZZZZZZZZZZZZZZZZZZZZZ"),
        Some(u128::MAX)
    );
}

#[test]
fn maximum_encoding_has_the_exact_adjacent_lower_value() {
    assert_eq!(base32::encode(u128::MAX - 1), "7ZZZZZZZZZZZZZZZZZZZZZZZZY");
}

#[test]
fn encode_decode_round_trip_handwritten_boundary_values() {
    let cases = [
        0_u128,
        1,
        31,
        32,
        1023,
        1024,
        1_u128 << 64,
        1_u128 << 127,
        u128::MAX - 1,
        u128::MAX,
    ];

    for value in cases {
        assert_eq!(base32::decode(&base32::encode(value)), Some(value));
    }
}

#[test]
fn decode_rejects_empty_and_short_values() {
    assert_eq!(base32::decode(""), None);
    assert_eq!(base32::decode("0000000000000000000000000"), None);
}

#[test]
fn decode_rejects_long_values() {
    assert_eq!(base32::decode("000000000000000000000000000"), None);
}

#[test]
fn decode_rejects_non_ascii_characters() {
    assert_eq!(base32::decode("000000000000000000000000é"), None);
}
