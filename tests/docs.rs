use std::fs;

fn readme() -> String {
    fs::read_to_string("README.md")
        .expect("README.md should be available from the crate directory")
        .replace("\r\n", "\n")
}

fn fenced_text_blocks(document: &str) -> Vec<&str> {
    document
        .split("```text\n")
        .skip(1)
        .map(|block| {
            block
                .split_once("\n```")
                .expect("every text block should have a closing fence")
                .0
        })
        .collect()
}

#[test]
fn includes_all_six_required_sections() {
    let document = readme();

    assert!(document.contains("## Install\n"));
    assert!(document.contains("## Usage\n"));
    assert!(document.contains("## Identifiers\n"));
    assert!(document.contains("## Bounds\n"));
    assert!(document.contains("## Exit codes\n"));
    assert!(document.contains("## Development\n"));
}

#[test]
fn documents_the_required_install_and_development_commands() {
    let document = readme();

    assert!(document.contains("`cargo build --release`"));
    assert!(document.contains("`cargo run -- ARGS`"));
    assert!(document.contains("cargo fmt --check\n"));
    assert!(document.contains("cargo clippy -- -D warnings\n"));
    assert!(document.contains("cargo test\n"));
}

#[test]
fn usage_block_matches_the_four_reference_lines() {
    let document = readme();

    assert!(document.contains(
        "```text\nusage: idpeek info [--epoch EPOCH] ID...\n       idpeek span [--epoch EPOCH] ID1 ID2\n       idpeek sort [--epoch EPOCH] [--desc] ID...\n       idpeek bound [--epoch EPOCH] [--max] KIND TIME\n```"
    ));
}

#[test]
fn info_reference_for_ulid_ksuid_and_objectid_is_verbatim() {
    let document = readme();

    assert!(document.contains(
        "`idpeek info 01ARZ3NDEKTSV4RRFFQ69G5FAV 0ujtsYcgvSTl8PAuAdqWYSMnLOv 507f1f77bcf86cd799439011` prints:\n\n```text\n01ARZ3NDEKTSV4RRFFQ69G5FAV\n  kind      ulid\n  time      2016-07-30T23:54:10.259Z\n  random    d6764c61efb99302bd5b\n0ujtsYcgvSTl8PAuAdqWYSMnLOv\n  kind      ksuid\n  time      2017-10-10T04:00:47.000Z\n  payload   b5a1cd34b5f99d1154fb6853345c9735\n507f1f77bcf86cd799439011\n  kind      objectid\n  time      2012-10-17T21:13:27.000Z\n  random    bcf86cd799\n  counter   4427793\n```"
    ));
}

#[test]
fn info_reference_for_uuid_v1_and_v7_is_verbatim() {
    let document = readme();

    assert!(document.contains(
        "`idpeek info C232AB00-9414-11EC-B3C8-9F6BDECED846 017F22E2-79B0-7CC3-98C4-DC0C0C07398F` prints:\n\n```text\nC232AB00-9414-11EC-B3C8-9F6BDECED846\n  kind      uuid\n  version   1 (time and node)\n  variant   rfc4122\n  time      2022-02-22T19:22:22.000Z\n  clock     13256\n  node      9f6bdeced846\n017F22E2-79B0-7CC3-98C4-DC0C0C07398F\n  kind      uuid\n  version   7 (unix time and random)\n  variant   rfc4122\n  time      2022-02-22T19:22:22.000Z\n```"
    ));
}

#[test]
fn discord_snowflake_reference_and_default_epoch_time_are_verbatim() {
    let document = readme();

    assert!(document.contains(
        "`idpeek info --epoch discord 175928847299117063` prints:\n\n```text\n175928847299117063\n  kind      snowflake\n  epoch     1420070400000\n  time      2016-04-30T11:18:25.796Z\n  worker    32\n  sequence  7\n```"
    ));
    assert!(document.contains(
        "With the default epoch, the same Snowflake ID has time `2012-03-03T13:01:20.453Z`."
    ));
}

#[test]
fn span_reference_and_swapped_result_are_verbatim() {
    let document = readme();

    assert!(document.contains(
        "`idpeek span 01ARZ3NDEKTSV4RRFFQ69G5FAV 507f1f77bcf86cd799439011` prints:\n\n```text\n01ARZ3NDEKTSV4RRFFQ69G5FAV  2016-07-30T23:54:10.259Z\n507f1f77bcf86cd799439011    2012-10-17T21:13:27.000Z\nthe second is 1382d 02:40:43.259 earlier than the first\n```"
    ));
    assert!(document.contains(
        "When the two span operands are swapped, the final line is:\n\n```text\nthe second is 1382d 02:40:43.259 later than the first\n```"
    ));
}

