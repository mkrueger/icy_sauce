//! Executable file format capabilities as specified in the SAUCE v00 standard.
//!
//! This module provides [`ExecutableCapabilities`] for describing executable files (.exe, .dll, .bat, etc.)
//! stored with SAUCE metadata. The SAUCE specification treats all executable formats uniformly
//! with a single type code (0) and no format-specific metadata fields.
//!
//! # SAUCE Field Mappings
//!
//! For executable files:
//! - **DataType**: Always `Executable` (8)
//! - **FileType**: Always `0` (no subtypes)
//! - **TInfo1-TInfo4**: Always `0` (no format-specific data)
//! - **TFlags**: Always `0` (no flags)
//! - **TInfoS**: Empty (no additional info string)
//!
//! # Example
//!
//! ```
//! use icy_sauce::{SauceRecordBuilder, SauceDataType, Capabilities, SauceDate};
//! use icy_sauce::ExecutableCapabilities;
//! use bstr::BString;
//!
//! let exe_caps = ExecutableCapabilities::new();
//! let sauce = SauceRecordBuilder::default()
//!     .title(BString::from("Setup Program"))?
//!     .author(BString::from("Developer"))?
//!     .date(SauceDate::new(2025, 11, 8))
//!     .data_type(SauceDataType::Executable)
//!     .capabilities(Capabilities::Executable(exe_caps))?
//!     .build();
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::{SauceDataType, SauceError, header::SauceHeader};

/// Executable file format capabilities.
///
/// The SAUCE specification treats all executable files uniformly without
/// format-specific metadata. This is a zero-sized marker type for
/// executable-specific SAUCE records.
///
/// # SAUCE Specification Details
///
/// Per SAUCE v00 spec, executable files have:
/// - No FileType subtypes (always 0)
/// - No format-specific fields (TInfo1-4 all 0)
/// - No rendering flags (TFlags = 0)
/// - No font or additional strings (TInfoS empty)
///
/// This design reflects that executables don't have a standardized display format
/// like text or graphics files do.
///
/// # Example
///
/// ```
/// use icy_sauce::ExecutableCapabilities;
/// let caps = ExecutableCapabilities::new();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutableCapabilities {
    // No specific fields needed - executable type is always 0
}

impl ExecutableCapabilities {
    /// Create new executable capabilities.
    ///
    /// Since executables have no format-specific metadata, this is a simple zero-argument
    /// constructor that returns a marker instance.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::ExecutableCapabilities;
    /// let caps = ExecutableCapabilities::new();
    /// ```
    pub fn new() -> Self {
        ExecutableCapabilities {}
    }

    /// Serialize executable capabilities into a SAUCE header.
    ///
    /// # Arguments
    ///
    /// * `header` - Mutable reference to the SAUCE header to populate
    ///
    /// # SAUCE Field Mappings
    ///
    /// Sets the following fields according to SAUCE spec:
    /// - **DataType**: `Executable` (8)
    /// - **FileType**: `0` (no executable subtypes)
    /// - **TInfo1-TInfo4**: All `0` (no format data)
    /// - **TFlags**: `0` (no rendering flags)
    /// - **TInfoS**: Empty string (no additional info)
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::{Capabilities, ExecutableCapabilities, SauceRecordBuilder};
    ///
    /// let caps = ExecutableCapabilities::new();
    /// let record = SauceRecordBuilder::default()
    ///     .capabilities(Capabilities::Executable(caps)).unwrap()
    ///     .build();
    /// assert_eq!(record.header().file_type, 0);
    /// ```
    pub(crate) fn encode_into_header(&self, header: &mut SauceHeader) -> crate::Result<()> {
        header.data_type = SauceDataType::Executable;
        header.file_type = 0; // Always 0 for executables

        // Executable formats have all TInfo fields set to 0 per spec
        header.t_info1 = 0;
        header.t_info2 = 0;
        header.t_info3 = 0;
        header.t_info4 = 0;

        // No flags or TInfoS for executables
        header.t_flags = 0;
        header.t_info_s.clear();

        Ok(())
    }
}

impl TryFrom<&SauceHeader> for ExecutableCapabilities {
    type Error = SauceError;

    /// Parse the executable marker from a SAUCE header.
    ///
    /// Only DataType is checked; FileType and other metadata fields are ignored.
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::UnsupportedDataType`] if DataType is not Executable.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::{header::SauceHeader, ExecutableCapabilities, SauceDataType};
    ///
    /// let header = SauceHeader {
    ///     data_type: SauceDataType::Executable,
    ///     ..SauceHeader::default()
    /// };
    /// let caps = ExecutableCapabilities::try_from(&header).unwrap();
    /// assert_eq!(caps, ExecutableCapabilities::new());
    /// ```
    fn try_from(header: &SauceHeader) -> crate::Result<Self> {
        if header.data_type != SauceDataType::Executable {
            return Err(SauceError::UnsupportedDataType(header.data_type));
        }
        Ok(ExecutableCapabilities {})
    }
}

impl Default for ExecutableCapabilities {
    /// Create default executable capabilities.
    ///
    /// Equivalent to calling [`new()`](Self::new).
    fn default() -> Self {
        Self::new()
    }
}
