//! Vector graphics format capabilities for SAUCE metadata.
//!
//! This module provides types for describing vector graphics files stored with SAUCE metadata.
//! Vector formats are scalable graphics that are resolution-independent.
//!
//! # Supported Vector Formats
//!
//! - **DXF**: AutoCAD Drawing Exchange Format
//! - **DWG**: AutoCAD Drawing Format
//! - **WPG (Vector)**: WordPerfect Graphics (vector mode)
//! - **3DS**: Autodesk 3D Studio
//!
//! # SAUCE Field Mappings
//!
//! Vector formats use:
//! - DataType: Vector (2)
//! - FileType: Format variant (0-3)
//! - TInfo1: Width in pixels (optional)
//! - TInfo2: Height in pixels (optional)
//! - TInfo3: Depth in pixels (optional for 3D formats)
//! - TInfo4: 0
//! - TFlags: 0
//! - TInfoS: Empty

use crate::{SauceDataType, SauceError, header::SauceHeader};

/// Vector graphics format enumeration for SAUCE metadata.
///
/// Covers 4 vector formats recognized by SAUCE, plus an unknown variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VectorFormat {
    /// DXF (AutoCAD Drawing Exchange Format)
    Dxf,
    /// DWG (AutoCAD Drawing Format)
    Dwg,
    /// WPG Vector (WordPerfect Graphics - vector)
    WpgVector,
    /// 3DS (Autodesk 3D Studio)
    ThreeDs,
    /// Unknown vector format with arbitrary file type code
    Unknown(u8),
}

impl VectorFormat {
    /// Parse vector format from SAUCE file type code.
    pub fn from_sauce(file_type: u8) -> Self {
        match file_type {
            0 => VectorFormat::Dxf,
            1 => VectorFormat::Dwg,
            2 => VectorFormat::WpgVector,
            3 => VectorFormat::ThreeDs,
            _ => VectorFormat::Unknown(file_type),
        }
    }

    /// Convert vector format back to SAUCE file type code.
    pub fn to_sauce(&self) -> u8 {
        match self {
            VectorFormat::Dxf => 0,
            VectorFormat::Dwg => 1,
            VectorFormat::WpgVector => 2,
            VectorFormat::ThreeDs => 3,
            VectorFormat::Unknown(val) => *val,
        }
    }
}

/// Vector graphics file capabilities for SAUCE records.
///
/// Represents vector-specific metadata parsed from or to be written to a SAUCE header.
/// This includes the vector format and optional dimensions.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorCapabilities {
    /// Vector graphics format
    pub format: VectorFormat,
}

impl VectorCapabilities {
    /// Create new vector capabilities with the specified format.
    pub fn new(format: VectorFormat) -> Self {
        Self { format }
    }

    /// Parse vector capabilities from a SAUCE header via the `TryFrom<&SauceHeader>` implementation.
    /// The former bespoke `from(&SauceHeader)` has been removed in favor of the standard trait.

    /// Serialize vector capabilities into a SAUCE header.
    pub(crate) fn encode_into_header(&self, header: &mut SauceHeader) -> crate::Result<()> {
        header.data_type = SauceDataType::Vector;
        header.file_type = self.format.to_sauce();

        // Set dimension fields
        header.t_info1 = 0;
        header.t_info2 = 0;
        header.t_info3 = 0;
        header.t_info4 = 0;

        // Vector formats don't use flags or TInfoS
        header.t_flags = 0;
        header.t_info_s.clear();

        Ok(())
    }
}

impl TryFrom<&SauceHeader> for VectorCapabilities {
    type Error = SauceError;
    fn try_from(header: &SauceHeader) -> crate::Result<Self> {
        if header.data_type != SauceDataType::Vector {
            return Err(SauceError::UnsupportedDataType(header.data_type));
        }
        Ok(VectorCapabilities { format: VectorFormat::from_sauce(header.file_type) })
    }
}

impl Default for VectorCapabilities {
    /// Create default vector capabilities with DXF format.
    fn default() -> Self {
        Self::new(VectorFormat::Dxf)
    }
}
