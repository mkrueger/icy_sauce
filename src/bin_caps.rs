use bstr::BString;

use crate::{SauceDataType, SauceError, header::SauceHeader};

use crate::char_caps::{AspectRatio, LetterSpacing};

/// Format discriminator for BinCaps (covers DataType 5 and 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinFormat {
    BinaryText, // DataType::BinaryText (width encoded in file_type = width/2)
    XBin,       // DataType::XBin (width = t_info1, height = t_info2)
}

impl BinFormat {
    pub fn from_data_type(dt: SauceDataType) -> crate::Result<Self> {
        match dt {
            SauceDataType::BinaryText => Ok(BinFormat::BinaryText),
            SauceDataType::XBin => Ok(BinFormat::XBin),
            _ => Err(SauceError::UnsupportedDataType(dt)),
        }
    }
}

/// Capabilities for BinaryText (DataType 5) and XBin (DataType 6).
/// BinaryText:
///   - width: even, 2..=510, stored as file_type = width/2
///   - height: derived from file_size or raw buffer length
///   - flags: ANSiFlags (iCE, letter-spacing, aspect ratio)
///   - font_name: optional (max 22 bytes, zero padded when serialized)
///
/// XBin:
///   - width: t_info1
///   - height: t_info2
///   - flags/font_name unused (always zero / empty)
#[derive(Debug, Clone)]
pub struct BinCaps {
    pub format: BinFormat,
    pub width: u16,
    pub height: u16,                // 0 for BinaryText (derived), explicit for XBin
    pub flags: u8,                  // ANSiFlags for BinaryText, always 0 for XBin
    pub font_name: Option<BString>, // Only BinaryText
}

impl BinCaps {
    /// Construct BinaryText capabilities.
    pub fn binary_text(width: u16) -> crate::Result<Self> {
        if width == 0 || width % 2 != 0 || width > 510 {
            return Err(SauceError::BinFileWidthLimitExceeded(width as i32));
        }
        Ok(Self {
            format: BinFormat::BinaryText,
            width,
            height: 0,
            flags: 0,
            font_name: None,
        })
    }

    /// Construct XBin capabilities.
    pub fn xbin(width: u16, height: u16) -> crate::Result<Self> {
        // Spec doesn’t forbid 0 explicitly, but width/height of 0 are meaningless for XBin.
        if width == 0 {
            return Err(SauceError::UnsupportedDataType(SauceDataType::XBin)); // Re-use existing error
        }
        if height == 0 {
            return Err(SauceError::UnsupportedDataType(SauceDataType::XBin));
        }
        Ok(Self {
            format: BinFormat::XBin,
            width,
            height,
            flags: 0,
            font_name: None,
        })
    }

    /// Add ANSi flags (BinaryText only).
    pub fn with_flags(
        mut self,
        use_ice: bool,
        letter_spacing: LetterSpacing,
        aspect_ratio: AspectRatio,
    ) -> Self {
        if self.format == BinFormat::BinaryText {
            self.flags = build_ansi_flags(use_ice, letter_spacing, aspect_ratio);
        }
        self
    }

    /// Set font for BinaryText (validated length).
    pub fn with_font(mut self, font: BString) -> crate::Result<Self> {
        if self.format != BinFormat::BinaryText {
            // Ignore silently or return an error; choose silent ignore to be lenient.
            return Ok(self);
        }
        if font.len() > 22 {
            return Err(SauceError::FontNameTooLong(font.len()));
        }
        if font.is_empty() {
            self.font_name = None;
        } else {
            self.font_name = Some(font);
        }
        Ok(self)
    }

    /// Parse BinCaps from header (returns None if data_type mismatches).
    pub fn from(header: &SauceHeader) -> crate::Result<Self> {
        let format = BinFormat::from_data_type(header.data_type)?;

        match format {
            BinFormat::BinaryText => {
                let width = (header.file_type as u16) * 2;
                if width == 0 {
                    return Err(SauceError::BinFileWidthLimitExceeded(0));
                }

                let font_name = if header.t_info_s.is_empty() {
                    None
                } else {
                    Some(header.t_info_s.clone())
                };
                Ok(Self {
                    format,
                    width,
                    height: 0,
                    flags: header.t_flags,
                    font_name,
                })
            }
            BinFormat::XBin => Ok(Self {
                format,
                width: header.t_info1,
                height: header.t_info2,
                flags: 0,
                font_name: None,
            }),
        }
    }

    /// Serialize into header based on format.
    pub fn write_to_header(&self, header: &mut SauceHeader) -> crate::Result<()> {
        match self.format {
            BinFormat::BinaryText => {
                header.data_type = SauceDataType::BinaryText;
                if self.width == 0 || self.width % 2 != 0 || self.width > 510 {
                    return Err(SauceError::BinFileWidthLimitExceeded(self.width as i32));
                }
                header.file_type = (self.width / 2) as u8;
                header.t_info1 = 0;
                header.t_info2 = 0;
                header.t_info3 = 0;
                header.t_info4 = 0;
                header.t_flags = self.flags;
                if let Some(font) = &self.font_name {
                    header.t_info_s.clone_from(font);
                } else {
                    header.t_info_s.clear();
                }
            }
            BinFormat::XBin => {
                header.data_type = SauceDataType::XBin;
                header.file_type = 0;
                header.t_info1 = self.width;
                header.t_info2 = self.height;
                header.t_info3 = 0;
                header.t_info4 = 0;
                header.t_flags = 0;
                header.t_info_s.clear();
            }
        }
        Ok(())
    }

    /// Height computed using stored FileSize (BinaryText only).
    /// FileSize must be the original content length (excluding SAUCE), per spec.
    pub fn calculate_binary_text_height(&self, file_size: u32) -> Option<u16> {
        if self.format != BinFormat::BinaryText {
            return None;
        }
        if self.width == 0 || file_size == 0 {
            return None;
        }
        let bytes_per_row = self.width as u32 * 2; // char + attribute
        let h = file_size / bytes_per_row;
        if h == 0 || h > u16::MAX as u32 {
            None
        } else {
            Some(h as u16)
        }
    }
}

/// Build ANSi flags byte.
/// Bits:
/// B (bit 0) – iCE
/// LS (bits 1–2) – letter spacing
/// AR (bits 3–4) – aspect ratio
fn build_ansi_flags(use_ice: bool, letter_spacing: LetterSpacing, aspect_ratio: AspectRatio) -> u8 {
    let mut flags = 0u8;
    if use_ice {
        flags |= 0b0000_0001;
    }
    flags |= match letter_spacing {
        LetterSpacing::Legacy => 0b0000_0000,
        LetterSpacing::EightPixel => 0b0000_0010,
        LetterSpacing::NinePixel => 0b0000_0100,
        LetterSpacing::Reserved => 0b0000_0000,
    };
    flags |= match aspect_ratio {
        AspectRatio::Legacy => 0b0000_0000,
        AspectRatio::LegacyDevice => 0b0000_1000,
        AspectRatio::Square => 0b0001_0000,
        AspectRatio::Reserved => 0b0000_0000,
    };
    flags
}
