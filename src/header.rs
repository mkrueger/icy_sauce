//! SAUCE v00 header structure and serialization.
//!
//! This module provides [`SauceHeader`], which represents the raw 128-byte SAUCE metadata block
//! as specified in the SAUCE v00 specification. The header is the core structure that stores
//! all SAUCE metadata fields for identification and format-specific information.
//!
//! # SAUCE Header Layout
//!
//! The SAUCE header is exactly 128 bytes and is appended to the end of the file, followed
//! by an optional comment block (if present):
//!
//! | Offset | Length | Field      | Type     | Description                    |
//! |--------|--------|------------|----------|--------------------------------|
//! | 0      | 5      | ID         | char[5]  | "SAUCE" magic bytes            |
//! | 5      | 2      | Version    | char[2]  | "00" for SAUCE v00             |
//! | 7      | 35     | Title      | char[35] | Artwork title (space-padded)   |
//! | 42     | 20     | Author     | char[20] | Creator name (space-padded)    |
//! | 62     | 20     | Group      | char[20] | Group/org (space-padded)       |
//! | 82     | 8      | Date       | char[8]  | CCYYMMDD format                |
//! | 90     | 4      | FileSize   | u32 LE   | Original file size in bytes    |
//! | 94     | 1      | DataType   | u8       | File category (0-8)            |
//! | 95     | 1      | FileType   | u8       | Format-specific type           |
//! | 96     | 2      | TInfo1     | u16 LE   | Type-dependent field 1         |
//! | 98     | 2      | TInfo2     | u16 LE   | Type-dependent field 2         |
//! | 100    | 2      | TInfo3     | u16 LE   | Type-dependent field 3         |
//! | 102    | 2      | TInfo4     | u16 LE   | Type-dependent field 4         |
//! | 104    | 1      | Comments   | u8       | Number of comment lines (0=none)|
//! | 105    | 1      | TFlags     | u8       | Type-dependent flags           |
//! | 106    | 22     | TInfoS     | char[22] | Type-dependent string (zero-pad)|
//!
//! **Total: 128 bytes**
//!
//! # Example
//!
//! ```no_run
//! use icy_sauce::header::SauceHeader;
//!
//! // Read a SAUCE header from file data
//! let file_data = std::fs::read("example.ans")?;
//! if let Some(header) = SauceHeader::from_bytes(&file_data)? {
//!     println!("Title: {}", String::from_utf8_lossy(&header.title));
//!     println!("Author: {}", String::from_utf8_lossy(&header.author));
//! } else {
//!     println!("No SAUCE record found");
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use bstr::BString;

use crate::util::{sauce_pad, trim_spaces, zero_pad, zero_trim};
use crate::{COMMENT_ID_LEN, COMMENT_LEN, SauceDataType, SauceDate, SauceError, limits};

pub(crate) const HDR_LEN: usize = 128;
const SAUCE_ID: &[u8; 5] = b"SAUCE";

/// Maximum length for the TInfoS field in bytes (zero-padded)
pub(crate) const TINFO_LEN: usize = 22;

#[derive(Clone, Debug, Default, PartialEq)]
/// Raw SAUCE v00 metadata header (128 bytes).
///
/// `SauceHeader` represents the complete SAUCE record structure as stored in files.
/// It contains identification, creator information, date, file metadata, and format-specific fields.
///
/// # Field Details
///
/// - **title**: Artwork/file title (up to 35 bytes, space-padded)
/// - **author**: Creator's name or handle (up to 20 bytes, space-padded)
/// - **group**: Group or organization name (up to 20 bytes, space-padded)
/// - **date**: Creation date in CCYYMMDD format (always 8 bytes)
/// - **file_size**: Size of original file excluding SAUCE record (4 bytes, little-endian)
/// - **data_type**: File category (0-8: see [`SauceDataType`])
/// - **file_type**: Format-specific type code (meaning depends on `data_type`)
/// - **t_info1-t_info4**: Type-dependent numeric fields (little-endian u16)
/// - **comments**: Number of comment lines in optional comment block (0 = no comments)
/// - **t_flags**: Type-dependent flags byte
/// - **t_info_s**: Type-dependent string (22 bytes, zero-padded)
///
/// # Serialization
///
/// Use [`read`](Self::read) to deserialize from file data, and [`write`](Self::write)
/// to serialize back to bytes. Both methods handle padding/trimming automatically.
///
/// # Note on Comments
///
/// If `comments > 0`, a separate SAUCE comment block precedes this header in the file.
/// The comment block consists of:
/// - "COMNT" (5 bytes)
/// - `comments` lines of 64 bytes each
pub struct SauceHeader {
    /// The title of the file (space-padded in storage, up to 35 bytes)
    pub title: BString,
    /// The (nick)name or handle of the creator (space-padded in storage, up to 20 bytes)
    pub author: BString,
    /// The group or company name (space-padded in storage, up to 20 bytes)
    pub group: BString,

