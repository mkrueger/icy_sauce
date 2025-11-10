use std::fmt::Display;

pub type Result<T> = std::result::Result<T, SauceError>;

mod capabilities;
pub use capabilities::*;
pub mod header;
pub mod record;
pub use record::*;

mod metadata;
pub use metadata::*;

pub mod builder;
pub use builder::*;

mod date;
pub use date::*;

mod errors;
pub use errors::*;

use crate::header::SauceHeader;

pub mod limits;

pub mod prelude; // public convenience re-exports

pub(crate) mod util;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum SauceDataType {
    /// None / Undefined (spec DataType 0)
    None = 0,
    /// A character based file.
    /// These files are typically interpreted sequentially. Also known as streams.
    #[default]
    Character = 1,
    /// Bitmap graphic and animation files.
    Bitmap = 2,
    /// A vector graphic file.
    Vector = 3,
    /// An audio file.
    Audio = 4,
    /// Raw memory copy of a text mode screen (.BIN file)
    BinaryText = 5,
    /// XBin or eXtended BIN file.
    XBin = 6,
    /// Archive file.
    Archive = 7,
    /// Executable file.
    Executable = 8,
    /// Any other value not covered by the spec (future / unknown).
    Undefined(u8),
}

impl Display for SauceDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SauceDataType::None => write!(f, "None"),
            SauceDataType::Character => write!(f, "Character"),
            SauceDataType::Bitmap => write!(f, "Bitmap"),
            SauceDataType::Vector => write!(f, "Vector"),
            SauceDataType::Audio => write!(f, "Audio"),
            SauceDataType::BinaryText => write!(f, "BinaryText"),
            SauceDataType::XBin => write!(f, "XBin"),
            SauceDataType::Archive => write!(f, "Archive"),
            SauceDataType::Executable => write!(f, "Executable"),
            SauceDataType::Undefined(val) => write!(f, "Undefined({})", val),
        }
    }
}

impl From<u8> for SauceDataType {
    fn from(byte: u8) -> SauceDataType {
        match byte {
            0 => SauceDataType::None,
            1 => SauceDataType::Character,
            2 => SauceDataType::Bitmap,
            3 => SauceDataType::Vector,
            4 => SauceDataType::Audio,
            5 => SauceDataType::BinaryText,
            6 => SauceDataType::XBin,
            7 => SauceDataType::Archive,
            8 => SauceDataType::Executable,
            other => SauceDataType::Undefined(other),
        }
    }
}

impl From<SauceDataType> for u8 {
    fn from(data_type: SauceDataType) -> u8 {
        match data_type {
            SauceDataType::None => 0,
            SauceDataType::Character => 1,
            SauceDataType::Bitmap => 2,
            SauceDataType::Vector => 3,
            SauceDataType::Audio => 4,
            SauceDataType::BinaryText => 5,
            SauceDataType::XBin => 6,
            SauceDataType::Archive => 7,
            SauceDataType::Executable => 8,
            SauceDataType::Undefined(byte) => byte,
        }
    }
}

/// Controls how SAUCE records and EOF markers are stripped from data.
///
/// SAUCE records are metadata blocks appended to files, typically preceded by
/// a DOS EOF marker (0x1A). This enum controls which records to remove and
/// whether to also remove associated EOF markers.
///
/// # EOF Marker Behavior
///
/// - EOF markers (0x1A bytes) were used in DOS to mark end-of-file
/// - Many SAUCE files have one EOF byte immediately before the SAUCE record
/// - This library only removes EOF bytes directly associated with SAUCE records
/// - Multiple/stacked EOF bytes are preserved (except the one tied to each record)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StripMode {
    /// Strip only the last SAUCE record, preserve any EOF markers.
    ///
    /// # Example
    /// ```text
    /// Before: "Content" + 0x1A + SAUCE
    /// After:  "Content" + 0x1A
    /// ```
    Last,

    /// Strip the last SAUCE record and one preceding EOF marker if present.
    /// This is the most common mode for cleaning files.
    ///
    /// # Example
    /// ```text
    /// Before: "Content" + 0x1A + SAUCE
    /// After:  "Content"
    /// ```
    LastStripFinalEof,

    /// Strip all contiguous SAUCE records, preserve trailing EOF markers.
    ///
    /// Records are considered contiguous only if separated by at most one EOF.
    /// Multiple EOF bytes between records will stop the iteration.
    ///
    /// # Example
    /// ```text
    /// Before: "Content" + 0x1A + SAUCE1 + 0x1A + SAUCE2 + 0x1A
    /// After:  "Content" + 0x1A
    ///
    /// But with multiple EOFs:
    /// Before: "Content" + 0x1A + SAUCE1 + 0x1A + 0x1A + SAUCE2
    /// After:  "Content" + 0x1A + 0x1A + SAUCE2
    ///         (iteration stops at double EOF)
    /// ```
    All,

    /// Strip all contiguous SAUCE records and final EOF marker.
    ///
    /// Like `All`, but also removes one trailing EOF after the last removed record.
    /// Most aggressive cleaning mode.
    ///
    /// # Example
    /// ```text
    /// Before: "Content" + 0x1A + SAUCE1 + 0x1A + SAUCE2 + 0x1A
    /// After:  "Content"
    /// ```
    AllStripFinalEof,
}

