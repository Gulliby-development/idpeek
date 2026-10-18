<p align="center"><img src="docs/logo.svg" width="96" alt="" /></p>

# idpeek

[![CI](https://github.com/Gulliby-development/idpeek/actions/workflows/ci.yml/badge.svg)](https://github.com/Gulliby-development/idpeek/actions/workflows/ci.yml) [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

idpeek is an offline command-line tool for people reading logs and databases. It reveals timestamps and embedded fields in UUIDs, ULIDs, KSUIDs, MongoDB ObjectIds and Snowflake IDs, compares and sorts timed identifiers, and creates time bounds for range queries.

## Install

Build the release executable with:

`cargo build --release`

Run the CLI through Cargo with:

`cargo run -- ARGS`

Replace `ARGS` with the desired command and its arguments.

The release build also contains a second executable, `idsort`: `idsort ARGS` does the same as `idpeek sort ARGS`, so a pipeline can end with `idsort -` to put the IDs it reads in creation order.

## Usage

The four commands are `info` to inspect IDs, `span` to compare two creation times, `sort` to order IDs by creation time and `bound` to make a minimum or maximum ID for a time.

The command forms are:

```text
usage: idpeek info [--epoch EPOCH] ID...
       idpeek span [--epoch EPOCH] ID1 ID2
       idpeek sort [--epoch EPOCH] [--desc] ID...
       idpeek bound [--epoch EPOCH] [--max] KIND TIME
```

The following reference invocations and output show the decoded fields:

`idpeek info 01ARZ3NDEKTSV4RRFFQ69G5FAV 0ujtsYcgvSTl8PAuAdqWYSMnLOv 507f1f77bcf86cd799439011` prints:

```text
01ARZ3NDEKTSV4RRFFQ69G5FAV
  kind      ulid
  time      2016-07-30T23:54:10.259Z
  random    d6764c61efb99302bd5b
0ujtsYcgvSTl8PAuAdqWYSMnLOv
  kind      ksuid
  time      2017-10-10T04:00:47.000Z
  payload   b5a1cd34b5f99d1154fb6853345c9735
507f1f77bcf86cd799439011
  kind      objectid
  time      2012-10-17T21:13:27.000Z
  random    bcf86cd799
  counter   4427793
```

`idpeek info C232AB00-9414-11EC-B3C8-9F6BDECED846 017F22E2-79B0-7CC3-98C4-DC0C0C07398F` prints:

```text
C232AB00-9414-11EC-B3C8-9F6BDECED846
  kind      uuid
  version   1 (time and node)
  variant   rfc4122
  time      2022-02-22T19:22:22.000Z
  clock     13256
  node      9f6bdeced846
017F22E2-79B0-7CC3-98C4-DC0C0C07398F
  kind      uuid
  version   7 (unix time and random)
  variant   rfc4122
  time      2022-02-22T19:22:22.000Z
```

`idpeek info --epoch discord 175928847299117063` prints:

```text
175928847299117063
  kind      snowflake
  epoch     1420070400000
  time      2016-04-30T11:18:25.796Z
  worker    32
  sequence  7
```

With the default epoch, the same Snowflake ID has time `2012-03-03T13:01:20.453Z`.

`idpeek span 01ARZ3NDEKTSV4RRFFQ69G5FAV 507f1f77bcf86cd799439011` prints:

```text
01ARZ3NDEKTSV4RRFFQ69G5FAV  2016-07-30T23:54:10.259Z
507f1f77bcf86cd799439011    2012-10-17T21:13:27.000Z
the second is 1382d 02:40:43.259 earlier than the first
```

When the two span operands are swapped, the final line is:

```text
the second is 1382d 02:40:43.259 later than the first
```

`idpeek sort 507f1f77bcf86cd799439011 01ARZ3NDEKTSV4RRFFQ69G5FAV 0ujtsYcgvSTl8PAuAdqWYSMnLOv 507f1f77bcf86cd799439012 017F22E2-79B0-7CC3-98C4-DC0C0C07398F` prints in ascending order:

```text
507f1f77bcf86cd799439011
507f1f77bcf86cd799439012
01ARZ3NDEKTSV4RRFFQ69G5FAV
0ujtsYcgvSTl8PAuAdqWYSMnLOv
017F22E2-79B0-7CC3-98C4-DC0C0C07398F
```

With `--desc`, the same IDs print in descending order, with equal-time IDs retaining their order:

```text
017F22E2-79B0-7CC3-98C4-DC0C0C07398F
0ujtsYcgvSTl8PAuAdqWYSMnLOv
01ARZ3NDEKTSV4RRFFQ69G5FAV
507f1f77bcf86cd799439011
507f1f77bcf86cd799439012
```

## Identifiers

Identifiers are recognized by their shape. UUIDs may be canonical or compact; their version and variant are reported, and versions 1, 6 and 7 carry a decoded time. ULIDs use Crockford base32 and include a timestamp and random field. KSUIDs use base62 and include a timestamp and payload. ObjectIds contain a Unix-seconds timestamp, random bytes and a counter. Snowflake IDs contain a timestamp, worker and sequence.

For Snowflake decoding, the default epoch is Twitter's. Use `--epoch discord`, `--epoch unix` or `--epoch EPOCH` to select Discord, Unix or a numeric millisecond epoch. In `info` and `sort`, a `-` operand reads IDs from standard input, one per line.

## Bounds

Use `bound KIND TIME` to construct the smallest identifier for a time, or add `--max` for the largest identifier at that time. Supported kinds are `ulid`, `objectid`, `ksuid`, `uuid7` and `snowflake`. TIME is `YYYY-MM-DD` or `YYYY-MM-DDTHH:MM:SS[.f]Z`, in UTC, with one to three fractional-second digits. ObjectId and KSUID bounds round down to the containing second. Snowflake bounds use the selected epoch.

Reference bound invocations and their one-line results:

- `idpeek bound ulid 2024-03-01` → `01HQVMZ1000000000000000000`
- `idpeek bound --max ulid 2024-03-01T12:30:00.5Z` → `01HQWZWASMZZZZZZZZZZZZZZZZ`
- `idpeek bound objectid 2024-03-01` → `65e11a800000000000000000`
- `idpeek bound uuid7 2024-03-01` → `018df74f-8400-7000-8000-000000000000`
- `idpeek bound --max uuid7 2024-03-01` → `018df74f-8400-7fff-bfff-ffffffffffff`
- `idpeek bound ksuid 2024-03-01` → `2d3ykafzYwGnkMqnex3Bf5k5Uye`
- `idpeek bound snowflake 2024-03-01` → `1763353455621046272`
- `idpeek bound --epoch discord snowflake 2024-03-01` → `1212912186163200000`

## Exit codes

The exit code is `0` when a command succeeds, `1` when an identifier or requested time cannot be used, and `2` for a usage error. Usage errors are reported on standard error; they do not produce standard output. The `info` command reports invalid IDs and continues with the remaining IDs.

## Development

The project uses only the Rust standard library. Run the formatting, lint and test checks with:

```text
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## License

MIT, see [LICENSE](LICENSE).

---

Built with BAKED
