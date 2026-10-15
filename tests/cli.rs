use idpeek::run;
use std::io::{self, Cursor, Read, Write};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const ULID: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
const OBJECT_ID: &str = "507f1f77bcf86cd799439011";
const KSUID: &str = "0ujtsYcgvSTl8PAuAdqWYSMnLOv";
const UUID_V1: &str = "C232AB00-9414-11EC-B3C8-9F6BDECED846";
const UUID_V7: &str = "017F22E2-79B0-7CC3-98C4-DC0C0C07398F";
const SNOWFLAKE: &str = "175928847299117063";

fn capture(args: &[&str], input: &[u8]) -> (i32, Vec<u8>, Vec<u8>) {
    let owned_args: Vec<String> = args.iter().map(|argument| (*argument).to_owned()).collect();
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = run(&owned_args, &mut stdin, &mut stdout, &mut stderr);

    (code, stdout, stderr)
}

fn assert_run(
    args: &[&str],
    input: &[u8],
    expected_code: i32,
    expected_stdout: &str,
    expected_stderr: &str,
) {
    let (code, stdout, stderr) = capture(args, input);
    assert_eq!(code, expected_code, "arguments: {args:?}");
    assert_eq!(stdout, expected_stdout.as_bytes(), "arguments: {args:?}");
    assert_eq!(stderr, expected_stderr.as_bytes(), "arguments: {args:?}");
}

#[test]
fn help_prints_the_exact_four_usage_lines() {
    assert_run(
        &["--help"],
        b"",
        0,
        "usage: idpeek info [--epoch EPOCH] ID...\n       idpeek span [--epoch EPOCH] ID1 ID2\n       idpeek sort [--epoch EPOCH] [--desc] ID...\n       idpeek bound [--epoch EPOCH] [--max] KIND TIME\n",
        "",
    );
}

#[test]
fn usage_errors_have_the_expected_messages_status_and_empty_stdout() {
    let cases: &[(&[&str], &str)] = &[
        (&[], "missing command (info, span, sort or bound)"),
        (&["other"], "unknown command \"other\""),
        (&["info"], "missing ID"),
        (&["span"], "missing ID"),
        (&["span", "one"], "missing ID"),
        (&["span", "one", "two", "three"], "too many arguments"),
        (&["sort"], "missing ID"),
        (&["bound"], "missing KIND"),
        (&["bound", "ulid"], "missing TIME"),
        (&["info", "--max", "x"], "unknown option \"--max\""),
        (&["sort", "--max", "x"], "unknown option \"--max\""),
        (
            &["bound", "--desc", "ulid", "2024-03-01"],
            "unknown option \"--desc\"",
        ),
        (&["info", "--epoch"], "--epoch needs a value"),
        (&["info", "--epoch=bad", "x"], "invalid --epoch \"bad\""),
        (&["info", "--epoch", "-1", "x"], "invalid --epoch \"-1\""),
        (
            &["info", "--epoch", "--desc", "x"],
            "invalid --epoch \"--desc\"",
        ),
        (
            &["bound", "ulid", "2024-03-01", "extra"],
            "too many arguments",
        ),
        (&["bound", "uuid4", "2024-03-01"], "invalid KIND \"uuid4\""),
        (
            &["bound", "ulid", "2024-02-30"],
            "invalid TIME \"2024-02-30\"",
        ),
    ];

    for (args, message) in cases {
        let (code, stdout, stderr) = capture(args, b"");
        assert_eq!(code, 2, "arguments: {args:?}");
        assert!(stdout.is_empty(), "arguments: {args:?}");
        assert_eq!(
            stderr,
            format!("idpeek: {message}\n").as_bytes(),
            "arguments: {args:?}"
        );
    }
}

#[test]
fn option_scanning_errors_precede_epoch_validation() {
    assert_run(
        &["info", "--epoch=bad", "--unknown"],
        b"",
        2,
        "",
        "idpeek: unknown option \"--unknown\"\n",
    );
}

#[test]
fn invalid_final_epoch_is_reported_after_option_scanning() {
    assert_run(
        &["info", "--epoch=1", "--epoch=bad", ULID],
        b"",
        2,
        "",
        "idpeek: invalid --epoch \"bad\"\n",
    );
}

#[test]
fn a_thirteen_digit_epoch_is_accepted() {
    assert_run(
        &["info", "--epoch=9999999999999", "175928847299117063"],
        b"",
        0,
        "175928847299117063\n  kind      snowflake\n  epoch     9999999999999\n  time      2288-03-20T05:05:05.795Z\n  worker    32\n  sequence  7\n",
        "",
    );
}

