use bstr::BString;
use icy_sauce::{
    SauceDataType, SauceError, SauceInformation, SauceInformationBuilder,
    bin_caps::{BinCaps, BinFormat},
    char_caps::{AspectRatio, LetterSpacing},
    header::SauceHeader,
};

#[test]
fn test_binary_text_height_calculation() {
    // Build proper BinaryText caps with even width = 80
    let caps = BinCaps::binary_text(80).unwrap();

    // FileSize is the size of the original data (without SAUCE & comments)
    // Height = file_size / (width * 2) = 8000 / (80 * 2) = 50
    let info = SauceInformationBuilder::default()
        .with_data_type(SauceDataType::BinaryText)
        .with_bin_caps(caps)
        .unwrap()
        .with_file_size(8000)
        .build();

    let mut data = Vec::new();
    info.write(&mut data).unwrap();
    let parsed = SauceInformation::read(&data).unwrap().unwrap();
    assert_eq!(
        parsed
            .get_bin_capabilities()
            .unwrap()
            .calculate_binary_text_height(parsed.file_size()),
        Some(50)
    );
}

#[test]
fn test_binary_text_odd_width_rejected() {
    // Width must be even (spec: encoded as width/2 in file_type)
    let err = BinCaps::binary_text(81).unwrap_err();
    match err {
        icy_sauce::SauceError::BinFileWidthLimitExceeded(w) => assert_eq!(w, 81),
        other => panic!("Unexpected error: {:?}", other),
    }
}

#[test]
fn test_binary_text_max_width() {
    // Maximum width is 510 (255 * 2)
    assert!(BinCaps::binary_text(510).is_ok());
    assert!(BinCaps::binary_text(512).is_err());
}

#[test]
fn test_binarytext_width_encoding() {
    let caps: BinCaps = BinCaps::binary_text(160).unwrap();

    let info = SauceInformationBuilder::default()
        .with_data_type(SauceDataType::BinaryText)
        .with_bin_caps(caps)
        .unwrap()
        .build();

    // Access file_type through binary capabilities
    let mut data = Vec::new();
    info.write(&mut data).unwrap();
    let parsed = SauceInformation::read(&data).unwrap().unwrap();
    let parsed_caps = parsed.get_bin_capabilities().unwrap();
    assert_eq!(parsed_caps.width, 160);
}

#[test]
fn test_binary_text_with_font() {
    // Test BinaryText with font name
    let caps = BinCaps::binary_text(80)
        .unwrap()
        .with_font(BString::from("IBM VGA"))
        .unwrap();

    let info = SauceInformationBuilder::default()
        .with_data_type(SauceDataType::BinaryText)
        .with_bin_caps(caps)
        .unwrap()
        .with_file_size(4000)
        .build();

    let mut data = Vec::new();
    info.write(&mut data).unwrap();
    let parsed = SauceInformation::read(&data).unwrap().unwrap();
    let parsed_caps = parsed.get_bin_capabilities().unwrap();

    assert_eq!(parsed_caps.font_name, Some(BString::from("IBM VGA")));
}

#[test]
fn test_binary_text_with_flags() {
    // Test BinaryText with ANSi flags
    let caps = BinCaps::binary_text(80).unwrap().with_flags(
        true,
        LetterSpacing::EightPixel,
        AspectRatio::Square,
    );

    let info = SauceInformationBuilder::default()
        .with_data_type(SauceDataType::BinaryText)
        .with_bin_caps(caps)
        .unwrap()
        .build();

    let mut data = Vec::new();
    info.write(&mut data).unwrap();
    let parsed = SauceInformation::read(&data).unwrap().unwrap();
    let parsed_caps = parsed.get_bin_capabilities().unwrap();

    // Check flag bits
    assert_ne!(parsed_caps.flags & 0b0000_0001, 0); // ICE flag set
    assert_ne!(parsed_caps.flags & 0b0000_0010, 0); // 8-pixel letter spacing
    assert_ne!(parsed_caps.flags & 0b0001_0000, 0); // Square aspect ratio
}

#[test]
fn test_binary_text_font_name_too_long() {
    // Font name max length is 22 bytes
    let long_font = BString::from("This font name is way too long!");
    let result = BinCaps::binary_text(80).unwrap().with_font(long_font);

    assert!(matches!(result, Err(SauceError::FontNameTooLong(_))));
}

#[test]
fn test_binary_text_empty_font() {
    // Empty font should be treated as None
    let caps = BinCaps::binary_text(80)
        .unwrap()
        .with_font(BString::from(""))
        .unwrap();

    assert!(caps.font_name.is_none());
}

