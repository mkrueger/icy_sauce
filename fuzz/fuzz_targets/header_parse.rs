#![no_main]
use libfuzzer_sys::fuzz_target;
use icy_sauce::header::SauceHeader;

fuzz_target!(|data: &[u8]| {
    // Try interpreting tail bytes as potential header+record
    if data.len() >= 128 {
        let _ = SauceHeader::from_bytes(data);
    }
});