#[test]
fn a_fourteen_digit_epoch_is_rejected() {
    assert_run(
        &["info", "--epoch=10000000000000", SNOWFLAKE],
        b"",
        2,
        "",
        "idpeek: invalid --epoch \"10000000000000\"\n",
    );
}

#[test]
fn the_last_named_epoch_option_wins() {
    assert_run(
        &["info", "--epoch=unix", "--epoch=discord", SNOWFLAKE],
        b"",
        0,
        "175928847299117063\n  kind      snowflake\n  epoch     1420070400000\n  time      2016-04-30T11:18:25.796Z\n  worker    32\n  sequence  7\n",
        "",
    );
}

#[test]
fn info_matches_the_ulid_ksuid_and_objectid_reference_blocks() {
    assert_run(
        &["info", ULID, KSUID, OBJECT_ID],
        b"",
        0,
        "01ARZ3NDEKTSV4RRFFQ69G5FAV\n  kind      ulid\n  time      2016-07-30T23:54:10.259Z\n  random    d6764c61efb99302bd5b\n0ujtsYcgvSTl8PAuAdqWYSMnLOv\n  kind      ksuid\n  time      2017-10-10T04:00:47.000Z\n  payload   b5a1cd34b5f99d1154fb6853345c9735\n507f1f77bcf86cd799439011\n  kind      objectid\n  time      2012-10-17T21:13:27.000Z\n  random    bcf86cd799\n  counter   4427793\n",
        "",
    );
}

#[test]
fn info_matches_the_uuid_v1_and_v7_reference_blocks() {
    assert_run(
        &["info", UUID_V1, UUID_V7],
        b"",
        0,
        "C232AB00-9414-11EC-B3C8-9F6BDECED846\n  kind      uuid\n  version   1 (time and node)\n  variant   rfc4122\n  time      2022-02-22T19:22:22.000Z\n  clock     13256\n  node      9f6bdeced846\n017F22E2-79B0-7CC3-98C4-DC0C0C07398F\n  kind      uuid\n  version   7 (unix time and random)\n  variant   rfc4122\n  time      2022-02-22T19:22:22.000Z\n",
        "",
    );
}

#[test]
fn info_uses_the_discord_epoch_for_snowflakes() {
    assert_run(
        &["info", "--epoch", "discord", SNOWFLAKE],
        b"",
        0,
        "175928847299117063\n  kind      snowflake\n  epoch     1420070400000\n  time      2016-04-30T11:18:25.796Z\n  worker    32\n  sequence  7\n",
        "",
    );
}

#[test]
fn info_continues_printing_valid_ids_after_invalid_ids() {
    assert_run(
        &["info", "abc", "80000000000000000000000000", ULID],
        b"",
        1,
        "01ARZ3NDEKTSV4RRFFQ69G5FAV\n  kind      ulid\n  time      2016-07-30T23:54:10.259Z\n  random    d6764c61efb99302bd5b\n",
        "idpeek: unknown ID format \"abc\"\nidpeek: invalid ULID \"80000000000000000000000000\"\n",
    );
}

#[test]
fn non_digit_text_of_snowflake_length_is_unknown_in_all_timed_commands() {
    assert_run(
        &["info", "abcdefghijklmno"],
        b"",
        1,
        "",
        "idpeek: unknown ID format \"abcdefghijklmno\"\n",
    );
    assert_run(
        &["span", "abcdefghijklmno", OBJECT_ID],
        b"",
        1,
        "",
        "idpeek: unknown ID format \"abcdefghijklmno\"\n",
    );
    assert_run(
        &["sort", "abcdefghijklmno"],
        b"",
        1,
        "",
        "idpeek: unknown ID format \"abcdefghijklmno\"\n",
    );
}

#[test]
fn info_reports_an_overflowing_ksuid_as_invalid_and_continues() {
    assert_run(
        &["info", "aWgEPTl1tmebfsQzFP4bxwgy80W", OBJECT_ID],
        b"",
        1,
        "507f1f77bcf86cd799439011\n  kind      objectid\n  time      2012-10-17T21:13:27.000Z\n  random    bcf86cd799\n  counter   4427793\n",
        "idpeek: invalid KSUID \"aWgEPTl1tmebfsQzFP4bxwgy80W\"\n",
    );
}

#[test]
fn info_preserves_lowercase_stdin_id_spelling_after_trimming() {
    assert_run(
        &["info", "-"],
        b"  01arz3ndektsv4rrffq69g5fav \r\n\r\n507f1f77bcf86cd799439011\n",
        0,
        "01arz3ndektsv4rrffq69g5fav\n  kind      ulid\n  time      2016-07-30T23:54:10.259Z\n  random    d6764c61efb99302bd5b\n507f1f77bcf86cd799439011\n  kind      objectid\n  time      2012-10-17T21:13:27.000Z\n  random    bcf86cd799\n  counter   4427793\n",
        "",
    );
}

