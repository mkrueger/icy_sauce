//! Character/text file format capabilities as specified in the SAUCE v00 standard.
//!
//! This module provides types for describing text-based file formats (ASCII, ANSI, RIPScript, etc.),
//! their character dimensions, and rendering options like ice colors and letter spacing.
//!
//! # Formats Supported
//!
//! - **ASCII**: Plain ASCII text files
//! - **ANSI**: ANSI escape sequences for color and formatting
//! - **ANSiMation**: ANSI with timing for animation playback
//! - **RIPScript**: Remote Imaging Protocol (pixel-based)
//! - **PCBoard**: PCBoard BBS text format
//! - **Avatar**: Avatar graphics format
//! - **HTML**: Hypertext Markup Language
//! - **Source Code**: Generic source code files
//! - **TundraDraw**: TundraDraw BBS graphics
//!
//! # Example
//!
//! ```
//! use icy_sauce::{CharacterCapabilities, CharacterFormat, LetterSpacing, AspectRatio};
//! use bstr::BString;
//!
//! let caps = CharacterCapabilities::with_font(
//!     CharacterFormat::Ansi,
//!     80,  // width
//!     25,  // height
//!     true,  // ice colors enabled
//!     LetterSpacing::NinePixel,
//!     AspectRatio::Square,
//!     Some(BString::from("IBM VGA")),
//! )?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use bstr::BString;

use crate::{SauceDataType, SauceError, header::SauceHeader};

/// ANSI flags bitmask for non-blink mode (ice colors).
/// When set (bit 0), the 16 background colors become available instead of blinking.
const ANSI_FLAG_NON_BLINK_MODE: u8 = 0b0000_0001;
/// ANSI flags bitmask for letter spacing (bits 1-2).
/// Values: 00=Legacy, 01=8-pixel, 10=9-pixel, 11=Reserved
const ANSI_MASK_LETTER_SPACING: u8 = 0b0000_0110;
const ANSI_LETTER_SPACING_LEGACY: u8 = 0b0000_0000;
const ANSI_LETTER_SPACING_8PX: u8 = 0b0000_0010;
const ANSI_LETTER_SPACING_9PX: u8 = 0b0000_0100;

/// ANSI flags bitmask for aspect ratio (bits 3-4).
/// Values: 00=Legacy, 01=LegacyDevice (needs stretching), 10=Square (modern), 11=Reserved
const ANSI_MASK_ASPECT_RATIO: u8 = 0b0001_1000;
const ANSI_ASPECT_RATIO_LEGACY: u8 = 0b0000_0000;
const ANSI_ASPECT_RATIO_STRETCH: u8 = 0b0000_1000;
const ANSI_ASPECT_RATIO_SQUARE: u8 = 0b0001_0000;

#[derive(Debug, Clone, Copy, PartialEq)]
/// Character format types as specified in the SAUCE v00 specification.
///
/// Each format represents a different text or graphics encoding standard used by
/// the BBS and retro computing community. The numeric values correspond to the
/// FileType field in the SAUCE header.
pub enum CharacterFormat {
    /// Plain ASCII text (value 0)
    Ascii,
    /// ANSI escape sequence art (value 1)
    Ansi,
    /// ANSI with animation/timing (value 2)
    AnsiMation,
    /// Remote Imaging Protocol - pixel-based graphics (value 3)
    RipScript,
    /// PCBoard BBS format (value 4)
    PCBoard,
    /// Avatar graphics format (value 5)
    Avatar,
    /// Hypertext Markup Language (value 6)
    Html,
    /// Generic source code (value 7)
    Source,
    /// TundraDraw format (value 8)
    TundraDraw,
    /// Unknown or unsupported format (preserves raw value)
    Unknown(u8),
}

impl CharacterFormat {
    /// Parse a character format from the SAUCE FileType byte.
    ///
    /// # Arguments
    ///
    /// * `file_type` - The 8-bit FileType value from the SAUCE header
    ///
    /// # Returns
    ///
    /// Returns the corresponding [`CharacterFormat`], or [`CharacterFormat::Unknown`]
    /// if the value is not recognized.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::CharacterFormat;
    /// let format = CharacterFormat::from_sauce(1);
    /// assert_eq!(format, CharacterFormat::Ansi);
    /// ```
    pub fn from_sauce(file_type: u8) -> Self {
        match file_type {
            0 => CharacterFormat::Ascii,
            1 => CharacterFormat::Ansi,
            2 => CharacterFormat::AnsiMation,
            3 => CharacterFormat::RipScript,
            4 => CharacterFormat::PCBoard,
            5 => CharacterFormat::Avatar,
            6 => CharacterFormat::Html,
            7 => CharacterFormat::Source,
            8 => CharacterFormat::TundraDraw,
            _ => CharacterFormat::Unknown(file_type),
        }
    }

