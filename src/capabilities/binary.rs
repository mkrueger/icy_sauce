//! Binary text and XBIN format capabilities as specified in the SAUCE v00 standard.
//!
//! This module provides types for describing binary text files (BinaryText and XBIN formats)
//! stored with SAUCE metadata. Both are text-based but with different storage mechanisms.
//!
//! # Formats Supported
//!
//! ## BinaryText (DataType = 5)
//!
//! Raw binary text data with width encoded in FileType:
//! - **Width**: Even number, 2-510 characters (stored as FileType = width/2)
//! - **Height**: Derived from FileSize (width × 2 bytes per row)
//! - **Colors**: Supports ICE colors (16 background colors)
//! - **Font**: Optional (max 22 bytes)
//! - **Rendering**: Letter spacing and aspect ratio support
//!
//! ## XBIN (DataType = 6)
//!
//! Extended Binary format with explicit dimensions:
//! - **Width**: TInfo1 (0-65535)
//! - **Height**: TInfo2 (0-65535)
//! - **Colors**: Basic colors only (no ICE)
//! - **Font**: Not supported
//! - **Rendering**: No special flags
//!
//! # Example
//!
//! ```
//! use icy_sauce::BinaryCapabilities;
//!
//! // Create BinaryText with 80-character width
//! let binary_text = BinaryCapabilities::binary_text(80).unwrap();
//! assert_eq!(binary_text.columns, 80);
//!
//! // Create XBIN with explicit dimensions
//! let xbin = BinaryCapabilities::xbin(100, 50).unwrap();
//! assert_eq!(xbin.columns, 100);
//! assert_eq!(xbin.lines, 50);
//! ```

use bstr::BString;

use crate::{SauceDataType, SauceError, header::SauceHeader};

use crate::character::{
    ANSI_ASPECT_RATIO_LEGACY, ANSI_ASPECT_RATIO_SQUARE, ANSI_ASPECT_RATIO_STRETCH,
    ANSI_FLAG_NON_BLINK_MODE, ANSI_LETTER_SPACING_8PX, ANSI_LETTER_SPACING_9PX,
    ANSI_LETTER_SPACING_LEGACY, ANSI_MASK_ASPECT_RATIO, ANSI_MASK_LETTER_SPACING, AspectRatio,
    LetterSpacing,
};

/// Binary text format discriminator.
///
/// Distinguishes between two SAUCE data types for binary text content:
/// - **BinaryText** (DataType = 5): Width encoded in FileType field
/// - **XBin** (DataType = 6): Explicit width and height in TInfo fields
///
/// # Example
///
/// ```
/// use icy_sauce::BinaryFormat;
/// use icy_sauce::SauceDataType;
///
/// let fmt = BinaryFormat::from_data_type(SauceDataType::BinaryText).unwrap();
/// assert_eq!(fmt, BinaryFormat::BinaryText);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFormat {
    /// BinaryText format (DataType = 5) with width encoded in FileType
    BinaryText,
    /// XBIN format (DataType = 6) with explicit dimensions
    XBin,
}

impl BinaryFormat {
    /// Parse format from SAUCE DataType.
    ///
    /// # Arguments
    ///
    /// * `dt` - The SAUCE DataType field
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::UnsupportedDataType`] if the data type is neither
    /// BinaryText nor XBin.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::BinaryFormat;
    /// use icy_sauce::SauceDataType;
    ///
    /// let fmt = BinaryFormat::from_data_type(SauceDataType::XBin).unwrap();
    /// assert_eq!(fmt, BinaryFormat::XBin);
    /// ```
    pub fn from_data_type(dt: SauceDataType) -> crate::Result<Self> {
        match dt {
            SauceDataType::BinaryText => Ok(BinaryFormat::BinaryText),
            SauceDataType::XBin => Ok(BinaryFormat::XBin),
            _ => Err(SauceError::UnsupportedDataType(dt)),
        }
    }
}