#[test]
fn info_processes_a_final_unterminated_stdin_line() {
    assert_run(
        &["info", "-"],
        b"507f1f77bcf86cd799439011",
        0,
        "507f1f77bcf86cd799439011\n  kind      objectid\n  time      2012-10-17T21:13:27.000Z\n  random    bcf86cd799\n  counter   4427793\n",
        "",
    );
}

#[test]
fn repeated_stdin_operands_consume_input_only_once_in_order() {
    assert_run(
        &["info", "-", OBJECT_ID, "-"],
        b"01ARZ3NDEKTSV4RRFFQ69G5FAV\n",
        0,
        "01ARZ3NDEKTSV4RRFFQ69G5FAV\n  kind      ulid\n  time      2016-07-30T23:54:10.259Z\n  random    d6764c61efb99302bd5b\n507f1f77bcf86cd799439011\n  kind      objectid\n  time      2012-10-17T21:13:27.000Z\n  random    bcf86cd799\n  counter   4427793\n",
        "",
    );
}

#[test]
fn info_with_empty_stdin_expansion_is_a_success() {
    assert_run(&["info", "-"], b" \t\r\n\t\n", 0, "", "");
}

#[test]
fn info_rejects_invalid_utf8_stdin_without_partial_output() {
    assert_run(
        &["info", "-"],
        &[b'1', b'2', b'3', 0xff, b'\n'],
        1,
        "",
        "idpeek: standard input is not valid UTF-8\n",
    );
}

struct FailedReader;

impl Read for FailedReader {
    fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("test input failure"))
    }
}

#[test]
fn info_reports_a_stdin_read_failure_without_output() {
    let arguments = vec!["info".to_owned(), "-".to_owned()];
    let mut stdin = FailedReader;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = run(&arguments, &mut stdin, &mut stdout, &mut stderr);

    assert_eq!(code, 1);
    assert!(stdout.is_empty());
    assert_eq!(stderr, b"idpeek: failed to read standard input\n");
}

#[test]
fn span_matches_the_reference_earlier_output() {
    assert_run(
        &["span", ULID, OBJECT_ID],
        b"",
        0,
        "01ARZ3NDEKTSV4RRFFQ69G5FAV  2016-07-30T23:54:10.259Z\n507f1f77bcf86cd799439011    2012-10-17T21:13:27.000Z\nthe second is 1382d 02:40:43.259 earlier than the first\n",
        "",
    );
}

#[test]
fn span_matches_the_reference_later_output_when_swapped() {
    assert_run(
        &["span", OBJECT_ID, ULID],
        b"",
        0,
        "507f1f77bcf86cd799439011    2012-10-17T21:13:27.000Z\n01ARZ3NDEKTSV4RRFFQ69G5FAV  2016-07-30T23:54:10.259Z\nthe second is 1382d 02:40:43.259 later than the first\n",
        "",
    );
}

#[test]
fn span_reports_equal_times_without_a_duration() {
    let zero_uuid7 = "00000000-0000-7000-8000-000000000000";
    assert_run(
        &["span", zero_uuid7, zero_uuid7],
        b"",
        0,
        "00000000-0000-7000-8000-000000000000  1970-01-01T00:00:00.000Z\n00000000-0000-7000-8000-000000000000  1970-01-01T00:00:00.000Z\nthe second is at the same time as the first\n",
        "",
    );
}

#[test]
fn span_failure_for_the_first_id_is_atomic() {
    assert_run(
        &["span", "abc", OBJECT_ID],
        b"",
        1,
        "",
        "idpeek: unknown ID format \"abc\"\n",
    );
}

#[test]
fn span_reports_when_the_first_id_has_no_time() {
    let version_three = "5df41881-3aed-3515-88a7-2f4a814cf09e";
    assert_run(
        &["span", version_three, OBJECT_ID],
        b"",
        1,
        "",
        "idpeek: \"5df41881-3aed-3515-88a7-2f4a814cf09e\" has no time\n",
    );
}

#[test]
fn span_reports_when_the_second_id_has_no_time_without_partial_output() {
    let version_three = "5df41881-3aed-3515-88a7-2f4a814cf09e";
    assert_run(
        &["span", OBJECT_ID, version_three],
        b"",
        1,
        "",
        "idpeek: \"5df41881-3aed-3515-88a7-2f4a814cf09e\" has no time\n",
    );
}