#[test]
fn ascending_sort_reference_is_verbatim() {
    let document = readme();

    assert!(document.contains(
        "`idpeek sort 507f1f77bcf86cd799439011 01ARZ3NDEKTSV4RRFFQ69G5FAV 0ujtsYcgvSTl8PAuAdqWYSMnLOv 507f1f77bcf86cd799439012 017F22E2-79B0-7CC3-98C4-DC0C0C07398F` prints in ascending order:\n\n```text\n507f1f77bcf86cd799439011\n507f1f77bcf86cd799439012\n01ARZ3NDEKTSV4RRFFQ69G5FAV\n0ujtsYcgvSTl8PAuAdqWYSMnLOv\n017F22E2-79B0-7CC3-98C4-DC0C0C07398F\n```"
    ));
}

#[test]
fn descending_sort_reference_is_verbatim_and_keeps_ties_in_order() {
    let document = readme();

    assert!(document.contains(
        "With `--desc`, the same IDs print in descending order, with equal-time IDs retaining their order:\n\n```text\n017F22E2-79B0-7CC3-98C4-DC0C0C07398F\n0ujtsYcgvSTl8PAuAdqWYSMnLOv\n01ARZ3NDEKTSV4RRFFQ69G5FAV\n507f1f77bcf86cd799439011\n507f1f77bcf86cd799439012\n```"
    ));
}

#[test]
fn all_eight_bound_invocations_and_results_are_verbatim() {
    let document = readme();

    assert!(document.contains("- `idpeek bound ulid 2024-03-01` → `01HQVMZ1000000000000000000`"));
    assert!(document.contains(
        "- `idpeek bound --max ulid 2024-03-01T12:30:00.5Z` → `01HQWZWASMZZZZZZZZZZZZZZZZ`"
    ));
    assert!(document.contains("- `idpeek bound objectid 2024-03-01` → `65e11a800000000000000000`"));
    assert!(
        document
            .contains("- `idpeek bound uuid7 2024-03-01` → `018df74f-8400-7000-8000-000000000000`")
    );
    assert!(document.contains(
        "- `idpeek bound --max uuid7 2024-03-01` → `018df74f-8400-7fff-bfff-ffffffffffff`"
    ));
    assert!(document.contains("- `idpeek bound ksuid 2024-03-01` → `2d3ykafzYwGnkMqnex3Bf5k5Uye`"));
    assert!(document.contains("- `idpeek bound snowflake 2024-03-01` → `1763353455621046272`"));
    assert!(
        document.contains(
            "- `idpeek bound --epoch discord snowflake 2024-03-01` → `1212912186163200000`"
        )
    );
}

#[test]
fn fenced_outputs_are_only_the_supplied_usage_and_reference_examples() {
    let document = readme();

    let expected = vec![
        "usage: idpeek info [--epoch EPOCH] ID...\n       idpeek span [--epoch EPOCH] ID1 ID2\n       idpeek sort [--epoch EPOCH] [--desc] ID...\n       idpeek bound [--epoch EPOCH] [--max] KIND TIME",
        "01ARZ3NDEKTSV4RRFFQ69G5FAV\n  kind      ulid\n  time      2016-07-30T23:54:10.259Z\n  random    d6764c61efb99302bd5b\n0ujtsYcgvSTl8PAuAdqWYSMnLOv\n  kind      ksuid\n  time      2017-10-10T04:00:47.000Z\n  payload   b5a1cd34b5f99d1154fb6853345c9735\n507f1f77bcf86cd799439011\n  kind      objectid\n  time      2012-10-17T21:13:27.000Z\n  random    bcf86cd799\n  counter   4427793",
        "C232AB00-9414-11EC-B3C8-9F6BDECED846\n  kind      uuid\n  version   1 (time and node)\n  variant   rfc4122\n  time      2022-02-22T19:22:22.000Z\n  clock     13256\n  node      9f6bdeced846\n017F22E2-79B0-7CC3-98C4-DC0C0C07398F\n  kind      uuid\n  version   7 (unix time and random)\n  variant   rfc4122\n  time      2022-02-22T19:22:22.000Z",
        "175928847299117063\n  kind      snowflake\n  epoch     1420070400000\n  time      2016-04-30T11:18:25.796Z\n  worker    32\n  sequence  7",
        "01ARZ3NDEKTSV4RRFFQ69G5FAV  2016-07-30T23:54:10.259Z\n507f1f77bcf86cd799439011    2012-10-17T21:13:27.000Z\nthe second is 1382d 02:40:43.259 earlier than the first",
        "the second is 1382d 02:40:43.259 later than the first",
        "507f1f77bcf86cd799439011\n507f1f77bcf86cd799439012\n01ARZ3NDEKTSV4RRFFQ69G5FAV\n0ujtsYcgvSTl8PAuAdqWYSMnLOv\n017F22E2-79B0-7CC3-98C4-DC0C0C07398F",
        "017F22E2-79B0-7CC3-98C4-DC0C0C07398F\n0ujtsYcgvSTl8PAuAdqWYSMnLOv\n01ARZ3NDEKTSV4RRFFQ69G5FAV\n507f1f77bcf86cd799439011\n507f1f77bcf86cd799439012",
        "cargo fmt --check\ncargo clippy -- -D warnings\ncargo test",
    ];

    assert_eq!(fenced_text_blocks(&document), expected);
}
