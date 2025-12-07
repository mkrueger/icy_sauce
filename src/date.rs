//! SAUCE date representation.
//!
//! `SauceDate` models the CCYYMMDD 8‑byte date stored inside a SAUCE header.
//! It is intentionally lightweight and does NOT (yet) validate calendar
//! correctness (e.g. month 13 or day 40 can appear). When the `chrono`
//! feature is enabled you can convert to/from `chrono::NaiveDate` for
//! validation.
//!
//! # Storage Format
//!
//! SAUCE dates are stored as 8 ASCII digits: `YYYYMMDD`. This type keeps
//! the split numeric components (`year`, `month`, `day`) for easy validation
//! or formatting without heap allocations.
//!
//! # Display vs Write
//!
//! - `Display` (`fmt`) renders the date as `YYYY/MM/DD` for readability
//!   (adds slashes).
//! - `write()` serializes the strict SAUCE wire format `YYYYMMDD` (no slashes).
//!
//! # Validation
//!
//! - `from_bytes` only checks byte length (== 8) and performs positional
//!   arithmetic; it does not enforce digit range or calendar validity.
//! - Use `TryFrom<SauceDate> for chrono::NaiveDate` (with `chrono` feature)
//!   to validate ranges.
//!
//! # Examples
//!
//! Creating and displaying:
//! ```
//! use icy_sauce::SauceDate;
//! let d = SauceDate::new(2025, 11, 8);
//! assert_eq!(d.to_string(), "2025/11/08");
//! ```
//!
//! Writing to a buffer (wire format):
//! ```
//! use icy_sauce::SauceDate;
//! let d = SauceDate::new(2025, 11, 8);
//! let mut buf = Vec::new();
//! d.write(&mut buf).unwrap();
//! assert_eq!(&buf, b"20251108");
//! ```
//!
//! Parsing raw bytes:
//! ```
//! use icy_sauce::SauceDate;
//! let raw = b"19991231";
//! let d = SauceDate::from_bytes(raw).unwrap();
//! assert_eq!(d.year, 1999);
//! assert_eq!(d.month, 12);
//! assert_eq!(d.day, 31);
//! ```
//!
//! Chrono conversion (with `chrono` feature):
//! ```
//! # #[cfg(feature="chrono")] {
//! use icy_sauce::SauceDate;
//! use chrono::NaiveDate;
//! let nd = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
//! let sd: SauceDate = nd.into();
//! assert_eq!(sd.to_string(), "2024/02/29");
//! let back = chrono::NaiveDate::try_from(sd).unwrap();
//! assert_eq!(back, nd);
//! # }
//! ```

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SauceDate {
    /// Full 4‑digit year (0–9999 typical; values outside are allowed but will
    /// trigger the fallback branch in `Display`).
    pub year: i32,
    /// Month value (1–12 typical; no range enforced at construction).
    pub month: u8,
    /// Day of month (1–31 typical; not range validated).
    pub day: u8,
}

impl std::fmt::Display for SauceDate {
    /// Format as human‑friendly `YYYY/MM/DD` if year is in `[0, 9999]`,
    /// otherwise fall back to unpadded year with slashes:
    /// ```
    /// use icy_sauce::SauceDate;
    /// assert_eq!(SauceDate::new(2025, 1, 2).to_string(), "2025/01/02");
    /// assert_eq!(SauceDate::new(12_345, 1, 2).to_string(), "12345/01/02");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.year >= 0 && self.year < 10_000 {
            write!(f, "{:04}/{:02}/{:02}", self.year, self.month, self.day)
        } else {
            write!(f, "{}/{:02}/{:02}", self.year, self.month, self.day)
        }
    }
}

