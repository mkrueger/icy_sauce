#![no_main]

use icy_sauce::{
    SauceRecord, SauceRecordBuilder, StripMode, header::SauceHeader, strip_sauce, strip_sauce_ex,
    strip_sauce_mut,
};
use libfuzzer_sys::fuzz_target;

const MODES: [StripMode; 4] = [
    StripMode::Last,
    StripMode::LastStripFinalEof,
    StripMode::All,
    StripMode::AllStripFinalEof,
];

fuzz_target!(|data: &[u8]| {
    // Exercise all implementations on arbitrary, potentially malformed tails.
    for mode in MODES {
        let stripped = strip_sauce(data, mode);
        let detailed = strip_sauce_ex(data, mode);
        assert_eq!(stripped, detailed.data);
        assert!(data.starts_with(stripped));
        let mut mutable = data.to_vec();
        assert_eq!(strip_sauce_mut(&mut mutable, mode), stripped);
    }

    // Any accepted record must serialize into a readable, equivalent record.
    if let Ok(Some(record)) = SauceRecord::from_bytes(data) {
        let encoded = record.to_bytes().unwrap();
        let reparsed = SauceRecord::from_bytes(&encoded).unwrap().unwrap();
        assert!(record == reparsed);
    }

    // Construct valid metadata so fuzzing reaches beyond magic-byte checks.
    let record = SauceRecordBuilder::default()
        .add_comment(data[..data.len().min(64)].into())
        .unwrap()
        .build();
    let mut valid = data.to_vec();
    valid.extend(record.to_bytes().unwrap());
    assert_eq!(strip_sauce(&valid, StripMode::LastStripFinalEof), data);

    // A deliberately invalid COMNT marker must never authorize deleting bytes.
    let mut malformed = data.to_vec();
    malformed.extend_from_slice(b"\x1aWRONG");
    malformed.extend_from_slice(&[b'X'; 64]);
    SauceHeader {
        comments: 1,
        ..Default::default()
    }
    .write(&mut malformed)
    .unwrap();
    for mode in MODES {
        assert_eq!(strip_sauce(&malformed, mode), malformed);
        assert_eq!(strip_sauce_ex(&malformed, mode).records_removed, 0);
    }
});