#[test]
fn sort_matches_the_reference_ascending_order() {
    assert_run(
        &[
            "sort",
            OBJECT_ID,
            ULID,
            KSUID,
            "507f1f77bcf86cd799439012",
            UUID_V7,
        ],
        b"",
        0,
        "507f1f77bcf86cd799439011\n507f1f77bcf86cd799439012\n01ARZ3NDEKTSV4RRFFQ69G5FAV\n0ujtsYcgvSTl8PAuAdqWYSMnLOv\n017F22E2-79B0-7CC3-98C4-DC0C0C07398F\n",
        "",
    );
}

#[test]
fn sort_matches_the_reference_descending_order_and_keeps_ties_stable() {
    assert_run(
        &[
            "sort",
            "--desc",
            OBJECT_ID,
            ULID,
            KSUID,
            "507f1f77bcf86cd799439012",
            UUID_V7,
        ],
        b"",
        0,
        "017F22E2-79B0-7CC3-98C4-DC0C0C07398F\n0ujtsYcgvSTl8PAuAdqWYSMnLOv\n01ARZ3NDEKTSV4RRFFQ69G5FAV\n507f1f77bcf86cd799439011\n507f1f77bcf86cd799439012\n",
        "",
    );
}

#[test]
fn sort_failure_is_atomic_and_reports_the_first_bad_id() {
    assert_run(
        &["sort", ULID, "bad-first", "bad-second"],
        b"",
        1,
        "",
        "idpeek: unknown ID format \"bad-first\"\n",
    );
}

#[test]
fn sort_rejects_a_valid_uuid_without_time_atomically() {
    assert_run(
        &[
            "sort",
            OBJECT_ID,
            "5df41881-3aed-3515-88a7-2f4a814cf09e",
            ULID,
        ],
        b"",
        1,
        "",
        "idpeek: \"5df41881-3aed-3515-88a7-2f4a814cf09e\" has no time\n",
    );
}

#[test]
fn sort_accepts_an_empty_stdin_expansion() {
    assert_run(&["sort", "-"], b"", 0, "", "");
}

#[test]
fn sort_expands_stdin_between_command_line_operands() {
    assert_run(
        &["sort", OBJECT_ID, "-", ULID],
        b"0ujtsYcgvSTl8PAuAdqWYSMnLOv\n",
        0,
        "507f1f77bcf86cd799439011\n01ARZ3NDEKTSV4RRFFQ69G5FAV\n0ujtsYcgvSTl8PAuAdqWYSMnLOv\n",
        "",
    );
}

#[test]
fn bound_matches_the_ulid_minimum_reference() {
    assert_run(
        &["bound", "ulid", "2024-03-01"],
        b"",
        0,
        "01HQVMZ1000000000000000000\n",
        "",
    );
}

#[test]
fn bound_matches_the_ulid_maximum_fractional_reference() {
    assert_run(
        &["bound", "--max", "ulid", "2024-03-01T12:30:00.5Z"],
        b"",
        0,
        "01HQWZWASMZZZZZZZZZZZZZZZZ\n",
        "",
    );
}

#[test]
fn bound_matches_the_objectid_reference() {
    assert_run(
        &["bound", "objectid", "2024-03-01"],
        b"",
        0,
        "65e11a800000000000000000\n",
        "",
    );
}

#[test]
fn bound_matches_the_uuid7_minimum_reference() {
    assert_run(
        &["bound", "uuid7", "2024-03-01"],
        b"",
        0,
        "018df74f-8400-7000-8000-000000000000\n",
        "",
    );
}

#[test]
fn bound_matches_the_uuid7_maximum_reference() {
    assert_run(
        &["bound", "--max", "uuid7", "2024-03-01"],
        b"",
        0,
        "018df74f-8400-7fff-bfff-ffffffffffff\n",
        "",
    );
}

#[test]
fn bound_matches_the_ksuid_reference() {
    assert_run(
        &["bound", "ksuid", "2024-03-01"],
        b"",
        0,
        "2d3ykafzYwGnkMqnex3Bf5k5Uye\n",
        "",
    );
}

#[test]
fn bound_matches_the_snowflake_default_epoch_reference() {
    assert_run(
        &["bound", "snowflake", "2024-03-01"],
        b"",
        0,
        "1763353455621046272\n",
        "",
    );
}

#[test]
fn bound_matches_the_snowflake_discord_epoch_reference() {
    assert_run(
        &["bound", "--epoch", "discord", "snowflake", "2024-03-01"],
        b"",
        0,
        "1212912186163200000\n",
        "",
    );
}

