//! SAUCE metadata information structures and parsing.
//!
//! This module provides high-level types for working with SAUCE metadata:
//! - [`MetaData`] - Basic metadata (title, author, group, comments)
//! - [`Capabilities`] - Format-specific capabilities
//! - [`SauceRecord`] - Complete SAUCE record with full parsing and serialization
//!
//! # SAUCE File Layout
//!
//! A file with SAUCE metadata has this structure (reading from end backwards):
//!
//! ```text
//! [File Content Data]
//! [0x1A EOF Marker] (1 byte)
//! [COMNT Comment Block] (if comments > 0)
//!   - "COMNT" ID (5 bytes)
//!   - Comment lines (64 bytes each, space-padded)
//! [SAUCE Header] (128 bytes)
//! ```
//!
//! # Example
//!
//! ```no_run
//! use icy_sauce::SauceRecord;
//!
//! // Read SAUCE metadata from a file
//! let file_data = std::fs::read("example.ans")?;
//! if let Some(sauce) = SauceRecord::from_bytes(&file_data)? {
//!     println!("Title: {}", String::from_utf8_lossy(sauce.title()));
//!     println!("Author: {}", String::from_utf8_lossy(sauce.author()));
//!     println!("Comments: {}", sauce.comments().len());
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::{
    cell::OnceCell,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};

use bstr::BString;

use crate::{
    Capabilities, MetaData, SauceDataType, SauceDate, SauceError, SauceRecordBuilder,
    VectorCapabilities,
    archive::ArchiveCapabilities,
    audio::AudioCapabilities,
    binary::BinaryCapabilities,
    bitmap::BitmapCapabilities,
    character::CharacterCapabilities,
    executable::ExecutableCapabilities,
    header::{HDR_LEN, SauceHeader},
    util::{sauce_pad, trim_spaces},
};

pub(crate) const COMMENT_LEN: usize = 64;
pub(crate) const COMMENT_ID_LEN: usize = 5;
const COMMENT_ID: [u8; COMMENT_ID_LEN] = *b"COMNT";

/// SAUCE information.
/// This is the main structure for SAUCE.
///
/// SAUCE metadata consits of a header and optional comments.
#[derive(PartialEq)]
pub struct SauceRecord {
    pub(crate) header: SauceHeader,

    /// Up to 255 comments, each 64 bytes long max.
    pub(crate) comments: Vec<BString>,

    pub(crate) cached_caps: OnceCell<Option<Capabilities>>,
}

// Custom Clone impl that resets cache
impl Clone for SauceRecord {
    fn clone(&self) -> Self {
        Self {
            header: self.header.clone(),
            comments: self.comments.clone(),
            cached_caps: OnceCell::new(), // fresh cache
        }
    }
}

impl SauceRecord {
    /// Attempt to parse a SAUCE record from a complete file buffer.
    ///
    /// This function expects the slice `data` to contain the entire file contents,
    /// including any trailing SAUCE header, optional comment block and the EOF marker (0x1A).
    /// It scans from the end using [`SauceHeader::from_bytes`].
    ///
    /// # Returns
    /// * `Ok(Some(record))` if a valid SAUCE header is found.
    /// * `Ok(None)` if no SAUCE record exists (file has no SAUCE metadata).
    /// * `Err(e)` if malformed SAUCE data is detected (e.g. truncated comment block).
    ///
    /// # Robustness
    /// The SAUCE specification tolerates certain minor deviations; for example a missing
    /// `COMNT` marker or missing EOF character are logged as warnings (via the `log` crate)
    /// but do not abort parsing. Severe structural problems (length mismatches) yield an error.
    ///
    /// # Performance
    /// Parsing is O(1) relative to the number of comments (bounded to 255) and otherwise
    /// proportional to the fixed header size. No heap allocations are performed except
    /// for copying comment lines and the header's owned strings.
    #[must_use]
    pub fn from_bytes(data: &[u8]) -> crate::Result<Option<Self>> {
        let Some(header) = SauceHeader::from_bytes(data)? else {
            return Ok(None);
        };

        let mut comments = Vec::new();
        if header.comments > 0 {
            let expected = HDR_LEN + header.comments as usize * COMMENT_LEN + COMMENT_ID_LEN;
            if data.len() < expected {
                return Err(SauceError::InvalidCommentBlock);
            }
            let mut cdata = &data[data.len() - expected..];
            if &cdata[..COMMENT_ID_LEN] != COMMENT_ID {
                // Non-fatal per spec: ignore comments
                log::warn!("SAUCE comment block missing COMNT ID - ignoring comments");
            } else {
                cdata = &cdata[COMMENT_ID_LEN..];
                for _ in 0..header.comments {
                    comments.push(trim_spaces(&cdata[..COMMENT_LEN]));
                    cdata = &cdata[COMMENT_LEN..];
                }
            }
        }

        // Check EOF marker at the correct position
        // EOF should be right before the SAUCE data (including comment block if present)
        let sauce_size = if header.comments > 0 {
            HDR_LEN + header.comments as usize * COMMENT_LEN + COMMENT_ID_LEN
        } else {
            HDR_LEN
        };

        // Non fatal warning
        if data.len() > sauce_size {
            let eof_pos = data.len() - sauce_size - 1;
            if data[eof_pos] != 0x1A {
                log::warn!("Missing EOF marker before SAUCE record");
            }
        }

        Ok(Some(SauceRecord {
            header,
            comments,
            cached_caps: OnceCell::new(),
        }))
    }

