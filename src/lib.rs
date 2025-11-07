use bstr::BString;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, SauceError>;

pub mod char_caps;
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

    #[error("Invalid comment ID: {0}")]
    InvalidCommentId(BString),

    #[error("Unsupported SAUCE date: {0}")]
    UnsupportedSauceDate(BString),

    #[error("Binary file width limit exceeded: {0}")]
    BinFileWidthLimitExceeded(i32),

    #[error("Wrong data type for operation: {0:?}")]
    WrongDataType(SauceDataType),

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

#[cfg(test)]
mod tests {
    use crate::{
        SauceDataType, SauceInformation, SauceInformationBuilder,
        char_caps::{CharCaps, ContentType},
        sauce_pad, sauce_trim, zero_trim,
    };
    use bstr::BString;

    #[test]
    fn test_sauce_trim() {
        let data = b"Hello World  ";
        assert_eq!(sauce_trim(data), BString::from("Hello World"));
        let data = b"Hello World\0\0";
        assert_eq!(sauce_trim(data), BString::from("Hello World"));

        let data = b"Hello World\t\0";
        assert_eq!(sauce_trim(data), BString::from("Hello World\t"));
        let data = b"Hello World\n ";
        assert_eq!(sauce_trim(data), BString::from("Hello World\n"));
        let data = b"    \0   ";
        assert_eq!(sauce_trim(data), BString::from(""));
    }

    #[test]
    fn test_sauce_pad() {
        let data = BString::from(b"Hello World");
        assert_eq!(sauce_pad(&data, 15), b"Hello World    ");

        let data = BString::from(b"Hello World");
        assert_eq!(sauce_pad(&data, 5), b"Hello");

        let data = BString::from(b"");
        assert_eq!(sauce_pad(&data, 1), b" ");
    }

    #[test]
    fn test_zero_trim() {
        let data = b"FONT NAME   \0\0\0"; // keep trailing spaces before zeros
        assert_eq!(zero_trim(data), BString::from("FONT NAME   "));
        let data = b"ABC";
        assert_eq!(zero_trim(data), BString::from("ABC"));
        let data = b"ABC\0DEF\0"; // internal zeros preserved
        assert_eq!(zero_trim(data), BString::from(b"ABC\0DEF".to_vec()));
    }

    #[test]
    fn test_binarytext_width_encoding() {
        let caps = CharCaps {
            content_type: ContentType::Ascii,
            width: 160,
            height: 0,
            use_ice: true,
            use_letter_spacing: false,
            use_aspect_ratio: false,
            font_opt: None,
        };
        let info = SauceInformationBuilder::default()
            .with_data_type(SauceDataType::BinaryText)
            .with_char_caps(caps)
            .unwrap()
            .build();
        assert_eq!(info.header.file_type, 80); // width/2 stored

        let mut data = Vec::new();
        info.write(&mut data, 1234).unwrap();
        let parsed = SauceInformation::read(&data).unwrap().unwrap();
        let parsed_caps = parsed.get_character_capabilities().unwrap();
        assert_eq!(parsed_caps.width, 160);
    }

    #[test]
    fn test_binarytext_width_invalid() {
        let caps = CharCaps {
            content_type: ContentType::Ascii,
            width: 161,
            height: 0,
            use_ice: false,
            use_letter_spacing: false,
            use_aspect_ratio: false,
            font_opt: None,
        };
        let err = SauceInformationBuilder::default()
            .with_data_type(SauceDataType::BinaryText)
            .with_char_caps(caps)
            .err()
            .expect("should error on odd width");
        match err {
            crate::SauceError::BinFileWidthLimitExceeded(w) => assert_eq!(w, 161),
            other => panic!("Unexpected error: {other:?}"),
        }
    }
}
