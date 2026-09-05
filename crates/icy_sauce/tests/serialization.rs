use icy_sauce::{SauceDataType, SauceDate, SauceError, SauceRecord, SauceRecordBuilder};
use proptest::prelude::*;

#[test]
fn invalid_dates_fail_before_writing_any_bytes() {
    for date in [
        SauceDate::new(-1, 1, 1),
        SauceDate::new(10000, 1, 1),
        SauceDate::new(i32::MIN, 1, 1),
        SauceDate::new(i32::MAX, 1, 1),
        SauceDate::new(2026, 100, 1),
        SauceDate::new(2026, 1, 100),
        SauceDate::new(2026, 255, 255),
    ] {
        let mut output = b"original".to_vec();
        assert!(matches!(
            date.write(&mut output),
            Err(SauceError::UnsupportedSauceDate(_))
        ));
        assert_eq!(output, b"original");

        let record = SauceRecordBuilder::default()
            .date(date)
            .add_comment("Comment".into())
            .unwrap()
            .build();
        assert!(matches!(
            record.header().write(&mut output),
            Err(SauceError::UnsupportedSauceDate(_))
        ));
        assert!(matches!(
            record.write(&mut output),
            Err(SauceError::UnsupportedSauceDate(_))
        ));
        assert!(matches!(
            record.write_without_eof(&mut output),
            Err(SauceError::UnsupportedSauceDate(_))
        ));
        assert_eq!(output, b"original");
        assert!(matches!(
            record.to_bytes(),
            Err(SauceError::UnsupportedSauceDate(_))
        ));
        assert!(matches!(
            record.to_bytes_without_eof(),
            Err(SauceError::UnsupportedSauceDate(_))
        ));
    }
}

#[test]
fn wire_dates_still_allow_unknown_and_non_calendar_values() {
    for date in [SauceDate::default(), SauceDate::new(9999, 99, 99)] {
        let record = SauceRecordBuilder::default().date(date.clone()).build();
        let encoded = record.to_bytes().unwrap();
        assert_eq!(encoded.len(), 129);
        let decoded = SauceRecord::from_bytes(&encoded).unwrap().unwrap();
        assert_eq!(decoded.date(), date);
    }
}

#[test]
fn equality_ignores_initialized_and_empty_capability_caches() {
    // Cover both a cached Some(capabilities) and a cached None.
    for data_type in [SauceDataType::Character, SauceDataType::Undefined(99)] {
        let original = SauceRecordBuilder::default().data_type(data_type).build();
        let cloned = original.clone();
        assert!(original == cloned);
        original.capabilities();
        assert!(original == cloned);
        assert!(cloned == original);
        assert!(original == original.clone());
        cloned.capabilities();
        assert!(original == cloned);

        let modified = original
            .to_builder()
            .title("Different".into())
            .unwrap()
            .build();
        assert!(original != modified);
    }
}

#[test]
fn clearing_comments_keeps_header_and_record_lengths_consistent() {
    let original = SauceRecordBuilder::default()
        .file_size(42)
        .add_comment("Original".into())
        .unwrap()
        .build();
    let cleared = original.to_builder().clear_comments().build();
    assert!(cleared.comments().is_empty());
    assert_eq!(cleared.header().comments, 0);
    assert_eq!(cleared.file_size(), 42);
    assert_eq!(cleared.record_len(), 128);
    assert_eq!(cleared.to_bytes().unwrap().len(), 129);
    assert!(original != cleared);
}

proptest! {
    #[test]
    fn serialized_dates_are_fixed_width_or_rejected(year in any::<i32>(), month in any::<u8>(), day in any::<u8>()) {
        let date = SauceDate::new(year, month, day);
        let mut bytes = Vec::new();
        let result = date.write(&mut bytes);
        if (0..=9999).contains(&year) && month <= 99 && day <= 99 {
            prop_assert!(result.is_ok());
            prop_assert_eq!(bytes.len(), 8);
            prop_assert_eq!(SauceDate::from_bytes(&bytes), Some(date));
        } else {
            prop_assert!(matches!(result, Err(SauceError::UnsupportedSauceDate(_))));
            prop_assert!(bytes.is_empty());
        }
    }

    #[test]
    fn representable_dates_round_trip(year in 0..=9999i32, month in 0..=99u8, day in 0..=99u8) {
        let record = SauceRecordBuilder::default().date(SauceDate::new(year, month, day)).build();
        let bytes = record.to_bytes().unwrap();
        prop_assert_eq!(bytes.len(), 129);
        let decoded = SauceRecord::from_bytes(&bytes).unwrap().unwrap();
        prop_assert!(record == decoded);
    }
}
