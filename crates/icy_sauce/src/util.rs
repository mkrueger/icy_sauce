use bstr::BString;

pub(crate) fn trim_spaces(buf: &[u8]) -> bstr::BString {
    let mut end = buf.len();
    while end > 0 {
        let b = buf[end - 1];
        if b != b' ' && b != 0 {
            break;
        }
        end -= 1;
    }
    bstr::BString::from(&buf[..end])
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

/// Decode a font string without altering the underlying raw TInfoS field.
pub(crate) fn decode_font(data: &[u8]) -> Option<BString> {
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    (end > 0).then(|| BString::from(&data[..end]))
}

pub(crate) fn validate_font(font: &[u8]) -> crate::Result<()> {
    if font.len() > crate::limits::MAX_FONT_NAME_LENGTH {
        return Err(crate::SauceError::FontNameTooLong(font.len()));
    }
    if font.contains(&0) {
        return Err(crate::SauceError::InvalidFontName);
    }
    Ok(())
}