impl Default for StripMode {
    /// Returns `LastStripFinalEof` - the most common use case for viewing/editing files.
    fn default() -> Self {
        StripMode::LastStripFinalEof
    }
}

/// Returns the end index of payload after removing one SAUCE record at the tail,
/// or None if no SAUCE header is found exactly at the tail.
fn attempt_single_strip(slice: &[u8]) -> Option<usize> {
    let header = SauceHeader::from_bytes(slice).ok().flatten()?;
    let sauce_len = header.total_length();
    if slice.len() < sauce_len {
        return None;
    }
    Some(slice.len() - sauce_len)
}

/// Consume at most one EOF (0x1A) directly preceding index `end`.
fn consume_single_eof(data: &[u8], end: usize) -> usize {
    if end > 0 && data[end - 1] == 0x1A {
        end - 1
    } else {
        end
    }
}

/// Returns true if the truncated slice ends with another SAUCE header.
/// This ONLY detects a header exactly at the tail (no scanning).
fn tail_has_sauce_header(slice: &[u8]) -> bool {
    SauceHeader::from_bytes(slice).ok().flatten().is_some()
}

/// Iteratively remove contiguous SAUCE records from the tail.
/// Contiguity rule: at most one EOF may separate successive SAUCE headers.
/// Returns new payload end or None if no record was removed.
fn strip_contiguous_records(data: &[u8]) -> Option<usize> {
    let mut cursor = data.len();
    let mut removed_any = false;

    loop {
        match attempt_single_strip(&data[..cursor]) {
            Some(end) => {
                removed_any = true;
                // Remove the SAUCE record plus one EOF tied to THAT record (if present)
                let next_cursor = consume_single_eof(data, end);
                // If no further header exactly at tail we stop and preserve remaining EOFs.
                if !tail_has_sauce_header(&data[..next_cursor]) {
                    cursor = next_cursor;
                    break;
                }
                cursor = next_cursor;
                continue;
            }
            None => break,
        }
    }
    if removed_any { Some(cursor) } else { None }
}

fn calculate_strip_position(data: &[u8], mode: StripMode) -> Option<usize> {
    match mode {
        StripMode::LastStripFinalEof => {
            attempt_single_strip(data).map(|end| consume_single_eof(data, end))
        }
        StripMode::Last => attempt_single_strip(data),
        StripMode::All => strip_contiguous_records(data),
        StripMode::AllStripFinalEof => {
            strip_contiguous_records(data).map(|end| consume_single_eof(data, end))
        }
    }
}

/// Strip SAUCE metadata from the end of data.
///
/// Returns a subslice of the input with SAUCE records and optionally EOF markers removed.
/// No allocation or copying occurs. Returns the original slice if no SAUCE is found.
///
/// # Arguments
///
/// * `data` - The data potentially containing SAUCE records
/// * `mode` - Controls which records to remove and EOF handling
///
/// # Examples
///
/// ## Basic usage - strip last SAUCE record
/// ```
/// use icy_sauce::{strip_sauce, StripMode};
///
/// # let file_data = vec![0u8; 100];
/// // Remove last SAUCE and its EOF marker (default)
/// let cleaned = strip_sauce(&file_data, StripMode::default());
///
/// // Remove last SAUCE but keep EOF markers
/// let cleaned = strip_sauce(&file_data, StripMode::Last);
/// ```
///
/// ## Multiple SAUCE records
/// ```
/// use icy_sauce::{strip_sauce, StripMode};
///
/// # let file_data = vec![0u8; 100];
/// // File edited multiple times may have multiple SAUCE records
/// let cleaned = strip_sauce(&file_data, StripMode::All);
/// ```
///
/// ## Contiguous records rule
///
/// For `All` modes, records separated by more than one EOF byte are not considered
/// contiguous. This preserves data integrity when multiple EOF bytes may be intentional:
///
/// ```text
/// Input:  "Content" + 0x1A + SAUCE1 + 0x1A + 0x1A + SAUCE2
///                     ^one EOF        ^double EOF (stops here)
///
/// StripMode::All result: "Content" + 0x1A + 0x1A + SAUCE2
/// (SAUCE1 removed, SAUCE2 preserved due to double EOF barrier)
/// ```
pub fn strip_sauce(data: &[u8], mode: StripMode) -> &[u8] {
    let pos = calculate_strip_position(data, mode).unwrap_or(data.len());
    &data[..pos]
}

