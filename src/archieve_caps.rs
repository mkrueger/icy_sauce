use crate::{SauceDataType, SauceError, header::SauceHeader};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArchiveFormat {
    Zip, // 0 - PKWare ZIP
    Arj, // 1 - ARJ by Robert K. Jung
    Lzh, // 2 - LZH by Haruyasu Yoshizaki (Yoshi)
    Arc, // 3 - ARC by S.E.A.
    Tar, // 4 - Unix TAR
    Zoo, // 5 - ZOO
    Rar, // 6 - RAR
    Uc2, // 7 - UC2
    Pak, // 8 - PAK
    Sqz, // 9 - SQZ

    Unknown(u8),
}

impl ArchiveFormat {
    /// Parse from file type byte
    pub fn from_sauce(file_type: u8) -> Self {
        match file_type {
            0 => ArchiveFormat::Zip,
            1 => ArchiveFormat::Arj,
            2 => ArchiveFormat::Lzh,
            3 => ArchiveFormat::Arc,
            4 => ArchiveFormat::Tar,
            5 => ArchiveFormat::Zoo,
            6 => ArchiveFormat::Rar,
            7 => ArchiveFormat::Uc2,
            8 => ArchiveFormat::Pak,
            9 => ArchiveFormat::Sqz,
            _ => ArchiveFormat::Unknown(file_type),
        }
    }

    /// Convert back to file type byte
    pub fn to_sauce(&self) -> u8 {
        match self {
            ArchiveFormat::Zip => 0,
            ArchiveFormat::Arj => 1,
            ArchiveFormat::Lzh => 2,
            ArchiveFormat::Arc => 3,
            ArchiveFormat::Tar => 4,
            ArchiveFormat::Zoo => 5,
            ArchiveFormat::Rar => 6,
            ArchiveFormat::Uc2 => 7,
            ArchiveFormat::Pak => 8,
            ArchiveFormat::Sqz => 9,
            ArchiveFormat::Unknown(ft) => *ft,
        }
    }

    /// Get common file extension for this archive format
    pub fn extension(&self) -> &str {
        match self {
            ArchiveFormat::Zip => "zip",
            ArchiveFormat::Arj => "arj",
            ArchiveFormat::Lzh => "lzh",
            ArchiveFormat::Arc => "arc",
            ArchiveFormat::Tar => "tar",
            ArchiveFormat::Zoo => "zoo",
            ArchiveFormat::Rar => "rar",
            ArchiveFormat::Uc2 => "uc2",
            ArchiveFormat::Pak => "pak",
            ArchiveFormat::Sqz => "sqz",
            ArchiveFormat::Unknown(_) => "",
        }
    }

    /// Check if this is a compressed archive format
    pub fn is_compressed(&self) -> bool {
        match self {
            ArchiveFormat::Tar => false, // TAR is uncompressed
            ArchiveFormat::Unknown(_) => false,
            _ => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArchiveCaps {
    pub format: ArchiveFormat,
}

impl ArchiveCaps {
    /// Create new archive capabilities
    pub fn new(format: ArchiveFormat) -> Self {
        ArchiveCaps { format }
    }

    pub(crate) fn from(header: &SauceHeader) -> crate::Result<Self> {
        if header.data_type != SauceDataType::Archive {
            return Err(SauceError::UnsupportedDataType(header.data_type));
        }

        let format = ArchiveFormat::from_sauce(header.file_type);

        Ok(ArchiveCaps { format })
    }

    pub(crate) fn write_to_header(&self, header: &mut SauceHeader) -> crate::Result<()> {
        header.data_type = SauceDataType::Archive;
        header.file_type = self.format.to_sauce();

        // Archive formats have all TInfo fields set to 0 per spec
        header.t_info1 = 0;
        header.t_info2 = 0;
        header.t_info3 = 0;
        header.t_info4 = 0;

        // No flags or TInfoS for archives
        header.t_flags = 0;
        header.t_info_s.clear();

        Ok(())
    }
}
