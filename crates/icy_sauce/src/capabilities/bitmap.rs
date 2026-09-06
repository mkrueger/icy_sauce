//! Bitmap and RIPScript capabilities for SAUCE metadata.
//!
//! This module provides types for describing bitmap images, animations, and RIPScript.
//! Vector graphics are handled separately by [`crate::vector`].
//!
//! # Format Categories
//!
//! **Bitmap Formats** (DataType=2): Raster image and animation formats
//! - GIF, PCX, LBM/IFF, TGA, FLI, FLC, BMP, GL, DL, WPG, PNG, JPG, MPEG, AVI
//!
//! **Special Format**: RIPScript (DataType=1 with FileType=3)
//! - Character-based graphics with fixed 640x350 pixel dimensions
//!
//! # Example
//!
//! ```
//! use icy_sauce::{BitmapCapabilities, BitmapFormat};
//!
//! let caps = BitmapCapabilities::new(BitmapFormat::Png);
//! assert_eq!(caps.width, 0);
//! assert_eq!(caps.height, 0);
//! ```

use crate::{SauceDataType, SauceError, header::SauceHeader};

#[derive(Debug, Clone, Copy, PartialEq)]
/// Bitmap and RIPScript format types for SAUCE metadata.
///
/// `BitmapFormat` enumerates bitmap formats and the special RIPScript format.
/// Vector formats use [`crate::VectorFormat`] instead.
///
/// # Bitmap Formats (DataType::Bitmap)
///
/// Raster image and video formats with pixel dimensions (values 0-13):
/// - **Gif** (0): Graphics Interchange Format
/// - **Pcx** (1): ZSoft Paintbrush
/// - **LbmIff** (2): Deluxe Paint (Amiga IFF/ILBM)
/// - **Tga** (3): Truevision TARGA
/// - **Fli** (4): Autodesk Animator
/// - **Flc** (5): Autodesk Animator Pro
/// - **Bmp** (6): Windows Bitmap
/// - **Gl** (7): GRASP animation format
/// - **Dl** (8): DL animation format
/// - **Wpg** (9): WordPerfect Graphics
/// - **Png** (10): Portable Network Graphics
/// - **Jpg** (11): JPEG Image Format
/// - **Mpg** (12): MPEG video
/// - **Avi** (13): Audio Video Interleave
///
/// # Special Format
///
/// - **RipScript**: Remote Imaging Protocol (Character DataType with FileType=3)
///   - Fixed 640x350 pixel dimensions
///   - 16-color display
///
/// # Unknown Variants
///
/// - **Unknown**: Preserves unrecognized (DataType, FileType) pairs for forward compatibility
///
/// # Example
///
/// ```
/// use icy_sauce::BitmapFormat;
/// use icy_sauce::SauceDataType;
///
/// let fmt = BitmapFormat::from_sauce(SauceDataType::Bitmap, 10);
/// assert_eq!(fmt, BitmapFormat::Png);
/// let (dt, ft) = fmt.to_sauce();
/// assert_eq!(ft, 10);
/// ```
pub enum BitmapFormat {
    // Bitmap formats (DataType::Bitmap)
    /// GIF (Graphics Interchange Format)
    Gif,
    /// PCX (ZSoft Paintbrush)
    Pcx,
    /// LBM/IFF (Deluxe Paint - Amiga format)
    LbmIff,
    /// TGA (Truevision TARGA)
    Tga,
    /// FLI (Autodesk Animator)
    Fli,
    /// FLC (Autodesk Animator Pro)
    Flc,
    /// BMP (Windows Bitmap)
    Bmp,
    /// GL (GRASP animation)
    Gl,
    /// DL (DL animation)
    Dl,
    /// WPG (WordPerfect Graphics)
    Wpg,
    /// PNG (Portable Network Graphics)
    Png,
    /// JPG (JPEG Image Format)
    Jpg,
    /// MPEG (Motion Picture Experts Group video)
    Mpg,
    /// AVI (Audio Video Interleave)
    Avi,

    /// RIPScript (Remote Imaging Protocol) - special character-based format
    RipScript,

    /// Unknown format (preserves original DataType and FileType for forward compatibility)
    Unknown(SauceDataType, u8),
}

