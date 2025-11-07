use std::fs;

use bstr::BString;
use icy_sauce::{
    SauceInformation, SauceInformationBuilder,
    char_caps::{AspectRatio, CharacterFormat, LetterSpacing},
};

#[test]
fn test_sauce_length() {
    let file = fs::read("tests/files/test1.pcb").unwrap();
    let info = SauceInformation::read(&file).unwrap().unwrap();

    // the EOF terminator is part of the sauce info according to spec that is what my old implementation did wrong
    assert_eq!(info.info_len(), 129);

    let new_info = SauceInformationBuilder::default().build().info_len();
    assert_eq!(new_info, 129);
}

#[test]
fn test_simple_file() {
    let file = fs::read("tests/files/test1.pcb").unwrap();
    let info = SauceInformation::read(&file).unwrap().unwrap();
    assert!(info.comments().is_empty());
    assert_eq!(info.info_len(), 129);
    assert_eq!(info.title(), &BString::from("Title"));
    assert_eq!(info.group(), &BString::from("Group"));
    assert_eq!(info.author(), &BString::from("Author"));

    let caps = info.get_character_capabilities().unwrap();
    assert_eq!(caps.format, CharacterFormat::PCBoard);
    assert_eq!(caps.width, 40);
    assert_eq!(caps.height, 42);
    assert_eq!(caps.aspect_ratio, AspectRatio::Legacy);
    assert!(!caps.use_ice);
    assert_eq!(caps.letter_spacing, LetterSpacing::Legacy);
}

#[test]
fn test_comments() {
    let file = fs::read("tests/files/test2.ans").unwrap();
    let info = SauceInformation::read(&file).unwrap().unwrap();
    assert_eq!(info.comments().len(), 2);
    assert_eq!(info.info_len(), 129 + 2 * 64 + 5);
    assert_eq!(info.title(), &BString::from("Title"));
    assert_eq!(info.group(), &BString::from("Group"));
    assert_eq!(info.author(), &BString::from("Author"));

    assert_eq!(info.comments()[0], BString::from("+9px & AR"));
    assert_eq!(info.comments()[1], BString::from("and 2 Comments!!!!"));

    let caps = info.get_character_capabilities().unwrap();
    assert_eq!(caps.format, CharacterFormat::Ansi);
    assert_eq!(caps.width, 80);
    assert_eq!(caps.height, 25);
    // Check if flags indicate 9px font and aspect ratio set
    assert_eq!(caps.letter_spacing, LetterSpacing::NinePixel);
    assert_ne!(caps.aspect_ratio, AspectRatio::Legacy);
    assert!(!caps.use_ice); // Not set in the file
}
