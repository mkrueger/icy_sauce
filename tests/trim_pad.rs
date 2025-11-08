use bstr::BString;
use icy_sauce::{
    Capabilities, CharacterCapabilities, CharacterFormat, SauceDataType, SauceRecord,
    SauceRecordBuilder,
};

fn build_character_record(
    title: BString,
    author: BString,
    group: BString,
    comment: Option<BString>,
    font: Option<BString>,
) -> SauceRecord {
    // Minimal character capabilities (no special flags; dimensions arbitrary)
    let char_caps = CharacterCapabilities::with_font(
        CharacterFormat::Ansi,
        80,
        25,
        true,
        icy_sauce::LetterSpacing::NinePixel,
        icy_sauce::AspectRatio::Square,
        font,
    )
    .unwrap();

    let mut builder = SauceRecordBuilder::default()
        .title(title)
        .unwrap()
        .author(author)
        .unwrap()
        .group(group)
        .unwrap()
        .data_type(SauceDataType::Character)
        .capabilities(Capabilities::Character(char_caps))
        .unwrap();

    if let Some(c) = comment {
        builder = builder.add_comment(c).unwrap();
    }

    builder.build()
}

fn round_trip(record: &SauceRecord) -> SauceRecord {
    let mut buf = Vec::new();
    record.write(&mut buf).unwrap();
    SauceRecord::from_bytes(&buf)
        .unwrap()
        .expect("SAUCE should parse")
}

#[test]
fn title_trims_trailing_spaces_after_round_trip() {
    let original = build_character_record(
        BString::from("Hello World  "), // trailing spaces that should be trimmed
        BString::from("Author"),
        BString::from("Group"),
        None,
        None,
    );
    let parsed = round_trip(&original);
    assert_eq!(parsed.title(), &BString::from("Hello World")); // trimmed
}

#[test]
fn author_trims_trailing_space_and_null() {
    // Include an internal null and trailing spaces + a trailing null; only trailing
    // space/null should be removed, internal null retained.
    let mut raw = b"Auth\0r  ".to_vec();
    raw.push(0); // explicit trailing null
    let author_with_noise = BString::from(raw);
    let original = build_character_record(
        BString::from("Title"),
        author_with_noise,
        BString::from("Group"),
        None,
        None,
    );
    let parsed = round_trip(&original);
    // Internal null stays; trailing spaces/null gone
    assert_eq!(parsed.author(), &BString::from(b"Auth\0r".to_vec()));
}

#[test]
fn group_trims_all_trailing_padding() {
    let original = build_character_record(
        BString::from("Title"),
        BString::from("Author"),
        BString::from("Group   "), // trailing spaces
        None,
        None,
    );
    let parsed = round_trip(&original);
    assert_eq!(parsed.group(), &BString::from("Group"));
}

#[test]
fn comment_trims_trailing_spaces() {
    let original = build_character_record(
        BString::from("Title"),
        BString::from("Author"),
        BString::from("Group"),
        Some(BString::from("Comment with pad   ")),
        None,
    );
    let parsed = round_trip(&original);
    assert_eq!(parsed.comments().len(), 1);
    assert_eq!(parsed.comments()[0], BString::from("Comment with pad"));
}

#[test]
fn comment_trims_trailing_nulls_and_spaces() {
    // Simulate user-supplied comment containing a trailing null + spaces
    let mut bytes = b"Mixed\0Data ".to_vec();
    bytes.extend_from_slice(b"  ");
    bytes.push(0);
    let original = build_character_record(
        BString::from("Title"),
        BString::from("Author"),
        BString::from("Group"),
        Some(BString::from(bytes)),
        None,
    );
    let parsed = round_trip(&original);
    assert_eq!(parsed.comments()[0], BString::from(b"Mixed\0Data".to_vec()));
}

#[test]
fn font_zero_padding_preserves_trailing_spaces_but_not_zeros() {
    // Font names reside in a zero-padded field; zero_trim should remove only trailing zeros.
    // We simulate trailing spaces we want preserved (they occur before the zero padding).
    let font_with_spaces = BString::from("FONT NAME   "); // trailing spaces intentional
    let original = build_character_record(
        BString::from("Title"),
        BString::from("Author"),
        BString::from("Group"),
        None,
        Some(font_with_spaces.clone()),
    );
    let parsed = round_trip(&original);
    // Extract capabilities again and inspect font via capabilities decoding.
    let caps = parsed.capabilities().expect("caps");
    match caps {
        Capabilities::Character(c) => {
            let font = c.font().expect("font should exist");
            // Spaces preserved (only trailing zeros would be trimmed; there were none).
            assert_eq!(font, &font_with_spaces);
        }
        _ => panic!("Expected character capabilities"),
    }
}

#[test]
fn raw_comment_block_layout_is_correct() {
    // Two comments -> ensure "COMNT" + (2 * 64) bytes exist before header.
    let original = build_character_record(
        BString::from("Title"),
        BString::from("Author"),
        BString::from("Group"),
        Some(BString::from("C1")),
        Some(BString::from("FONT")),
    )
    .to_builder()
    .add_comment(BString::from("Second line"))
    .unwrap()
    .build();

    let mut buf = Vec::new();
    original.write(&mut buf).unwrap();

    // Layout: [0x1A][COMNT][comment1 64][comment2 64][header 128]
    assert!(buf.len() >= 1 + 5 + 64 * 2 + 128);
    let eof = buf[0];
    assert_eq!(eof, 0x1A);

    let tag = &buf[1..6];
    assert_eq!(tag, b"COMNT");

    let c1 = &buf[6..6 + 64];
    let c2 = &buf[6 + 64..6 + 128];

    // Each comment is exactly 64 bytes, space padded.
    assert_eq!(c1.len(), 64);
    assert_eq!(c2.len(), 64);
    assert!(c1.starts_with(b"C1"));
    assert!(c2.starts_with(b"Second line"));

    // Header should occupy the last 128 bytes.
    let header_len = 128;
    let header_slice = &buf[buf.len() - header_len..];
    assert_eq!(header_slice.len(), header_len);
    assert_eq!(&header_slice[0..5], b"SAUCE");
}

#[test]
fn title_truncation_and_round_trip_preserves_trimmed_core() {
    // Builder refuses >35, but we can simulate near-limit with extra spaces.
    let long_with_spaces = "A".repeat(33) + "  ";
    let original = build_character_record(
        BString::from(long_with_spaces.clone()),
        BString::from("Author"),
        BString::from("Group"),
        None,
        None,
    );
    let parsed = round_trip(&original);
    // Trailing spaces trimmed
    assert_eq!(parsed.title(), &BString::from("A".repeat(33)));
}

#[test]
fn empty_title_author_group_round_trip() {
    let original = build_character_record(
        BString::from(""),
        BString::from(""),
        BString::from(""),
        Some(BString::from("  ")), // becomes empty after trim
        None,
    );
    let parsed = round_trip(&original);
    assert_eq!(parsed.title(), &BString::from(""));
    assert_eq!(parsed.author(), &BString::from(""));
    assert_eq!(parsed.group(), &BString::from(""));
    assert_eq!(parsed.comments()[0], BString::from("")); // trimmed
}