/// Binary text and XBIN format capabilities.
///
/// `BinaryCapabilities` describes binary text files (BinaryText or XBIN format) with their dimensions,
/// rendering options, and fonts. The structure differs based on format:
///
/// # BinaryText (DataType = 5)
///
/// - **Width**: Even number from 2 to 510 characters (stored as FileType = width/2)
/// - **Height**: Derived from FileSize (bytes_per_row = width × 2)
/// - **Flags**: ANSi flags for ICE colors, letter spacing, aspect ratio
/// - **Font**: Optional font name (max 22 bytes, zero-padded in SAUCE)
///
/// SAUCE Field Mappings:
/// - FileType: width/2 (must result in even number ≤ 510)
/// - TInfo1-4: All 0
/// - TFlags: ANSi flags (ICE, letter spacing, aspect ratio)
/// - TInfoS: Font name (zero-padded)
///
/// # XBIN (DataType = 6)
///
/// - **Width**: Arbitrary width from TInfo1 (0-65535)
/// - **Height**: Arbitrary height from TInfo2 (0-65535)
/// - **Flags**: Always 0 (no special rendering)
/// - **Font**: Not supported (always None)
///
/// SAUCE Field Mappings:
/// - FileType: Always 0
/// - TInfo1: Width
/// - TInfo2: Height
/// - TInfo3-4: All 0
/// - TFlags: Always 0
/// - TInfoS: Always empty
///
/// # Example
///
/// ```
/// use icy_sauce::{BinaryCapabilities, BinaryFormat};
/// use icy_sauce::{LetterSpacing, AspectRatio};
/// use bstr::BString;
///
/// let mut binary_text = BinaryCapabilities::binary_text(80).unwrap();
/// binary_text.ice_colors = true;
/// binary_text.letter_spacing = LetterSpacing::NinePixel;
/// binary_text.aspect_ratio = AspectRatio::Square;
/// binary_text.set_font(BString::from("IBM VGA")).unwrap();
/// assert_eq!(binary_text.columns, 80);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryCapabilities {
    /// Binary text format (BinaryText or XBIN)
    pub format: BinaryFormat,
    /// Width in characters
    pub columns: u16,
    /// Height in lines (0 for BinaryText, explicit for XBIN)
    pub lines: u16,

    /// Whether ICE colors (16 background colors) are enabled
    pub ice_colors: bool,
    /// Letter spacing mode (8px vs 9px)
    pub letter_spacing: LetterSpacing,
    /// Pixel aspect ratio for rendering
    pub aspect_ratio: AspectRatio,
    /// Optional font name (max 22 bytes)
    font_opt: Option<BString>,
}

impl BinaryCapabilities {
    /// Create BinaryText format capabilities.
    ///
    /// # Arguments
    ///
    /// * `width` - Width in characters (must be even, 2-510)
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::BinFileWidthLimitExceeded`] if width is:
    /// - Zero
    /// - Odd (not evenly divisible by 2)
    /// - Greater than 510
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::BinaryCapabilities;
    ///
    /// let binary_text = BinaryCapabilities::binary_text(80).unwrap();
    /// assert_eq!(binary_text.columns, 80);
    /// ```
    pub fn binary_text(columns: u16) -> crate::Result<Self> {
        if columns == 0 || columns % 2 != 0 || columns > 510 {
            return Err(SauceError::BinFileWidthLimitExceeded(columns as i32));
        }
        Ok(Self {
            format: BinaryFormat::BinaryText,
            columns,
            lines: 0,
            ice_colors: false,
            letter_spacing: LetterSpacing::Legacy,
            aspect_ratio: AspectRatio::Legacy,
            font_opt: None,
        })
    }

    /// Create XBIN format capabilities.
    ///
    /// # Arguments
    ///
    /// * `width` - Width in characters (must be > 0)
    /// * `lines` - Height in lines (must be > 0)
    ///
    /// NOTE: This is the only info saved in a sauce record for xbin.
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::UnsupportedDataType`] if width or lines is 0.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::BinaryCapabilities;
    ///
    /// let xbin = BinaryCapabilities::xbin(100, 50).unwrap();
    /// assert_eq!(xbin.columns, 100);
    /// assert_eq!(xbin.lines, 50);
    /// ```
    pub fn xbin(columns: u16, lines: u16) -> crate::Result<Self> {
        // Spec doesn't forbid 0 explicitly, but width/height of 0 are meaningless for XBin.
        if columns == 0 {
            return Err(SauceError::UnsupportedDataType(SauceDataType::XBin)); // Re-use existing error
        }
        if lines == 0 {
            return Err(SauceError::UnsupportedDataType(SauceDataType::XBin));
        }
        Ok(Self {
            format: BinaryFormat::XBin,
            columns,
            lines,
            ice_colors: false,
            letter_spacing: LetterSpacing::Legacy,
            aspect_ratio: AspectRatio::Legacy,
            font_opt: None,
        })
    }

    /// Get a reference to the optional font name.
    ///
    /// # Returns
    ///
    /// `Some(&font)` if a font has been set, or `None` if not.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::BinaryCapabilities;
    /// let caps = BinaryCapabilities::binary_text(80).unwrap();
    /// assert_eq!(caps.font(), None);
    /// ```
    pub fn font(&self) -> Option<&BString> {
        self.font_opt.as_ref()
    }

