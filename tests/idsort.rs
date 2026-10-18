use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn sorts_id_arguments_in_ascending_creation_order() {
    let output = Command::new(env!("CARGO_BIN_EXE_idsort"))
        .args([
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            "507f1f77bcf86cd799439011",
            "0ujtsYcgvSTl8PAuAdqWYSMnLOv",
        ])
        .output()
        .expect("the idsort executable should start");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        output.stdout,
        b"507f1f77bcf86cd799439011\n01ARZ3NDEKTSV4RRFFQ69G5FAV\n0ujtsYcgvSTl8PAuAdqWYSMnLOv\n"
    );
    assert_eq!(output.stderr, b"");
}

#[test]
fn sorts_id_arguments_in_descending_creation_order() {
    let output = Command::new(env!("CARGO_BIN_EXE_idsort"))
        .args([
            "--desc",
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            "507f1f77bcf86cd799439011",
            "0ujtsYcgvSTl8PAuAdqWYSMnLOv",
        ])
        .output()
        .expect("the idsort executable should start");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        output.stdout,
        b"0ujtsYcgvSTl8PAuAdqWYSMnLOv\n01ARZ3NDEKTSV4RRFFQ69G5FAV\n507f1f77bcf86cd799439011\n"
    );
    assert_eq!(output.stderr, b"");
}

#[test]
fn sorts_ids_from_real_stdin_with_crlf_and_blank_lines() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_idsort"))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the idsort executable should start");

    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(
            b"01ARZ3NDEKTSV4RRFFQ69G5FAV\r\n\r\n507f1f77bcf86cd799439011\r\n0ujtsYcgvSTl8PAuAdqWYSMnLOv\r\n",
        )
        .expect("the test input should be written");
    let output = child
        .wait_with_output()
        .expect("the idsort process should finish");

    assert_eq!(output.status.code(), Some(0), "idsort - stdin invocation");
    assert_eq!(
        output.stdout,
        b"507f1f77bcf86cd799439011\n01ARZ3NDEKTSV4RRFFQ69G5FAV\n0ujtsYcgvSTl8PAuAdqWYSMnLOv\n"
    );
    assert_eq!(output.stderr, b"");
}

#[test]
fn invalid_id_reports_the_exact_diagnostic() {
    let output = Command::new(env!("CARGO_BIN_EXE_idsort"))
        .args(["01ARZ3NDEKTSV4RRFFQ69G5FAV", "nope"])
        .output()
        .expect("the idsort executable should start");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"idpeek: unknown ID format \"nope\"\n");
}

#[test]
fn missing_id_and_help_report_usage_errors() {
    let missing = Command::new(env!("CARGO_BIN_EXE_idsort"))
        .output()
        .expect("the idsort executable should start");

    assert_eq!(missing.status.code(), Some(2));
    assert_eq!(missing.stdout, b"");
    assert_eq!(missing.stderr, b"idpeek: missing ID\n");

    let help = Command::new(env!("CARGO_BIN_EXE_idsort"))
        .arg("--help")
        .output()
        .expect("the idsort executable should start");

    assert_eq!(help.status.code(), Some(2));
    assert_eq!(help.stdout, b"");
    assert_eq!(help.stderr, b"idpeek: unknown option \"--help\"\n");
}