    /// Convert to the SAUCE FileType byte representation.
    ///
    /// # Returns
    ///
    /// The 8-bit FileType value suitable for writing to a SAUCE header.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::CharacterFormat;
    /// assert_eq!(CharacterFormat::Ansi.to_sauce(), 1);
    /// assert_eq!(CharacterFormat::Unknown(99).to_sauce(), 99);
    /// ```
    pub fn to_sauce(&self) -> u8 {
        match self {
            CharacterFormat::Ascii => 0,
            CharacterFormat::Ansi => 1,
            CharacterFormat::AnsiMation => 2,
            CharacterFormat::RipScript => 3,
            CharacterFormat::PCBoard => 4,
            CharacterFormat::Avatar => 5,
            CharacterFormat::Html => 6,
            CharacterFormat::Source => 7,
            CharacterFormat::TundraDraw => 8,
            CharacterFormat::Unknown(ft) => *ft,
        }
    }

    /// Check if this format supports ANSI flags (ice colors, letter spacing, aspect ratio).
    ///
    /// Only ASCII, ANSI, and ANSiMation formats support the extended rendering options
    /// stored in the TFlags byte of the SAUCE header.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::CharacterFormat;
    /// assert!(CharacterFormat::Ansi.supports_ansi_flags());
    /// assert!(!CharacterFormat::Html.supports_ansi_flags());
    /// ```
    pub fn supports_ansi_flags(&self) -> bool {
        matches!(
            self,
            CharacterFormat::Ascii | CharacterFormat::Ansi | CharacterFormat::AnsiMation
        )
    }

    /// Check if this format stores character grid dimensions (width/height).
    ///
    /// Most formats store character dimensions in TInfo1 (width) and TInfo2 (height),
    /// except HTML and Source which have variable dimensions.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::CharacterFormat;
    /// assert!(CharacterFormat::Ansi.has_dimensions());
    /// assert!(!CharacterFormat::Html.has_dimensions());
    /// ```
    pub fn has_dimensions(&self) -> bool {
        matches!(
            self,
            CharacterFormat::Ascii
                | CharacterFormat::Ansi
                | CharacterFormat::AnsiMation
                | CharacterFormat::PCBoard
                | CharacterFormat::Avatar
                | CharacterFormat::TundraDraw
        )
    }

    /// Check if this format is a streaming format with variable height.
    ///
    /// Streaming formats can have content of any height, unlike fixed-size animations.
    /// RIPScript and HTML are not considered streaming despite being variable-height.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::CharacterFormat;
    /// assert!(CharacterFormat::Ansi.is_stream());
    /// assert!(!CharacterFormat::AnsiMation.is_stream());
    /// ```
    pub fn is_stream(&self) -> bool {
        matches!(
            self,
            CharacterFormat::Ascii
                | CharacterFormat::Ansi
                | CharacterFormat::PCBoard
                | CharacterFormat::Avatar
                | CharacterFormat::TundraDraw
        )
    }

