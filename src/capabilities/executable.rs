//! Executable file format capabilities as specified in the SAUCE v00 standard.
//!
//! This module provides [`ExecutableCapabilities`] for describing executable files (.exe, .dll, .bat, etc.)
//! stored with SAUCE metadata. The SAUCE specification treats all executable formats uniformly
//! with a single type code (0) and no format-specific metadata fields.
//!
//! # SAUCE Field Mappings
//!
//! For executable files:
//! - **DataType**: Always `Executable` (4)
//! - **FileType**: Always `0` (no subtypes)
//! - **TInfo1-TInfo4**: Always `0` (no format-specific data)
//! - **TFlags**: Always `0` (no flags)
//! - **TInfoS**: Empty (no additional info string)
//!
//! # Example
//!
//! ```
//! use icy_sauce::{SauceRecordBuilder, SauceDataType, Capabilities};
//! use icy_sauce::ExecutableCapabilities;
//! use bstr::BString;
//! use chrono::Local;
//!
//! let exe_caps = ExecutableCapabilities::new();
//! let sauce = SauceRecordBuilder::default()
//!     .title(BString::from("Setup Program"))?
//!     .author(BString::from("Developer"))?
//!     .date(Local::now().naive_local().date())
//!     .data_type(SauceDataType::Executable)
//!     .capabilities(Capabilities::Executable(exe_caps))?
//!     .build();
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::{SauceDataType, SauceError, header::SauceHeader};

/// Executable file format capabilities.
///
/// The SAUCE specification treats all executable files uniformly without
/// format-specific metadata. This is a marker type that stores no data beyond
/// its type discriminant, simplifying the API for executable-specific SAUCE records.
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
#[derive(Debug, Clone)]
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

    /// Parse executable capabilities from a SAUCE header.
    ///
    /// # Arguments
    ///
    /// * `header` - The SAUCE header to parse
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::UnsupportedDataType`] if the header's DataType is not
    /// [`SauceDataType::Executable`].
    ///
    /// # Behavior
    ///
    /// While the SAUCE specification requires FileType to be 0 for executables,
    /// this implementation is tolerant of malformed SAUCE records and does not
    /// strictly enforce this constraint.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Internal parsing example (ignored because `ExecutableCapabilities::from` is not public)
    /// # use icy_sauce::{header::SauceHeader, SauceDataType};
    /// # use icy_sauce::ExecutableCapabilities;
    /// let mut header = SauceHeader::default();
    /// header.data_type = SauceDataType::Executable;
    /// let caps = ExecutableCapabilities::from(&header)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub(crate) fn from(header: &SauceHeader) -> crate::Result<Self> {
        if header.data_type != SauceDataType::Executable {
            return Err(SauceError::UnsupportedDataType(header.data_type));
        }

        // File type should be 0 for executables
        // We don't enforce this strictly to be tolerant of malformed SAUCE

        Ok(ExecutableCapabilities {})
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
    /// - **DataType**: `Executable` (4)
    /// - **FileType**: `0` (no executable subtypes)
    /// - **TInfo1-TInfo4**: All `0` (no format data)
    /// - **TFlags**: `0` (no rendering flags)
    /// - **TInfoS**: Empty string (no additional info)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Internal serialization example (ignored because `encode_into_header` is not public)
    /// # use icy_sauce::{header::SauceHeader};
    /// # use icy_sauce::ExecutableCapabilities;
    /// let caps = ExecutableCapabilities::new();
    /// let mut header = SauceHeader::default();
    /// caps.encode_into_header(&mut header)?;
    /// assert_eq!(header.file_type, 0);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
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

impl Default for ExecutableCapabilities {
    /// Create default executable capabilities.
    ///
    /// Equivalent to calling [`new()`](Self::new).
    fn default() -> Self {
        Self::new()
    }
}
