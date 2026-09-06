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
//! - [`binary`] - Binary text formats (BinaryText, XBin)
//! - [`character`] - Character/text formats (ANSI, ASCII, etc.)
//! - [`executable`] - Executable file formats
//! - [`bitmap`] - Bitmap images, animations, and RIPScript
//! - [`vector`] - Vector graphics formats
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
/// - [`Bitmap`](Capabilities::Bitmap) - Bitmap images, animations, and RIPScript
/// - [`Vector`](Capabilities::Vector) - Vector graphics
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
/// Parse header fields using each capability type's `TryFrom<&SauceHeader>`
/// implementation. Write capabilities through
/// [`SauceRecordBuilder::capabilities`](crate::SauceRecordBuilder::capabilities):
///
/// ```
/// use icy_sauce::header::SauceHeader;
/// use icy_sauce::{Capabilities, SauceDataType, SauceRecordBuilder, VectorCapabilities, VectorFormat};
///
/// // Parse from header
/// let header = SauceHeader {
///     data_type: SauceDataType::Vector,
///     file_type: VectorFormat::Dxf.to_sauce(),
///     ..SauceHeader::default()
/// };
/// let caps = VectorCapabilities::try_from(&header).unwrap();
///
/// // Write into a new record through the public builder
/// let record = SauceRecordBuilder::default()
///     .capabilities(Capabilities::Vector(caps)).unwrap()
///     .build();
/// assert_eq!(record.header().data_type, SauceDataType::Vector);
/// assert_eq!(record.header().file_type, 0);
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

    /// Bitmap and RIPScript capabilities.
    ///
    /// For bitmap images and animations (GIF, PNG, JPG, etc.), plus RIPScript.
    /// Contains pixel dimensions and color depth. When reading a record,
    /// [`SauceRecord::capabilities`](crate::SauceRecord::capabilities) returns
    /// RIPScript as [`Capabilities::Character`] because its DataType is Character;
    /// use `BitmapCapabilities::try_from(record.header())` for the bitmap view.
    Bitmap(BitmapCapabilities),

    /// Vector format capabilities.
    ///
    /// For scalable graphics formats (DXF, DWG, WPG Vector, 3DS).
    /// Contains only the format identifier, not dimensions, bounding boxes, or layers.
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
