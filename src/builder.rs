//! Builder for constructing SAUCE metadata records.
//!
//! This module provides a fluent API for creating valid SAUCE records with proper
//! validation and constraint enforcement. The builder pattern ensures that SAUCE
//! records are well-formed before being serialized.
//!
//! # Example
//!
//! ```no_run
//! use icy_sauce::{SauceRecordBuilder, SauceDataType, Capabilities};
//! use icy_sauce::{CharacterCapabilities, CharacterFormat, LetterSpacing, AspectRatio};
//! use bstr::BString;
//! use chrono::Local;
//!
//! let char_caps = CharacterCapabilities::with_font(
//!     CharacterFormat::Ansi,
//!     80, 25,
//!     false,
//!     LetterSpacing::EightPixel,
//!     AspectRatio::Square,
//!     Some(BString::from("IBM VGA")),
//! ).unwrap();
//!
//! let sauce = SauceRecordBuilder::default()
//!     .title(BString::from("My ANSI Art")).unwrap()
//!     .author(BString::from("Artist Name")).unwrap()
//!     .group(BString::from("Art Group")).unwrap()
//!     .date(Local::now().naive_local().date())
//!     .data_type(SauceDataType::Character)
//!     .capabilities(Capabilities::Character(char_caps)).unwrap()
//!     .add_comment(BString::from("Created with passion")).unwrap()
//!     .build();
//! ```

use bstr::BString;
use chrono::NaiveDate;

use crate::{
    COMMENT_LEN, Capabilities, MetaData, SauceDataType, SauceError,
    header::{AUTHOR_GROUP_LEN, SauceHeader, TITLE_LEN},
};

/// Builder for constructing SAUCE metadata records with validation.
///
/// This builder enforces SAUCE specification constraints as each field is set,
/// ensuring that only valid records can be created. All text fields are validated
/// against their maximum lengths specified in the SAUCE v00 specification.
///
/// # Field Constraints
///
/// - **Title**: maximum 35 bytes (space-padded)
/// - **Author**: maximum 20 bytes (space-padded)
/// - **Group**: maximum 20 bytes (space-padded)
/// - **Comments**: maximum 255 comments, each 64 bytes (space-padded)
/// - **File Size**: 32-bit unsigned integer
/// - **Data Type**: enumerated value (0-8 per spec)
///
/// # Default Values
///
/// - Empty strings for title, author, group
/// - No comments
/// - DataType: None
/// - FileSize: 0
///
/// # Errors
///
/// Methods return [`Err`] if field constraints are violated:
/// - String fields that exceed their maximum length
/// - Comment count exceeds 255
/// - Individual comment length exceeds 64 bytes
#[derive(Default)]
pub struct SauceRecordBuilder {
    /// Raw SAUCE header being constructed
    pub(crate) header: SauceHeader,

    /// Comment lines; up to 255 comments, each 64 bytes max (space-padded).
    /// These are validated as added via [`comment`](Self::comment).
    pub(crate) comments: Vec<BString>,
}