    /// Efficiently parse a SAUCE record from a file path.
    ///
    /// Instead of reading the entire file, only the trailing window large enough to hold
    /// the maximum possible SAUCE payload (header + comment block + COMNT marker + EOF) is read.
    /// This keeps memory usage low for large artwork files.
    ///
    /// # Arguments
    /// * `path` - Path to the file on disk.
    ///
    /// # Returns
    /// Same semantics as [`from_bytes`](Self::from_bytes).
    ///
    /// # Errors
    /// I/O failures are wrapped in [`SauceError::IoError`]. Structural SAUCE issues yield
    /// specific `SauceError` variants.
    #[must_use]
    pub fn from_path(path: &std::path::Path) -> crate::Result<Option<Self>> {
        const MAX_SAUCE_WINDOW: u64 = 128 + 5 + 255 * 64 + 1;
        let mut f = File::open(path).map_err(|e| SauceError::io_error(path, e))?;
        let file_len = f
            .metadata()
            .map_err(|e| SauceError::io_error(path, e))?
            .len();
        let read_len = MAX_SAUCE_WINDOW.min(file_len);
        f.seek(SeekFrom::End(-(read_len as i64)))
            .map_err(|e| SauceError::io_error(path, e))?;
        let mut buf = vec![0u8; read_len as usize];
        f.read_exact(&mut buf)
            .map_err(|e| SauceError::io_error(path, e))?;
        // Reuse existing logic
        Self::from_bytes(&buf)
    }

