//! Archive format capabilities as specified in the SAUCE v00 standard.
//!
//! This module provides types for describing compressed archive files with SAUCE metadata.
//! Archive files have minimal metadata - just the format type, with all other fields fixed.
//!
//! # SAUCE Field Mappings
//!
//! All archive formats use:
//! - DataType: Archive (7)
//! - FileType: Format variant (0-9 for ZIP through SQZ)
//! - TInfo1-4: All 0
//! - TFlags: 0
//! - TInfoS: Empty
//!
//! # Example
//!
//! ```
//! use icy_sauce::{ArchiveCapabilities, ArchiveFormat, SauceRecordBuilder, Capabilities, SauceDataType};
//!
//! let caps = ArchiveCapabilities::new(ArchiveFormat::Zip);
//! let sauce = SauceRecordBuilder::default()
//!     .data_type(SauceDataType::Archive)
//!     .capabilities(Capabilities::Archive(caps))
//!     .unwrap()
//!     .build();
//! assert_eq!(sauce.header().file_type, 0); // Zip maps to FileType 0
//! ```

use crate::{SauceDataType, SauceError, header::SauceHeader};

/// Archive file format enumeration.
///
/// Defines the 10 archive formats supported by SAUCE v00 specification.
/// Each format corresponds to a specific FileType value (0-9).
///
/// # Example
///
/// ```
/// use icy_sauce::ArchiveFormat;
///
/// let format = ArchiveFormat::from_sauce(0);
/// assert_eq!(format, ArchiveFormat::Zip);
/// assert_eq!(format.to_sauce(), 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    /// ZIP format: PKWare ZIP (file type code 0)
    Zip,
    /// ARJ format: ARJ by Robert K. Jung (file type code 1)
    Arj,
    /// LZH format: LZH by Haruyasu Yoshizaki (file type code 2)
    Lzh,
    /// ARC format: ARC by S.E.A. (file type code 3)
    Arc,
    /// TAR format: Unix TAR (uncompressed, file type code 4)
    Tar,
    /// ZOO format: ZOO (file type code 5)
    Zoo,
    /// RAR format: WinRAR (file type code 6)
    Rar,
    /// UC2 format: UC2 (file type code 7)
    Uc2,
    /// PAK format: PAK (file type code 8)
    Pak,
    /// SQZ format: SQZ (file type code 9)
    Sqz,

    /// Unknown archive format with arbitrary file type code
    Unknown(u8),
}

impl ArchiveFormat {
    /// Parse archive format from SAUCE FileType byte.
    ///
    /// # Arguments
    ///
    /// * `file_type` - The FileType byte from SAUCE header (0-9)
    ///
    /// # Returns
    ///
    /// The corresponding [`ArchiveFormat`] variant, or `Unknown` for unrecognized values.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::ArchiveFormat;
    ///
    /// assert_eq!(ArchiveFormat::from_sauce(0), ArchiveFormat::Zip);
    /// assert_eq!(ArchiveFormat::from_sauce(1), ArchiveFormat::Arj);
    /// assert_eq!(ArchiveFormat::from_sauce(99), ArchiveFormat::Unknown(99));
    /// ```
    pub fn from_sauce(file_type: u8) -> Self {
        match file_type {
            0 => ArchiveFormat::Zip,
            1 => ArchiveFormat::Arj,
            2 => ArchiveFormat::Lzh,
            3 => ArchiveFormat::Arc,
            4 => ArchiveFormat::Tar,
            5 => ArchiveFormat::Zoo,
            6 => ArchiveFormat::Rar,
            7 => ArchiveFormat::Uc2,
            8 => ArchiveFormat::Pak,
            9 => ArchiveFormat::Sqz,
            _ => ArchiveFormat::Unknown(file_type),
        }
    }

    /// Convert to SAUCE FileType byte.
    ///
    /// # Returns
    ///
    /// The FileType byte value (0-9 for known formats, original value for Unknown).
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::ArchiveFormat;
    ///
    /// assert_eq!(ArchiveFormat::Zip.to_sauce(), 0);
    /// assert_eq!(ArchiveFormat::Rar.to_sauce(), 6);
    /// assert_eq!(ArchiveFormat::Unknown(99).to_sauce(), 99);
    /// ```
    pub fn to_sauce(&self) -> u8 {
        match self {
            ArchiveFormat::Zip => 0,
            ArchiveFormat::Arj => 1,
            ArchiveFormat::Lzh => 2,
            ArchiveFormat::Arc => 3,
            ArchiveFormat::Tar => 4,
            ArchiveFormat::Zoo => 5,
            ArchiveFormat::Rar => 6,
            ArchiveFormat::Uc2 => 7,
            ArchiveFormat::Pak => 8,
            ArchiveFormat::Sqz => 9,
            ArchiveFormat::Unknown(ft) => *ft,
        }
    }

