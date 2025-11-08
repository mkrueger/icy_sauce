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

use bstr::{BString, ByteSlice};
use chrono::NaiveDate;

use crate::{
    Capabilities, MetaData, SauceDataType, SauceError, SauceRecordBuilder, VectorCapabilities,
    archive::ArchiveCapabilities,
    audio::AudioCapabilities,
    binary::BinaryCapabilities,
    bitmap::BitmapCapabilities,
    character::CharacterCapabilities,
    executable::ExecutableCapabilities,
    header::{HDR_LEN, SauceHeader},
    sauce_pad, sauce_trim,
};

pub(crate) const COMMENT_LEN: usize = 64;
const COMMENT_ID_LEN: usize = 5;
const COMMENT_ID: [u8; COMMENT_ID_LEN] = *b"COMNT";

/// SAUCE information.
/// This is the main structure for SAUCE.
///
/// SAUCE metadata consits of a header and optional comments.
#[derive(Clone, PartialEq)]
pub struct SauceRecord {
    pub(crate) header: SauceHeader,

    /// Up to 255 comments, each 64 bytes long max.
    pub(crate) comments: Vec<BString>,
}

impl SauceRecord {
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
                    comments.push(sauce_trim(&cdata[..COMMENT_LEN]));
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

        Ok(Some(SauceRecord { header, comments }))
    }

    /// Write the SAUCE record to a writer.
    ///
    /// Serializes the complete SAUCE information including the header and any comment blocks
    /// to the provided writer. The data is written in the following order:
    ///
    /// 1. Optional comment block (if comments exist)
    /// 2. SAUCE header (128 bytes)
    ///
    /// # Arguments
    ///
    /// * `writer` - The writer to serialize the SAUCE data to
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::IoError`] if writing fails.
    ///
    /// # Notes
    ///
    /// - Comments are automatically padded to 64 bytes each
    /// - The header fields are padded according to spec (spaces or zeros)
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::SauceRecordBuilder;
    /// use bstr::BString;
    ///
    /// let sauce = SauceRecordBuilder::default()
    ///     .title(BString::from("My Art")).unwrap()
    ///     .build();
    ///
    /// let mut buffer = Vec::new();
    /// sauce.write(&mut buffer).unwrap();
    /// assert!(buffer.len() >= 128); // At least the header
    /// ```
    pub fn write<A: std::io::Write>(&self, writer: &mut A, append_eof: bool) -> crate::Result<()> {
        // EOF Char.
        if append_eof {
            if let Err(err) = writer.write_all(&[0x1A]) {
                return Err(SauceError::IoError(err));
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
                return Err(SauceError::IoError(err));
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
    /// assert_eq!(sauce.record_len(), 129 + 5 + 64); // header + EOF + COMNT + 1 comment
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
    /// use icy_sauce::SauceRecordBuilder;
    /// use chrono::{NaiveDate, Datelike};
    ///
    /// let sauce = SauceRecordBuilder::default()
    ///     .date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
    ///     .build();
    /// let date = sauce.date().unwrap();
    /// assert_eq!(date.year(), 2024);
    /// ```
    pub fn date(&self) -> crate::Result<NaiveDate> {
        match NaiveDate::parse_from_str(&self.header.date.to_str_lossy(), "%Y%m%d") {
            Ok(d) => Ok(d),
            Err(_) => Err(SauceError::UnsupportedSauceDate(self.header.date.clone())),
        }
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
        match self.header.data_type {
            SauceDataType::Character => match CharacterCapabilities::from(&self.header) {
                Ok(caps) => Some(Capabilities::Character(caps)),
                Err(_) => None,
            },
            SauceDataType::BinaryText | SauceDataType::XBin => {
                match BinaryCapabilities::from(&self.header) {
                    Ok(caps) => Some(Capabilities::Binary(caps)),
                    Err(_) => None,
                }
            }
            SauceDataType::Bitmap => match BitmapCapabilities::from(&self.header) {
                Ok(caps) => Some(Capabilities::Bitmap(caps)),
                Err(_) => None,
            },
            SauceDataType::Vector => match VectorCapabilities::from(&self.header) {
                Ok(caps) => Some(Capabilities::Vector(caps)),
                Err(_) => None,
            },
            SauceDataType::Audio => match AudioCapabilities::from(&self.header) {
                Ok(caps) => Some(Capabilities::Audio(caps)),
                Err(_) => None,
            },
            SauceDataType::Archive => match ArchiveCapabilities::from(&self.header) {
                Ok(caps) => Some(Capabilities::Archive(caps)),
                Err(_) => None,
            },
            SauceDataType::Executable => match ExecutableCapabilities::from(&self.header) {
                Ok(caps) => Some(Capabilities::Executable(caps)),
                Err(_) => None,
            },
            _ => None,
        }
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
