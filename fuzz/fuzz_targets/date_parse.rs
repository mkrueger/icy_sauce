#![no_main]
use libfuzzer_sys::fuzz_target;
use icy_sauce::SauceDate;

fuzz_target!(|data: &[u8]| {
    // Expect exactly 8 digits for a date, but fuzz anything
    let _ = SauceDate::from_bytes(data);
});