    /// Set the font name with validation.
    ///
    /// # Arguments
    ///
    /// * `font` - The font name to set (max 22 bytes), or empty to clear
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::FontNameTooLong`] if the font name exceeds 22 bytes.
    ///
    /// # Behavior
    ///
    /// - Passing an empty `BString` clears the font (equivalent to [`clear_font`](Self::clear_font))
    /// - Non-empty strings up to 22 bytes are stored
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::BinaryCapabilities;
    /// use bstr::BString;
    /// let mut caps = BinaryCapabilities::binary_text(80).unwrap();
    /// caps.set_font(BString::from("IBM VGA")).unwrap();
    /// assert_eq!(caps.font(), Some(&BString::from("IBM VGA")));
    /// ```
    pub fn set_font(&mut self, font: BString) -> crate::Result<()> {
        if font.len() > 22 {
            return Err(SauceError::FontNameTooLong(font.len()));
        }
        if font.is_empty() {
            self.font_opt = None;
            return Ok(());
        }
        self.font_opt = Some(font);
        Ok(())
    }

    /// Clear the font name, setting it to `None`.
    ///
    /// This is equivalent to calling [`set_font`](Self::set_font) with an empty `BString`.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::BinaryCapabilities;
    /// let mut caps = BinaryCapabilities::binary_text(80).unwrap();
    /// caps.remove_font();
    /// assert_eq!(caps.font(), None);
    /// ```
    pub fn remove_font(&mut self) {
        self.font_opt = None;
    }

    /// Parse binary text capabilities from a SAUCE header.
    ///
    /// # Arguments
    ///
    /// * `header` - The SAUCE header to parse
    ///
    /// # Returns
    ///
    /// Binary text capabilities extracted from header fields.
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::UnsupportedDataType`] if DataType is neither BinaryText nor XBIN.
    /// Returns [`SauceError::BinFileWidthLimitExceeded`] if width is 0 or invalid.
    ///
    /// # Format-Specific Parsing
    ///
    /// **BinaryText**:
    /// - Width: (FileType × 2)
    /// - Flags: TFlags field
    /// - Font: TInfoS field (if non-empty)
    ///
    /// **XBIN**:
    /// - Width: TInfo1
    /// - Height: TInfo2
    /// - Flags/Font: Ignored
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::{header::SauceHeader, SauceDataType};
    /// use icy_sauce::BinaryCapabilities;
    ///
    /// let mut header = SauceHeader::default();
    /// header.data_type = SauceDataType::BinaryText;
    /// header.file_type = 40; // width = 80
    /// use std::convert::TryFrom;
    /// let caps = BinaryCapabilities::try_from(&header).unwrap();
    /// assert_eq!(caps.columns, 80);
    /// ```
    /// This example uses the `TryFrom<&SauceHeader>` implementation on `BinaryCapabilities`.
    /// The bespoke `from(&SauceHeader)` constructor has been removed in favor of the
    /// standard conversion trait.

    /// Serialize binary text capabilities into a SAUCE header.
    ///
    /// # Arguments
    ///
    /// * `header` - Mutable reference to the SAUCE header to populate
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::BinFileWidthLimitExceeded`] if BinaryText width is invalid.
    ///
    /// # Behavior
    ///
    /// **BinaryText**:
    /// - Sets DataType = BinaryText
    /// - FileType = width/2
    /// - TInfo1-4: All 0
    /// - TFlags: Self.flags (ANSi flags)
    /// - TInfoS: Font name (if present)
    ///
    /// **XBIN**:
    /// - Sets DataType = XBIN
    /// - FileType: 0
    /// - TInfo1: width, TInfo2: height
    /// - TInfo3-4: All 0
    /// - TFlags: 0, TInfoS: Empty
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::BinaryCapabilities;
    /// use icy_sauce::header::SauceHeader;
    ///
    /// let caps = BinaryCapabilities::binary_text(80).unwrap();
    /// let mut header = SauceHeader::default();
    /// caps.encode_into_header(&mut header).unwrap();
    /// assert_eq!(header.file_type, 40);
    /// ```
    pub fn encode_into_header(&self, header: &mut SauceHeader) -> crate::Result<()> {
        match self.format {
            BinaryFormat::BinaryText => {
                header.data_type = SauceDataType::BinaryText;
                if self.columns == 0 || self.columns % 2 != 0 || self.columns > 510 {
                    return Err(SauceError::BinFileWidthLimitExceeded(self.columns as i32));
                }
                header.file_type = (self.columns / 2) as u8;
                header.t_info1 = 0;
                header.t_info2 = 0;
                header.t_info3 = 0;
                header.t_info4 = 0;

                // Build flags byte
                header.t_flags = if self.ice_colors {
                    ANSI_FLAG_NON_BLINK_MODE
                } else {
                    0
                };

                // Add letter spacing bits
                header.t_flags |= match self.letter_spacing {
                    LetterSpacing::Legacy => ANSI_LETTER_SPACING_LEGACY,
                    LetterSpacing::EightPixel => ANSI_LETTER_SPACING_8PX,
                    LetterSpacing::NinePixel => ANSI_LETTER_SPACING_9PX,
                    LetterSpacing::Reserved => ANSI_LETTER_SPACING_LEGACY, // fallback
                };

                // Add aspect ratio bits
                header.t_flags |= match self.aspect_ratio {
                    AspectRatio::Legacy => ANSI_ASPECT_RATIO_LEGACY,
                    AspectRatio::LegacyDevice => ANSI_ASPECT_RATIO_STRETCH,
                    AspectRatio::Square => ANSI_ASPECT_RATIO_SQUARE,
                    AspectRatio::Reserved => ANSI_ASPECT_RATIO_LEGACY, // fallback
                };

                if let Some(font) = &self.font_opt {
                    header.t_info_s.clone_from(font);
                } else {
                    header.t_info_s.clear();
                }
            }
            BinaryFormat::XBin => {
                header.data_type = SauceDataType::XBin;
                header.file_type = 0;
                header.t_info1 = self.columns;
                header.t_info2 = self.lines;
                header.t_info3 = 0;
                header.t_info4 = 0;
                header.t_flags = 0;
                header.t_info_s.clear();
            }
        }
        Ok(())
    }

