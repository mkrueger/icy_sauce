use bstr::BString;
use icy_sauce::{SauceRecord, SauceRecordBuilder};

#[test]
fn test_empty_sauce() {
    // Test reading data with no SAUCE
    let data = b"Just some data without SAUCE";
    let result = SauceRecord::from_bytes(data).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_malformed_sauce_id() {
    // Test data with wrong SAUCE ID
    let mut data = vec![0u8; 128];
    data[0..5].copy_from_slice(b"WRONG");
    let result = SauceRecord::from_bytes(&data).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_comments_without_valid_id() {
    // Per spec: "Non fatal, No comment present" when comment ID doesn't match
    let info = SauceRecordBuilder::default()
        .title("Test".into())
        .unwrap()
        .add_comment(BString::from("Comment 1"))
        .unwrap()
        .build();

    let mut data = Vec::new();
    info.write(&mut data).unwrap();

    // Corrupt the comment ID
    let comment_id_pos = data.len() - 128 - 64 - 5;
    data[comment_id_pos..comment_id_pos + 5].copy_from_slice(b"WRONG");

    // Should still parse, just without comments
    let parsed = SauceRecord::from_bytes(&data).unwrap().unwrap();
    assert_eq!(parsed.title(), &BString::from("Test"));
    assert_eq!(parsed.comments().len(), 0); // Comments ignored due to bad ID
    assert_eq!(parsed.header().comments, 0);
    assert_eq!(parsed.record_len(), 128);

    let encoded = parsed.to_bytes().unwrap();
    assert_eq!(encoded.len(), 129);
    let reparsed = SauceRecord::from_bytes(&encoded).unwrap().unwrap();
    assert!(parsed == reparsed);

    // Adding comments after a lenient parse must also produce a valid block.
    let modified = parsed
        .to_builder()
        .add_comment("Replacement".into())
        .unwrap()
        .build();
    let reparsed = SauceRecord::from_bytes(&modified.to_bytes().unwrap())
        .unwrap()
        .unwrap();
    assert!(modified == reparsed);
}

#[test]
fn test_maximum_comments() {
    // Test with maximum 255 comments
    let mut builder = SauceRecordBuilder::default();

    for i in 0..255 {
        builder = builder
            .add_comment(BString::from(format!("Comment {}", i)))
            .unwrap();
    }

    // Adding 256th comment should fail
    let result = builder.add_comment(BString::from("One too many"));
    assert!(result.is_err());
}
