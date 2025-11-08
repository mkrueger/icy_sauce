//! Graphics format capabilities as specified in the SAUCE v00 standard.
//!
//! This module provides types for describing graphics files (bitmap, vector, and RIPScript)
//! stored with SAUCE metadata. Supported formats include common image, animation, and vector types.
//!
//! # Format Categories
//!
//! **Bitmap Formats** (DataType=1): Raster image and animation formats
//! - GIF, PCX, LBM/IFF, TGA, FLI, FLC, BMP, GL, DL, WPG, PNG, JPG, MPEG, AVI
//!
//! **Vector Formats** (DataType=2): Scalable graphics formats
//! - DXF, DWG, WPG (vector), 3DS
//!
//! **Special Format**: RIPScript (DataType=0 with FileType=3)
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
/// Graphics format types supported by the SAUCE v00 specification.
///
/// `BitmapFormat` enumerates all bitmap, vector, and special graphics formats
/// that can be stored with SAUCE metadata. Formats are organized by data type:
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
/// # Vector Formats (DataType::Vector)
///
/// Scalable graphics formats without pixel dimensions (values 0-3):
/// - **Dxf** (0): AutoCAD Drawing Exchange Format
/// - **Dwg** (1): AutoCAD Drawing Format
/// - **WpgVector** (2): WordPerfect Graphics (vector)
/// - **ThreeDs** (3): Autodesk 3D Studio
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
    /// - Vector formats: DataType=Vector with FileType 0-3
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
/// Graphics/pixel format capabilities.
///
/// `BitmapCapabilities` describes bitmap or vector graphics files stored with SAUCE metadata,
/// including format type, resolution, and color depth for bitmap images.
///
/// # Fields
///
/// - **format**: The graphics format type (see [`BitmapFormat`])
/// - **width**: Image width in pixels (0 for vector formats)
/// - **height**: Image height in pixels (0 for vector formats)
/// - **pixel_depth**: Color depth in bits per pixel (0 for vector formats)
///
/// # Format-Specific Values
///
/// **Bitmap Formats**: Store width, height, and pixel depth
/// - TInfo1: Width (pixels)
/// - TInfo2: Height (pixels)
/// - TInfo3: Bit depth (bits per pixel)
///
/// **Vector Formats**: No pixel information (all dimensions = 0)
/// - TInfo1-3: Always 0
///
/// **RIPScript**: Fixed 640×350, 16-color display
/// - Width: 640 pixels (fixed)
/// - Height: 350 pixels (fixed)
/// - Pixel depth: 16 colors (fixed)
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
    /// Color depth in bits per pixel
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

    /// Parse graphics capabilities from a SAUCE header.
    ///
    /// Extracts format and dimensions based on the data type and format.
    ///
    /// # Arguments
    ///
    /// * `header` - The SAUCE header to parse
    ///
    /// # Returns
    ///
    /// Graphics capabilities with format-specific dimensions and pixel depth.
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::UnsupportedDataType`] if DataType is not Bitmap or Vector.
    ///
    /// # Format-Specific Parsing
    ///
    /// - **Bitmap/Vector formats**: Width from TInfo1, Height from TInfo2, Depth from TInfo3
    /// - **RIPScript**: Fixed 640×350 @ 16 colors (4 bits)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Internal parsing example (ignored because `BitmapCapabilities::from` is not public)
    /// use icy_sauce::{header::SauceHeader, SauceDataType};
    /// use icy_sauce::{BitmapCapabilities, BitmapFormat};
    ///
    /// let mut header = SauceHeader::default();
    /// header.data_type = SauceDataType::Bitmap;
    /// header.file_type = 10; // PNG
    /// header.t_info1 = 640;
    /// header.t_info2 = 480;
    /// header.t_info3 = 24;
    /// use std::convert::TryFrom;
    /// let caps = BitmapCapabilities::try_from(&header).unwrap();
    /// assert_eq!(caps.format, BitmapFormat::Png);
    /// assert_eq!(caps.width, 640);
    /// ```
    /// The bespoke internal `from(&SauceHeader)` has been replaced by a `TryFrom<&SauceHeader>`
    /// implementation for idiomatic conversion.

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
    /// - DataType = Bitmap or Vector (based on format)
    /// - FileType = Format variant
    /// - TInfo1 = Width (or 640 for RIPScript)
    /// - TInfo2 = Height (or 350 for RIPScript)
    /// - TInfo3 = Pixel depth (or 4 for RIPScript)
    /// - TInfo4 = 0
    /// - TFlags = 0
    /// - TInfoS = Empty
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Internal serialization example (ignored because `encode_into_header` is not public)
    /// use icy_sauce::{BitmapCapabilities, BitmapFormat};
    /// use icy_sauce::header::SauceHeader;
    /// use icy_sauce::SauceDataType;
    ///
    /// let caps = BitmapCapabilities {
    ///     format: BitmapFormat::Png,
    ///     width: 800,
    ///     height: 600,
    ///     pixel_depth: 32,
    /// };
    /// let mut header = SauceHeader::default();
    /// caps.encode_into_header(&mut header).unwrap();
    /// assert_eq!(header.data_type, SauceDataType::Bitmap);
    /// assert_eq!(header.t_info1, 800);
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
    fn try_from(header: &SauceHeader) -> crate::Result<Self> {
        let graphics_format = BitmapFormat::from_sauce(header.data_type, header.file_type);
        let (width, height, pixel_depth) = match header.data_type {
            SauceDataType::Character if header.file_type == 3 => (640, 350, 16),
            SauceDataType::Bitmap => (header.t_info1, header.t_info2, header.t_info3),
            _ => return Err(SauceError::UnsupportedDataType(header.data_type)),
        };
        Ok(BitmapCapabilities { format: graphics_format, width, height, pixel_depth })
    }
}