impl BitmapFormat {
    /// Parse a graphics format from SAUCE data type and file type bytes.
    ///
    /// # Arguments
    ///
    /// * `data_type` - The SAUCE DataType field
    /// * `file_type` - The SAUCE FileType field
    ///
    /// # Returns
    ///
    /// The corresponding [`BitmapFormat`], or [`BitmapFormat::Unknown`] if the
    /// combination is not recognized.
    ///
    /// # Special Cases
    ///
    /// - RIPScript: DataType=Character with FileType=3
    /// - Bitmap formats: DataType=Bitmap with FileType 0-13
    /// - Other data types, including Vector, produce [`BitmapFormat::Unknown`]
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::BitmapFormat;
    /// use icy_sauce::SauceDataType;
    ///
    /// let fmt = BitmapFormat::from_sauce(SauceDataType::Bitmap, 6);
    /// assert_eq!(fmt, BitmapFormat::Bmp);
    /// ```
    pub fn from_sauce(data_type: SauceDataType, file_type: u8) -> Self {
        match data_type {
            SauceDataType::Character if file_type == 3 => BitmapFormat::RipScript,
            SauceDataType::Bitmap => match file_type {
                0 => BitmapFormat::Gif,
                1 => BitmapFormat::Pcx,
                2 => BitmapFormat::LbmIff,
                3 => BitmapFormat::Tga,
                4 => BitmapFormat::Fli,
                5 => BitmapFormat::Flc,
                6 => BitmapFormat::Bmp,
                7 => BitmapFormat::Gl,
                8 => BitmapFormat::Dl,
                9 => BitmapFormat::Wpg,
                10 => BitmapFormat::Png,
                11 => BitmapFormat::Jpg,
                12 => BitmapFormat::Mpg,
                13 => BitmapFormat::Avi,
                _ => BitmapFormat::Unknown(data_type, file_type),
            },
            _ => BitmapFormat::Unknown(data_type, file_type),
        }
    }

