use idpeek::{args, run};
use std::io::Cursor;

const ULID: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
const OBJECT_ID: &str = "507f1f77bcf86cd799439011";
const OBJECT_ID_NEXT: &str = "507f1f77bcf86cd799439012";
const KSUID: &str = "0ujtsYcgvSTl8PAuAdqWYSMnLOv";
const UUID_V7: &str = "017F22E2-79B0-7CC3-98C4-DC0C0C07398F";

fn capture(arguments: &[&str], input: &[u8]) -> (i32, String, String) {
    let owned: Vec<String> = arguments
        .iter()
        .map(|argument| (*argument).to_owned())
        .collect();
    let mut stdin = Cursor::new(input.to_vec());
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run(&owned, &mut stdin, &mut stdout, &mut stderr);
    (
        code,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

fn parse(arguments: &[&str]) -> Result<args::ParsedArgs, String> {
    let owned: Vec<String> = arguments
        .iter()
        .map(|argument| (*argument).to_owned())
        .collect();
    args::parse(&owned)
}

#[test]
fn unique_is_recorded_for_sort() {
    let parsed = parse(&["sort", "--unique", ULID]).unwrap();

    assert_eq!(
        parsed.command,
        args::Command::Sort {
            ids: vec![ULID.to_owned()],
            descending: false,
            unique: true,
        }
    );
}

#[test]
fn unique_can_follow_the_operands_and_combine_with_desc() {
    let parsed = parse(&["sort", ULID, "--desc", "--unique"]).unwrap();

    assert_eq!(
        parsed.command,
        args::Command::Sort {
            ids: vec![ULID.to_owned()],
            descending: true,
            unique: true,
        }
    );
}

#[test]
fn unique_is_an_unknown_option_for_the_other_commands() {
    for command in ["info", "span", "bound"] {
        assert_eq!(
            parse(&[command, "--unique", ULID]),
            Err("unknown option \"--unique\"".to_owned()),
            "command: {command}"
        );
    }
}

#[test]
fn sort_without_unique_keeps_repeated_ids() {
    let (code, stdout, stderr) = capture(&["sort", OBJECT_ID, ULID, OBJECT_ID], b"");

    assert_eq!(code, 0);
    assert_eq!(stdout, format!("{OBJECT_ID}\n{OBJECT_ID}\n{ULID}\n"));
    assert_eq!(stderr, "");
}

#[test]
fn unique_prints_each_repeated_id_once() {
    let (code, stdout, stderr) = capture(&["sort", "--unique", OBJECT_ID, ULID, OBJECT_ID], b"");

    assert_eq!(code, 0);
    assert_eq!(stdout, format!("{OBJECT_ID}\n{ULID}\n"));
    assert_eq!(stderr, "");
}

#[test]
fn unique_keeps_the_first_of_equal_time_ids_in_operand_order() {
    let (code, stdout, _) = capture(
        &[
            "sort",
            "--unique",
            OBJECT_ID_NEXT,
            OBJECT_ID,
            OBJECT_ID_NEXT,
        ],
        b"",
    );

    assert_eq!(code, 0);
    assert_eq!(stdout, format!("{OBJECT_ID_NEXT}\n{OBJECT_ID}\n"));
}

#[test]
fn unique_with_desc_reverses_the_times_only() {
    let (code, stdout, _) = capture(
        &["sort", "--desc", "--unique", KSUID, UUID_V7, KSUID, ULID],
        b"",
    );

    assert_eq!(code, 0);
    assert_eq!(stdout, format!("{UUID_V7}\n{KSUID}\n{ULID}\n"));
}

#[test]
fn unique_compares_the_text_as_typed() {
    let braced = format!("{{{UUID_V7}}}");
    let (code, stdout, _) = capture(&["sort", "--unique", UUID_V7, &braced, UUID_V7], b"");

    assert_eq!(code, 0);
    assert_eq!(stdout, format!("{UUID_V7}\n{braced}\n"));
}

#[test]
fn unique_also_drops_repeats_read_from_standard_input() {
    let input = format!("{ULID}\n{OBJECT_ID}\n{ULID}\n");
    let (code, stdout, _) = capture(&["sort", "--unique", "-", OBJECT_ID], input.as_bytes());

    assert_eq!(code, 0);
    assert_eq!(stdout, format!("{OBJECT_ID}\n{ULID}\n"));
}

#[test]
fn unique_still_fails_on_an_unknown_id_before_printing() {
    let (code, stdout, stderr) = capture(&["sort", "--unique", ULID, "bad", "bad"], b"");

    assert_eq!(code, 1);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "idpeek: unknown ID format \"bad\"\n");
}
