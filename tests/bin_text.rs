use bstr::BString;
use icy_sauce::{
    Capabilities, SauceDataType, SauceError, SauceRecord, SauceRecordBuilder,
    binary::{BinaryCapabilities, BinaryFormat},
    character::{AspectRatio, LetterSpacing},
    header::SauceHeader,
};

#[test]
fn test_binary_text_height_calculation() {
    let caps = BinaryCapabilities::binary_text(80).unwrap();
    let info = SauceRecordBuilder::default()
        .data_type(SauceDataType::BinaryText)
        .capabilities(Capabilities::Binary(caps))
        .unwrap()
        .file_size(8000)
        .build();

    let mut data = Vec::new();
    info.write(&mut data).unwrap();
    let parsed = SauceRecord::from_bytes(&data).unwrap().unwrap();

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
    let err = BinaryCapabilities::binary_text(81).unwrap_err();
    match err {
        SauceError::BinFileWidthLimitExceeded(w) => assert_eq!(w, 81),
        other => panic!("Unexpected error: {:?}", other),
    }
}

#[test]
fn test_binary_text_max_width() {
    assert!(BinaryCapabilities::binary_text(510).is_ok());
    assert!(BinaryCapabilities::binary_text(512).is_err());
}

#[test]
fn test_binarytext_width_encoding() {
    let caps = BinaryCapabilities::binary_text(160).unwrap();
    let info = SauceRecordBuilder::default()
        .data_type(SauceDataType::BinaryText)
        .capabilities(Capabilities::Binary(caps))
        .unwrap()
        .build();

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
    let mut caps = BinaryCapabilities::binary_text(80).unwrap();
    caps.set_font(BString::from("IBM VGA")).unwrap();

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
        assert_eq!(bin_caps.font(), Some(&BString::from("IBM VGA")));
    } else {
        panic!("Expected Binary capabilities");
    }
}

#[test]
fn test_binary_text_flags() {
    let mut caps = BinaryCapabilities::binary_text(80).unwrap();
    caps.ice_colors = true;
    caps.letter_spacing = LetterSpacing::EightPixel;
    caps.aspect_ratio = AspectRatio::Square;

    let info = SauceRecordBuilder::default()
        .data_type(SauceDataType::BinaryText)
        .capabilities(Capabilities::Binary(caps.clone()))
        .unwrap()
        .build();

    // Round-trip: ensure high-level fields preserved
    let mut data = Vec::new();
    info.write(&mut data).unwrap();
    let parsed = SauceRecord::from_bytes(&data).unwrap().unwrap();

    if let Some(Capabilities::Binary(bin_caps)) = parsed.capabilities() {
        assert!(bin_caps.ice_colors);
        assert_eq!(bin_caps.letter_spacing, LetterSpacing::EightPixel);
        assert_eq!(bin_caps.aspect_ratio, AspectRatio::Square);
    } else {
        panic!("Expected Binary capabilities");
    }

    // Low-level bit check
    let mut header = SauceHeader::default();
    header.data_type = SauceDataType::BinaryText;
    caps.encode_into_header(&mut header).unwrap();
    assert_ne!(header.t_flags & 0b0000_0001, 0); // ICE flag
    assert_ne!(header.t_flags & 0b0000_0010, 0); // 8-pixel spacing
    assert_ne!(header.t_flags & 0b0001_0000, 0); // Square aspect ratio
}

#[test]
fn test_binary_text_font_too_long() {
    let long_font = BString::from("This font name is way too long!");
    let mut caps = BinaryCapabilities::binary_text(80).unwrap();
    let result = caps.set_font(long_font);
    assert!(matches!(result, Err(SauceError::FontNameTooLong(_))));
}

#[test]
fn test_binary_text_empty_font() {
    let mut caps = BinaryCapabilities::binary_text(80).unwrap();
    caps.set_font(BString::from("")).unwrap();
    assert!(caps.font().is_none());
}

#[test]
fn test_xbin_roundtrip() {
    let test_cases = vec![(80, 25), (132, 50), (256, 100), (1, 1), (65535, 65535)];

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
            assert!(!bin_caps.ice_colors);
            assert_eq!(bin_caps.letter_spacing, LetterSpacing::Legacy);
            assert_eq!(bin_caps.aspect_ratio, AspectRatio::Legacy);
            assert!(bin_caps.font().is_none());
        } else {
            panic!("Expected Binary capabilities");
        }
    }
}

#[test]
fn test_xbin_ignores_flags_and_font() {
    let mut caps = BinaryCapabilities::xbin(80, 25).unwrap();
    caps.ice_colors = true;
    caps.letter_spacing = LetterSpacing::EightPixel;
    caps.aspect_ratio = AspectRatio::Square;
    caps.set_font(BString::from("Ignored")).unwrap();

    let info = SauceRecordBuilder::default()
        .data_type(SauceDataType::XBin)
        .capabilities(Capabilities::Binary(caps))
        .unwrap()
        .build();

    let mut data = Vec::new();
    info.write(&mut data).unwrap();
    let parsed = SauceRecord::from_bytes(&data).unwrap().unwrap();

    if let Some(Capabilities::Binary(bin_caps)) = parsed.capabilities() {
        // Parsed values should reset to defaults for XBin
        assert!(!bin_caps.ice_colors);
        assert_eq!(bin_caps.letter_spacing, LetterSpacing::Legacy);
        assert_eq!(bin_caps.aspect_ratio, AspectRatio::Legacy);
        assert!(bin_caps.font().is_none());
    } else {
        panic!("Expected Binary capabilities");
    }
}

#[test]
fn test_get_bin_caps_wrong_type() {
    let info = SauceRecordBuilder::default()
        .data_type(SauceDataType::Character)
        .build();

    match info.capabilities() {
        Some(Capabilities::Binary(_)) => panic!("Should not return Binary for Character type"),
        _ => {}
    }

    let info = SauceRecordBuilder::default()
        .data_type(SauceDataType::Audio)
        .build();

    match info.capabilities() {
        Some(Capabilities::Binary(_)) => panic!("Should not return Binary for Audio type"),
        _ => {}
    }
}

#[test]
fn test_binary_text_all_valid_widths() {
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
    assert!(BinaryCapabilities::binary_text(510).is_ok());
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
    assert!(!caps.ice_colors);
    assert!(caps.font().is_none());
    assert!(BinaryCapabilities::xbin(0, 25).is_err());
    assert!(BinaryCapabilities::xbin(80, 0).is_err());
}

#[test]
fn test_binary_text_height_calculation2() {
    let caps = BinaryCapabilities::binary_text(80).unwrap();
    assert_eq!(caps.binary_text_height_from_file_size(8000), Some(50));
    assert_eq!(caps.binary_text_height_from_file_size(4000), Some(25));
    assert_eq!(caps.binary_text_height_from_file_size(0), None);
}

#[test]
fn test_write_to_header_binary_text() {
    let mut caps = BinaryCapabilities::binary_text(80).unwrap();
    caps.ice_colors = true;
    caps.letter_spacing = LetterSpacing::EightPixel;
    caps.aspect_ratio = AspectRatio::Square;

    let mut header = SauceHeader::default();
    header.data_type = SauceDataType::BinaryText;
    caps.encode_into_header(&mut header).unwrap();
    assert_eq!(header.file_type, 40);
    assert_ne!(header.t_flags & 0b0000_0001, 0);
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
