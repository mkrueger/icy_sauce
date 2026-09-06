# icy_sauce

A Rust library for reading and writing SAUCE (Standard Architecture for Universal Comment Extensions) metadata records. SAUCE is a metadata protocol widely used in the ANSI art and BBS scenes to embed information about artwork files.

This workspace contains the `icy_sauce` library and the `icy_sauce_cli` package,
which provides the `sauce` executable. These tools handle metadata, not the decoding
or rendering of artwork, images, or audio.

**Upcoming release:** library and CLI **0.4.0** (not yet published).
Both packages use the same release version, even when only one has major changes.
See [CHANGELOG.md](CHANGELOG.md) for release notes and
[Migrating to 0.4](#migrating-to-04) for library API changes.

## What is SAUCE?

SAUCE is a metadata format created in 1994 by ACiD Productions to standardize how information about digital artwork and other files is stored. The SAUCE record is appended to the end of files and contains:

- Title, Author, and Group information
- Creation date
- File type specifications
- Format-specific metadata (dimensions, fonts, etc.)
- Comments

## Features

- **SAUCE v00 Metadata Support**: Reads and writes headers and comments using the revision 5 field definitions
- **Multiple Format Support**:
  - Character formats (ANSI, ASCII, PCBoard, Avatar, RipScript, etc.)
  - Binary text formats (BinaryText, XBin)
  - Graphics formats (GIF, PNG, JPG, PCX, etc.)
  - Audio formats (MOD, S3M, XM, IT, etc.)
  - Archive formats (ZIP, ARJ, RAR, etc.)
  - Vector formats (DXF, DWG, WPG)
- **Type-Safe API**: Strongly typed capabilities for each format type
- **Builder Pattern**: Convenient builder for creating SAUCE records
- **Comment Support**: Read and write up to 255 comments per record
- **Byte-Preserving Metadata**: Uses `bstr` to retain CP437 and other legacy bytes without implicit UTF-8 conversion

## Installation

Requires Rust 1.89 or newer. CI tests both the minimum supported Rust version
and the current stable toolchain.

Add this to your `Cargo.toml`:

```toml
[dependencies]
icy_sauce = "0.4.0"
bstr = "1.12" # Used explicitly in the examples below
```

The version above is the upcoming release. Until it is published, use a checkout
of this repository and a path dependency on `crates/icy_sauce`.

### Optional Features

No optional features are enabled by default.

| Feature | Adds |
|---------|------|
| `chrono` | Conversions between `SauceDate` and `chrono::NaiveDate`, including calendar validation |
| `serde` | `Serialize` and `Deserialize` for `MetaData` (title, author, group, comments), not the full `SauceRecord` |

Enable features with `icy_sauce = { version = "0.4.0", features = ["chrono", "serde"] }`.
The CLI's JSON format is independent of the library's optional `serde` feature.

## Basic Usage

### Reading SAUCE

```rust,no_run
use icy_sauce::prelude::*; // brings common types into scope
use bstr::ByteSlice;
use std::path::Path;

fn main() -> icy_sauce::Result<()> {
    // Reads a bounded trailing window rather than loading the entire file.
    if let Some(sauce) = SauceRecord::from_path(Path::new("artwork.ans"))? {
        println!("Title: {}", sauce.title());
        println!("Author: {}", sauce.author());
        println!("Group: {}", sauce.group());

        // Get format-specific information
        if let Some(caps) = sauce.capabilities() {
            match caps {
                Capabilities::Character(c) => {
                    println!("Character format: {:?} ({}x{})", c.format, c.columns, c.lines);
                }
                Capabilities::Bitmap(b) => {
                    println!("Bitmap: {:?} ({}x{} @ {}bpp)", b.format, b.width, b.height, b.pixel_depth);
                }
                Capabilities::Binary(b) => {
                    match b.format {
                        BinaryFormat::BinaryText => {
                            println!("BinaryText width: {}", b.columns);
                            if let Some(h) = b.binary_text_height_from_file_size(sauce.file_size()) {
                                println!("Derived height: {}", h);
                            }
                            println!("ICE colors: {}", b.ice_colors);
                            println!("Letter spacing: {:?}", b.letter_spacing);
                            println!("Aspect ratio: {:?}", b.aspect_ratio);
                            if let Some(font) = b.font() {
                                println!("Font: {}", font.to_str_lossy());
                            }
                        }
                        BinaryFormat::XBin => {
                            println!("XBin dimensions: {}x{}", b.columns, b.lines);
                        }
                    }
                }
                Capabilities::Vector(v) => {
                    println!("Vector: {:?}", v.format);
                }
                _ => {}
            }
        }
    }

    Ok(())
}
```

For an existing in-memory file buffer, use `SauceRecord::from_bytes(&data)`.
Both readers return `Ok(None)` when no trailing SAUCE record is found and `Err`
for unsupported versions or structural errors. Unknown data/file type values can
be retained in the raw header even when no typed capabilities are available.

### Writing SAUCE

```rust
use icy_sauce::prelude::*;
use bstr::BString;

fn main() -> icy_sauce::Result<()> {
    let content = b"Your file content here...";
    // Create character capabilities for an 80x25 ANSI file
    let mut caps = CharacterCapabilities::new(CharacterFormat::Ansi)
        .dimensions(80, 25);
    caps.set_font(BString::from("IBM VGA"))?;

    let sauce = SauceRecordBuilder::default()
        .title(BString::from("My Artwork"))?
        .author(BString::from("Artist"))?
        .group(BString::from("Art Group"))?
        .date(SauceDate::new(2024, 1, 15))
        .file_size(content.len() as u32)
        .capabilities(Capabilities::Character(caps))?
        .add_comment(BString::from("Created with love"))?
        .build();

    // Build the complete file in memory: payload + EOF + SAUCE.
    let mut output = content.to_vec();
    sauce.write(&mut output)?;
    assert!(output.starts_with(content));

    Ok(())
}
```

The library does not calculate `file_size` or replace existing metadata for you.
Set the payload size explicitly (0 for unknown or larger than `u32::MAX`), and
strip an existing record before appending a replacement. `write()` includes one
leading EOF byte; `write_without_eof()` omits it. The CLI performs file replacement
and payload-size handling for you.

### Migrating to 0.4

`SauceRecord::to_bytes()` and `to_bytes_without_eof()` now return `Result<Vec<u8>>`.
Callers migrating from the previous infallible methods must handle the result,
for example with `let bytes = sauce.to_bytes()?;`.

All date writers reject years outside 0–9999 and month/day values above 99 with
`SauceError::UnsupportedSauceDate`, before writing any part of the date or record.
Calendar validation remains optional: unknown dates (`00000000`) and other
eight-digit dates are still supported. The `chrono` feature can validate calendar dates.

Font setters now accept at most 21 bytes, reserving a byte for the NUL terminator,
and reject embedded NULs. Capability font decoders stop at the first NUL.
The raw `SauceHeader::t_info_s` field and builder's `t_info_s` setter still preserve
up to 22 raw bytes, including legacy unterminated values and bytes after an embedded
NUL. Use `record.to_builder()` for lossless edits to such records rather than
decoding and re-encoding their capabilities.

`SauceRecordBuilder::metadata()` now applies comments as well as title, author,
and group. Supplied comments replace existing comments, including clearing them
when the list is empty, matching `MetaData::to_builder()`.

`SauceRecordBuilder::capabilities()` selects the matching data and file types,
including when switching an existing record to Character capabilities. Basic
metadata, comments, date, and file size are preserved.

### Stripping SAUCE Metadata

You can remove one or more SAUCE records (and optionally their preceding EOF 0x1A marker) from the end of a file buffer without copying the data using `strip_sauce`.

`StripMode` variants:

| Mode | Removes | EOF Handling | Use Case |
|------|---------|--------------|----------|
| `Last` | Last SAUCE record | Preserves all EOF bytes | Keep legacy EOF marker but drop metadata |
| `LastStripFinalEof` (default) | Last SAUCE record | Removes a single EOF directly before the record | Clean view of payload |
| `All` | All contiguous SAUCE records (separated by ≤1 EOF each) | Removes one preceding EOF per record; preserves additional EOF bytes | Multi-edit cleanup |
| `AllStripFinalEof` | All contiguous SAUCE records | Like `All`, then removes one additional EOF from the remaining data | Aggressive full cleanup |

Contiguous multi-record stripping stops if more than one consecutive EOF (0x1A 0x1A ...) separates records—stacked EOFs form a barrier.

```rust
use icy_sauce::{strip_sauce, SauceRecordBuilder, StripMode};

let mut data = b"Content".to_vec();
data.extend(SauceRecordBuilder::default().build().to_bytes().unwrap());
let cleaned = strip_sauce(&data, StripMode::default()); // LastStripFinalEof
assert_eq!(cleaned, b"Content");

// Keep EOF marker but remove SAUCE
let keep_eof = strip_sauce(&data, StripMode::Last);

// Remove multiple contiguous SAUCE records and their associated EOFs
let multi = strip_sauce(&data, StripMode::All);

// Most aggressive: remove all contiguous SAUCE records and one trailing EOF
let aggressive = strip_sauce(&data, StripMode::AllStripFinalEof);
```

Multi-record example:

```text
"Content" 0x1A SAUCE1 0x1A SAUCE2       -> StripMode::All ->  "Content"

"Content" 0x1A SAUCE1 0x1A 0x1A SAUCE2 -> StripMode::All ->  "Content" 0x1A SAUCE1 0x1A  (extra EOF blocks chain)
```

Headers must be at the exact end of the input. An EOF after the final header prevents stripping in every mode.

Stripping stops at a malformed or truncated advertised comment block and leaves
that record intact. Read-only parsing may ignore a missing `COMNT` marker, but
destructive operations do not guess which preceding bytes belong to metadata.

#### Getting Strip Statistics

Use `strip_sauce_ex` for metadata about the operation:

```rust
use icy_sauce::{strip_sauce_ex, SauceRecordBuilder, StripMode};

let mut data = b"Content".to_vec();
data.extend(SauceRecordBuilder::default().build().to_bytes().unwrap());
let result = strip_sauce_ex(&data, StripMode::AllStripFinalEof);
println!("Removed {} record(s), {} EOF byte(s); new length {}",
         result.records_removed, result.eof_bytes_removed, result.data.len());
```

If no SAUCE record is found, the original slice is returned unchanged.

## Command Line Tool

The workspace includes the `sauce` command-line utility for viewing, adding,
altering, and removing SAUCE metadata.

### CLI Installation

From the repository root (also works before publication):

```bash
cargo install --path crates/icy_sauce_cli
```

After CLI 0.4.0 is published, it can be installed with
`cargo install icy_sauce_cli --version 0.4.0`. The executable is named `sauce`,
not `icy_sauce_cli`.

### Usage

```bash
sauce view artwork.ans --comments
sauce view artwork.ans --json
sauce add artwork.ans --title "My artwork"
sauce alter artwork.ans --group "My group"
sauce remove artwork.ans --strip-eof
```

Use `sauce <command> --help` for the full option list and `sauce info` for SAUCE
field definitions. `add` refuses an existing record unless `--force` is supplied;
`alter` requires one. Raw format fields use `--data-type`, `--file-type`,
`--tinfo1` through `--tinfo4`, `--tflags`, and `--tinfos`.

### JSON and Comment Editing

```bash
# Export metadata, then apply it to an existing record.
sauce view artwork.ans --json > metadata.json
sauce alter artwork.ans --from-json metadata.json --group "New group"

# Read a partial update from stdin; other fields remain unchanged.
printf '%s\n' '{"title":"New title","comments":[]}' | sauce alter artwork.ans --from-json -

# Append comments or clear the comment block.
sauce alter artwork.ans --add-comment "Additional note"
sauce alter artwork.ans --clear-comments
```

JSON uses the keys `title`, `author`, `group`, `date`, `comments`, `file_size`,
`data_type`, `file_type`, `tinfo1`, `tinfo2`, `tinfo3`, `tinfo4`, `tflags`, and
`tinfos`. Dates accept `YYYYMMDD` or `YYYY-MM-DD`; `0000-00-00` clears the date.
Omitted or `null` fields leave existing values unchanged on `alter`; use an empty
string or empty comment array to clear the corresponding text fields.
`view --json` emits `null` if no record exists; that output is not an importable
metadata object. JSON is not a byte-exact backup for legacy encodings.

### Editing and Safety Semantics

- `alter` preserves untouched metadata bytes, raw fields, and the original `file_size`.
- `add` calculates `file_size` from the payload, using 0 if it cannot fit in `u32`.
    An explicit JSON `file_size` overrides this value for both `add` and `alter`.
- CLI options override JSON fields. CLI `--comment` values replace JSON comments.
    On `alter`, omitted JSON comments preserve existing comments, while `"comments": []`
    clears them. `--clear-comments` takes precedence; `--add-comment` appends afterward.
- Changes use same-directory temporary files and atomic replacement, preserving
    permission bits and following symlinks. Atomic replacement creates a new inode;
    other hard links keep their old contents, and ownership, ACLs, and extended
    attributes are not copied.
- On Unix, the parent directory is synced after atomic replacement to strengthen
    crash durability. If that sync fails, the command returns an error stating that
    the file was already replaced but crash durability is uncertain; no rollback
    is attempted. Other platforms retain their existing file-sync behavior.
- Mutating commands resolve symlinks before reading and keep that resolved target
    for writing. Retargeting the input symlink does not redirect the write.
    File identity and content checks reject detected changes before replacement.
    These checks are best-effort, not transaction isolation: avoid concurrent writers
    or changes to the resolved path or its parent directories during an operation.
    A change after the final check can still race with the rename.
- CLI text input is UTF-8. JSON export uses lossy UTF-8 for legacy byte strings,
    so JSON is not a lossless backup format for CP437 metadata. The library itself
    preserves these bytes, as does `alter` for fields not explicitly replaced.
- JSON exports explicitly include `"date": "0000-00-00"` for an unknown date and
    `"tinfos": ""` for an empty font/raw TInfoS field. Reimporting these values clears
    the corresponding fields; omitting them in a partial JSON update preserves them.
- Human-readable output escapes terminal control characters and bidirectional
    overrides in metadata, paths, and errors. JSON retains its normal serialization.
- Modification commands reject an ambiguous comment block in the trailing record
    without changing the file. With `remove --all`, valid trailing records can be
    removed before stripping stops at an older malformed or unsupported header.
    If a SAUCE header remains at the resulting tail, the CLI reports partial removal
    and exits with status 1; the already removed records are not restored.

### Removal Modes and Exit Status

| Command options | Library mode | Effect |
|-----------------|--------------|--------|
| `remove` | `Last` | Remove the last record, keep its EOF byte |
| `remove --strip-eof` | `LastStripFinalEof` | Also remove one preceding EOF |
| `remove --all` | `All` | Remove contiguous records and one associated EOF per record |
| `remove --all --strip-eof` | `AllStripFinalEof` | Also remove one extra EOF from the remaining payload |

| Exit code | Meaning |
|-----------|---------|
| 0 | Success; includes `view`/`remove` finding no metadata |
| 1 | Parsing, validation, I/O, partial-removal, or durability error |
| 2 | Invalid command-line syntax or arguments |

An exit code of 1 does **not** always mean the file is unchanged: partial removal
and a failed directory sync happen after replacement. Read the error message
before retrying a modifying command.

## Supported Data Types

### Character Files

- ASCII, ANSI, ANSiMation
- PCBoard, Avatar, TundraDraw
- RipScript, HTML, Source code

### Graphics Files

- **Bitmap**: GIF, PCX, LBM/IFF, TGA, FLI/FLC, BMP, GL, DL, WPG, PNG, JPG, MPG, AVI
- **Vector**: DXF, DWG, WPG, 3DS

### Binary Text

- BinaryText (.BIN files) – even width (2–510), height derived from file size
- XBin – explicit width & height (u16), no font or rendering flags

### Audio Files

- Tracker: MOD, 669, STM, S3M, MTM, FAR, ULT, AMF, DMF, OKT, XM, IT
- Other: ROL, CMF, MIDI, VOC, WAV, SMP

### Archives

- ZIP, ARJ, LZH, ARC, TAR, ZOO, RAR, UC2, PAK, SQZ

## Advanced Usage

### Working with Comments

```rust
use icy_sauce::prelude::*;
use bstr::BString;

let sauce = SauceRecordBuilder::default()
    .title(BString::from("Art")).unwrap()
    .add_comment(BString::from("First comment")).unwrap()
    .add_comment(BString::from("Second comment")).unwrap()
    .build();

for comment in sauce.comments() {
    println!("Comment: {}", comment);
}
```

### Binary Text Files

```rust
use icy_sauce::prelude::*;
use bstr::BString;
use icy_sauce::{LetterSpacing, AspectRatio};

// BinaryText (width must be even; height can be derived from file size)
let mut bin_caps = BinaryCapabilities::binary_text(160).unwrap(); // 160 columns
bin_caps.ice_colors = true;
bin_caps.letter_spacing = LetterSpacing::NinePixel;
bin_caps.aspect_ratio = AspectRatio::Legacy;
bin_caps.set_font(BString::from("IBM VGA")).unwrap();

// XBin with explicit dimensions
let xbin_caps = BinaryCapabilities::xbin(80, 50).unwrap();
```

To compute height of a BinaryText record:

```rust
use icy_sauce::{BinaryCapabilities, Capabilities, SauceRecordBuilder};

let record = SauceRecordBuilder::default()
    .file_size(80 * 2 * 25)
    .capabilities(Capabilities::Binary(BinaryCapabilities::binary_text(80).unwrap()))
    .unwrap()
    .build();
if let Some(Capabilities::Binary(caps)) = record.capabilities() {
    assert_eq!(caps.binary_text_height_from_file_size(record.file_size()), Some(25));
}
```

### Bitmap & Vector Graphics

```rust
use icy_sauce::prelude::*;

let mut caps = BitmapCapabilities::new(BitmapFormat::Png);
caps.width = 640;
caps.height = 480;
caps.pixel_depth = 24;

let vector = VectorCapabilities::new(VectorFormat::Dxf);
```

### Audio Files

```rust
use icy_sauce::prelude::*;

let caps = AudioCapabilities { format: AudioFormat::S3m, sample_rate: 0 }; // tracker formats ignore sample_rate
```

### Archives

```rust
use icy_sauce::prelude::*;

let caps = ArchiveCapabilities { format: ArchiveFormat::Zip };
```

## String Encoding

SAUCE strings are typically encoded in CP437 (DOS codepage). This library uses `bstr::BString` for all text fields:

```rust
use bstr::{BString, ByteSlice};
let title = BString::from(b"My \x01 ASCII Art");
println!("Title: {}", title.to_str_lossy());
```

`to_str_lossy()` is UTF-8 display conversion, **not** CP437 decoding. Use an
explicit codepage converter if you need faithful Unicode text. Field limits count
bytes, not characters: title 35, author/group 20, each comment 64 (up to 255 lines),
validated font name 21, and raw TInfoS 22. Library display examples do not escape
terminal controls; use the CLI's escaped output for untrusted metadata.

## Error Handling

```rust
use icy_sauce::{SauceError, SauceRecordBuilder};

let sauce_result = SauceRecordBuilder::default().title("x".repeat(36).into());
match sauce_result {
    Err(SauceError::TitleTooLong(len)) => println!("Title is {} bytes, max is 35", len),
    Err(SauceError::CommentLimitExceeded) => println!("Cannot add more than 255 comments"),
    _ => {}
}
```

## Type-Safe Capabilities

```rust
use icy_sauce::prelude::*;
let char_caps = CharacterCapabilities::new(CharacterFormat::Ansi).dimensions(80, 25);
let caps = Capabilities::Character(char_caps);

match caps {
    Capabilities::Character(c) => println!("Character format with {} columns", c.columns),
    Capabilities::Bitmap(b) => println!("Bitmap format: {:?}", b.format),
    Capabilities::Vector(v) => println!("Vector format: {:?}", v.format),
    Capabilities::Audio(a) => println!("Audio format: {:?}", a.format),
    _ => {}
}
```

## BinaryCapabilities Quick Reference

| Field          | BinaryText Meaning                                | XBin Meaning                  |
|----------------|----------------------------------------------------|-------------------------------|
| `columns`      | Width (even 2–510)                                 | Width (0–65535, >0 recommended) |
| `lines`        | Always 0 (height derived from file size)           | Explicit height               |
| `ice_colors`   | Enables 16 background colors (non-blink mode)      | Ignored                       |
| `letter_spacing` | 8/9 pixel or legacy spacing                      | Ignored (always legacy)       |
| `aspect_ratio` | Legacy / LegacyDevice / Square                     | Ignored (legacy)              |
| `font()`       | Optional font name (≤21 bytes when writing, plus NUL) | Always `None`                 |

## Specifications

Implements SAUCE v00.5 Spec:

- [SAUCE Specification](http://www.acid.org/info/sauce/sauce.htm)

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).

## Contributing

Issues and PRs welcome: <https://github.com/mkrueger/icy_sauce>.

### Validation

Run from the workspace root:

```bash
cargo test --workspace
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo fmt --all -- --check
cargo fmt --manifest-path fuzz/Cargo.toml --all -- --check
cargo clippy --manifest-path fuzz/Cargo.toml --all-targets -- -D warnings
```

CI currently runs on Linux with Rust 1.89.0 and stable. It tests default and
all-feature configurations and checks formatting, Clippy, and Rustdoc. Local
success does not replace checking the workflow results before a release.

The fuzz package is a separate workspace. With `cargo-fuzz` and a nightly
toolchain installed, bounded smoke runs can be started from the repository root:

```bash
cargo +nightly fuzz run date_parse -- -runs=1000
cargo +nightly fuzz run header_parse -- -runs=1000
cargo +nightly fuzz run strip_logic -- -runs=1000
```

`strip_logic` checks agreement between stripping APIs, record round trips, and
preservation of payloads with malformed comment markers. Longer fuzz sessions
are recommended before release; smoke runs are not exhaustive.

For release verification, `cargo package --workspace` builds both packages using
the local library version. Publish the library before publishing the dependent CLI.
Keep the library dependency in the CLI manifest aligned with the library release;
both packages share the same release version. Replace the unreleased labels in this
README and [CHANGELOG.md](CHANGELOG.md) when publishing.

## Related Projects

- [icy_tools](https://github.com/mkrueger/icy_tools)
- [bstr](https://github.com/BurntSushi/bstr)
