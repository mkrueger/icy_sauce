use bstr::BString;

use crate::SauceRecordBuilder;

/// Basic SAUCE metadata without format-specific capabilities.
///
/// `MetaData` contains the title, author, group, and comments from a SAUCE record,
/// but does not include the format-specific information (character dimensions, audio codec, etc.).
/// This is useful for lightweight metadata queries or bulk metadata operations.
///
/// # Field Details
///
/// - **title**: Artwork/file title (up to 35 bytes, space-trimmed)
/// - **author**: Creator's name or handle (up to 20 bytes, space-trimmed)
/// - **group**: Group or organization name (up to 20 bytes, space-trimmed)
/// - **comments**: Optional comment lines (up to 255, each 64 bytes)
///
/// # Example
///
/// ```
/// use icy_sauce::MetaData;
/// use bstr::BString;
///
/// let meta = MetaData {
///     title: BString::from("My Artwork"),
///     author: BString::from("Artist"),
///     group: BString::from("Group"),
///     comments: vec![],
/// };
/// assert!(!meta.is_empty());
/// ```
#[derive(Default, Clone, PartialEq)]
pub struct MetaData {
    /// The title of the file (space-trimmed)
    pub title: BString,
    /// The (nick)name or handle of the creator (space-trimmed)
    pub author: BString,
    /// The group or company the creator is employed by (space-trimmed)
    pub group: BString,

    /// Optional comment lines (up to 255, each 64 bytes)
    pub comments: Vec<BString>,
}

impl MetaData {
    pub fn to_builder(&self) -> crate::Result<SauceRecordBuilder> {
        let mut builder = SauceRecordBuilder::default();
        builder = builder.title(self.title.clone())?;
        builder = builder.author(self.author.clone())?;
        builder = builder.group(self.group.clone())?;
        for comment in &self.comments {
            builder = builder.add_comment(comment.clone())?;
        }
        Ok(builder)
    }

    pub fn is_empty(&self) -> bool {
        self.title.is_empty()
            && self.author.is_empty()
            && self.group.is_empty()
            && self.comments.is_empty()
    }
}