    /// Convert to SAUCE data type and file type bytes.
    ///
    /// # Returns
    ///
    /// A tuple `(data_type, file_type)` suitable for writing to a SAUCE header.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::BitmapFormat;
    /// use icy_sauce::SauceDataType;
    ///
    /// let (dt, ft) = BitmapFormat::Png.to_sauce();
    /// assert_eq!(dt, SauceDataType::Bitmap);
    /// assert_eq!(ft, 10);
    /// ```
    pub fn to_sauce(&self) -> (SauceDataType, u8) {
        match self {
            // RipScript special case
            BitmapFormat::RipScript => (SauceDataType::Character, 3),

            // Bitmap formats
            BitmapFormat::Gif => (SauceDataType::Bitmap, 0),
            BitmapFormat::Pcx => (SauceDataType::Bitmap, 1),
            BitmapFormat::LbmIff => (SauceDataType::Bitmap, 2),
            BitmapFormat::Tga => (SauceDataType::Bitmap, 3),
            BitmapFormat::Fli => (SauceDataType::Bitmap, 4),
            BitmapFormat::Flc => (SauceDataType::Bitmap, 5),
            BitmapFormat::Bmp => (SauceDataType::Bitmap, 6),
            BitmapFormat::Gl => (SauceDataType::Bitmap, 7),
            BitmapFormat::Dl => (SauceDataType::Bitmap, 8),
            BitmapFormat::Wpg => (SauceDataType::Bitmap, 9),
            BitmapFormat::Png => (SauceDataType::Bitmap, 10),
            BitmapFormat::Jpg => (SauceDataType::Bitmap, 11),
            BitmapFormat::Mpg => (SauceDataType::Bitmap, 12),
            BitmapFormat::Avi => (SauceDataType::Bitmap, 13),

            BitmapFormat::Unknown(dt, ft) => (*dt, *ft),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Bitmap and RIPScript capabilities.
///
/// `BitmapCapabilities` describes bitmap images, animations, and RIPScript,
/// including format, resolution, and color depth. Vector formats use
/// [`crate::VectorCapabilities`] instead.
///
/// # Fields
///
/// - **format**: The graphics format type (see [`BitmapFormat`])
/// - **width**: Image width in pixels
/// - **height**: Image height in pixels
/// - **pixel_depth**: Bits per pixel for bitmap images; 16 for RIPScript
///
/// # Format-Specific Values
///
/// **Bitmap Formats**: Store width, height, and pixel depth
/// - TInfo1: Width (pixels)
/// - TInfo2: Height (pixels)
/// - TInfo3: Bit depth (bits per pixel)
///
/// **RIPScript**: Fixed 640×350, 16-color display
/// - Width: 640 pixels (fixed)
/// - Height: 350 pixels (fixed)
/// - Pixel depth: This API reads and writes the value 16 (color count, not bits per pixel)
///
/// [`Self::new`] initializes all dimensions to 0, including for RIPScript.
/// RIPScript's fixed values are applied when encoding or parsing a header.
///
/// # Example
///
/// ```
/// use icy_sauce::{BitmapCapabilities, BitmapFormat};
///
/// let mut caps = BitmapCapabilities::new(BitmapFormat::Bmp);
/// caps.width = 1024;
/// caps.height = 768;
/// caps.pixel_depth = 24;
/// assert_eq!(caps.width, 1024);
/// ```
pub struct BitmapCapabilities {
    /// The graphics format type
    pub format: BitmapFormat,
    /// Image width in pixels
    pub width: u16,
    /// Image height in pixels
    pub height: u16,
    /// Color depth in bits per pixel for bitmaps; 16 (color count) for RIPScript
    pub pixel_depth: u16,
}

impl BitmapCapabilities {
    /// Create new graphics capabilities with zero dimensions.
    ///
    /// # Arguments
    ///
    /// * `graphics_format` - The [`BitmapFormat`] for this graphics file
    ///
    /// # Default Values
    ///
    /// - Width: 0
    /// - Height: 0
    /// - Pixel depth: 0
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::{BitmapCapabilities, BitmapFormat};
    /// let caps = BitmapCapabilities::new(BitmapFormat::Png);
    /// assert_eq!(caps.width, 0);
    /// ```
    pub fn new(graphics_format: BitmapFormat) -> Self {
        BitmapCapabilities {
            format: graphics_format,
            width: 0,
            height: 0,
            pixel_depth: 0,
        }
    }

    /// Serialize graphics capabilities into a SAUCE header.
    ///
    /// # Arguments
    ///
    /// * `header` - Mutable reference to the SAUCE header to populate
    ///
    /// # Errors
    ///
    /// Never fails (returns `Ok(())`).
    ///
    /// # Behavior
    ///
    /// Sets header fields based on format:
    /// - DataType = Bitmap, Character for RIPScript, or the preserved unknown data type
    /// - FileType = Format variant
    /// - TInfo1 = Width (or 640 for RIPScript)
    /// - TInfo2 = Height (or 350 for RIPScript)
    /// - TInfo3 = Pixel depth (or 16 for RIPScript)
    /// - TInfo4 = 0 for known formats; unchanged for unknown formats
    /// - TFlags = 0 for known formats; unchanged for unknown formats
    /// - TInfoS = Empty for known formats; unchanged for unknown formats
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::{BitmapCapabilities, BitmapFormat, Capabilities, SauceRecordBuilder};
    /// use icy_sauce::SauceDataType;
    ///
    /// let caps = BitmapCapabilities {
    ///     format: BitmapFormat::Png,
    ///     width: 800,
    ///     height: 600,
    ///     pixel_depth: 32,
    /// };
    /// let record = SauceRecordBuilder::default()
    ///     .capabilities(Capabilities::Bitmap(caps)).unwrap()
    ///     .build();
    /// assert_eq!(record.header().data_type, SauceDataType::Bitmap);
    /// assert_eq!(record.header().t_info1, 800);
    /// ```
    pub(crate) fn encode_into_header(&self, header: &mut SauceHeader) -> crate::Result<()> {
        let (data_type, file_type) = self.format.to_sauce();
        header.data_type = data_type;
        header.file_type = file_type;

        match self.format {
            BitmapFormat::RipScript => {
                // RipScript always has fixed values
                header.t_info1 = 640;
                header.t_info2 = 350;
                header.t_info3 = 16;
                header.t_info4 = 0;
                header.t_flags = 0;
                header.t_info_s.clear();
            }
            BitmapFormat::Gif
            | BitmapFormat::Pcx
            | BitmapFormat::LbmIff
            | BitmapFormat::Tga
            | BitmapFormat::Fli
            | BitmapFormat::Flc
            | BitmapFormat::Bmp
            | BitmapFormat::Gl
            | BitmapFormat::Dl
            | BitmapFormat::Wpg
            | BitmapFormat::Png
            | BitmapFormat::Jpg
            | BitmapFormat::Mpg
            | BitmapFormat::Avi => {
                // Bitmap formats store dimensions
                header.t_info1 = self.width;
                header.t_info2 = self.height;
                header.t_info3 = self.pixel_depth;
                header.t_info4 = 0;
                header.t_flags = 0;
                header.t_info_s.clear();
            }
            BitmapFormat::Unknown(_, _) => {
                // Preserve whatever values are there
                header.t_info1 = self.width;
                header.t_info2 = self.height;
                header.t_info3 = self.pixel_depth;
            }
        }

        Ok(())
    }

    /// Check if this is an animated format.
    ///
    /// Animated formats include FLI, FLC, GL, DL, MPEG, and AVI which have
    /// temporal information that should be considered during playback.
    ///
    /// # Returns
    ///
    /// `true` for animation formats, `false` for static formats.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::{BitmapCapabilities, BitmapFormat};
    /// assert!(BitmapCapabilities::new(BitmapFormat::Fli).is_animated());
    /// assert!(!BitmapCapabilities::new(BitmapFormat::Png).is_animated());
    /// ```
    pub fn is_animated(&self) -> bool {
        matches!(
            self.format,
            BitmapFormat::Fli
                | BitmapFormat::Flc
                | BitmapFormat::Gl
                | BitmapFormat::Dl
                | BitmapFormat::Mpg
                | BitmapFormat::Avi
        )
    }
}

impl TryFrom<&SauceHeader> for BitmapCapabilities {
    type Error = SauceError;

    /// Parse bitmap or RIPScript capabilities from a SAUCE header.
    ///
    /// Bitmap records read width, height, and pixel depth from TInfo1-3.
    /// RIPScript (Character with FileType 3) returns fixed values of 640, 350,
    /// and 16, regardless of the header's TInfo fields. Here 16 is the color
    /// count, not bits per pixel. Unknown bitmap file types are preserved.
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::UnsupportedDataType`] for data types other than
    /// Bitmap or Character with FileType 3, including Vector.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::{header::SauceHeader, SauceDataType, BitmapCapabilities, BitmapFormat};
    ///
    /// let mut header = SauceHeader {
    ///     data_type: SauceDataType::Bitmap,
    ///     file_type: 10, // PNG
    ///     t_info1: 640,
    ///     t_info2: 480,
    ///     t_info3: 24,
    ///     ..SauceHeader::default()
    /// };
    /// let caps = BitmapCapabilities::try_from(&header).unwrap();
    /// assert_eq!(caps.format, BitmapFormat::Png);
    /// assert_eq!((caps.width, caps.height, caps.pixel_depth), (640, 480, 24));
    ///
    /// header.data_type = SauceDataType::Character;
    /// header.file_type = 3; // RIPScript
    /// let caps = BitmapCapabilities::try_from(&header).unwrap();
    /// assert_eq!((caps.width, caps.height, caps.pixel_depth), (640, 350, 16));
    ///
    /// header.data_type = SauceDataType::Vector;
    /// assert!(BitmapCapabilities::try_from(&header).is_err());
    /// ```
    fn try_from(header: &SauceHeader) -> crate::Result<Self> {
        let graphics_format = BitmapFormat::from_sauce(header.data_type, header.file_type);
        let (width, height, pixel_depth) = match header.data_type {
            SauceDataType::Character if header.file_type == 3 => (640, 350, 16),
            SauceDataType::Bitmap => (header.t_info1, header.t_info2, header.t_info3),
            _ => return Err(SauceError::UnsupportedDataType(header.data_type)),
        };
        Ok(BitmapCapabilities {
            format: graphics_format,
            width,
            height,
            pixel_depth,
        })
    }
}
