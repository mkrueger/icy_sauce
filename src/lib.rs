use std::fmt::Display;

use bstr::BString;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, SauceError>;

pub mod archieve_caps;
pub mod audio_caps;
pub mod bin_caps;
pub mod char_caps;
pub mod executable_caps;
pub mod pixel_caps;

pub mod header;
pub mod info;
pub use info::*;

pub mod builder;
pub use builder::*;

#[derive(Error, Debug)]
pub enum SauceError {
    #[error("Unsupported SAUCE version: {0}")]
    UnsupportedSauceVersion(BString),

    #[error("Invalid comment block")]
    InvalidCommentBlock,

    #[error("Unsupported SAUCE date: {0}")]
    UnsupportedSauceDate(BString),

    #[error("Binary file width limit exceeded: {0}")]
    BinFileWidthLimitExceeded(i32),

    #[error("Unsupported data type for operation: {0:?}")]
    UnsupportedDataType(SauceDataType),

    #[error("IO error: {0}")]
    IoError(std::io::Error),

    #[error("Comment limit exceeded (255)")]
    CommentLimitExceeded,

    #[error("Comment too long: {0} bytes only up to 64 bytes are allowed.")]
    CommentTooLong(usize),

    #[error("Title too long: {0} bytes only up to 35 bytes are allowed.")]
    TitleTooLong(usize),

    #[error("Author too long: {0} bytes only up to 20 bytes are allowed.")]
    AuthorTooLong(usize),

    #[error("Group too long: {0} bytes only up to 20 bytes are allowed.")]
    GroupTooLong(usize),

    #[error("Font name too long: {0} bytes only up to 22 bytes are allowed.")]
    FontNameTooLong(usize),

    #[error("Missing EOF marker (0x1A) before SAUCE record")]
    MissingEofMarker,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum SauceDataType {
    /// None / Undefined (spec DataType 0)
    None = 0,
    /// A character based file.
    /// These files are typically interpreted sequentially. Also known as streams.
    #[default]
    Character = 1,
    /// Bitmap graphic and animation files.
    Bitmap = 2,
    /// A vector graphic file.
    Vector = 3,
    /// An audio file.
    Audio = 4,
    /// Raw memory copy of a text mode screen (.BIN file)
    BinaryText = 5,
    /// XBin or eXtended BIN file.
    XBin = 6,
    /// Archive file.
    Archive = 7,
    /// Executable file.
    Executable = 8,
    /// Any other value not covered by the spec (future / unknown).
    Undefined(u8),
}

impl Display for SauceDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SauceDataType::None => write!(f, "None"),
            SauceDataType::Character => write!(f, "Character"),
            SauceDataType::Bitmap => write!(f, "Bitmap"),
            SauceDataType::Vector => write!(f, "Vector"),
            SauceDataType::Audio => write!(f, "Audio"),
            SauceDataType::BinaryText => write!(f, "BinaryText"),
            SauceDataType::XBin => write!(f, "XBin"),
            SauceDataType::Archive => write!(f, "Archive"),
            SauceDataType::Executable => write!(f, "Executable"),
            SauceDataType::Undefined(val) => write!(f, "Undefined({})", val),
        }
    }
}

impl From<u8> for SauceDataType {
    fn from(byte: u8) -> SauceDataType {
        match byte {
            0 => SauceDataType::None,
            1 => SauceDataType::Character,
            2 => SauceDataType::Bitmap,
            3 => SauceDataType::Vector,
            4 => SauceDataType::Audio,
            5 => SauceDataType::BinaryText,
            6 => SauceDataType::XBin,
            7 => SauceDataType::Archive,
            8 => SauceDataType::Executable,
            other => SauceDataType::Undefined(other),
        }
    }
}

impl From<SauceDataType> for u8 {
    fn from(data_type: SauceDataType) -> u8 {
        match data_type {
            SauceDataType::None => 0,
            SauceDataType::Character => 1,
            SauceDataType::Bitmap => 2,
            SauceDataType::Vector => 3,
            SauceDataType::Audio => 4,
            SauceDataType::BinaryText => 5,
            SauceDataType::XBin => 6,
            SauceDataType::Archive => 7,
            SauceDataType::Executable => 8,
            SauceDataType::Undefined(byte) => byte,
        }
    }
}

/// Trims the trailing whitespace and null bytes from the data.
/// This is sauce specific - no other thing than space should be trimmed, however some implementations use null bytes instead of spaces.
pub(crate) fn sauce_trim(data: &[u8]) -> BString {
    let end = sauce_len_rev(data);
    BString::new(data[..end].to_vec())
}

fn sauce_len_rev(data: &[u8]) -> usize {
    let mut end = data.len();
    while end > 0 {
        let b = data[end - 1];
        if b != 0 && b != b' ' {
            break;
        }
        end -= 1;
    }
    end
}

/// Pads trailing whitespaces or cut too long data.
pub(crate) fn sauce_pad(str: &BString, len: usize) -> Vec<u8> {
    let mut data = str.to_vec();
    data.resize(len, b' ');
    data
}

/// Pads trailing \0 or cut too long data.
pub(crate) fn zero_pad(str: &BString, len: usize) -> Vec<u8> {
    let mut data = str.to_vec();
    data.resize(len, 0);
    data
}

/// Trim only trailing zero bytes (binary zero padding) – for zero padded fields like TInfoS.
pub(crate) fn zero_trim(data: &[u8]) -> BString {
    let mut end = data.len();
    while end > 0 && data[end - 1] == 0 {
        end -= 1;
    }
    BString::new(data[..end].to_vec())
}
