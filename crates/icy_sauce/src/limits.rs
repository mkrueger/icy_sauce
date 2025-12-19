//! SAUCE specification limits.
//!
//! These constants define the maximum field lengths and counts imposed by the
//! SAUCE v00 specification. They are used for validation when building or
//! parsing a `SauceRecord`.
//!
//! Reference summary (header layout):
//! - Title: 35 bytes (space‑padded)
//! - Author: 20 bytes (space‑padded)
//! - Group: 20 bytes (space‑padded)
//! - Date: 8 bytes (ASCII CCYYMMDD)
//! - Comments: Up to 255 lines, each exactly 64 bytes (space‑padded), preceded
//!   by a 5‑byte "COMNT" tag.
//!
//! Use these limits when performing custom validation or trimming input.

/// Maximum number of bytes for the title field (space‑padded to this length).
pub const MAX_TITLE_LENGTH: usize = 35;

/// Maximum number of bytes for the author field (space‑padded to this length).
pub const MAX_AUTHOR_LENGTH: usize = 20;

/// Maximum number of bytes for the group field (space‑padded to this length).
pub const MAX_GROUP_LENGTH: usize = 20;

/// Maximum number of bytes for a single comment line before padding/truncation.
pub const MAX_COMMENT_LENGTH: usize = 64;

/// Maximum number of comment lines permitted by the SAUCE spec.
pub const MAX_COMMENTS: usize = 255;

/// Exact number of bytes for the date field (CCYYMMDD ASCII digits).
pub const DATE_LENGTH: usize = 8;

/// Maximum number of bytes for the font name in binary capabilities.
pub const MAX_FONT_NAME_LENGTH: usize = 22;