impl SauceRecordBuilder {
    /// Set the title field.
    ///
    /// # Arguments
    ///
    /// * `title` - The artwork title (max 35 bytes)
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::TitleTooLong`] if the title exceeds 35 bytes.
    ///
    /// # Example
    ///
    /// ```
    /// # use icy_sauce::SauceRecordBuilder;
    /// # use bstr::BString;
    /// let builder = SauceRecordBuilder::default()
    ///     .title(BString::from("Winter Scene"))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn title(mut self, title: BString) -> crate::Result<Self> {
        if title.len() > TITLE_LEN {
            return Err(SauceError::TitleTooLong(title.len()));
        }
        self.header.title = title;
        Ok(self)
    }

    /// Set the author field.
    ///
    /// # Arguments
    ///
    /// * `author` - The creator's name or handle (max 20 bytes)
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::AuthorTooLong`] if the author exceeds 20 bytes.
    ///
    /// # Example
    ///
    /// ```
    /// # use icy_sauce::SauceRecordBuilder;
    /// # use bstr::BString;
    /// let builder = SauceRecordBuilder::default()
    ///     .author(BString::from("ArtistHandle"))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn author(mut self, author: BString) -> crate::Result<Self> {
        if author.len() > AUTHOR_GROUP_LEN {
            return Err(SauceError::AuthorTooLong(author.len()));
        }
        self.header.author = author;
        Ok(self)
    }

    /// Set the group field.
    ///
    /// # Arguments
    ///
    /// * `group` - The group or organization name (max 20 bytes)
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::GroupTooLong`] if the group exceeds 20 bytes.
    ///
    /// # Example
    ///
    /// ```
    /// # use icy_sauce::SauceRecordBuilder;
    /// # use bstr::BString;
    /// let builder = SauceRecordBuilder::default()
    ///     .group(BString::from("Cool Group"))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn group(mut self, group: BString) -> crate::Result<Self> {
        if group.len() > AUTHOR_GROUP_LEN {
            return Err(SauceError::GroupTooLong(group.len()));
        }
        self.header.group = group;
        Ok(self)
    }

    /// Set the creation date.
    ///
    /// # Arguments
    ///
    /// * `date` - The creation date as a [`chrono::NaiveDate`]
    ///
    /// The date is automatically formatted to YYYYMMDD as required by the SAUCE specification.
    ///
    /// # Example
    ///
    /// ```
    /// # use icy_sauce::SauceRecordBuilder;
    /// # use chrono::NaiveDate;
    /// let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    /// let builder = SauceRecordBuilder::default()
    ///     .date(date);
    /// ```
    pub fn date(mut self, date: NaiveDate) -> Self {
        self.header.date = date.format("%Y%m%d").to_string().into();
        self
    }

    /// Set the original file size (excluding SAUCE metadata).
    ///
    /// # Arguments
    ///
    /// * `file_size` - Size of the file content in bytes (not including SAUCE record)
    ///
    /// Per SAUCE spec, this field is optional but recommended. Set to 0 for files larger than 4GB
    /// or when the size is unknown.
    ///
    /// # Example
    ///
    /// ```
    /// # use icy_sauce::SauceRecordBuilder;
    /// let builder = SauceRecordBuilder::default()
    ///     .file_size(8192);
    /// ```
    pub fn file_size(mut self, file_size: u32) -> Self {
        self.header.file_size = file_size;
        self
    }

    /// Set the data type of the file.
    ///
    /// # Arguments
    ///
    /// * `data_type` - The [`SauceDataType`] indicating the file format
    ///
    /// The data type is required in all SAUCE records and determines how the format-specific
    /// fields (TInfo1-TInfo4, TFlags, TInfoS) are interpreted.
    ///
    /// # Example
    ///
    /// ```
    /// # use icy_sauce::SauceRecordBuilder;
    /// # use icy_sauce::SauceDataType;
    /// let builder = SauceRecordBuilder::default()
    ///     .data_type(SauceDataType::Character);
    /// ```
    pub fn data_type(mut self, data_type: SauceDataType) -> Self {
        self.header.data_type = data_type;
        self
    }

    /// Set format-specific capabilities.
    ///
    /// # Arguments
    ///
    /// * `caps` - A [`Capabilities`] enum containing format-specific metadata
    ///
    /// This method serializes the capabilities into the appropriate header fields
    /// (TInfo1-TInfo4, TFlags, TInfoS) based on the capability type. The capabilities
    /// must be compatible with the data type set via [`data_type`](Self::data_type).
    ///
    /// # Errors
    ///
    /// Returns an error if the capabilities cannot be serialized (e.g., invalid dimensions,
    /// font name too long, invalid values for the data type, etc.).
    ///
    /// # Example
    ///
    /// ```
    /// # use icy_sauce::{SauceRecordBuilder, Capabilities};
    /// # use icy_sauce::BinaryCapabilities;
    /// let bin_caps = BinaryCapabilities::binary_text(80).unwrap();
    /// let builder = SauceRecordBuilder::default()
    ///     .capabilities(Capabilities::Binary(bin_caps)).unwrap();
    /// ```
    pub fn capabilities(mut self, caps: Capabilities) -> crate::Result<Self> {
        match caps {
            Capabilities::Character(c) => c.encode_into_header(&mut self.header)?,
            Capabilities::Binary(c) => c.encode_into_header(&mut self.header)?,
            Capabilities::Bitmap(c) => c.encode_into_header(&mut self.header)?,
            Capabilities::Vector(c) => c.encode_into_header(&mut self.header)?,
            Capabilities::Audio(c) => c.encode_into_header(&mut self.header)?,
            Capabilities::Executable(c) => c.encode_into_header(&mut self.header)?,
            Capabilities::Archive(c) => c.encode_into_header(&mut self.header)?,
        }
        Ok(self)
    }

    /// Apply metadata from a [`MetaData`] struct.
    ///
    /// # Arguments
    ///
    /// * `info` - Metadata containing title, author, and group
    ///
    /// This is a convenience method for bulk-applying basic metadata. It validates all
    /// fields just like the individual setters.
    ///
    /// # Errors
    ///
    /// Returns validation errors if any field (title, author, or group) exceeds its
    /// maximum length.
    ///
    /// # Example
    ///
    /// ```
    /// # use icy_sauce::{SauceRecordBuilder, MetaData};
    /// # use bstr::BString;
    /// let meta = MetaData {
    ///     title: BString::from("Artwork"),
    ///     author: BString::from("Artist"),
    ///     group: BString::from("Group"),
    ///     comments: Vec::new(),
    /// };
    /// let builder = SauceRecordBuilder::default()
    ///     .metadata(meta)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn metadata(self, info: MetaData) -> crate::Result<Self> {
        self.title(info.title)?
            .author(info.author)?
            .group(info.group)
    }

    /// Add a comment line to the SAUCE record.
    ///
    /// # Arguments
    ///
    /// * `comment` - A comment line (max 64 bytes)
    ///
    /// Comments are stored in order and can be retrieved from the final [`crate::SauceRecord`].
    /// The SAUCE specification supports up to 255 comment lines, each up to 64 bytes.
    ///
    /// # Errors
    ///
    /// Returns:
    /// - [`SauceError::CommentLimitExceeded`] if you attempt to add more than 255 comments
    /// - [`SauceError::CommentTooLong`] if the comment exceeds 64 bytes
    ///
    /// # Example
    ///
    /// ```
    /// # use icy_sauce::SauceRecordBuilder;
    /// # use bstr::BString;
    /// let builder = SauceRecordBuilder::default()
    ///     .add_comment(BString::from("First comment line"))?
    ///     .add_comment(BString::from("Second comment line"))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_comment(mut self, comment: BString) -> crate::Result<Self> {
        if self.comments.len() >= 255 {
            return Err(SauceError::CommentLimitExceeded);
        }
        if comment.len() > COMMENT_LEN {
            return Err(SauceError::CommentTooLong(comment.len()));
        }
        self.comments.push(comment);
        self.header.comments = self.comments.len() as u8;
        Ok(self)
    }

    /// Finalize the builder and return a [`crate::SauceRecord`] record.
    ///
    /// This method consumes the builder and returns a fully constructed SAUCE record
    /// ready for serialization via [`crate::SauceRecord::write`].
    ///
    /// # Example
    ///
    /// ```
    /// # use icy_sauce::{SauceRecordBuilder, SauceDataType};
    /// # use bstr::BString;
    /// # use chrono::Local;
    /// let sauce = SauceRecordBuilder::default()
    ///     .title(BString::from("My Art")).unwrap()
    ///     .author(BString::from("Me")).unwrap()
    ///     .group(BString::from("Group")).unwrap()
    ///     .date(Local::now().naive_local().date())
    ///     .data_type(SauceDataType::Character)
    ///     .build();
    ///
    /// // Write to file
    /// let mut output = Vec::new();
    /// sauce.write(&mut output).unwrap();
    /// ```
    pub fn build(self) -> crate::SauceRecord {
        crate::SauceRecord {
            header: self.header,
            comments: self.comments,
        }
    }
}
