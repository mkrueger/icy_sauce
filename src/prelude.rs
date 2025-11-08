//! Crate prelude: convenient re-exports of the most commonly used
//! types, enums, and builders for working with SAUCE metadata.
//!
//! Importing the prelude lets you get started quickly without a long
//! list of individual `use` statements:
//!
//! ```
//! use icy_sauce::prelude::*;
//! use bstr::BString;
//!
//! // Build a simple SAUCE record for an ANSI text file.
//! let sauce = SauceRecordBuilder::default()
//!     .title(BString::from("Example")).unwrap()
//!     .author(BString::from("Me")).unwrap()
//!     .group(BString::from("Group")).unwrap()
//!     .date(SauceDate::new(2025, 11, 8))
//!     .data_type(SauceDataType::Character)
//!     .capabilities(Capabilities::Character(CharacterCapabilities::new(CharacterFormat::Ansi))).unwrap()
//!     .add_comment(BString::from("Rendered with iCE colors"))?.build();
//!
//! assert_eq!(sauce.date().year, 2025);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! The prelude intentionally omits rarely-used internals (low-level header
//! parsing helpers, internal padding helpers, etc.) to keep the import
//! surface focused and stable.
//!
//! ## Re-exported Items
//!
//! - Core record & builder: [`SauceRecord`], [`SauceRecordBuilder`]
//! - Date handling: [`SauceDate`]
//! - Data type enum & error: [`SauceDataType`], [`SauceError`]
//! - Result alias: [`Result`]
//! - Unified capabilities enum: [`Capabilities`]
//! - Capability structs & format enums for all categories
//!   - Character: [`CharacterCapabilities`], [`CharacterFormat`], [`LetterSpacing`], [`AspectRatio`]
//!   - Binary: [`BinaryCapabilities`], [`BinaryFormat`]
//!   - Bitmap: [`BitmapCapabilities`], [`BitmapFormat`]
//!   - Vector: [`VectorCapabilities`], [`VectorFormat`]
//!   - Audio: [`AudioCapabilities`], [`AudioFormat`]
//!   - Archive: [`ArchiveCapabilities`], [`ArchiveFormat`]
//!   - Executable: [`ExecutableCapabilities`]
//!
//! If you need lower-level access (e.g. raw header operations) you can
//! still import from the crate root (e.g. `use icy_sauce::header::SauceHeader`).

pub use crate::{
    // Core types
    SauceRecord,
    SauceRecordBuilder,
    SauceDate,
    SauceDataType,
    SauceError,
    Result,
    // Unified enum
    Capabilities,
    // Character
    CharacterCapabilities,
    CharacterFormat,
    LetterSpacing,
    AspectRatio,
    // Binary
    BinaryCapabilities,
    BinaryFormat,
    // Bitmap
    BitmapCapabilities,
    BitmapFormat,
    // Vector
    VectorCapabilities,
    VectorFormat,
    // Audio
    AudioCapabilities,
    AudioFormat,
    // Archive
    ArchiveCapabilities,
    ArchiveFormat,
    // Executable
    ExecutableCapabilities,
};
