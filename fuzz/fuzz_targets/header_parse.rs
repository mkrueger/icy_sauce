#![no_main]

use icy_sauce::header::SauceHeader;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(Some(header)) = SauceHeader::from_bytes(data) {
        let mut encoded = Vec::new();
        header.write(&mut encoded).unwrap();
        assert_eq!(encoded.len(), 128);
        assert_eq!(SauceHeader::from_bytes(&encoded).unwrap(), Some(header));
    }
});
