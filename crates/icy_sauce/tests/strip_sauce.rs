use bstr::BString;
use icy_sauce::{SauceRecordBuilder, StripMode, strip_sauce, strip_sauce_mut};

/// Helper to create minimal SAUCE record bytes (no comments)
fn create_sauce_record() -> Vec<u8> {
    let sauce = SauceRecordBuilder::default()
        .title(BString::from("Test"))
        .unwrap()
        .build();
    sauce.to_bytes()
}

/// Helper to create SAUCE with comment block
fn create_sauce_with_comments(num_comments: u8) -> Vec<u8> {
    let mut builder = SauceRecordBuilder::default();
    for i in 0..num_comments {
        builder = builder
            .add_comment(BString::from(format!("Comment {}", i)))
            .unwrap();
    }
    let sauce = builder.build();
    sauce.to_bytes()
}

#[test]
fn test_no_sauce_present() {
    let data = b"Just some regular file content";

    // All modes should return original data
    assert_eq!(strip_sauce(data, StripMode::LastStripFinalEof), data);
    assert_eq!(strip_sauce(data, StripMode::Last), data);
    assert_eq!(strip_sauce(data, StripMode::All), data);
    assert_eq!(strip_sauce(data, StripMode::All), data);
}

#[test]
fn test_last_mode_single_eof() {
    let mut data = b"Content".to_vec();
    // to_bytes() already includes one EOF at the start
    data.extend_from_slice(&create_sauce_record());

    let stripped = strip_sauce(&data, StripMode::LastStripFinalEof);
    assert_eq!(stripped, b"Content");
}

#[test]
fn test_last_mode_multiple_eof() {
    let mut data = b"Content".to_vec();
    data.push(0x1A); // Extra EOF 1
    data.push(0x1A); // Extra EOF 2
    // to_bytes() includes one EOF, so total = 3 EOFs
    data.extend_from_slice(&create_sauce_record());

    let stripped = strip_sauce(&data, StripMode::LastStripFinalEof);
    // Should remove SAUCE + single EOF, preserve other 2
    assert_eq!(stripped, b"Content\x1A\x1A");
}

#[test]
fn test_trailing_eof() {
    let mut data = b"Content".to_vec();
    data.push(0x1A); // Extra EOF 1
    data.push(0x1A); // Extra EOF 2
    // to_bytes() includes one EOF, so total = 3 EOFs
    data.extend_from_slice(&create_sauce_record());
    data.push(0x1A); // Extra EOF 3

    let stripped = strip_sauce(&data, StripMode::All);
    // Should remove SAUCE + single EOF, preserve other 2
    assert_eq!(stripped, data);

    let stripped = strip_sauce(&data, StripMode::All);
    // Should remove SAUCE + single EOF, preserve other 2
    assert_eq!(stripped, data);

    let stripped = strip_sauce(&data, StripMode::LastStripFinalEof);
    // Should remove SAUCE + single EOF, preserve other 2
    assert_eq!(stripped, data);

    let stripped = strip_sauce(&data, StripMode::Last);
    // Should remove SAUCE + single EOF, preserve other 2
    assert_eq!(stripped, data);
}

#[test]
fn test_last_mode_multiple_eof_preserve() {
    let mut data = b"Content".to_vec();
    data.push(0x1A); // Extra EOF 1
    data.push(0x1A); // Extra EOF 2
    // to_bytes() includes one EOF, so total = 3 EOFs
    data.extend_from_slice(&create_sauce_record());

    let stripped = strip_sauce(&data, StripMode::Last);
    // Should remove SAUCE + single EOF, preserve other 2
    assert_eq!(stripped, b"Content\x1A\x1A\x1A");
}

#[test]
fn test_last_mode_no_eof() {
    let mut data = b"Content".to_vec();
    // Use to_bytes_without_eof to avoid the automatic EOF
    let sauce = SauceRecordBuilder::default()
        .title(BString::from("Test"))
        .unwrap()
        .build();
    data.extend_from_slice(&sauce.to_bytes_without_eof());

    let stripped = strip_sauce(&data, StripMode::LastStripFinalEof);
    assert_eq!(stripped, b"Content");
}

#[test]
fn test_last_preserve_eof_single_eof() {
    let mut data = b"Content".to_vec();
    // to_bytes() already has EOF
    data.extend_from_slice(&create_sauce_record());

    let stripped = strip_sauce(&data, StripMode::Last);
    // Should preserve the EOF
    assert_eq!(stripped, b"Content\x1A");
}

