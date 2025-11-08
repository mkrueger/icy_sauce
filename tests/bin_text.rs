use bstr::BString;
use icy_sauce::{
    Capabilities, SauceDataType, SauceError, SauceRecord, SauceRecordBuilder,
    binary::{BinaryCapabilities, BinaryFormat},
    character::{AspectRatio, LetterSpacing},
    header::SauceHeader,
};

#[test]
fn test_binary_text_height_calculation() {
    // Build proper BinaryText caps with even width = 80
    let caps = BinaryCapabilities::binary_text(80).unwrap();

    // FileSize is the size of the original data (without SAUCE & comments)
    // Height = file_size / (width * 2) = 8000 / (80 * 2) = 50
    let info = SauceRecordBuilder::default()
        .data_type(SauceDataType::BinaryText)
        .capabilities(Capabilities::Binary(caps))
        .unwrap()
        .file_size(8000)
        .build();

    let mut data = Vec::new();
    info.write(&mut data).unwrap();
    let parsed = SauceRecord::from_bytes(&data).unwrap().unwrap();

    // Extract BinCaps from the unified capabilities enum
    if let Some(Capabilities::Binary(bin_caps)) = parsed.capabilities() {
        assert_eq!(
            bin_caps.binary_text_height_from_file_size(parsed.file_size()),
            Some(50)
        );
    } else {
        panic!("Expected Binary capabilities");
    }
}

#[test]
fn test_binary_text_odd_width_rejected() {
    // Width must be even (spec: encoded as width/2 in file_type)
    let err = BinaryCapabilities::binary_text(81).unwrap_err();
    match err {
        icy_sauce::SauceError::BinFileWidthLimitExceeded(w) => assert_eq!(w, 81),
        other => panic!("Unexpected error: {:?}", other),
    }
}

#[test]
fn test_binary_text_max_width() {
    // Maximum width is 510 (255 * 2)
    assert!(BinaryCapabilities::binary_text(510).is_ok());
    assert!(BinaryCapabilities::binary_text(512).is_err());
}

#[test]
fn test_binarytext_width_encoding() {
    let caps: BinaryCapabilities = BinaryCapabilities::binary_text(160).unwrap();

    let info = SauceRecordBuilder::default()
        .data_type(SauceDataType::BinaryText)
        .capabilities(Capabilities::Binary(caps))
        .unwrap()
        .build();

    // Access file_type through binary capabilities
    let mut data = Vec::new();
    info.write(&mut data).unwrap();
    let parsed = SauceRecord::from_bytes(&data).unwrap().unwrap();

    if let Some(Capabilities::Binary(bin_caps)) = parsed.capabilities() {
        assert_eq!(bin_caps.columns, 160);
    } else {
        panic!("Expected Binary capabilities");
    }
}

#[test]
fn test_binary_text_font() {
    // Test BinaryText with font name
    let caps = BinaryCapabilities::binary_text(80)
        .unwrap()
        .font(BString::from("IBM VGA"))
        .unwrap();

    let info = SauceRecordBuilder::default()
        .data_type(SauceDataType::BinaryText)
        .capabilities(Capabilities::Binary(caps))
        .unwrap()
        .file_size(4000)
        .build();

    let mut data = Vec::new();
    info.write(&mut data).unwrap();
    let parsed = SauceRecord::from_bytes(&data).unwrap().unwrap();

    if let Some(Capabilities::Binary(bin_caps)) = parsed.capabilities() {
        assert_eq!(bin_caps.font, Some(BString::from("IBM VGA")));
    } else {
        panic!("Expected Binary capabilities");
    }
}

#[test]
fn test_binary_text_flags() {
    // Test BinaryText with ANSi flags
    let caps = BinaryCapabilities::binary_text(80).unwrap().flags(
        true,
        LetterSpacing::EightPixel,
        AspectRatio::Square,
    );

    let info = SauceRecordBuilder::default()
        .data_type(SauceDataType::BinaryText)
        .capabilities(Capabilities::Binary(caps))
        .unwrap()
        .build();

    let mut data = Vec::new();
    info.write(&mut data).unwrap();
    let parsed = SauceRecord::from_bytes(&data).unwrap().unwrap();

    if let Some(Capabilities::Binary(bin_caps)) = parsed.capabilities() {
        // Check flag bits
        assert_ne!(bin_caps.flags & 0b0000_0001, 0); // ICE flag set
        assert_ne!(bin_caps.flags & 0b0000_0010, 0); // 8-pixel letter spacing
        assert_ne!(bin_caps.flags & 0b0001_0000, 0); // Square aspect ratio
    } else {
        panic!("Expected Binary capabilities");
    }
}

#[test]
fn test_binary_text_font_too_long() {
    // Font name max length is 22 bytes
    let long_font = BString::from("This font name is way too long!");
    let result = BinaryCapabilities::binary_text(80).unwrap().font(long_font);

    assert!(matches!(result, Err(SauceError::FontNameTooLong(_))));
}

