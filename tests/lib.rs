use std::fs;

use bstr::BString;
use icy_sauce::{
    Capabilities, SauceDataType, SauceDate, SauceRecord, SauceRecordBuilder,
    binary::BinaryCapabilities,
    character::{AspectRatio, CharacterCapabilities, CharacterFormat, LetterSpacing},
};

#[test]
fn test_write1() {
    let file = fs::read("tests/files/test1.pcb").unwrap();
    let info = SauceRecord::from_bytes(&file).unwrap().unwrap();

    let mut write_to = Vec::new();
    info.write(&mut write_to).unwrap();
    let info2 = SauceRecord::from_bytes(&write_to).unwrap().unwrap();

    assert_eq!(info.title(), info2.title());
    assert_eq!(info.group(), info2.group());
    assert_eq!(info.author(), info2.author());
}

#[test]
fn test_write2() {
    let file = fs::read("tests/files/test2.ans").unwrap();
    let info = SauceRecord::from_bytes(&file).unwrap().unwrap();

    let mut write_to = Vec::new();
    info.write(&mut write_to).unwrap();
    let info2 = SauceRecord::from_bytes(&write_to).unwrap().unwrap();

    assert_eq!(info.title(), info2.title());
    assert_eq!(info.group(), info2.group());
    assert_eq!(info.author(), info2.author());
}

#[test]
fn test_builder() {
    let caps = BinaryCapabilities::xbin(112, 90).unwrap();
    // font_opt is None by default in new()

    let builder = SauceRecordBuilder::default()
        .title("Title".into())
        .unwrap()
        .author("Author".into())
        .unwrap()
        .group("Group".into())
        .unwrap()
        .date(SauceDate::new(1976, 12, 30))
        .data_type(icy_sauce::SauceDataType::XBin)
        .capabilities(Capabilities::Binary(caps))
        .unwrap();

    let mut write_to = Vec::new();
    builder.build().write(&mut write_to).unwrap();
    let info2 = SauceRecord::from_bytes(&write_to).unwrap().unwrap();

    assert_eq!(info2.title(), &BString::from("Title"));
    assert_eq!(info2.group(), &BString::from("Group"));
    assert_eq!(info2.author(), &BString::from("Author"));
    assert_eq!(info2.data_type(), icy_sauce::SauceDataType::XBin);
    assert_eq!(info2.date(), SauceDate::new(1976, 12, 30));

    // Use the unified capabilities method
    if let Some(Capabilities::Binary(caps)) = info2.capabilities() {
        assert_eq!(caps.columns, 112);
        assert_eq!(caps.lines, 90);
    } else {
        panic!("Expected Binary capabilities");
    }
}

#[test]
fn test_build_comments() {
    let builder = SauceRecordBuilder::default()
        .title("Title".into())
        .unwrap()
        .author("Author".into())
        .unwrap()
        .group("Group".into())
        .unwrap()
        .add_comment(BString::new("This is a comment".into()))
        .unwrap()
        .add_comment(BString::new("This is another comment".into()))
        .unwrap();

    let mut write_to = Vec::new();
    builder.build().write(&mut write_to).unwrap();
    let info2 = SauceRecord::from_bytes(&write_to).unwrap().unwrap();

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
            let caps = CharacterCapabilities::with_font(
                CharacterFormat::Ansi,
                80,
                25,
                false,
                letter_spacing,
                aspect_ratio,
                Some(BString::from("IBM VGA")),
            )
            .unwrap();

            let info = SauceRecordBuilder::default()
                .data_type(SauceDataType::Character)
                .capabilities(Capabilities::Character(caps))
                .unwrap()
                .build();

            let mut data = Vec::new();
            info.write(&mut data).unwrap();
            let parsed = SauceRecord::from_bytes(&data).unwrap().unwrap();

            // Use the unified capabilities method
            if let Some(Capabilities::Character(parsed_caps)) = parsed.capabilities() {
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
                    parsed_caps.font(),
                    Some(&BString::from("IBM VGA")),
                    "Font mismatch"
                );
            } else {
                panic!("Expected Character capabilities");
            }
        }
    }
}

use proptest::collection::vec;
use proptest::proptest;

proptest! {
    #[test]
    fn round_trip_character(
        // Generate byte vectors directly to ensure exact length control
        title_bytes in vec(0x21u8..=0xFE, 0..=35),  // CP437 printable range
        author_bytes in vec(0x21u8..=0xFE, 0..=20),
        width in 1u16..200,
        height in 1u16..200
    ) {
        let caps = CharacterCapabilities::new(CharacterFormat::Ansi)
            .dimensions(width, height);
        let record = SauceRecordBuilder::default()
            .title(BString::from(title_bytes))?
            .author(BString::from(author_bytes))?
            .capabilities(Capabilities::Character(caps))?
            .build();

        let mut buf = Vec::new();
        record.write(&mut buf)?;
        let parsed = SauceRecord::from_bytes(&buf)?.unwrap();

        assert_eq!(parsed.title(), record.title());
        assert_eq!(parsed.author(), record.author());
        assert_eq!(parsed.capabilities(), record.capabilities());
    }
}