    pub date: SauceDate,

    /// Size of the original file in bytes (excluding SAUCE metadata)
    pub file_size: u32,

    /// Type of data (see [`SauceDataType`])
    pub data_type: SauceDataType,

    /// Type-specific code; meaning depends on `data_type`
    pub file_type: u8,

    /// Type-dependent numeric information field 1 (little-endian u16)
    pub t_info1: u16,
    /// Type-dependent numeric information field 2 (little-endian u16)
    pub t_info2: u16,
    /// Type-dependent numeric information field 3 (little-endian u16)
    pub t_info3: u16,
    /// Type-dependent numeric information field 4 (little-endian u16)
    pub t_info4: u16,

    /// Number of lines in the optional SAUCE comment block.
    /// 0 indicates no comment block is present.
    pub comments: u8,

    /// Type-dependent flags byte
    pub t_flags: u8,

    /// Type-dependent string information field (zero-padded, up to 22 bytes)
    pub t_info_s: BString,
}

impl SauceHeader {
    /// Deserialize a SAUCE header from the end of file data.
    ///
    /// Searches for a valid SAUCE header in the last 128 bytes of the provided data.
    /// If found and valid, returns the parsed header. The header can be followed by a
    /// comment block (indicated by `header.comments > 0`).
    ///
    /// # Arguments
    ///
    /// * `data` - Complete file data to search (must be at least 128 bytes to contain a header)
    ///
    /// # Returns
    ///
    /// - `Ok(Some(header))` if a valid SAUCE v00 header is found
    /// - `Ok(None)` if no valid header is found
    /// - `Err` if data is present but malformed (e.g., wrong version)
    ///
    /// # Errors
    ///
    /// Returns:
    /// - [`SauceError::UnsupportedSauceVersion`] if the header has a version other than "00"
    ///
    /// # Behavior
    ///
    /// - Looks for "SAUCE" magic bytes at `data.len() - 128`
    /// - Validates version field is exactly "00"
    /// - Automatically trims space-padded string fields
    /// - Parses little-endian numeric fields
    /// - Does not validate `data_type` - unknown types are accepted
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use icy_sauce::header::SauceHeader;
    /// let file_data = std::fs::read("example.ans")?;
    /// match SauceHeader::from_bytes(&file_data)? {
    ///     Some(header) => println!("Found SAUCE: {}", String::from_utf8_lossy(&header.title)),
    ///     None => println!("No SAUCE record"),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn from_bytes(data: &[u8]) -> crate::Result<Option<Self>> {
        if data.len() < HDR_LEN {
            return Ok(None);
        }
        let header_start = data.len() - HDR_LEN;

        let mut header = &data[header_start..];
        if SAUCE_ID != &header[..5] {
            return Ok(None);
        }
        header = &header[5..];

        if b"00" != &header[0..2] {
            return Err(SauceError::UnsupportedSauceVersion(BString::new(
                header[0..2].to_vec(),
            )));
        }
        header = &header[2..];

        let title = trim_spaces(&header[0..limits::MAX_TITLE_LENGTH]);
        header = &header[limits::MAX_TITLE_LENGTH..];
        let author = trim_spaces(&header[0..limits::MAX_AUTHOR_LENGTH]);
        header = &header[limits::MAX_AUTHOR_LENGTH..];
        let group = trim_spaces(&header[0..limits::MAX_GROUP_LENGTH]);
        header = &header[limits::MAX_GROUP_LENGTH..];

        let (date_bytes, rest) = header.split_at(8);
        let date = SauceDate::from_bytes(date_bytes).unwrap_or_default();
        let (size_bytes, rest) = rest.split_at(4);
        let file_size = u32::from_le_bytes(size_bytes.try_into().unwrap());
        header = rest;

        let data_type = SauceDataType::from(header[0]);
        let file_type = header[1];
        let t_info1 = header[2] as u16 + ((header[3] as u16) << 8);
        let t_info2 = header[4] as u16 + ((header[5] as u16) << 8);
        let t_info3 = header[6] as u16 + ((header[7] as u16) << 8);
        let t_info4 = header[8] as u16 + ((header[9] as u16) << 8);
        let num_comments = header[10];
        let t_flags = header[11];
        header = &header[12..];

