#![no_main]

use icy_sauce::SauceDate;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Some(date) = SauceDate::from_bytes(data) {
        let mut encoded = Vec::new();
        date.write(&mut encoded).unwrap();
        assert_eq!(encoded.len(), 8);
        assert_eq!(SauceDate::from_bytes(&encoded), Some(date));
    }
});