    /// Serialize this SAUCE record (including EOF marker) to a fresh `Vec<u8>`.
    ///
    /// Useful for appending to existing file content or for tests that need a full
    /// byte representation. Use [`to_bytes_without_eof`](Self::to_bytes_without_eof)
    /// if you need only the SAUCE payload without the leading EOF marker.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.record_len() + 1); // +1 for EOF
        let _ = self.write(&mut buf);
        buf
    }

    /// Serialize this SAUCE record without the EOF marker.
    ///
    /// The SAUCE specification places a single 0x1A byte before the comment block / header.    
    /// Certain embedding contexts already manage this EOF marker externally; for those scenarios
    /// this variant avoids duplication.
    pub fn to_bytes_without_eof(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.record_len());
        let _ = self.write_without_eof(&mut buf);
        buf
    }

    /// Write SAUCE with EOF marker (standard format).
    pub fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        self.write_internal(writer, true)
    }

    /// Write SAUCE without EOF marker (for special cases).
    pub fn write_without_eof<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        self.write_internal(writer, false)
    }

    /// Internal unified writer for both public write variants.
    ///
    /// When `eof` is true an EOF marker (0x1A) is prepended. Comments (if any) are serialized
    /// with a `COMNT` marker followed by each space‑padded 64 byte line. Finally the header
    /// (128 bytes) is written. In case of any I/O error an appropriate [`SauceError::IoError`]
    /// is returned.
    fn write_internal<W: Write>(&self, writer: &mut W, eof: bool) -> crate::Result<()> {
        // EOF Char.
        if eof {
            if let Err(err) = writer.write_all(&[0x1A]) {
                return Err(SauceError::io_error("<writer>", err));
            }
        }

        if !self.comments.is_empty() {
            let length = COMMENT_ID_LEN + self.comments.len() * COMMENT_LEN;
            let mut comment_info = Vec::with_capacity(length);
            comment_info.extend(&COMMENT_ID);
            for comment in &self.comments {
                comment_info.extend(sauce_pad(comment, COMMENT_LEN));
            }
            assert_eq!(comment_info.len(), length);
            if let Err(err) = writer.write_all(&comment_info) {
                return Err(SauceError::io_error("<writer>", err));
            }
        }
        self.header.write(writer)?;
        Ok(())
    }

    /// Get the total byte length of this SAUCE record.
    ///
    /// Returns the total number of bytes that would be written by [`write()`](Self::write),
    /// including:
    /// - 128 bytes for the SAUCE header
    /// - Optional comment block if comments exist:
    ///   - 5 bytes for "COMNT" marker
    ///   - (number of comments × 64) bytes for comment data
    ///
    /// NOTE the EOF is not included!
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::SauceRecordBuilder;
    /// use bstr::BString;
    ///
    /// let sauce = SauceRecordBuilder::default()
    ///     .add_comment(BString::from("Test")).unwrap()
    ///     .build();
    /// assert_eq!(sauce.record_len(), 128 + 5 + 64); // header + COMNT + 1 comment
    /// ```
    pub fn record_len(&self) -> usize {
        if self.comments.is_empty() {
            HDR_LEN
        } else {
            HDR_LEN + self.header.comments as usize * COMMENT_LEN + COMMENT_ID_LEN
        }
    }

    /// Get the original file size (before SAUCE was added).
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::SauceRecordBuilder;
    ///
    /// let sauce = SauceRecordBuilder::default()
    ///     .file_size(1024)
    ///     .build();
    /// assert_eq!(sauce.file_size(), 1024);
    /// ```
    pub fn file_size(&self) -> u32 {
        self.header.file_size
    }

    /// Get the title field.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::SauceRecordBuilder;
    /// use bstr::BString;
    ///
    /// let sauce = SauceRecordBuilder::default()
    ///     .title(BString::from("My Title")).unwrap()
    ///     .build();
    /// assert_eq!(sauce.title(), &BString::from("My Title"));
    /// ```
    pub fn title(&self) -> &BString {
        &self.header.title
    }

    /// Get the author field.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::SauceRecordBuilder;
    /// use bstr::BString;
    ///
    /// let sauce = SauceRecordBuilder::default()
    ///     .author(BString::from("Artist")).unwrap()
    ///     .build();
    /// assert_eq!(sauce.author(), &BString::from("Artist"));
    /// ```
    pub fn author(&self) -> &BString {
        &self.header.author
    }

    /// Get the group field.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::SauceRecordBuilder;
    /// use bstr::BString;
    ///
    /// let sauce = SauceRecordBuilder::default()
    ///     .group(BString::from("Art Group")).unwrap()
    ///     .build();
    /// assert_eq!(sauce.group(), &BString::from("Art Group"));
    /// ```
    pub fn group(&self) -> &BString {
        &self.header.group
    }

    /// Get the data type.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::{SauceRecordBuilder, SauceDataType};
    ///
    /// let sauce = SauceRecordBuilder::default()
    ///     .data_type(SauceDataType::Character)
    ///     .build();
    /// assert_eq!(sauce.data_type(), SauceDataType::Character);
    /// ```
    pub fn data_type(&self) -> SauceDataType {
        self.header.data_type
    }

    /// Get a reference to the raw SAUCE header.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::SauceRecordBuilder;
    ///
    /// let sauce = SauceRecordBuilder::default().build();
    /// let header = sauce.header();
    /// assert_eq!(header.file_type, 0);
    /// ```
    pub fn header(&self) -> &SauceHeader {
        &self.header
    }

    /// Get the comment lines.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::SauceRecordBuilder;
    /// use bstr::BString;
    ///
    /// let sauce = SauceRecordBuilder::default()
    ///     .add_comment(BString::from("Line 1")).unwrap()
    ///     .add_comment(BString::from("Line 2")).unwrap()
    ///     .build();
    /// assert_eq!(sauce.comments().len(), 2);
    /// assert_eq!(sauce.comments()[0], BString::from("Line 1"));
    /// ```
    pub fn comments(&self) -> &[BString] {
        &self.comments
    }

    /// Parse and return the date from the SAUCE record.
    ///
    /// The date is stored as CCYYMMDD in the SAUCE header.
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::UnsupportedSauceDate`] if the date cannot be parsed or is invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::{SauceRecordBuilder, SauceDate};
    ///
    /// let sauce = SauceRecordBuilder::default()
    ///     .date(SauceDate::new(2024, 1, 15))
    ///     .build();
    /// let date = sauce.date();
    /// assert_eq!(date.year, 2024);
    /// ```
    pub fn date(&self) -> SauceDate {
        self.header.date.clone()
    }

    /// Get format-specific capabilities.
    ///
    /// Returns the appropriate capability structure based on the data type,
    /// or `None` if the data type has no associated capabilities.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::{SauceRecordBuilder, SauceDataType, Capabilities};
    /// use icy_sauce::{CharacterCapabilities, CharacterFormat};
    ///
    /// let caps = CharacterCapabilities::new(CharacterFormat::Ansi);
    /// let sauce = SauceRecordBuilder::default()
    ///     .data_type(SauceDataType::Character)
    ///     .capabilities(Capabilities::Character(caps)).unwrap()
    ///     .build();
    ///
    /// match sauce.capabilities() {
    ///     Some(Capabilities::Character(c)) => {
    ///         println!("Character file: {}x{}", c.columns, c.lines);
    ///     }
    ///     _ => {}
    /// }
    /// ```
    pub fn capabilities(&self) -> Option<Capabilities> {
        self.cached_caps
            .get_or_init(|| match self.header.data_type {
                SauceDataType::Character => CharacterCapabilities::try_from(&self.header)
                    .ok()
                    .map(Capabilities::Character),
                SauceDataType::BinaryText | SauceDataType::XBin => {
                    BinaryCapabilities::try_from(&self.header)
                        .ok()
                        .map(Capabilities::Binary)
                }
                SauceDataType::Bitmap => BitmapCapabilities::try_from(&self.header)
                    .ok()
                    .map(Capabilities::Bitmap),
                SauceDataType::Vector => VectorCapabilities::try_from(&self.header)
                    .ok()
                    .map(Capabilities::Vector),
                SauceDataType::Audio => AudioCapabilities::try_from(&self.header)
                    .ok()
                    .map(Capabilities::Audio),
                SauceDataType::Archive => ArchiveCapabilities::try_from(&self.header)
                    .ok()
                    .map(Capabilities::Archive),
                SauceDataType::Executable => ExecutableCapabilities::try_from(&self.header)
                    .ok()
                    .map(Capabilities::Executable),
                _ => None,
            })
            .clone()
    }

    /// Extract basic metadata information.
    ///
    /// Returns a [`MetaData`] containing just the title, author, group,
    /// and comments from this SAUCE record.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::SauceRecordBuilder;
    /// use bstr::BString;
    ///
    /// let sauce = SauceRecordBuilder::default()
    ///     .title(BString::from("Title")).unwrap()
    ///     .author(BString::from("Author")).unwrap()
    ///     .build();
    ///
    /// let meta = sauce.metadata();
    /// assert_eq!(meta.title, BString::from("Title"));
    /// assert_eq!(meta.author, BString::from("Author"));
    /// ```
    pub fn metadata(&self) -> MetaData {
        MetaData {
            title: self.header.title.clone(),
            author: self.header.author.clone(),
            group: self.header.group.clone(),
            comments: self.comments.clone(),
        }
    }

    /// Convert this SAUCE record to a builder for modification.
    ///
    /// This allows you to create a modified copy of an existing SAUCE record.
    ///
    /// # Errors
    ///
    /// Returns an error if the capabilities cannot be converted to builder format.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::SauceRecordBuilder;
    /// use bstr::BString;
    ///
    /// let original = SauceRecordBuilder::default()
    ///     .title(BString::from("Original")).unwrap()
    ///     .build();
    ///
    /// let modified = original.to_builder()
    ///     .title(BString::from("Modified")).unwrap()
    ///     .build();
    ///
    /// assert_eq!(modified.title(), &BString::from("Modified"));
    /// ```
    pub fn to_builder(&self) -> SauceRecordBuilder {
        SauceRecordBuilder {
            header: self.header.clone(),
            comments: self.comments.clone(),
        }
    }
}
