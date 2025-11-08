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
- **CP437 Support**: Works with `bstr` for proper DOS codepage handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
icy_sauce = "0.3.0"
```

## Basic Usage

### Reading SAUCE

```rust
use icy_sauce::prelude::*; // brings common types into scope
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read("artwork.ans")?;
    
    if let Some(sauce) = SauceRecord::from_bytes(&data)? {
        println!("Title: {}", sauce.title());
        println!("Author: {}", sauce.author());
        println!("Group: {}", sauce.group());
        
        // Get format-specific information
        if let Some(caps) = sauce.capabilities() { // cached internally
            match caps {
                Capabilities::Character(c) => {
                    println!("Character format: {:?} ({}x{})", c.format, c.columns, c.lines);
                }
                Capabilities::Bitmap(b) => {
                    println!("Bitmap: {:?} ({}x{} @ {}bpp)", b.format, b.width, b.height, b.pixel_depth);
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
    // Create character capabilities for an 80x25 ANSI file (builder-style mutating methods retained)
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

## Command Line Tool

This library includes a command-line utility for inspecting SAUCE records in files. You can use it directly with `cargo run --example`:

### Installation

```bash
# Run directly from the repository
cargo run --example print_sauce <FILE>

# Or install it locally
cargo install --path . --example print_sauce
```

### Usage

```bash
# Basic usage - show SAUCE information
cargo run --example print_sauce artwork.ans

# Show comments
cargo run --example print_sauce artwork.ans --comments
cargo run --example print_sauce artwork.ans -c

# Show raw technical details
cargo run --example print_sauce artwork.ans --raw
cargo run --example print_sauce artwork.ans -r

# Show everything
cargo run --example print_sauce artwork.ans -c -r
```

### Example Output

```
$ cargo run --example print_sauce demo.ans --comments
SAUCE Information for 'demo.ans'
============================================================
Title:    Winter Scene
Author:   ArtistName
Group:    Cool Group
Date:     2024-01-15
Type:     Character

Character File Information:
  Format:     Ansi
  Dimensions: 80x25 (columns x rows)
  iCE Colors: Yes
  Letter Spacing: NinePixel
  Aspect Ratio:   Legacy
  Font:       IBM VGA

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
- BinaryText (.BIN files)
- XBin (extended BIN)

### Audio Files
- Tracker formats: MOD, 669, STM, S3M, MTM, FAR, ULT, AMF, DMF, OKT, XM, IT
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
    .add_comment(BString::from("Up to 255 comments allowed"))?
    .build();

// Read comments
for comment in sauce.comments() {
    println!("Comment: {}", comment);
}
```

### Binary Text Files

```rust
use icy_sauce::prelude::*;

// For .BIN files with specific width
let caps = BinaryCapabilities::binary_text(160)?  // 160 columns (must be even)
    .flags(true, LetterSpacing::NinePixel, AspectRatio::Legacy);

// For XBin files with explicit dimensions
let caps = BinaryCapabilities::xbin(80, 50)?;
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

### Archive Files

```rust
use icy_sauce::prelude::*;

let caps = ArchiveCapabilities { format: ArchiveFormat::Zip };
```

## String Encoding

SAUCE strings are typically encoded in CP437 (DOS codepage). This library uses `bstr::BString` for all text fields, allowing you to handle the encoding as needed:

```rust
use bstr::BString;

// Create from CP437 bytes
let title = BString::from(b"My \x01 ASCII Art");  // ☺ smiley in CP437

// Convert to UTF-8 for display (lossy)
println!("Title: {}", title.to_str_lossy());
```

## Error Handling

The library provides detailed error types for various failure conditions:

```rust
use icy_sauce::SauceError;

match sauce_result {
    Err(SauceError::TitleTooLong(len)) => {
        println!("Title is {} bytes, max is 35", len);
    }
    Err(SauceError::CommentLimitExceeded) => {
        println!("Cannot add more than 255 comments");
    }
    _ => {}
}
```

## Type-Safe Capabilities

Each file type has its own strongly-typed capability structure, ensuring you can only set valid metadata:

```rust
use icy_sauce::prelude::*;

// Character files have dimensions and font settings
let char_caps = CharacterCapabilities::new(CharacterFormat::Ansi).dimensions(80, 25);

// Convert to general capabilities
let caps = Capabilities::Character(char_caps);

// Pattern match to access specific capabilities
match caps {
    Capabilities::Character(c) => println!("Character format with {} columns", c.columns),
    Capabilities::Bitmap(b) => println!("Bitmap format: {:?}", b.format),
    Capabilities::Vector(v) => println!("Vector format: {:?}", v.format),
    Capabilities::Audio(a) => println!("Audio format: {:?}", a.format),
    _ => {}
}
```

## Specifications

This library implements the SAUCE v00 specification. For detailed information about the SAUCE format, visit:
- [SAUCE Specification](http://www.acid.org/info/sauce/sauce.htm)

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests on the [GitHub repository](https://github.com/mkrueger/icy_sauce).

## Related Projects

- [icy_tools](https://github.com/mkrueger/icy_tools) - ANSI/ASCII art editor and viewer
- [bstr](https://github.com/BurntSushi/bstr) - Byte string utilities (used for CP437 handling)