#[test]
fn test_xbin_roundtrip() {
    // Test XBin with various dimensions
    let test_cases = vec![
        (80, 25),
        (132, 50),
        (256, 100),
        (1, 1),         // minimum valid
        (65535, 65535), // maximum u16
    ];

    for (width, height) in test_cases {
        let caps = BinCaps::xbin(width, height).unwrap();

        let info = SauceInformationBuilder::default()
            .with_data_type(SauceDataType::XBin)
            .with_bin_caps(caps)
            .unwrap()
            .build();

        let mut data = Vec::new();
        info.write(&mut data).unwrap();
        let parsed = SauceInformation::read(&data).unwrap().unwrap();
        let parsed_caps = parsed.get_bin_capabilities().unwrap();

        assert_eq!(parsed_caps.format, BinFormat::XBin);
        assert_eq!(parsed_caps.width, width);
        assert_eq!(parsed_caps.height, height);
        assert_eq!(parsed_caps.flags, 0); // XBin doesn't use flags
        assert!(parsed_caps.font_name.is_none()); // XBin doesn't use font
    }
}

#[test]
fn test_xbin_ignores_flags_and_font() {
    // XBin should ignore flags and font settings
    let caps = BinCaps::xbin(80, 25)
        .unwrap()
        .with_flags(true, LetterSpacing::EightPixel, AspectRatio::Square)
        .with_font(BString::from("Ignored"))
        .unwrap();

    assert_eq!(caps.flags, 0);
    assert!(caps.font_name.is_none());
}

#[test]
fn test_get_bin_caps_wrong_type() {
    // Test that get_bin_capabilities returns None for non-binary types
    let info = SauceInformationBuilder::default()
        .with_data_type(SauceDataType::Character)
        .build();

    assert!(info.get_bin_capabilities().is_err());

    let info = SauceInformationBuilder::default()
        .with_data_type(SauceDataType::Audio)
        .build();

    assert!(info.get_bin_capabilities().is_err());
}

#[test]
fn test_binary_text_all_valid_widths() {
    // Test a sampling of valid even widths
    for width in (2..=510).step_by(2).take(10) {
        let caps = BinCaps::binary_text(width).unwrap();
        assert_eq!(caps.width, width);
    }
}

#[test]
fn test_sauce_information_calculate_height_methods() {
    let caps = BinCaps::binary_text(80).unwrap();

    let info = SauceInformationBuilder::default()
        .with_data_type(SauceDataType::BinaryText)
        .with_bin_caps(caps)
        .unwrap()
        .with_file_size(8000)
        .build();

    // Since calculation moved to BinCaps, use it directly
    let bin_caps = info.get_bin_capabilities().unwrap();
    assert_eq!(
        bin_caps.calculate_binary_text_height(info.file_size()),
        Some(50)
    );
}

#[test]
fn test_binary_text_width_constraints() {
    assert!(BinCaps::binary_text(80).is_ok());
    assert!(BinCaps::binary_text(160).is_ok());
    assert!(BinCaps::binary_text(510).is_ok()); // max
    assert!(BinCaps::binary_text(0).is_err());
    assert!(BinCaps::binary_text(81).is_err());
    assert!(BinCaps::binary_text(511).is_err());
    assert!(BinCaps::binary_text(512).is_err());
}

#[test]
fn test_xbin_creation() {
    let caps = BinCaps::xbin(80, 25).unwrap();
    assert_eq!(caps.format, BinFormat::XBin);
    assert_eq!(caps.width, 80);
    assert_eq!(caps.height, 25);
    assert_eq!(caps.flags, 0);
    assert!(caps.font_name.is_none());
    // invalid dimensions
    assert!(BinCaps::xbin(0, 25).is_err());
    assert!(BinCaps::xbin(80, 0).is_err());
}

#[test]
fn test_binary_text_height_calculation2() {
    let caps = BinCaps::binary_text(80).unwrap();
    assert_eq!(caps.calculate_binary_text_height(8000), Some(50)); // 8000 / (80*2)
    assert_eq!(caps.calculate_binary_text_height(4000), Some(25));
    assert_eq!(caps.calculate_binary_text_height(0), None);
}

#[test]
fn test_write_to_header_binary_text() {
    let caps = BinCaps::binary_text(80).unwrap().with_flags(
        true,
        LetterSpacing::EightPixel,
        AspectRatio::Square,
    );

    let mut header = SauceHeader::default();
    header.data_type = SauceDataType::BinaryText;
    caps.write_to_header(&mut header).unwrap();
    assert_eq!(header.file_type, 40); // 80/2
    assert_eq!(header.t_flags & 0b0000_0001, 0b0000_0001);
}

#[test]
fn test_write_to_header_xbin() {
    let caps = BinCaps::xbin(132, 50).unwrap();
    let mut header = SauceHeader::default();
    header.data_type = SauceDataType::XBin;
    caps.write_to_header(&mut header).unwrap();
    assert_eq!(header.file_type, 0);
    assert_eq!(header.t_info1, 132);
    assert_eq!(header.t_info2, 50);
    assert_eq!(header.t_flags, 0);
}
