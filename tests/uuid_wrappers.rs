use idpeek::{detect, run};
use std::io::Cursor;

const BARE: &str = "017F22E2-79B0-7CC3-98C4-DC0C0C07398F";

fn info(args: &[&str]) -> (i32, String, String) {
    let owned: Vec<String> = args.iter().map(|argument| (*argument).to_owned()).collect();
    let mut stdin = Cursor::new(Vec::new());
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run(&owned, &mut stdin, &mut stdout, &mut stderr);
    (
        code,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

#[test]
fn braces_around_a_canonical_uuid_decode_like_the_bare_uuid() {
    let wrapped = detect::decode("{017F22E2-79B0-7CC3-98C4-DC0C0C07398F}", 0).unwrap();

    assert_eq!(wrapped, detect::decode(BARE, 0).unwrap());
    assert_eq!(wrapped.time_ms, Some(1_645_557_742_000));
}

#[test]
fn braces_around_a_compact_uuid_decode() {
    let wrapped = detect::decode("{00000000000070008000000000000000}", 0).unwrap();

    assert_eq!(wrapped.kind, "uuid");
    assert_eq!(wrapped.time_ms, Some(0));
}

#[test]
fn urn_prefix_is_accepted_in_any_case() {
    let expected = detect::decode(BARE, 0).unwrap();

    assert_eq!(
        detect::decode("urn:uuid:017F22E2-79B0-7CC3-98C4-DC0C0C07398F", 0).unwrap(),
        expected
    );
    assert_eq!(
        detect::decode("URN:UUID:017f22e2-79b0-7cc3-98c4-dc0c0c07398f", 0).unwrap(),
        expected
    );
    assert_eq!(
        detect::decode("Urn:Uuid:017F22E279B07CC398C4DC0C0C07398F", 0).unwrap(),
        expected
    );
}

#[test]
fn a_wrapper_that_does_not_match_is_an_unknown_format() {
    let cases = [
        "{017F22E2-79B0-7CC3-98C4-DC0C0C07398F",
        "017F22E2-79B0-7CC3-98C4-DC0C0C07398F}",
        "{}",
        "{abc}",
        "urn:uuid:",
        "urn:uuid:abc",
        "urn:uuidx017F22E2-79B0-7CC3-98C4-DC0C0C07398F",
        "[017F22E2-79B0-7CC3-98C4-DC0C0C07398F]",
    ];

    for input in cases {
        assert_eq!(
            detect::decode(input, 0),
            Err(format!("unknown ID format \"{input}\"")),
            "{input}"
        );
    }
}

#[test]
fn a_wrapped_value_that_is_not_a_uuid_reports_the_input_as_given() {
    assert_eq!(
        detect::decode("{017F22E2-79B0-7CC3-98C4-DC0C0C07398G}", 0),
        Err("invalid UUID \"{017F22E2-79B0-7CC3-98C4-DC0C0C07398G}\"".to_owned())
    );
    assert_eq!(
        detect::decode("urn:uuid:017F22E2-79B0-7CC3-98C4-DC0C0C07398G", 0),
        Err("invalid UUID \"urn:uuid:017F22E2-79B0-7CC3-98C4-DC0C0C07398G\"".to_owned())
    );
}

#[test]
fn nested_wrappers_are_not_unwrapped_twice() {
    assert_eq!(
        detect::decode("{{017F22E2-79B0-7CC3-98C4-DC0C0C07398F}}", 0),
        Err("unknown ID format \"{{017F22E2-79B0-7CC3-98C4-DC0C0C07398F}}\"".to_owned())
    );
    assert_eq!(
        detect::decode("{urn:uuid:017F22E2-79B0-7CC3-98C4-DC0C0C07398F}", 0),
        Err("unknown ID format \"{urn:uuid:017F22E2-79B0-7CC3-98C4-DC0C0C07398F}\"".to_owned())
    );
}

#[test]
fn info_prints_a_wrapped_uuid_as_typed() {
    let (code, stdout, stderr) = info(&["info", "{017F22E2-79B0-7CC3-98C4-DC0C0C07398F}"]);

    assert_eq!(code, 0);
    assert_eq!(
        stdout,
        "{017F22E2-79B0-7CC3-98C4-DC0C0C07398F}\n  kind      uuid\n  version   7 (unix time and random)\n  variant   rfc4122\n  time      2022-02-22T19:22:22.000Z\n"
    );
    assert_eq!(stderr, "");
}

#[test]
fn sort_orders_wrapped_and_bare_uuids_together() {
    let (code, stdout, stderr) = info(&[
        "sort",
        "urn:uuid:017F22E2-79B0-7CC3-98C4-DC0C0C07398F",
        "{C232AB00-9414-11EC-B3C8-9F6BDECED846}",
        "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    ]);

    assert_eq!(code, 0);
    assert_eq!(
        stdout,
        "01ARZ3NDEKTSV4RRFFQ69G5FAV\nurn:uuid:017F22E2-79B0-7CC3-98C4-DC0C0C07398F\n{C232AB00-9414-11EC-B3C8-9F6BDECED846}\n"
    );
    assert_eq!(stderr, "");
}

#[test]
fn info_reports_a_bad_wrapped_uuid_and_continues() {
    let (code, stdout, stderr) = info(&["info", "{not a uuid}", BARE]);

    assert_eq!(code, 1);
    assert_eq!(stderr, "idpeek: unknown ID format \"{not?a?uuid}\"\n");
    assert!(stdout.starts_with("017F22E2-79B0-7CC3-98C4-DC0C0C07398F\n"));
}
