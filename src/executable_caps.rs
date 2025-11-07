use crate::{SauceDataType, SauceError, header::SauceHeader};

/// Executable capabilities - SAUCE spec only has one type (0)
/// for any executable file (.exe, .dll, .bat, etc.)
#[derive(Debug, Clone)]
pub struct ExecutableCaps {
    // No specific fields needed - executable type is always 0
}

impl ExecutableCaps {
    /// Create new executable capabilities
    pub fn new() -> Self {
        ExecutableCaps {}
    }

    pub(crate) fn from(header: &SauceHeader) -> crate::Result<Self> {
        if header.data_type != SauceDataType::Executable {
            return Err(SauceError::UnsupportedDataType(header.data_type));
        }

        // File type should be 0 for executables
        // We don't enforce this strictly to be tolerant of malformed SAUCE

        Ok(ExecutableCaps {})
    }

    pub(crate) fn write_to_header(&self, header: &mut SauceHeader) -> crate::Result<()> {
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

impl Default for ExecutableCaps {
    fn default() -> Self {
        Self::new()
    }
}
