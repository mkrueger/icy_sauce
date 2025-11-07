use bstr::BString;

use crate::{SauceDataType, SauceError, header::SauceHeader};

/// | Field    | Type | Size | Descritption
/// |----------|------|------|-------------
/// | ID       | char | 5    | SAUCE comment block ID. This should be equal to "COMNT".
/// | Line 1   | char | 64   | Line of text.
/// | ...      |      |      |
/// | Line n   | char | 64   | Last line of text
const ANSI_FLAG_NON_BLINK_MODE: u8 = 0b0000_0001;
const ANSI_MASK_LETTER_SPACING: u8 = 0b0000_0110;
const ANSI_LETTER_SPACING_LEGACY: u8 = 0b0000_0000;
const ANSI_LETTER_SPACING_8PX: u8 = 0b0000_0010;
const ANSI_LETTER_SPACING_9PX: u8 = 0b0000_0100;

const ANSI_MASK_ASPECT_RATIO: u8 = 0b0001_1000;
const ANSI_ASPECT_RATIO_LEGACY: u8 = 0b0000_0000;
const ANSI_ASPECT_RATIO_STRETCH: u8 = 0b0000_1000;
const ANSI_ASPECT_RATIO_SQUARE: u8 = 0b0001_0000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CharacterFormat {
    Ascii,      // 0
    Ansi,       // 1
    AnsiMation, // 2
    RipScript,  // 3
    PCBoard,    // 4
    Avatar,     // 5
    Html,       // 6
    Source,     // 7
    TundraDraw, // 8

    Unknown(u8),
}

impl CharacterFormat {
    /// Parse from file type byte
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

    /// Convert back to file type byte
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

    /// Check if this format supports ANSi flags (ice colors, letter spacing, aspect ratio)
    pub fn supports_ansi_flags(&self) -> bool {
        matches!(
            self,
            CharacterFormat::Ascii | CharacterFormat::Ansi | CharacterFormat::AnsiMation
        )
    }

    /// Check if this format stores character dimensions
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

    /// Check if this is a streaming format (variable height)
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

    /// Check if this format requires fixed screen size
    pub fn is_animation(&self) -> bool {
        matches!(self, CharacterFormat::AnsiMation)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LetterSpacing {
    Legacy,     // 00
    EightPixel, // 01
    NinePixel,  // 10
    Reserved,   // 11
}

impl LetterSpacing {
    pub fn use_letter_spacing(self) -> bool {
        self == LetterSpacing::NinePixel
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AspectRatio {
    Legacy,       // 00
    LegacyDevice, // 01 (needs stretching)
    Square,       // 10 (modern, no stretch)
    Reserved,     // 11
}

impl AspectRatio {
    pub fn use_aspect_ratio(self) -> bool {
        self == AspectRatio::LegacyDevice
    }
}

#[derive(Debug, Clone)]
pub struct CharCaps {
    pub format: CharacterFormat,
    pub width: u16,
    pub height: u16,
    pub use_ice: bool,
    pub letter_spacing: LetterSpacing,
    pub aspect_ratio: AspectRatio,
    font_opt: Option<BString>,
}

impl CharCaps {
    pub fn new(character_format: CharacterFormat) -> Self {
        CharCaps {
            format: character_format,
            width: 80,
            height: 25,
            use_ice: false,
            letter_spacing: LetterSpacing::Legacy,
            aspect_ratio: AspectRatio::Legacy,
            font_opt: None,
        }
    }

    /// Create a new CharCaps with all fields specified, validating the font
    pub fn with_font(
        format: CharacterFormat,
        width: u16,
        height: u16,
        use_ice: bool,
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

        Ok(CharCaps {
            format,
            width,
            height,
            use_ice,
            letter_spacing,
            aspect_ratio,
            font_opt: font,
        })
    }

    pub(crate) fn from(header: &SauceHeader) -> crate::Result<Self> {
        let format = CharacterFormat::from_sauce(header.file_type);
        let width;
        let height;
        let mut use_ice = false;
        let mut letter_spacing = LetterSpacing::Legacy;
        let mut aspect_ratio = AspectRatio::Legacy;
        let mut font_opt = None;

        match header.data_type {
            SauceDataType::Character => {
                // Check if format supports ANSi flags
                if format.supports_ansi_flags() {
                    width = header.t_info1;
                    height = header.t_info2;
                    use_ice =
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
                    width = 80;
                    height = 25;
                } else if format.has_dimensions() {
                    // PCBoard, Avatar, TundraDraw
                    width = header.t_info1;
                    height = header.t_info2;
                } else {
                    // Html, Source - no dimensions
                    width = 0;
                    height = 0;
                }
            }
            _ => {
                return Err(SauceError::UnsupportedDataType(header.data_type));
            }
        }

        Ok(CharCaps {
            format,
            width,
            height,
            use_ice,
            letter_spacing,
            aspect_ratio,
            font_opt,
        })
    }

    /// Get the font name
    pub fn font_opt(&self) -> Option<&BString> {
        self.font_opt.as_ref()
    }

    /// Set the font name with validation
    pub fn set_font(&mut self, font_name: BString) -> crate::Result<()> {
        if font_name.len() > 22 {
            return Err(SauceError::FontNameTooLong(font_name.len()));
        }
        if font_name.is_empty() {
            self.font_opt = None;
            return Ok(());
        }
        self.font_opt = Some(font_name);
        Ok(())
    }

    /// Clear the font name
    pub fn clear_font(&mut self) {
        self.font_opt = None;
    }

    pub(crate) fn write_to_header(&self, header: &mut SauceHeader) -> crate::Result<()> {
        match header.data_type {
            SauceDataType::BinaryText => {
                // BinaryText: file_type encodes width/2; width must be even and <= 510.
                if self.width == 0 || self.width % 2 != 0 || self.width > 510 {
                    return Err(SauceError::BinFileWidthLimitExceeded(self.width as i32));
                }
                header.file_type = (self.width / 2) as u8;
                header.t_info1 = 0;
                header.t_info2 = 0;
                header.t_info3 = 0;
                header.t_info4 = 0;
                header.t_flags = if self.use_ice {
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
                header.t_info1 = self.width;
                header.t_info2 = self.height;
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
                        header.t_info1 = self.width;
                        header.t_info2 = self.height;
                        header.t_info3 = 0;
                        header.t_info4 = 0;

                        // Build flags byte
                        header.t_flags = if self.use_ice {
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
                        header.t_info1 = self.width;
                        header.t_info2 = self.height;
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
                        header.t_info1 = self.width;
                        header.t_info2 = self.height;
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