#[test]
fn bound_maximum_sets_the_snowflake_low_twenty_two_bits() {
    assert_run(
        &[
            "bound",
            "--epoch",
            "unix",
            "--max",
            "snowflake",
            "1970-01-01",
        ],
        b"",
        0,
        "4194303\n",
        "",
    );
}

#[test]
fn bound_floors_subsecond_time_for_objectid_and_ksuid() {
    assert_run(
        &["bound", "objectid", "2024-03-01T00:00:00.999Z"],
        b"",
        0,
        "65e11a800000000000000000\n",
        "",
    );
    assert_run(
        &["bound", "ksuid", "2014-05-13T16:53:20.999Z"],
        b"",
        0,
        "000000000000000000000000000\n",
        "",
    );
}

#[test]
fn bound_reports_out_of_range_as_runtime_failure() {
    assert_run(
        &["bound", "ulid", "1969-12-31"],
        b"",
        1,
        "",
        "idpeek: time is out of range for ulid\n",
    );
}

#[test]
fn bound_distinguishes_invalid_calendar_date_from_range_failure() {
    assert_run(
        &["bound", "ulid", "2024-02-30"],
        b"",
        2,
        "",
        "idpeek: invalid TIME \"2024-02-30\"\n",
    );
}

struct FailedWriter;

impl Write for FailedWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("test output failure"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn a_stdout_write_failure_is_reported_on_stderr() {
    let arguments = vec!["info".to_owned(), ULID.to_owned()];
    let mut stdin = Cursor::new(b"");
    let mut stdout = FailedWriter;
    let mut stderr = Vec::new();

    let code = run(&arguments, &mut stdin, &mut stdout, &mut stderr);

    assert_eq!(code, 1);
    assert_eq!(stderr, b"idpeek: failed to write standard output\n");
}

fn run_binary_with_stdin(args: &[&str], input: &[u8]) -> (ExitStatus, Vec<u8>, Vec<u8>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_idpeek"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the idpeek test binary should start");

    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(input)
        .expect("the test input should be written");

    let started = Instant::now();
    let status = wait_with_timeout(&mut child, started);

    let mut stdout = Vec::new();
    child
        .stdout
        .take()
        .expect("stdout should be piped")
        .read_to_end(&mut stdout)
        .expect("stdout should be readable");
    let mut stderr = Vec::new();
    child
        .stderr
        .take()
        .expect("stderr should be piped")
        .read_to_end(&mut stderr)
        .expect("stderr should be readable");

    (status, stdout, stderr)
}

fn wait_with_timeout(child: &mut Child, started: Instant) -> ExitStatus {
    let timeout = Duration::from_secs(30);
    loop {
        if let Some(status) = child
            .try_wait()
            .expect("the idpeek process should be waitable")
        {
            return status;
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            panic!("the idpeek process exceeded its 30-second timeout");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn real_process_reads_literal_crlf_stdin_and_prints_lf_reports() {
    let (status, stdout, stderr) = run_binary_with_stdin(
        &["info", "-"],
        b"  01arz3ndektsv4rrffq69g5fav \r\n\r\n507f1f77bcf86cd799439011\n",
    );

    assert_eq!(status.code(), Some(0));
    assert_eq!(
        stdout,
        b"01arz3ndektsv4rrffq69g5fav\n  kind      ulid\n  time      2016-07-30T23:54:10.259Z\n  random    d6764c61efb99302bd5b\n507f1f77bcf86cd799439011\n  kind      objectid\n  time      2012-10-17T21:13:27.000Z\n  random    bcf86cd799\n  counter   4427793\n"
    );
    assert_eq!(stderr, b"");
}

#[test]
fn real_process_preserves_carriage_return_on_final_unterminated_stdin_line() {
    let (status, stdout, stderr) =
        run_binary_with_stdin(&["info", "-"], b"01ARZ3NDEKTSV4RRFFQ69G5FAV\r");

    assert_eq!(status.code(), Some(1));
    assert_eq!(stdout, b"");
    assert_eq!(
        stderr,
        b"idpeek: invalid KSUID \"01ARZ3NDEKTSV4RRFFQ69G5FAV?\"\n"
    );
}

#[test]
fn real_process_rejects_invalid_utf8_bytes_written_to_stdin() {
    let (status, stdout, stderr) =
        run_binary_with_stdin(&["info", "-"], &[b'1', b'2', b'3', 0xff, b'\n']);

    assert_eq!(status.code(), Some(1));
    assert_eq!(stdout, b"");
    assert_eq!(stderr, b"idpeek: standard input is not valid UTF-8\n");
}