    /// Get the typical file extension for this archive format.
    ///
    /// # Returns
    ///
    /// A string slice with the common file extension (lowercase, without dot).
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::ArchiveFormat;
    ///
    /// assert_eq!(ArchiveFormat::Zip.extension(), "zip");
    /// assert_eq!(ArchiveFormat::Rar.extension(), "rar");
    /// assert_eq!(ArchiveFormat::Unknown(99).extension(), "");
    /// ```
    pub fn extension(&self) -> &str {
        match self {
            ArchiveFormat::Zip => "zip",
            ArchiveFormat::Arj => "arj",
            ArchiveFormat::Lzh => "lzh",
            ArchiveFormat::Arc => "arc",
            ArchiveFormat::Tar => "tar",
            ArchiveFormat::Zoo => "zoo",
            ArchiveFormat::Rar => "rar",
            ArchiveFormat::Uc2 => "uc2",
            ArchiveFormat::Pak => "pak",
            ArchiveFormat::Sqz => "sqz",
            ArchiveFormat::Unknown(_) => "",
        }
    }

    /// Check if this format typically provides compression.
    ///
    /// # Returns
    ///
    /// `true` if the format usually compresses data, `false` if it's primarily
    /// an archive format without compression (like TAR).
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::ArchiveFormat;
    ///
    /// assert!(ArchiveFormat::Zip.is_compressed());
    /// assert!(!ArchiveFormat::Tar.is_compressed());
    /// ```
    pub fn is_compressed(&self) -> bool {
        match self {
            ArchiveFormat::Tar => false, // TAR is uncompressed
            ArchiveFormat::Unknown(_) => false,
            _ => true,
        }
    }
}

/// Check if this format typically provides compression.
///
/// # Returns
///
/// `true` if the format usually compresses data, `false` if it's primarily
/// an archive format without compression (like TAR).
///
/// # Example
///
/// ```
/// use icy_sauce::ArchiveFormat;
/// assert!(ArchiveFormat::Zip.is_compressed());
/// assert!(!ArchiveFormat::Tar.is_compressed());
/// ```
#[derive(Debug, Clone)]
pub struct ArchiveCapabilities {
    /// Archive format (ZIP, RAR, TAR, etc.)
    pub format: ArchiveFormat,
}

impl ArchiveCapabilities {
    /// Create new archive capabilities.
    ///
    /// Constructs a new `ArchiveCapabilities` with the specified archive format.
    ///
    /// # Arguments
    ///
    /// * `format` - The archive format
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::{ArchiveCapabilities, ArchiveFormat};
    /// let archive = ArchiveCapabilities::new(ArchiveFormat::Zip);
    /// assert_eq!(archive.format, ArchiveFormat::Zip);
    /// ```
    pub fn new(format: ArchiveFormat) -> Self {
        ArchiveCapabilities { format }
    }

    /// Parse archive capabilities from SAUCE header.
    ///
    /// Extracts archive-specific metadata from a SAUCE header. Only valid when
    /// header.data_type is Archive (11).
    ///
    /// # Arguments
    ///
    /// * `header` - SAUCE header to parse
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::UnsupportedDataType`] if header.data_type is not Archive.
    ///
    /// # SAUCE Field Mapping
    ///
    /// * FileType → ArchiveFormat (0-9)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Parsing from a raw header is an internal operation (pub(crate));
    /// // use SauceRecord::read() for public parsing.
    /// ```
    pub(crate) fn from(header: &SauceHeader) -> crate::Result<Self> {
        if header.data_type != SauceDataType::Archive {
            return Err(SauceError::UnsupportedDataType(header.data_type));
        }

        let format = ArchiveFormat::from_sauce(header.file_type);

        Ok(ArchiveCapabilities { format })
    }

    /// Write archive capabilities to SAUCE header.
    ///
    /// Encodes archive metadata into a SAUCE header for serialization. Sets DataType to Archive (11)
    /// and encodes the format in FileType. All TInfo fields are set to 0 per SAUCE specification
    /// as archives have no additional metadata.
    ///
    /// # Arguments
    ///
    /// * `header` - SAUCE header to modify
    ///
    /// # SAUCE Field Mapping
    ///
    /// * DataType → Archive (11)
    /// * FileType ← ArchiveFormat::to_sauce() (0-9)
    /// * TInfo1, TInfo2, TInfo3, TInfo4 → 0
    /// * TFlags → 0
    /// * TInfoS → empty
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Writing directly to a header is internal (pub(crate)); use the builder:
    /// // let caps = ArchiveCapabilities::new(ArchiveFormat::Rar);
    /// // SauceRecordBuilder::default()
    /// //     .data_type(SauceDataType::Archive)
    /// //     .capabilities(Capabilities::Archive(caps));
    /// ```
    pub(crate) fn encode_into_header(&self, header: &mut SauceHeader) -> crate::Result<()> {
        header.data_type = SauceDataType::Archive;
        header.file_type = self.format.to_sauce();

        // Archive formats have all TInfo fields set to 0 per spec
        header.t_info1 = 0;
        header.t_info2 = 0;
        header.t_info3 = 0;
        header.t_info4 = 0;

        // No flags or TInfoS for archives
        header.t_flags = 0;
        header.t_info_s.clear();

        Ok(())
    }
}
