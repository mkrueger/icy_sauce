# Changelog

Notable changes to icy_sauce are documented here.

## 0.4.0 — 2026-09-06

Release versions: library `icy_sauce` **0.4.0**, CLI `icy_sauce_cli` **0.4.0**
(executable: `sauce`). Both packages use the same release version. The CLI advances
from 0.1.0 to align with the library, including the JSON, editing, and
failure-reporting changes described below.

### Breaking changes

- `SauceRecord::to_bytes()` and `to_bytes_without_eof()` now return
  `Result<Vec<u8>>`. Callers must handle serialization errors.
- Validated font names are limited to 21 bytes, reserving the final TInfoS byte
  for a NUL terminator. Embedded NUL bytes are rejected. Raw TInfoS APIs still
  support all 22 bytes for lossless handling of existing records.
- `SauceRecordBuilder::metadata()` now replaces comments as well as title,
  author, and group. An empty comment list clears existing comments.

### Fixed

- Validate date field widths before serialization instead of silently truncating
  values. Invalid dates are rejected before any record bytes are written;
  calendar-date validation remains intentionally lenient.
- Ignore the lazy capabilities cache when comparing records for equality.
- Normalize the comment count when lenient parsing ignores a missing COMNT block.
- Validate advertised comment blocks before stripping records. Invalid or
  truncated blocks are preserved instead of deleting unrelated payload bytes.
- Decode font names only up to the first NUL while preserving the underlying
  raw TInfoS bytes, including legacy unterminated fields.
- Apply consistent comment validation and replacement through
  `MetaData::to_builder()` and `SauceRecordBuilder::metadata()`.
- Make `SauceRecordBuilder::capabilities()` consistently select the matching data
  type, including when switching an existing record to Character capabilities.

### CLI

- Resolve and pin the input target before reading, so a retargeted symlink cannot
  redirect the subsequent write to an unrelated file. Check file identity and
  contents before replacement and reject detected concurrent changes.
- Preserve unspecified metadata and raw header bytes when altering records.
- Resolve command-line overrides before validating imported JSON values.
- Distinguish omitted JSON comments from an explicit empty list; retain explicit
  replacement, append, and clear behavior.
- Export unknown dates and empty TInfoS fields explicitly so reimporting a
  snapshot clears those fields. Omitted fields still preserve existing values
  in partial JSON updates.
- Calculate payload file size when adding metadata unless explicitly overridden.
- Reject unsafe alterations, forced replacements, and removals when the trailing
  record advertises an invalid or truncated comment block.
- Report partial removal with exit status 1 when `remove --all` leaves a SAUCE
  header at the resulting tail. Valid trailing records are still removed; the
  remaining header and preceding bytes are preserved.
- Replace files atomically using a temporary file in the same directory while
  preserving permission bits and following symlink targets. Hard-link updates,
  ownership, ACLs, and extended attributes are not preserved.
- Sync the parent directory after atomic replacement on Unix. Directory sync
  failures explicitly report that replacement completed but crash durability is
  uncertain, rather than implying the original file remains unchanged.
- Escape terminal control and bidirectional formatting characters in
  human-readable metadata, paths, and errors without changing JSON export values.
- Read only the bounded trailing metadata window when viewing files.
- Parse date arguments safely without slicing through UTF-8 characters.
- Correct BinaryText help: width is `FileType * 2`, with nonzero even widths up
  to 510 columns.

### Added

- `SauceRecordBuilder::clear_comments()`.
- Regression and property tests covering serialization, metadata preservation,
  malformed comment blocks, fonts, terminal output, and CLI file replacement.
- A stripping fuzz target and serialization round-trip assertions in the date
  and header fuzz targets.

### Maintenance

- Run CI for pushes and pull requests targeting `main`, including strict Clippy
  and Rustdoc checks, default/all-feature tests, and a Rust 1.89 MSRV test job.
- Declare Rust 1.89 as the minimum supported version and include the README in
  both published packages.
- Repair the standalone fuzz workspace and its library dependency path.
- Add a version requirement to the CLI's library dependency so both crates can
  be packaged for publication.
- Resolve Clippy warnings and broken Rustdoc links, correct stale capability
  examples, and convert ignored documentation examples into runnable tests.
- Document migration steps, byte-preservation behavior, safe stripping,
  atomic-write limitations, validation, fuzzing, and publication order in the
  [README](README.md).