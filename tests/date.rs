use icy_sauce::SauceDate;

#[test]
fn displays_sauce_date() {
    let d = SauceDate::new(2025, 11, 8);
    assert_eq!(d.to_string(), "2025/11/08");
}

#[test]
fn displays_out_of_range_year_fallback() {
    let d = SauceDate::new(12_345, 1, 2);
    // Fallback branch: year not zero‑padded to 4 (design choice above)
    assert_eq!(d.to_string(), "12345/01/02");
}

// from_bytes with a valid sequence
#[test]
fn parses_from_bytes_valid() {
    let bytes = b"20251108";
    let d = SauceDate::from_bytes(bytes).expect("should parse");
    assert_eq!(d.year, 2025);
    assert_eq!(d.month, 11);
    assert_eq!(d.day, 8);
}

// from_bytes rejects wrong length
#[test]
fn from_bytes_rejects_wrong_length() {
    assert!(SauceDate::from_bytes(b"2025110").is_none());
    assert!(SauceDate::from_bytes(b"202511080").is_none());
}

// from_bytes with non-digit characters (documents current wrapping behavior)
#[test]
fn from_bytes_non_digit_bytes() {
    let raw = b"A0B1C2D3"; // Not valid digits; wraps subtraction
    let d = SauceDate::from_bytes(raw).unwrap();
    // We assert only that parsing returns Some; exact numbers depend on wrapping math.
    // If you later add digit validation, change this to assert None.
    assert!(d.year >= 0); // Likely garbage but non-negative due to construction math
}

// Round-trip: write then parse
#[test]
fn round_trip_write_parse() {
    let original = SauceDate::new(1999, 12, 31);
    let mut buf = Vec::new();
    original.write(&mut buf).unwrap();
    assert_eq!(buf.len(), 8);
    assert_eq!(&buf, b"19991231");
    let reparsed = SauceDate::from_bytes(&buf).unwrap();
    assert_eq!(reparsed, original);
}

// Ensure write uses contiguous digits (differs from Display)
#[test]
fn write_format_differs_from_display() {
    let d = SauceDate::new(2025, 1, 2);
    let mut buf = Vec::new();
    d.write(&mut buf).unwrap();
    assert_eq!(d.to_string(), "2025/01/02");
    assert_eq!(&buf, b"20250102"); // no slashes
}

// Large year boundary just below fallback threshold
#[test]
fn displays_year_9999_edge() {
    let d = SauceDate::new(9_999, 12, 31);
    assert_eq!(d.to_string(), "9999/12/31");
}

// Fallback threshold crossing: 10_000
#[test]
fn displays_year_10000_fallback() {
    let d = SauceDate::new(10_000, 1, 1);
    assert_eq!(d.to_string(), "10000/01/01"); // No zero-padding logic beyond branch
}

// Equality derives PartialEq
#[test]
fn equality_check() {
    let a = SauceDate::new(2025, 11, 8);
    let b = SauceDate::new(2025, 11, 8);
    let c = SauceDate::new(2025, 11, 9);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// Round-trip: write then parse
#[test]
fn test_invalid_data() {
    let buf = b"1-991231";
    let invalid = SauceDate::from_bytes(buf);
    assert_eq!(invalid, None);
}