#[test]
fn test_last_preserve_eof_multiple_eof() {
    let mut data = b"Content".to_vec();
    data.push(0x1A); // Extra 1
    data.push(0x1A); // Extra 2
    // to_bytes() adds one more = 3 total
    data.extend_from_slice(&create_sauce_record());

    let stripped = strip_sauce(&data, StripMode::Last);
    // Should preserve all EOFs
    assert_eq!(stripped, b"Content\x1A\x1A\x1A");
}

#[test]
fn test_multiple_sauce_records() {
    let mut data = b"Content".to_vec();

    // First SAUCE (to_bytes includes EOF)
    data.extend_from_slice(&create_sauce_record());

    // Second SAUCE (to_bytes includes EOF)
    data.extend_from_slice(&create_sauce_record());

    let stripped = strip_sauce(&data, StripMode::All);
    // Should remove both SAUCE + one EOF each, preserve content
    assert_eq!(stripped, b"Content");
}

#[test]
fn test_multiple_sauce_with_multiple_eof_between() {
    let mut data = b"Content".to_vec();

    // First SAUCE (includes EOF)
    data.push(0x1A); // Extra EOF (E1) before first record
    let first = create_sauce_record();
    data.extend_from_slice(&first);

    // Second SAUCE (includes EOF)
    data.push(0x1A); // Extra EOF (E2) before second record
    let second = create_sauce_record();
    data.extend_from_slice(&second);

    let stripped = strip_sauce(&data, StripMode::All);
    // Layout (C=Content, e=extra EOF, rE=record EOF inside to_bytes, R=record bytes):
    // C e(E1) rE(first) R1 e(E2) rE(second) R2
    // Algorithm pre-consumes rE(second), strips R2, consumes rE(second) only (leaving e(E2)).
    // Next iteration sees tail: C e(E1) rE(first) R1 e(E2) -> pre-consume e(E2)
    // Tail ends with e(E2) not R1 header; cannot strip further due to stacked separation.
    // Expected remaining: C e(E1) first_record e(E2)
    let mut expected = b"Content".to_vec();
    expected.push(0x1A); // E1
    expected.extend_from_slice(&first); // includes its own leading EOF
    expected.push(0x1A); // E2 leftover
    assert_eq!(stripped, &expected);
}

#[test]
fn test_stacked_eof_stops_multi_strip() {
    let mut data = b"Content".to_vec();

    // Construct: Content, extra EOF A, extra EOF B (stacked), first record, extra EOF C, extra EOF D (stacked), second record
    data.push(0x1A); // A
    data.push(0x1A); // B (stacked before first record)
    let first = create_sauce_record();
    data.extend_from_slice(&first);
    data.push(0x1A); // C
    data.push(0x1A); // D (stacked before second record)
    let second = create_sauce_record();
    data.extend_from_slice(&second);

    let stripped = strip_sauce(&data, StripMode::All);
    // After stripping second record: D consumed with its record, C remains; stacked A,B still block
    // traversal to earlier record because more than one EOF separated it historically.
    // Expected: Content A B first_record C D. Second record removed; its own leading EOF consumed.
    // Both stacked EOFs after first record (C,D) remain because we stop when tail no longer ends with a header.
    let mut expected = b"Content".to_vec();
    expected.push(0x1A); // A
    expected.push(0x1A); // B
    expected.extend_from_slice(&first);
    expected.push(0x1A); // C
    expected.push(0x1A); // D
    assert_eq!(stripped, &expected);

    // Last mode only strips last record, same result.
    let stripped_last = strip_sauce(&data, StripMode::LastStripFinalEof);
    assert_eq!(stripped_last, &expected);
}

#[test]
fn test_all_mode_removes_final_eof() {
    let mut data = b"Content".to_vec();
    // to_bytes includes EOF
    data.extend_from_slice(&create_sauce_record());

    let stripped = strip_sauce(&data, StripMode::All);
    // Should remove SAUCE and EOF
    assert_eq!(stripped, b"Content");
}

#[test]
fn test_all_mode_multiple_trailing_eof() {
    let mut data = b"Content".to_vec();
    data.push(0x1A); // Extra 1
    data.push(0x1A); // Extra 2
    // to_bytes adds one more = 3 total EOFs before SAUCE
    data.extend_from_slice(&create_sauce_record());
    let stripped = strip_sauce(&data, StripMode::All);

    // New semantics: StripMode::All removes only the single EOF directly preceding the SAUCE record.
    // Three EOFs existed (extra1, extra2, record EOF). Only record EOF is removed, leaving the two extras.
    assert_eq!(stripped, b"Content\x1A\x1A");
}

