//! Format-specific capability types for SAUCE metadata.
//!
//! This module provides specialized capability structures for each SAUCE data type,
//! allowing type-safe access to format-specific metadata fields. The capabilities
//! are organized by file format category, with each type mapping to specific SAUCE
//! header fields.
//!
//! # Module Organization
//!
//! Each submodule corresponds to a SAUCE data type category:
//!
//! - [`archive`] - Compressed archive formats (ZIP, RAR, etc.)
//! - [`audio`] - Sound and music formats (MOD, S3M, WAV, etc.)
//! - [`bin`] - Binary text formats (BinaryText, XBin)
//! - [`char`] - Character/text formats (ANSI, ASCII, etc.)
//! - [`executable`] - Executable file formats
//! - [`pixel`] - Graphics formats (bitmap and vector)
//!
//! # Unified Access
//!
//! The [`Capabilities`] enum provides a unified interface for accessing
//! all capability types through pattern matching.
//!
//! # Example
//!
//! ```no_run
//! use icy_sauce::{Capabilities, CharacterCapabilities};
//! use icy_sauce::SauceRecord;
//!
//! // Parse SAUCE from file data
//! let data = std::fs::read("artwork.ans").unwrap();
//! if let Ok(Some(sauce)) = SauceRecord::from_bytes(&data) {
//!     // Access capabilities through unified enum
//!     match sauce.capabilities() {
//!         Some(Capabilities::Character(caps)) => {
//!             println!("Text file: {}x{}", caps.columns, caps.lines);
//!         }
//!         Some(Capabilities::Bitmap(caps)) => {
//!             println!("Image: {}x{} @ {}bpp", caps.width, caps.height, caps.pixel_depth);
//!         }
//!         _ => println!("Other format"),
//!     }
//! }
//! ```

pub mod archive;
pub use crate::archive::{ArchiveCapabilities, ArchiveFormat};
pub mod audio;
pub use crate::audio::{AudioCapabilities, AudioFormat};
pub mod binary;
pub use crate::binary::{BinaryCapabilities, BinaryFormat};
pub mod character;
pub use crate::character::{AspectRatio, CharacterCapabilities, CharacterFormat, LetterSpacing};
pub mod executable;
pub use crate::executable::ExecutableCapabilities;

pub mod bitmap;
pub use crate::bitmap::{BitmapCapabilities, BitmapFormat};

pub mod vector;
pub use crate::vector::{VectorCapabilities, VectorFormat};

/// Unified enumeration of all format-specific capabilities.
///
/// This enum provides a type-safe way to access format-specific metadata
/// for any SAUCE data type. Each variant corresponds to a specific category
/// of file formats with its own set of capabilities.
///
/// # Variants
///
/// - [`Character`](Capabilities::Character) - Text and ANSI art files
/// - [`Binary`](Capabilities::Binary) - Binary text and XBin files
/// - [`Graphics`](Capabilities::Graphics) - Bitmap and vector graphics
/// - [`Audio`](Capabilities::Audio) - Sound and music files
/// - [`Archive`](Capabilities::Archive) - Compressed archives
/// - [`Executable`](Capabilities::Executable) - Program files
///
/// # Usage
///
/// Capabilities are typically obtained from a [`SauceRecord`](crate::SauceRecord)
/// record and accessed through pattern matching:
///
/// ```
/// use icy_sauce::Capabilities;
/// use icy_sauce::{CharacterCapabilities, CharacterFormat, LetterSpacing, AspectRatio};
///
/// // Create character capabilities using the public constructor
/// let char_caps = CharacterCapabilities::new(CharacterFormat::Ansi);
///
/// // Wrap in enum
/// let caps = Capabilities::Character(char_caps);
///
/// // Access through pattern matching
/// match caps {
///     Capabilities::Character(c) => {
///         println!("Character format: {:?}", c.format);
///         println!("Dimensions: {}x{}", c.columns, c.lines);
///     }
///     Capabilities::Binary(b) => {
///         println!("Binary format with width: {}", b.columns);
///     }
///     _ => println!("Other format"),
/// }
/// ```
///
/// # Conversion
///
/// Each capability type can be converted to/from SAUCE header fields through
/// their respective `TryFrom<&SauceHeader>` implementations and `encode_into_header()` methods:
///
/// ```ignore
/// // Internal conversion example (ignored because `from` and `encode_into_header` are not public)
/// use icy_sauce::header::SauceHeader;
/// use icy_sauce::{CharacterCapabilities, Capabilities, SauceDataType};
///
/// // Parse from header
/// let mut header = SauceHeader::default();
/// header.data_type = SauceDataType::Character;
/// use std::convert::TryFrom;
/// let char_caps = CharacterCapabilities::try_from(&header).unwrap();
/// let caps = Capabilities::Character(char_caps);
///
/// // Write back to header
/// match caps {
///     Capabilities::Character(c) => c.encode_into_header(&mut header).unwrap(),
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Capabilities {
    /// Character/text format capabilities.
    ///
    /// For ASCII, ANSI, ANSiMation, PCBoard, Avatar, HTML, Source, and other text formats.
    /// Contains dimensions, font information, and rendering settings.
    Character(CharacterCapabilities),

    /// Binary text format capabilities.
    ///
    /// For BinaryText (.BIN) and XBin formats.
    /// Contains width, height (for XBin), and display flags.
    Binary(BinaryCapabilities),

    /// Graphics format capabilities.
    ///
    /// For bitmap (GIF, PNG, JPG, etc.) and vector (DXF, DWG, etc.) formats.
    /// Contains pixel dimensions and color depth.
    Bitmap(BitmapCapabilities),

    /// Vector format capabilities.
    ///
    /// For scalable graphics formats (DXF, DWG, WPG Vector, 3DS).
    /// Contains vector-specific metadata such as bounding box and layer information.
    Vector(VectorCapabilities),

    /// Audio format capabilities.
    ///
    /// For tracker modules (MOD, S3M, XM, IT), MIDI, WAV, and other sound formats.
    /// Contains format type and optional sample rate.
    Audio(AudioCapabilities),

    /// Archive format capabilities.
    ///
    /// For compressed archives (ZIP, RAR, ARJ, etc.).
    /// Contains only the format type identifier.
    Archive(ArchiveCapabilities),

    /// Executable format capabilities.
    ///
    /// For program files. This is a marker type with no additional metadata.
    Executable(ExecutableCapabilities),
}
