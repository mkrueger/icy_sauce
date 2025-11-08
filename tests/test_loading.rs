use std::fs;

use bstr::BString;
use icy_sauce::{
    Capabilities, SauceRecord, SauceRecordBuilder,
    character::{AspectRatio, CharacterFormat, LetterSpacing},
};

#[test]
fn test_sauce_length() {
    let file = fs::read("tests/files/test1.pcb").unwrap();
    let info = SauceRecord::from_bytes(&file).unwrap().unwrap();

    // the EOF terminator is part of the sauce info according to spec that is what my old implementation did wrong
    assert_eq!(info.record_len(), 128);

    let new_info = SauceRecordBuilder::default().build().record_len();
    assert_eq!(new_info, 128);
}

#[test]
fn test_simple_file() {
    let file = fs::read("tests/files/test1.pcb").unwrap();
    let info = SauceRecord::from_bytes(&file).unwrap().unwrap();
    assert!(info.comments().is_empty());
    assert_eq!(info.record_len(), 128);
    assert_eq!(info.title(), &BString::from("Title"));
    assert_eq!(info.group(), &BString::from("Group"));
    assert_eq!(info.author(), &BString::from("Author"));

    // Use the unified capabilities method with pattern matching
    if let Some(Capabilities::Character(caps)) = info.capabilities() {
        assert_eq!(caps.format, CharacterFormat::PCBoard);
        assert_eq!(caps.columns, 40);
        assert_eq!(caps.lines, 42);
        assert_eq!(caps.aspect_ratio, AspectRatio::Legacy);
        assert!(!caps.ice_colors);
        assert_eq!(caps.letter_spacing, LetterSpacing::Legacy);
    } else {
        panic!("Expected Character capabilities");
    }
}

#[test]
fn test_comments() {
    let file = fs::read("tests/files/test2.ans").unwrap();
    let info = SauceRecord::from_bytes(&file).unwrap().unwrap();
    assert_eq!(info.comments().len(), 2);
    assert_eq!(info.record_len(), 128 + 2 * 64 + 5);
    assert_eq!(info.title(), &BString::from("Title"));
    assert_eq!(info.group(), &BString::from("Group"));
    assert_eq!(info.author(), &BString::from("Author"));

    assert_eq!(info.comments()[0], BString::from("+9px & AR"));
    assert_eq!(info.comments()[1], BString::from("and 2 Comments!!!!"));

    // Use the unified capabilities method with pattern matching
    if let Some(Capabilities::Character(caps)) = info.capabilities() {
        assert_eq!(caps.format, CharacterFormat::Ansi);
        assert_eq!(caps.columns, 80);
        assert_eq!(caps.lines, 25);
        // Check if flags indicate 9px font and aspect ratio set
        assert_eq!(caps.letter_spacing, LetterSpacing::NinePixel);
        assert_ne!(caps.aspect_ratio, AspectRatio::Legacy);
        assert!(!caps.ice_colors); // Not set in the file
    } else {
        panic!("Expected Character capabilities");
    }
}