#[test]
fn test_all_preserve_trailing_eof() {
    let mut data = b"Content".to_vec();
    data.push(0x1A);
    data.push(0x1A);
    // to_bytes includes EOF
    data.extend_from_slice(&create_sauce_record());

    let stripped = strip_sauce(&data, StripMode::All);
    // Should preserve the content + remaining EOFs after stripping
    assert_eq!(stripped, b"Content\x1A\x1A");
}

#[test]
fn test_sauce_with_comments() {
    let mut data = b"Content".to_vec();
    // to_bytes includes EOF
    data.extend_from_slice(&create_sauce_with_comments(2));

    let stripped = strip_sauce(&data, StripMode::LastStripFinalEof);
    assert_eq!(stripped, b"Content");
}

#[test]
fn test_sauce_with_max_comments() {
    let mut data = b"Content".to_vec();
    // to_bytes includes EOF
    data.extend_from_slice(&create_sauce_with_comments(255));

    let stripped = strip_sauce(&data, StripMode::LastStripFinalEof);
    assert_eq!(stripped, b"Content");
}

#[test]
fn test_content_ending_with_eof_no_sauce() {
    let data = b"Content\x1A\x1A\x1A";

    // Without SAUCE, all modes preserve everything
    assert_eq!(strip_sauce(data, StripMode::LastStripFinalEof), data);
    assert_eq!(strip_sauce(data, StripMode::All), data);
}

#[test]
fn test_empty_content_with_sauce() {
    let mut data = Vec::new();
    data.extend_from_slice(&create_sauce_record());

    let stripped = strip_sauce(&data, StripMode::LastStripFinalEof);
    assert_eq!(stripped, b"");
}

#[test]
fn test_strip_sauce_mut() {
    let mut data = b"Content".to_vec();
    // to_bytes includes EOF
    data.extend_from_slice(&create_sauce_record());

    let original_len = data.len();
    {
        let stripped = strip_sauce_mut(&mut data, StripMode::LastStripFinalEof);
        assert_eq!(stripped, b"Content");
        // Verify slice is shorter
        assert_eq!(stripped.len(), 7);
    }
    // Original vec unchanged
    assert_eq!(data.len(), original_len);
}

#[test]
fn test_default_mode() {
    let mut data = b"Content".to_vec();
    // to_bytes includes EOF
    data.extend_from_slice(&create_sauce_record());

    // Default should be Last
    let stripped = strip_sauce(&data, StripMode::default());
    assert_eq!(stripped, b"Content");
}

#[test]
fn test_truncated_sauce_not_stripped() {
    let sauce = create_sauce_record();
    let truncated = &sauce[..sauce.len() - 50]; // Incomplete SAUCE

    let mut data = b"Content".to_vec();
    data.extend_from_slice(truncated);

    // Should not strip incomplete SAUCE
    let stripped = strip_sauce(&data, StripMode::LastStripFinalEof);
    assert_eq!(stripped.len(), data.len());
}

#[test]
fn test_three_sauce_records_all_mode() {
    let mut data = b"Content".to_vec();

    // Three SAUCE records each with EOF from to_bytes
    for _ in 0..3 {
        data.extend_from_slice(&create_sauce_record());
    }

    let stripped = strip_sauce(&data, StripMode::All);
    assert_eq!(stripped, b"Content");
}

#[test]
fn test_payload_containing_eof_bytes() {
    // Payload legitimately ends with EOF bytes
    let mut data = b"Binary\x1A\x1A".to_vec();
    // to_bytes adds The SAUCE's EOF
    data.extend_from_slice(&create_sauce_record());

    let stripped = strip_sauce(&data, StripMode::LastStripFinalEof);
    // Should preserve payload EOF bytes
    assert_eq!(stripped, b"Binary\x1A\x1A");
}

#[test]
fn test_last_preserve_eof_with_no_eof() {
    let mut data = b"Content".to_vec();
    // Use to_bytes_without_eof
    let sauce = SauceRecordBuilder::default()
        .title(BString::from("Test"))
        .unwrap()
        .build();
    data.extend_from_slice(&sauce.to_bytes_without_eof());

    let stripped = strip_sauce(&data, StripMode::Last);
    assert_eq!(stripped, b"Content");
}
