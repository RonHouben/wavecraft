# Implementation Progress: CLI Dev Server Reuse

## Todo
- [x] Create shared in-memory parameter host in `wavecraft-bridge`.
- [x] Export shared host from `wavecraft-bridge`.
- [x] Create shared plugin param loader utility (FFI loader).
- [x] Export loader from shared crate.
- [x] Refactor CLI `DevServerHost` to wrap shared host.
- [x] Refactor CLI loader usage to shared loader.
- [x] Use shared `db_to_linear` (or shared meter utility).
- [x] Add/update tests for shared host and loader.
- [x] Update CLI tests as needed.

## Optional
- [x] Unify `MeterFrame` types between protocol and metering crates.
- [x] Move `MeterGenerator` to shared dev module.
- [x] Align standalone host with shared host.
