use idpeek::ksuid::{
    Decoded, EPOCH_UNIX_SECONDS, bound, decode, decode_bytes, encode, payload_hex,
};

const ZERO: &str = "000000000000000000000000000";
const ONE: &str = "000000000000000000000000001";
const MAXIMUM: &str = "aWgEPTl1tmebfsQzFP4bxwgy80V";
const REFERENCE: &str = "0ujtsYcgvSTl8PAuAdqWYSMnLOv";

#[test]
fn decode_reference_ksuid_has_the_expected_time_and_payload() {
    let decoded = decode(REFERENCE).unwrap();
    assert_eq!(decoded.time_ms, 1_507_608_047_000);
    assert_eq!(payload_hex(&decoded), "b5a1cd34b5f99d1154fb6853345c9735");
}

#[test]
fn decode_zero_is_the_ksuid_epoch_with_zero_payload() {
    let decoded = decode(ZERO).unwrap();
    assert_eq!(decoded.time_ms, 1_400_000_000_000);
    assert_eq!(decoded.payload, [0; 16]);
}

#[test]
fn decode_maximum_uses_the_last_32_bit_timestamp_and_payload() {
    let decoded = decode(MAXIMUM).unwrap();
    assert_eq!(decoded.time_ms, 5_694_967_295_000);
    assert_eq!(decoded.payload, [u8::MAX; 16]);
}

#[test]
fn decode_rejects_the_first_base62_value_above_160_bits() {
    assert_eq!(decode("aWgEPTl1tmebfsQzFP4bxwgy80W"), None);
}

#[test]
fn decode_rejects_a_value_that_is_one_character_short() {
    assert_eq!(decode("0ujtsYcgvSTl8PAuAdqWYSMnLO"), None);
}

#[test]
fn decode_rejects_a_value_that_is_one_character_long() {
    assert_eq!(decode("00ujtsYcgvSTl8PAuAdqWYSMnLOv"), None);
}

#[test]
fn decode_rejects_non_ascii_text() {
    assert_eq!(decode("0ujtsYcgvSTl8PAuAdqWYSMnLö"), None);
}

#[test]
fn decode_bytes_rejects_punctuation_outside_the_alphabet() {
    assert_eq!(decode_bytes("0ujtsYcgvSTl8PAuAdqWYSMnLO!"), None);
}

#[test]
fn decode_bytes_rejects_ascii_punctuation_in_the_first_position() {
    assert_eq!(decode_bytes("!ujtsYcgvSTl8PAuAdqWYSMnLOv"), None);
}

#[test]
fn decode_bytes_maps_twenty_seven_zeroes_to_twenty_zero_bytes() {
    assert_eq!(ZERO.len(), 27);
    assert_eq!(decode_bytes(ZERO), Some([0; 20]));
}

#[test]
fn decode_bytes_maps_one_to_the_last_byte() {
    let mut expected = [0; 20];
    expected[19] = 1;
    assert_eq!(decode_bytes(ONE), Some(expected));
}

#[test]
fn encode_zero_uses_all_twenty_seven_positions() {
    assert_eq!(encode(&[0; 20]), ZERO);
    assert_eq!(ZERO.len(), 27);
}

#[test]
fn encode_one_is_left_padded_to_the_fixed_width() {
    let mut value = [0; 20];
    value[19] = 1;
    assert_eq!(encode(&value), ONE);
    assert_eq!(ONE.len(), 27);
}

#[test]
fn encode_uses_the_uppercase_base62_digit_after_nine() {
    let mut value = [0; 20];
    value[19] = 10;
    assert_eq!(encode(&value), "00000000000000000000000000A");
}

#[test]
fn encode_uses_lowercase_base62_digits_after_uppercase() {
    let mut value = [0; 20];
    value[19] = 36;
    assert_eq!(encode(&value), "00000000000000000000000000a");
}

#[test]
fn encode_maximum_matches_the_documented_160_bit_boundary() {
    assert_eq!(encode(&[u8::MAX; 20]), MAXIMUM);
    assert_eq!(decode_bytes(MAXIMUM), Some([u8::MAX; 20]));
}

#[test]
fn encode_and_decode_round_trip_the_reference_bytes() {
    let bytes = decode_bytes(REFERENCE).unwrap();
    assert_eq!(encode(&bytes), REFERENCE);
}

#[test]
fn bound_minimum_has_zero_timestamp_offset_and_payload() {
    assert_eq!(bound(1_400_000_000_000, false), Some(ZERO.to_owned()));
}

#[test]
fn bound_rejects_a_timestamp_before_the_epoch_even_by_one_millisecond() {
    assert_eq!(bound(1_399_999_999_999, false), None);
}

#[test]
fn bound_floors_milliseconds_within_the_epoch_second() {
    assert_eq!(bound(1_400_000_000_999, false), Some(ZERO.to_owned()));
}

#[test]
fn bound_advances_at_the_next_whole_second() {
    let result = bound(1_400_000_001_000, false).unwrap();
    let bytes = decode_bytes(&result).unwrap();
    assert_eq!(result.len(), 27);
    assert_eq!(&bytes[..4], &[0, 0, 0, 1]);
    assert_eq!(&bytes[4..], &[0; 16]);
}

#[test]
fn bound_maximum_uses_the_maximum_timestamp_and_payload() {
    assert_eq!(bound(5_694_967_295_000, true), Some(MAXIMUM.to_owned()));
}

#[test]
fn bound_maximum_accepts_the_last_second_through_its_final_millisecond() {
    let result = bound(5_694_967_295_999, false).unwrap();
    let bytes = decode_bytes(&result).unwrap();
    assert_eq!(result.len(), 27);
    assert_eq!(&bytes[..4], &[u8::MAX; 4]);
    assert_eq!(&bytes[4..], &[0; 16]);
}

#[test]
fn bound_rejects_a_timestamp_after_the_maximum_second() {
    assert_eq!(bound(5_694_967_296_000, false), None);
}

#[test]
fn bound_uses_the_expected_epoch_constant() {
    assert_eq!(EPOCH_UNIX_SECONDS, 1_400_000_000);
    let decoded = Decoded {
        time_ms: 1_400_000_000_000,
        payload: [0; 16],
    };
    assert_eq!(decode(ZERO), Some(decoded));
}