impl SauceDate {
    /// Construct a new `SauceDate` without validation.
    ///
    /// For strict validation (e.g., rejecting month > 12), enable the
    /// `chrono` feature and attempt a conversion to `NaiveDate`.
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        SauceDate { year, month, day }
    }

    /// Parse an 8‑byte `YYYYMMDD` ASCII slice into a `SauceDate`.
    ///
    /// Returns `None` if:
    /// - Slice length != 8
    /// - Any byte is not an ASCII digit ('0'-'9')
    ///
    /// Does not validate the logical ranges of month/day.
    ///
    /// ```
    /// use icy_sauce::SauceDate;
    /// assert!(SauceDate::from_bytes(b"20251108").is_some());
    /// assert!(SauceDate::from_bytes(b"2025").is_none());
    /// assert!(SauceDate::from_bytes(b"ABCD1108").is_none()); // Non-digits rejected
    /// ```
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 8 {
            return None;
        }

        // Validate all bytes are ASCII digits first
        if !bytes.iter().all(|&b| b.is_ascii_digit()) {
            return None;
        }

        // Safe helper to convert two ASCII digits to a number
        let parse_two_digits = |pair: &[u8]| -> u8 {
            // We know these are valid digits from the check above
            (pair[0] - b'0') * 10 + (pair[1] - b'0')
        };

        // Parse year (4 digits)
        let year = (bytes[0] - b'0') as i32 * 1000
            + (bytes[1] - b'0') as i32 * 100
            + (bytes[2] - b'0') as i32 * 10
            + (bytes[3] - b'0') as i32;

        let month = parse_two_digits(&bytes[4..6]);
        let day = parse_two_digits(&bytes[6..8]);

        Some(SauceDate { year, month, day })
    }

    /// Write the strict SAUCE wire format (`YYYYMMDD`) to a writer.
    ///
    /// ```
    /// use icy_sauce::SauceDate;
    /// let mut buf = Vec::new();
    /// SauceDate::new(2025, 11, 8).write(&mut buf).unwrap();
    /// assert_eq!(&buf, b"20251108");
    /// ```
    pub fn write<A: std::io::Write>(&self, writer: &mut A) -> crate::Result<()> {
        write!(writer, "{:04}{:02}{:02}", self.year, self.month, self.day)
            .map_err(|e| crate::SauceError::io_error("<date>", e))?;
        Ok(())
    }

    /// Attempt conversion to `chrono::NaiveDate` if the feature is enabled,
    /// returning `None` on invalid ranges.
    ///
    /// ```
    /// # #[cfg(feature="chrono")] {
    /// use icy_sauce::SauceDate;
    /// let valid = SauceDate::new(2024, 2, 29).to_naive_date();
    /// assert!(valid.is_some());
    /// let invalid = SauceDate::new(2024, 13, 40).to_naive_date();
    /// assert!(invalid.is_none());
    /// # }
    /// ```
    #[cfg(feature = "chrono")]
    pub fn to_naive_date(&self) -> Option<chrono::NaiveDate> {
        chrono::NaiveDate::from_ymd_opt(self.year, self.month as u32, self.day as u32)
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::NaiveDate> for SauceDate {
    /// Infallible conversion from a valid `NaiveDate`.
    ///
    /// ```
    /// # #[cfg(feature="chrono")] {
    /// use chrono::NaiveDate;
    /// use icy_sauce::SauceDate;
    /// let nd = NaiveDate::from_ymd_opt(2025, 11, 8).unwrap();
    /// let sd: SauceDate = nd.into();
    /// assert_eq!(sd.to_string(), "2025/11/08");
    /// # }
    /// ```
    fn from(d: chrono::NaiveDate) -> Self {
        use chrono::Datelike;
        SauceDate::new(d.year(), d.month() as u8, d.day() as u8)
    }
}

#[cfg(feature = "chrono")]
impl std::convert::TryFrom<SauceDate> for chrono::NaiveDate {
    type Error = ();
    /// Fallible conversion performing range validation.
    ///
    /// ```
    /// # #[cfg(feature="chrono")] {
    /// use icy_sauce::SauceDate;
    /// use chrono::NaiveDate;
    /// let good = chrono::NaiveDate::try_from(SauceDate::new(2025, 11, 8));
    /// assert!(good.is_ok());
    /// let bad = chrono::NaiveDate::try_from(SauceDate::new(2025, 13, 40));
    /// assert!(bad.is_err());
    /// # }
    /// ```
    fn try_from(value: SauceDate) -> Result<Self, Self::Error> {
        chrono::NaiveDate::from_ymd_opt(value.year, value.month as u32, value.day as u32).ok_or(())
    }
}

#[cfg(feature = "chrono")]
impl std::convert::TryFrom<&SauceDate> for chrono::NaiveDate {
    type Error = ();
    fn try_from(value: &SauceDate) -> Result<Self, Self::Error> {
        chrono::NaiveDate::from_ymd_opt(value.year, value.month as u32, value.day as u32).ok_or(())
    }
}
