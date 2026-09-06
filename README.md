# icy_sauce

A Rust library for reading and writing SAUCE (Standard Architecture for Universal Comment Extensions) metadata records. SAUCE is a metadata protocol widely used in the ANSI art and BBS scenes to embed information about artwork files.

## What is SAUCE?

SAUCE is a metadata format created in 1994 by ACiD Productions to standardize how information about digital artwork and other files is stored. The SAUCE record is appended to the end of files and contains:

- Title, Author, and Group information
- Creation date
- File type specifications
- Format-specific metadata (dimensions, fonts, etc.)
- Comments

## Features

- **Full SAUCE Specification Support**: Implements the complete SAUCE v00 specification
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

Add this to your `Cargo.toml`:

```toml
[dependencies]
icy_sauce = "0.4.0"
```

## Basic Usage

### Reading SAUCE

```rust
use icy_sauce::prelude::*; // brings common types into scope
use bstr::ByteSlice;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read("artwork.ans")?;
    
    if let Some(sauce) = SauceRecord::from_bytes(&data)? {
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

### Writing SAUCE

```rust
use icy_sauce::prelude::*;
use bstr::BString;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create character capabilities for an 80x25 ANSI file
    let mut caps = CharacterCapabilities::new(CharacterFormat::Ansi)
        .dimensions(80, 25);
    caps.set_font(BString::from("IBM VGA"))?;

    let sauce = SauceRecordBuilder::default()
        .title(BString::from("My Artwork"))?
        .author(BString::from("Artist"))?
        .group(BString::from("Art Group"))?
        .date(SauceDate::new(2024, 1, 15))
        .capabilities(Capabilities::Character(caps))?
        .add_comment(BString::from("Created with love"))?
        .build();

    // Write to file with content
    let mut output = Vec::new();
    output.extend_from_slice(b"Your file content here...");
    sauce.write(&mut output)?;
    
    Ok(())
}
```

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
use icy_sauce::{strip_sauce, StripMode};

// Assume `data` contains file payload + EOF + SAUCE
let cleaned = strip_sauce(&data, StripMode::default()); // LastStripFinalEof

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
use icy_sauce::{strip_sauce_ex, StripMode};

let result = strip_sauce_ex(&data, StripMode::AllStripFinalEof);
println!("Removed {} record(s), {} EOF byte(s); new length {}", 
         result.records_removed, result.eof_bytes_removed, result.data.len());
```

If no SAUCE record is found, the original slice is returned unchanged.

## Command Line Tool

The workspace includes the `sauce` command-line utility for viewing, adding,
altering, and removing SAUCE metadata.

### CLI Installation

```bash
cargo install --path crates/icy_sauce_cli
```

### Usage

```bash
sauce view artwork.ans --comments
sauce view artwork.ans --json
sauce add artwork.ans --title "My artwork"
sauce alter artwork.ans --group "My group"
sauce remove artwork.ans --strip-eof
```

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
- CLI text input is UTF-8. JSON export uses lossy UTF-8 for legacy byte strings,
    so JSON is not a lossless backup format for CP437 metadata. The library itself
    preserves these bytes, as does `alter` for fields not explicitly replaced.
- Human-readable output escapes terminal control characters and bidirectional
    overrides in metadata, paths, and errors. JSON retains its normal serialization.
- Modification commands reject ambiguous comment blocks without changing the file.

### Example Output

```text
SAUCE Information for 'demo.ans'
============================================================
Title:    Winter Scene
Author:   ArtistName
Group:    Cool Group
Date:     2024-01-15
Type:     Character

Character File Information:
  Format:        Ansi
  Dimensions:    80x25
  iCE Colors:    Yes
  Letter Spacing: NinePixel
  Aspect Ratio:   Legacy
  Font:          IBM VGA

Comments (2):
----------------------------------------
  1: Created for the winter artpack
  2: Inspired by snowy mountains
```

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
    .title(BString::from("Art"))?
    .add_comment(BString::from("First comment"))?
    .add_comment(BString::from("Second comment"))?
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
let mut bin_caps = BinaryCapabilities::binary_text(160)?; // 160 columns
bin_caps.ice_colors = true;
bin_caps.letter_spacing = LetterSpacing::NinePixel;
bin_caps.aspect_ratio = AspectRatio::Legacy;
bin_caps.set_font(BString::from("IBM VGA"))?;

// XBin with explicit dimensions
let xbin_caps = BinaryCapabilities::xbin(80, 50)?;
```

To compute height of a BinaryText file after parsing:

```rust
if let Some(h) = bin_caps.binary_text_height_from_file_size(record.file_size()) {
    println!("Derived height: {}", h);
}
```

### Bitmap & Vector Graphics

```rust
use icy_sauce::prelude::*;

let mut caps = BitmapCapabilities::new(BitmapFormat::Png);
caps.width = 640;
caps.height = 480;
caps.pixel_depth = 24;
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

## Error Handling

```rust
use icy_sauce::SauceError;

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
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo fmt --all -- --check
cargo fmt --manifest-path fuzz/Cargo.toml --all -- --check
```

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

## Related Projects

- [icy_tools](https://github.com/mkrueger/icy_tools)
- [bstr](https://github.com/BurntSushi/bstr)