    /// Check if this format is an animation that requires fixed screen dimensions.
    ///
    /// ANSiMation requires both width and height to be fixed for proper playback timing.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::CharacterFormat;
    /// assert!(CharacterFormat::AnsiMation.is_animation());
    /// assert!(!CharacterFormat::Ansi.is_animation());
    /// ```
    pub fn is_animation(&self) -> bool {
        matches!(self, CharacterFormat::AnsiMation)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Letter spacing mode for ANSI text, stored in bits 1-2 of TFlags.
///
/// Letter spacing controls the horizontal character width used when rendering,
/// affecting how tight or loose the text appears on screen.
pub enum LetterSpacing {
    /// Legacy / undefined spacing (value 0)
    Legacy,
    /// 8-pixel character width (value 1)
    EightPixel,
    /// 9-pixel character width (value 2)
    NinePixel,
    /// Reserved value (value 3) - not standardized
    Reserved,
}

impl LetterSpacing {
    /// Check if this format uses the 9-pixel letter spacing.
    ///
    /// Returns `true` only for [`LetterSpacing::NinePixel`], which indicates
    /// that modern 9-pixel wide characters should be used instead of 8-pixel legacy.
    pub fn use_letter_spacing(self) -> bool {
        self == LetterSpacing::NinePixel
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Pixel aspect ratio for ANSI text rendering, stored in bits 3-4 of TFlags.
///
/// The aspect ratio determines how pixels should be displayed on different hardware:
/// - Legacy systems had rectangular (non-square) pixels
/// - Modern displays use square pixels
pub enum AspectRatio {
    /// Legacy (undefined) aspect ratio (value 0)
    Legacy,
    /// Legacy device ratio requiring vertical stretch (value 1)
    LegacyDevice,
    /// Square pixels, modern aspect ratio (value 2)
    Square,
    /// Reserved value (value 3) - not standardized
    Reserved,
}

impl AspectRatio {
    /// Check if this aspect ratio requires stretching.
    ///
    /// Returns `true` only for [`AspectRatio::LegacyDevice`], which indicates
    /// that content should be vertically stretched to correct for rectangular pixels.
    pub fn use_aspect_ratio(self) -> bool {
        self == AspectRatio::LegacyDevice
    }
}

/// Character/text file format capabilities and display options.
///
/// `CharacterCapabilities` describes how a text-based SAUCE file should be displayed, including:
/// - The character encoding format (ANSI, ASCII, RIPScript, etc.)
/// - Screen dimensions in characters (width × height)
/// - Rendering options like ice colors and letter spacing
/// - Optional font name for the display
///
/// # Field Constraints
///
/// - **Width/Height**: Range depends on format; typically 0-65535
/// - **Font Name**: Max 22 bytes (space-padded in SAUCE header TInfoS field)
/// - **ICE Colors**: Only for ANSI-compatible formats
/// - **Letter Spacing/Aspect Ratio**: Only for ANSI-compatible formats
///
/// # Example
///
/// ```
/// use icy_sauce::{CharacterCapabilities, CharacterFormat, LetterSpacing, AspectRatio};
/// use bstr::BString;
///
/// let caps = CharacterCapabilities::new(CharacterFormat::Ansi);
/// assert_eq!(caps.columns, 80);
/// assert_eq!(caps.lines, 25);
/// ```
#[derive(Debug, Clone)]
pub struct CharacterCapabilities {
    /// The character encoding format
    pub format: CharacterFormat,
    /// Width in characters
    pub columns: u16,
    /// Height in characters (lines)
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

impl CharacterCapabilities {
    /// Create a new `CharacterCapabilities` with default values.
    ///
    /// # Arguments
    ///
    /// * `character_format` - The [`CharacterFormat`] for this content
    ///
    /// # Defaults
    ///
    /// - Width: 80 characters
    /// - Height: 25 characters
    /// - ICE colors: disabled
    /// - Letter spacing: Legacy (8-pixel)
    /// - Aspect ratio: Legacy
    /// - Font: None
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::{CharacterCapabilities, CharacterFormat};
    /// let caps = CharacterCapabilities::new(CharacterFormat::Ansi);
    /// assert_eq!(caps.columns, 80);
    /// assert_eq!(caps.lines, 25);
    /// assert!(!caps.ice_colors);
    /// ```
    pub fn new(character_format: CharacterFormat) -> Self {
        CharacterCapabilities {
            format: character_format,
            columns: 80,
            lines: 25,
            ice_colors: false,
            letter_spacing: LetterSpacing::Legacy,
            aspect_ratio: AspectRatio::Legacy,
            font_opt: None,
        }
    }

    /// Create a new `CharacterCapabilities` with all fields specified and font validation.
    ///
    /// This constructor is more explicit than [`new`](Self::new) and validates
    /// all parameters, especially the font name length.
    ///
    /// # Arguments
    ///
    /// * `format` - The character encoding format
    /// * `width` - Display width in characters
    /// * `height` - Display height in characters (lines)
    /// * `ice_colors` - Whether ICE colors (16 background colors) are available
    /// * `letter_spacing` - Letter spacing mode for rendering
    /// * `aspect_ratio` - Pixel aspect ratio for rendering
    /// * `font` - Optional font name (max 22 bytes)
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::FontNameTooLong`] if the font name exceeds 22 bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::{CharacterCapabilities, CharacterFormat, LetterSpacing, AspectRatio};
    /// use bstr::BString;
    ///
    /// let caps = CharacterCapabilities::with_font(
    ///     CharacterFormat::Ansi,
    ///     80, 25,
    ///     true,
    ///     LetterSpacing::NinePixel,
    ///     AspectRatio::Square,
    ///     Some(BString::from("IBM VGA")),
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_font(
        format: CharacterFormat,
        columns: u16,
        lines: u16,
        ice_colors: bool,
        letter_spacing: LetterSpacing,
        aspect_ratio: AspectRatio,
        font: Option<BString>,
    ) -> crate::Result<Self> {
        // Validate font length if provided
        if let Some(ref f) = font {
            if f.len() > 22 {
                return Err(SauceError::FontNameTooLong(f.len()));
            }
        }

        Ok(CharacterCapabilities {
            format,
            columns,
            lines,
            ice_colors,
            letter_spacing,
            aspect_ratio,
            font_opt: font,
        })
    }

    pub(crate) fn from(header: &SauceHeader) -> crate::Result<Self> {
        let format = CharacterFormat::from_sauce(header.file_type);
        let columns;
        let lines;
        let mut ice_colors = false;
        let mut letter_spacing = LetterSpacing::Legacy;
        let mut aspect_ratio = AspectRatio::Legacy;
        let mut font_opt = None;

        match header.data_type {
            SauceDataType::Character => {
                // Check if format supports ANSi flags
                if format.supports_ansi_flags() {
                    columns = header.t_info1;
                    lines = header.t_info2;
                    ice_colors =
                        (header.t_flags & ANSI_FLAG_NON_BLINK_MODE) == ANSI_FLAG_NON_BLINK_MODE;

                    // Parse letter spacing
                    letter_spacing = match header.t_flags & ANSI_MASK_LETTER_SPACING {
                        ANSI_LETTER_SPACING_LEGACY => LetterSpacing::Legacy,
                        ANSI_LETTER_SPACING_8PX => LetterSpacing::EightPixel,
                        ANSI_LETTER_SPACING_9PX => LetterSpacing::NinePixel,
                        _ => LetterSpacing::Reserved,
                    };

                    // Parse aspect ratio
                    aspect_ratio = match header.t_flags & ANSI_MASK_ASPECT_RATIO {
                        ANSI_ASPECT_RATIO_LEGACY => AspectRatio::Legacy,
                        ANSI_ASPECT_RATIO_STRETCH => AspectRatio::LegacyDevice,
                        ANSI_ASPECT_RATIO_SQUARE => AspectRatio::Square,
                        _ => AspectRatio::Reserved,
                    };

                    font_opt = Some(header.t_info_s.clone());
                } else if format == CharacterFormat::RipScript {
                    // RipScript stores pixel dimensions, not character dimensions
                    // Default character dimensions for RipScript
                    columns = 80;
                    lines = 25;
                } else if format.has_dimensions() {
                    // PCBoard, Avatar, TundraDraw
                    columns = header.t_info1;
                    lines = header.t_info2;
                } else {
                    // Html, Source - no dimensions
                    columns = 0;
                    lines = 0;
                }
            }
            _ => {
                return Err(SauceError::UnsupportedDataType(header.data_type));
            }
        }

        Ok(CharacterCapabilities {
            format,
            columns,
            lines,
            ice_colors,
            letter_spacing,
            aspect_ratio,
            font_opt,
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
    /// use icy_sauce::{CharacterCapabilities, CharacterFormat};
    /// use bstr::BString;
    /// let caps = CharacterCapabilities::new(CharacterFormat::Ansi);
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
    /// # use icy_sauce::{CharacterCapabilities, CharacterFormat};
    /// # use bstr::BString;
    /// let mut caps = CharacterCapabilities::new(CharacterFormat::Ansi);
    /// caps.set_font(BString::from("IBM VGA"))?;
    /// assert_eq!(caps.font(), Some(&BString::from("IBM VGA")));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
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
    /// # use icy_sauce::{CharacterCapabilities, CharacterFormat};
    /// # use bstr::BString;
    /// let mut caps = CharacterCapabilities::new(CharacterFormat::Ansi);
    /// caps.remove_font();
    /// assert_eq!(caps.font(), None);
    /// ```
    pub fn remove_font(&mut self) {
        self.font_opt = None;
    }

    /// Serialize these capabilities into a SAUCE header for file storage.
    ///
    /// This method populates the format-specific fields in the SAUCE header based on
    /// the data type and character format. Each data type (Character, BinaryText, XBin)
    /// has different field layouts as specified by the SAUCE standard.
    ///
    /// # Arguments
    ///
    /// * `header` - Mutable reference to the [`SauceHeader`] to populate
    ///
    /// # Errors
    ///
    /// Returns errors for invalid field values:
    /// - [`SauceError::BinFileWidthLimitExceeded`]: Binary text width must be even and ≤510
    /// - [`SauceError::UnsupportedDataType`]: Unsupported SAUCE data type
    ///
    /// # SAUCE Field Mappings
    ///
    /// **For Character data type:**
    /// - FileType: Character format code (0-8 or unknown)
    /// - TInfo1: Character width
    /// - TInfo2: Character height
    /// - TFlags: ICE colors, letter spacing, aspect ratio (for ANSI formats)
    /// - TInfoS: Font name (for ANSI formats)
    ///
    /// **For BinaryText data type:**
    /// - FileType: Width/2 (width must be even, max 510)
    /// - TFlags: ICE colors flag
    /// - TInfoS: Font name
    ///
    /// **For XBin data type:**
    /// - TInfo1: Character width
    /// - TInfo2: Character height
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Internal serialization example (ignored because `encode_into_header` is not public)
    /// # use icy_sauce::{CharacterCapabilities, CharacterFormat};
    /// # use icy_sauce::header::SauceHeader;
    /// let caps = CharacterCapabilities::new(CharacterFormat::Ansi);
    /// let mut header = SauceHeader::default();
    /// caps.encode_into_header(&mut header)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub(crate) fn encode_into_header(&self, header: &mut SauceHeader) -> crate::Result<()> {
        match header.data_type {
            SauceDataType::BinaryText => {
                // BinaryText: file_type encodes width/2; width must be even and <= 510.
                if self.columns == 0 || self.columns % 2 != 0 || self.columns > 510 {
                    return Err(SauceError::BinFileWidthLimitExceeded(self.columns as i32));
                }
                header.file_type = (self.columns / 2) as u8;
                header.t_info1 = 0;
                header.t_info2 = 0;
                header.t_info3 = 0;
                header.t_info4 = 0;
                header.t_flags = if self.ice_colors {
                    ANSI_FLAG_NON_BLINK_MODE
                } else {
                    0
                };
                if let Some(font) = &self.font_opt {
                    header.t_info_s.clone_from(font);
                } else {
                    header.t_info_s.clear();
                }
            }
            SauceDataType::XBin => {
                header.file_type = 0; // XBin has no file type
                header.t_info1 = self.columns;
                header.t_info2 = self.lines;
                header.t_info3 = 0;
                header.t_info4 = 0;
                header.t_flags = 0;
                header.t_info_s.clear();
            }
            SauceDataType::Character => {
                header.file_type = self.format.to_sauce();

                match self.format {
                    CharacterFormat::Ascii
                    | CharacterFormat::Ansi
                    | CharacterFormat::AnsiMation => {
                        // Formats that support ANSi flags
                        header.t_info1 = self.columns;
                        header.t_info2 = self.lines;
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

                    CharacterFormat::RipScript => {
                        // RipScript MUST have fixed pixel values per SAUCE spec
                        header.t_info1 = 640; // Pixel width
                        header.t_info2 = 350; // Pixel height
                        header.t_info3 = 16; // Number of colors
                        header.t_info4 = 0; // Must be 0
                        header.t_flags = 0; // No flags
                        header.t_info_s.clear(); // No font
                    }

                    CharacterFormat::PCBoard
                    | CharacterFormat::Avatar
                    | CharacterFormat::TundraDraw => {
                        // These formats have dimensions but no flags
                        header.t_info1 = self.columns;
                        header.t_info2 = self.lines;
                        header.t_info3 = 0;
                        header.t_info4 = 0;
                        header.t_flags = 0;
                        header.t_info_s.clear();
                    }

                    CharacterFormat::Html | CharacterFormat::Source => {
                        // HTML and Source have all zeros per spec
                        header.t_info1 = 0;
                        header.t_info2 = 0;
                        header.t_info3 = 0;
                        header.t_info4 = 0;
                        header.t_flags = 0;
                        header.t_info_s.clear();
                    }

                    CharacterFormat::Unknown(_) => {
                        // For unknown types, try to preserve width/height
                        header.t_info1 = self.columns;
                        header.t_info2 = self.lines;
                        header.t_info3 = 0;
                        header.t_info4 = 0;
                        header.t_flags = 0;
                        header.t_info_s.clear();
                    }
                }
            }
            _ => {
                return Err(SauceError::UnsupportedDataType(header.data_type));
            }
        }
        Ok(())
    }
}