        // Sanity check: remaining slice must be exactly 22 bytes for TInfoS.
        // This cannot fail at runtime since all offsets are derived from the fixed
        // 128-byte header structure, but catches offset calculation errors during development.
        debug_assert_eq!(header.len(), TINFO_LEN);

        let t_info_s = zero_trim(header); // zero-padded field
        Ok(Some(Self {
            title,
            author,
            group,
            date,
            file_size,
            data_type,
            file_type,
            t_info1,
            t_info2,
            t_info3,
            t_info4,
            comments: num_comments,
            t_flags,
            t_info_s,
        }))
    }

    /// Compute the total serialized length of this header plus its optional
    /// comment block and leading COMNT marker.
    ///
    /// This does NOT include the leading EOF (0x1A) byte that precedes the
    /// comment block/header in full SAUCE records. For the complete size used
    /// when writing a record with EOF, add 1 to this value.
    ///
    /// Formula:
    /// ```text
    /// total = 128 (header) + (comments == 0 ? 0 : 5 + comments * 64)
    /// ```
    ///
    /// # Example
    /// ```
    /// use icy_sauce::header::SauceHeader;
    /// let mut h = SauceHeader::default();
    /// assert_eq!(h.total_length(), 128);
    /// h.comments = 2; // two comment lines
    /// assert_eq!(h.total_length(), 128 + 5 + 2 * 64);
    /// ```
    pub fn total_length(&self) -> usize {
        if self.comments == 0 {
            HDR_LEN
        } else {
            HDR_LEN + COMMENT_ID_LEN + self.comments as usize * COMMENT_LEN
        }
    }

    /// Serialize this SAUCE header to bytes.
    ///
    /// Writes exactly 128 bytes in SAUCE v00 format, with all string fields properly
    /// padded and numeric fields in little-endian byte order. This writes only the header;
    /// any comment block must be written separately beforehand.
    ///
    /// # Arguments
    ///
    /// * `writer` - Mutable writer to append the 128-byte header to
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::IoError`] if writing fails.
    ///
    /// # Comments Handling
    ///
    /// If `self.comments > 0`, a separate SAUCE comment block must be written BEFORE
    /// this header. The comment block format is:
    /// - 5 bytes: "COMNT"
    /// - `comments` lines of exactly 64 bytes each (space-padded)
    ///
    /// # Field Padding
    ///
    /// - **title**: Padded/trimmed to exactly 35 bytes with spaces
    /// - **author**: Padded/trimmed to exactly 20 bytes with spaces
    /// - **group**: Padded/trimmed to exactly 20 bytes with spaces
    /// - **date**: Padded/trimmed to exactly 8 bytes (typically CCYYMMDD)
    /// - **t_info_s**: Padded/trimmed to exactly 22 bytes with null bytes
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use icy_sauce::header::SauceHeader;
    /// let header = SauceHeader::default();
    /// let mut output = Vec::new();
    /// header.write(&mut output)?;
    /// assert_eq!(output.len(), 128);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn write<A: std::io::Write>(&self, writer: &mut A) -> crate::Result<()> {
        let mut sauce_info = Vec::with_capacity(HDR_LEN);
        sauce_info.extend(SAUCE_ID);
        sauce_info.extend(b"00");
        sauce_info.extend(sauce_pad(&self.title, limits::MAX_TITLE_LENGTH));
        sauce_info.extend(sauce_pad(&self.author, limits::MAX_AUTHOR_LENGTH));
        sauce_info.extend(sauce_pad(&self.group, limits::MAX_GROUP_LENGTH));
        self.date.write(&mut sauce_info)?;
        sauce_info.extend(self.file_size.to_le_bytes());
        sauce_info.push(self.data_type.into());
        sauce_info.push(self.file_type);
        sauce_info.extend(&self.t_info1.to_le_bytes());
        sauce_info.extend(&self.t_info2.to_le_bytes());
        sauce_info.extend(&self.t_info3.to_le_bytes());
        sauce_info.extend(&self.t_info4.to_le_bytes());
        sauce_info.push(self.comments);
        sauce_info.push(self.t_flags);
        sauce_info.extend(zero_pad(&self.t_info_s, TINFO_LEN));

        // Sanity check: serialized header must be exactly 128 bytes.
        // This cannot fail at runtime since all field sizes are fixed constants,
        // but catches serialization errors during development.
        debug_assert_eq!(sauce_info.len(), HDR_LEN);

        if let Err(err) = writer.write_all(&sauce_info) {
            return Err(SauceError::io_error("<writer>", err));
        }
        Ok(())
    }
}
