# Implementation Progress: CLI Dev Server Reuse

## Todo
- [ ] Create shared in-memory parameter host in `wavecraft-bridge`.
- [ ] Export shared host from `wavecraft-bridge`.
- [ ] Create shared plugin param loader utility (FFI loader).
- [ ] Export loader from shared crate.
- [ ] Refactor CLI `DevServerHost` to wrap shared host.
- [ ] Refactor CLI loader usage to shared loader.
- [ ] Use shared `db_to_linear` (or shared meter utility).
- [ ] Add/update tests for shared host and loader.
- [ ] Update CLI tests as needed.

## Optional
- [ ] Unify `MeterFrame` types between protocol and metering crates.
- [ ] Move `MeterGenerator` to shared dev module.
- [ ] Align standalone host with shared host.