    /// Calculate BinaryText height from file size.
    ///
    /// For BinaryText format, height is not stored explicitly but can be derived
    /// from the original file size and width using: height = file_size / (width × 2)
    ///
    /// # Arguments
    ///
    /// * `file_size` - The original file size in bytes (from SAUCE FileSize field)
    ///
    /// # Returns
    ///
    /// `Some(height)` if height can be calculated, or `None` if:
    /// - This is XBIN format (not BinaryText)
    /// - Width is 0
    /// - File size is 0
    /// - Calculated height would exceed u16::MAX
    ///
    /// # Formula
    ///
    /// Each character row requires `width × 2` bytes (character + attribute).
    /// Height = FileSize ÷ (Width × 2)
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::BinaryCapabilities;
    ///
    /// let binary_text = BinaryCapabilities::binary_text(80).unwrap();
    /// let file_size = 80 * 2 * 25; // 80 chars wide, 25 lines
    /// let height = binary_text.binary_text_height_from_file_size(file_size as u32).unwrap();
    /// assert_eq!(height, 25);
    /// ```
    pub fn binary_text_height_from_file_size(&self, file_size: u32) -> Option<u16> {
        if self.format != BinaryFormat::BinaryText {
            return None;
        }
        if self.columns == 0 || file_size == 0 {
            return None;
        }
        let bytes_per_row = self.columns as u32 * 2; // char + attribute
        let h = file_size / bytes_per_row;
        if h == 0 || h > u16::MAX as u32 {
            None
        } else {
            Some(h as u16)
        }
    }
}

impl TryFrom<&SauceHeader> for BinaryCapabilities {
    type Error = SauceError;
    fn try_from(header: &SauceHeader) -> crate::Result<Self> {
        let format = BinaryFormat::from_data_type(header.data_type)?;
        match format {
            BinaryFormat::BinaryText => {
                let width = (header.file_type as u16) * 2;
                if width == 0 {
                    return Err(SauceError::BinFileWidthLimitExceeded(0));
                }
                let ice_colors =
                    (header.t_flags & ANSI_FLAG_NON_BLINK_MODE) == ANSI_FLAG_NON_BLINK_MODE;
                let letter_spacing = match header.t_flags & ANSI_MASK_LETTER_SPACING {
                    ANSI_LETTER_SPACING_LEGACY => LetterSpacing::Legacy,
                    ANSI_LETTER_SPACING_8PX => LetterSpacing::EightPixel,
                    ANSI_LETTER_SPACING_9PX => LetterSpacing::NinePixel,
                    _ => LetterSpacing::Reserved,
                };
                let aspect_ratio = match header.t_flags & ANSI_MASK_ASPECT_RATIO {
                    ANSI_ASPECT_RATIO_LEGACY => AspectRatio::Legacy,
                    ANSI_ASPECT_RATIO_STRETCH => AspectRatio::LegacyDevice,
                    ANSI_ASPECT_RATIO_SQUARE => AspectRatio::Square,
                    _ => AspectRatio::Reserved,
                };
                let font_opt = if header.t_info_s.is_empty() {
                    None
                } else {
                    Some(header.t_info_s.clone())
                };
                Ok(Self {
                    format,
                    columns: width,
                    lines: 0,
                    ice_colors,
                    letter_spacing,
                    aspect_ratio,
                    font_opt,
                })
            }
            BinaryFormat::XBin => Ok(Self {
                format,
                columns: header.t_info1,
                lines: header.t_info2,
                ice_colors: false,
                letter_spacing: LetterSpacing::Legacy,
                aspect_ratio: AspectRatio::Legacy,
                font_opt: None,
            }),
        }
    }
}
