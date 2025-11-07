use std::fs;

use bstr::BString;
use chrono::NaiveDate;
use icy_sauce::{
    SauceDataType, SauceInformation, SauceInformationBuilder,
    bin_caps::BinCaps,
    char_caps::{AspectRatio, CharCaps, CharacterFormat, LetterSpacing},
};

#[test]
fn test_write1() {
    let file = fs::read("tests/files/test1.pcb").unwrap();
    let info = SauceInformation::read(&file).unwrap().unwrap();

    let mut write_to = Vec::new();
    info.write(&mut write_to).unwrap();
    let info2 = SauceInformation::read(&write_to).unwrap().unwrap();

    assert_eq!(info.title(), info2.title());
    assert_eq!(info.group(), info2.group());
    assert_eq!(info.author(), info2.author());
}

#[test]
fn test_write2() {
    let file = fs::read("tests/files/test2.ans").unwrap();
    let info = SauceInformation::read(&file).unwrap().unwrap();

    let mut write_to = Vec::new();
    info.write(&mut write_to).unwrap();
    let info2 = SauceInformation::read(&write_to).unwrap().unwrap();

    assert_eq!(info.title(), info2.title());
    assert_eq!(info.group(), info2.group());
    assert_eq!(info.author(), info2.author());
}

#[test]
fn test_builder() {
    let caps = BinCaps::xbin(112, 90).unwrap();
    // font_opt is None by default in new()

    let builder = SauceInformationBuilder::default()
        .with_title("Title".into())
        .unwrap()
        .with_author("Author".into())
        .unwrap()
        .with_group("Group".into())
        .unwrap()
        .with_date(NaiveDate::from_ymd_opt(1976, 12, 30).unwrap())
        .with_data_type(icy_sauce::SauceDataType::XBin)
        .with_bin_caps(caps)
        .unwrap();

    let mut write_to = Vec::new();
    builder.build().write(&mut write_to).unwrap();
    let info2 = SauceInformation::read(&write_to).unwrap().unwrap();

    assert_eq!(info2.title(), &BString::from("Title"));
    assert_eq!(info2.group(), &BString::from("Group"));
    assert_eq!(info2.author(), &BString::from("Author"));
    assert_eq!(info2.data_type(), icy_sauce::SauceDataType::XBin);
    assert_eq!(
        info2.date().unwrap(),
        NaiveDate::from_ymd_opt(1976, 12, 30).unwrap()
    );
    let caps = info2.get_bin_capabilities().unwrap();
    assert_eq!(caps.width, 112);
    assert_eq!(caps.height, 90);
}

#[test]
fn test_build_comments() {
    let builder = SauceInformationBuilder::default()
        .with_title("Title".into())
        .unwrap()
        .with_author("Author".into())
        .unwrap()
        .with_group("Group".into())
        .unwrap()
        .with_comment(BString::new("This is a comment".into()))
        .unwrap()
        .with_comment(BString::new("This is another comment".into()))
        .unwrap();

    let mut write_to = Vec::new();
    builder.build().write(&mut write_to).unwrap();
    let info2 = SauceInformation::read(&write_to).unwrap().unwrap();

    assert_eq!(info2.title(), &BString::from("Title"));
    assert_eq!(info2.group(), &BString::from("Group"));
    assert_eq!(info2.author(), &BString::from("Author"));
    assert_eq!(info2.comments().len(), 2);
    assert_eq!(info2.comments()[0], BString::from("This is a comment"));
    assert_eq!(
        info2.comments()[1],
        BString::from("This is another comment")
    );
}

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
fn test_letter_spacing_aspect_ratio_roundtrip() {
    // Test all combinations of letter spacing and aspect ratio
    for &letter_spacing in &[
        LetterSpacing::Legacy,
        LetterSpacing::EightPixel,
        LetterSpacing::NinePixel,
    ] {
        for &aspect_ratio in &[
            AspectRatio::Legacy,
            AspectRatio::LegacyDevice,
            AspectRatio::Square,
        ] {
            let caps = CharCaps::with_font(
                CharacterFormat::Ansi,
                80,
                25,
                false,
                letter_spacing,
                aspect_ratio,
                Some(BString::from("IBM VGA")),
            )
            .unwrap();

            let info = SauceInformationBuilder::default()
                .with_data_type(SauceDataType::Character)
                .with_char_caps(caps)
                .unwrap()
                .build();

            let mut data = Vec::new();
            info.write(&mut data).unwrap();
            let parsed = SauceInformation::read(&data).unwrap().unwrap();
            let parsed_caps = parsed.get_character_capabilities().unwrap();

            assert_eq!(
                parsed_caps.letter_spacing, letter_spacing,
                "Letter spacing mismatch for {:?}",
                letter_spacing
            );
            assert_eq!(
                parsed_caps.aspect_ratio, aspect_ratio,
                "Aspect ratio mismatch for {:?}",
                aspect_ratio
            );
            assert_eq!(
                parsed_caps.font_opt(),
                Some(&BString::from("IBM VGA")),
                "Font mismatch"
            );
        }
    }
}

/// Trims the trailing whitespace and null bytes from the data.
/// This is sauce specific - no other thing than space should be trimmed, however some implementations use null bytes instead of spaces.
pub(crate) fn sauce_trim(data: &[u8]) -> BString {
    let end = sauce_len_rev(data);
    BString::new(data[..end].to_vec())
}

/// Pads trailing whitespaces or cut too long data.
pub(crate) fn sauce_pad(str: &BString, len: usize) -> Vec<u8> {
    let mut data = str.to_vec();
    data.resize(len, b' ');
    data
}

/// Pads trailing \0 or cut too long data.
pub(crate) fn _zero_pad(str: &BString, len: usize) -> Vec<u8> {
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
