use icy_sauce::{
    AspectRatio, BinaryCapabilities, Capabilities, CharacterCapabilities, CharacterFormat,
    LetterSpacing, MetaData, SauceDataType, SauceError, SauceRecord, SauceRecordBuilder, StripMode,
    header::SauceHeader, strip_sauce, strip_sauce_ex, strip_sauce_mut,
};
use proptest::prelude::*;

const MODES: [StripMode; 4] = [
    StripMode::Last,
    StripMode::LastStripFinalEof,
    StripMode::All,
    StripMode::AllStripFinalEof,
];

fn invalid_comments(payload_len: usize, comments: u8) -> Vec<u8> {
    let mut bytes = vec![b'A'; payload_len];
    bytes.push(0x1a);
    SauceHeader {
        comments,
        ..Default::default()
    }
    .write(&mut bytes)
    .unwrap();
    bytes
}

#[test]
fn malformed_comment_block_is_never_stripped_as_payload() {
    let bytes = invalid_comments(100, 1);
    assert!(SauceRecord::from_bytes(&bytes).unwrap().is_some());
    for mode in MODES {
        assert_eq!(strip_sauce(&bytes, mode), bytes);
        let detailed = strip_sauce_ex(&bytes, mode);
        assert_eq!(detailed.data, bytes);
        assert_eq!(detailed.records_removed, 0);
        assert_eq!(detailed.eof_bytes_removed, 0);
        let mut mutable = bytes.clone();
        assert_eq!(strip_sauce_mut(&mut mutable, mode), bytes);
    }
}

#[test]
fn multi_strip_stops_before_an_invalid_earlier_record() {
    let earlier = invalid_comments(100, 1);
    let mut bytes = earlier.clone();
    bytes.extend(SauceRecordBuilder::default().build().to_bytes().unwrap());
    for mode in [StripMode::All, StripMode::AllStripFinalEof] {
        assert_eq!(strip_sauce(&bytes, mode), earlier);
        let detailed = strip_sauce_ex(&bytes, mode);
        assert_eq!(detailed.data, earlier);
        assert_eq!(detailed.records_removed, 1);
        assert_eq!(detailed.eof_bytes_removed, 1);
    }
}

#[test]
fn decoded_fonts_stop_at_nul_but_raw_fields_remain_lossless() {
    for data_type in [SauceDataType::Character, SauceDataType::BinaryText] {
        for raw in [
            b"IBM VGA\0junk".to_vec(),
            b"\0junk".to_vec(),
            vec![b'X'; 22],
        ] {
            let record = SauceRecordBuilder::default()
                .data_type(data_type)
                .file_type(if data_type == SauceDataType::Character {
                    1
                } else {
                    40
                })
                .t_info_s(raw.clone().into())
                .unwrap()
                .build();
            let bytes = record.to_bytes().unwrap();
            let parsed = SauceRecord::from_bytes(&bytes).unwrap().unwrap();
            assert_eq!(parsed.header().t_info_s.as_slice(), raw);
            let font = match parsed.capabilities().unwrap() {
                Capabilities::Character(c) => c.font_opt,
                Capabilities::Binary(c) => c.font_opt,
                _ => unreachable!(),
            };
            let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
            assert_eq!(
                font.as_ref().map(|f| f.as_slice()),
                (end > 0).then_some(&raw[..end])
            );
            assert_eq!(parsed.to_builder().build().to_bytes().unwrap(), bytes);
        }
    }
}

#[test]
fn font_writers_reserve_a_terminator_and_reject_embedded_nuls() {
    let mut character = CharacterCapabilities::new(CharacterFormat::Ansi);
    let mut binary = BinaryCapabilities::binary_text(80).unwrap();
    character.set_font(vec![b'X'; 21].into()).unwrap();
    binary.set_font(vec![b'X'; 21].into()).unwrap();
    for caps in [
        Capabilities::Character(character.clone()),
        Capabilities::Binary(binary.clone()),
    ] {
        let record = SauceRecordBuilder::default()
            .capabilities(caps)
            .unwrap()
            .build();
        let bytes = record.to_bytes().unwrap();
        assert_eq!(bytes.last(), Some(&0));
        assert_eq!(&bytes[bytes.len() - 22..bytes.len() - 1], &[b'X'; 21]);
    }
    for font in [vec![b'X'; 22], b"IBM\0VGA".to_vec()] {
        assert!(character.set_font(font.clone().into()).is_err());
        assert!(binary.set_font(font.clone().into()).is_err());
        assert!(
            CharacterCapabilities::with_font(
                CharacterFormat::Ansi,
                80,
                25,
                false,
                LetterSpacing::Legacy,
                AspectRatio::Legacy,
                Some(font.clone().into()),
            )
            .is_err()
        );
        // Public fields cannot bypass serialization validation.
        character.font_opt = Some(font.clone().into());
        binary.font_opt = Some(font.into());
        assert!(
            SauceRecordBuilder::default()
                .capabilities(Capabilities::Character(character.clone()))
                .is_err()
        );
        assert!(
            SauceRecordBuilder::default()
                .capabilities(Capabilities::Binary(binary.clone()))
                .is_err()
        );
    }
}

#[test]
fn metadata_replaces_comments_and_uses_the_same_validation() {
    let original = SauceRecordBuilder::default()
        .file_size(42)
        .add_comment("Old".into())
        .unwrap()
        .build();
    let metadata = MetaData {
        title: "Title".into(),
        comments: vec!["New".into()],
        ..Default::default()
    };
    let modified = original
        .to_builder()
        .metadata(metadata.clone())
        .unwrap()
        .build();
    assert_eq!(modified.metadata(), metadata);
    assert_eq!(modified.file_size(), 42);
    assert_eq!(metadata.to_builder().unwrap().build().metadata(), metadata);
    assert!(
        modified
            .to_builder()
            .metadata(MetaData::default())
            .unwrap()
            .build()
            .comments()
            .is_empty()
    );

    assert!(matches!(
        SauceRecordBuilder::default().metadata(MetaData {
            comments: vec![vec![b'X'; 65].into()],
            ..Default::default()
        }),
        Err(SauceError::CommentTooLong(65))
    ));
    assert!(matches!(
        SauceRecordBuilder::default().metadata(MetaData {
            comments: vec!["Comment".into(); 256],
            ..Default::default()
        }),
        Err(SauceError::CommentLimitExceeded)
    ));
}

proptest! {
    #[test]
    fn arbitrary_missing_comment_counts_preserve_payload(payload_len in 0..17000usize, comments in 1..=255u8) {
        let data = invalid_comments(payload_len, comments);
        for mode in MODES {
            prop_assert_eq!(strip_sauce(&data, mode), data.as_slice());
            prop_assert_eq!(strip_sauce_ex(&data, mode).records_removed, 0);
        }
    }
}