#[test]
fn test_binary_text_empty_font() {
    // Empty font should be treated as None
    let caps = BinaryCapabilities::binary_text(80)
        .unwrap()
        .font(BString::from(""))
        .unwrap();

    assert!(caps.font.is_none());
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
        let caps = BinaryCapabilities::xbin(width, height).unwrap();

        let info = SauceRecordBuilder::default()
            .data_type(SauceDataType::XBin)
            .capabilities(Capabilities::Binary(caps))
            .unwrap()
            .build();

        let mut data = Vec::new();
        info.write(&mut data).unwrap();
        let parsed = SauceRecord::from_bytes(&data).unwrap().unwrap();

        if let Some(Capabilities::Binary(bin_caps)) = parsed.capabilities() {
            assert_eq!(bin_caps.format, BinaryFormat::XBin);
            assert_eq!(bin_caps.columns, width);
            assert_eq!(bin_caps.lines, height);
            assert_eq!(bin_caps.flags, 0); // XBin doesn't use flags
            assert!(bin_caps.font.is_none()); // XBin doesn't use font
        } else {
            panic!("Expected Binary capabilities");
        }
    }
}

#[test]
fn test_xbin_ignores_flags_and_font() {
    // XBin should ignore flags and font settings
    let caps = BinaryCapabilities::xbin(80, 25)
        .unwrap()
        .flags(true, LetterSpacing::EightPixel, AspectRatio::Square)
        .font(BString::from("Ignored"))
        .unwrap();

    assert_eq!(caps.flags, 0);
    assert!(caps.font.is_none());
}

#[test]
fn test_get_bin_caps_wrong_type() {
    // Test that capabilities returns None for non-binary types
    let info = SauceRecordBuilder::default()
        .data_type(SauceDataType::Character)
        .build();

    // capabilities() returns None if the type doesn't match
    match info.capabilities() {
        Some(Capabilities::Binary(_)) => panic!("Should not return Binary for Character type"),
        _ => {} // Expected: either None or Character capabilities
    }

    let info = SauceRecordBuilder::default()
        .data_type(SauceDataType::Audio)
        .build();

    match info.capabilities() {
        Some(Capabilities::Binary(_)) => panic!("Should not return Binary for Audio type"),
        _ => {} // Expected: either None or Audio capabilities
    }
}

#[test]
fn test_binary_text_all_valid_widths() {
    // Test a sampling of valid even widths
    for width in (2..=510).step_by(2).take(10) {
        let caps = BinaryCapabilities::binary_text(width).unwrap();
        assert_eq!(caps.columns, width);
    }
}

#[test]
fn test_sauce_information_calculate_height_methods() {
    let caps = BinaryCapabilities::binary_text(80).unwrap();

    let info = SauceRecordBuilder::default()
        .data_type(SauceDataType::BinaryText)
        .capabilities(Capabilities::Binary(caps))
        .unwrap()
        .file_size(8000)
        .build();

    // Since calculation moved to BinCaps, use it directly
    if let Some(Capabilities::Binary(bin_caps)) = info.capabilities() {
        assert_eq!(
            bin_caps.binary_text_height_from_file_size(info.file_size()),
            Some(50)
        );
    } else {
        panic!("Expected Binary capabilities");
    }
}

#[test]
fn test_binary_text_width_constraints() {
    assert!(BinaryCapabilities::binary_text(80).is_ok());
    assert!(BinaryCapabilities::binary_text(160).is_ok());
    assert!(BinaryCapabilities::binary_text(510).is_ok()); // max
    assert!(BinaryCapabilities::binary_text(0).is_err());
    assert!(BinaryCapabilities::binary_text(81).is_err());
    assert!(BinaryCapabilities::binary_text(511).is_err());
    assert!(BinaryCapabilities::binary_text(512).is_err());
}

#[test]
fn test_xbin_creation() {
    let caps = BinaryCapabilities::xbin(80, 25).unwrap();
    assert_eq!(caps.format, BinaryFormat::XBin);
    assert_eq!(caps.columns, 80);
    assert_eq!(caps.lines, 25);
    assert_eq!(caps.flags, 0);
    assert!(caps.font.is_none());
    // invalid dimensions
    assert!(BinaryCapabilities::xbin(0, 25).is_err());
    assert!(BinaryCapabilities::xbin(80, 0).is_err());
}

#[test]
fn test_binary_text_height_calculation2() {
    let caps = BinaryCapabilities::binary_text(80).unwrap();
    assert_eq!(caps.binary_text_height_from_file_size(8000), Some(50)); // 8000 / (80*2)
    assert_eq!(caps.binary_text_height_from_file_size(4000), Some(25));
    assert_eq!(caps.binary_text_height_from_file_size(0), None);
}

#[test]
fn test_write_to_header_binary_text() {
    let caps = BinaryCapabilities::binary_text(80).unwrap().flags(
        true,
        LetterSpacing::EightPixel,
        AspectRatio::Square,
    );

    let mut header = SauceHeader::default();
    header.data_type = SauceDataType::BinaryText;
    caps.encode_into_header(&mut header).unwrap();
    assert_eq!(header.file_type, 40); // 80/2
    assert_eq!(header.t_flags & 0b0000_0001, 0b0000_0001);
}

#[test]
fn test_write_to_header_xbin() {
    let caps = BinaryCapabilities::xbin(132, 50).unwrap();
    let mut header = SauceHeader::default();
    header.data_type = SauceDataType::XBin;
    caps.encode_into_header(&mut header).unwrap();
    assert_eq!(header.file_type, 0);
    assert_eq!(header.t_info1, 132);
    assert_eq!(header.t_info2, 50);
    assert_eq!(header.t_flags, 0);
}