/// Mutable version of [`strip_sauce`].
///
/// Identical semantics to [`strip_sauce`] but returns a mutable slice.
/// Useful when you need to modify the content after stripping.
///
/// # Example
/// ```
/// use icy_sauce::{strip_sauce_mut, StripMode};
///
/// let mut file_data = vec![65u8; 100]; // 'A' repeated
/// let cleaned = strip_sauce_mut(&mut file_data, StripMode::default());
/// cleaned[0] = 66; // Change first 'A' to 'B'
/// ```
pub fn strip_sauce_mut(data: &mut [u8], mode: StripMode) -> &mut [u8] {
    let pos = calculate_strip_position(data, mode).unwrap_or(data.len());
    &mut data[..pos]
}

/// Extended strip result with metadata about what was removed.
///
/// Returned by [`strip_sauce_ex`] to provide details about the stripping operation.
#[derive(Debug, Clone)]
pub struct StripResult<'a> {
    /// The data with SAUCE/EOF removed
    pub data: &'a [u8],
    /// Number of SAUCE records removed
    pub records_removed: usize,
    /// Number of EOF (0x1A) bytes removed
    pub eof_bytes_removed: usize,
}

impl<'a> StripResult<'a> {
    /// Calculate total bytes removed from original
    pub fn bytes_removed(&self, original_len: usize) -> usize {
        original_len - self.data.len()
    }
}

/// Strip SAUCE with detailed information about what was removed.
///
/// Like [`strip_sauce`] but returns metadata about the operation.
/// Useful for logging, debugging, or UI feedback.
///
/// # Example
/// ```
/// use icy_sauce::{strip_sauce_ex, StripMode};
///
/// # let file_data = vec![0u8; 100];
/// let result = strip_sauce_ex(&file_data, StripMode::AllStripFinalEof);
///
/// if result.records_removed > 0 {
///     println!("Removed {} SAUCE record(s) and {} EOF byte(s)",
///              result.records_removed, result.eof_bytes_removed);
///     println!("File reduced by {} bytes",
///              result.bytes_removed(file_data.len()));
/// }
/// ```
pub fn strip_sauce_ex(data: &[u8], mode: StripMode) -> StripResult<'_> {
    let mut records = 0;
    let mut eof_count = 0;
    let new_end = match mode {
        StripMode::LastStripFinalEof | StripMode::Last => attempt_single_strip(data).map(|end| {
            records = 1;
            let consumed_end = if mode == StripMode::LastStripFinalEof {
                let c = consume_single_eof(data, end);
                if c != end {
                    eof_count += 1;
                }
                c
            } else {
                end
            };
            consumed_end
        }),
        StripMode::All | StripMode::AllStripFinalEof => {
            let mut cursor = data.len();
            loop {
                match attempt_single_strip(&data[..cursor]) {
                    Some(end) => {
                        records += 1;
                        let c = consume_single_eof(data, end);
                        if c != end {
                            eof_count += 1;
                        }
                        if !tail_has_sauce_header(&data[..c]) {
                            cursor = c;
                            break;
                        }
                        cursor = c;
                    }
                    None => break,
                }
            }
            if records > 0 {
                if mode == StripMode::AllStripFinalEof {
                    let final_c = consume_single_eof(data, cursor);
                    if final_c != cursor {
                        eof_count += 1;
                    }
                    cursor = final_c;
                }
                Some(cursor)
            } else {
                None
            }
        }
    };
    let end = new_end.unwrap_or(data.len());
    StripResult {
        data: &data[..end],
        records_removed: records,
        eof_bytes_removed: eof_count,
    }
}
