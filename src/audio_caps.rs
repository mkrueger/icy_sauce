use crate::{SauceDataType, header::SauceHeader};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioFormat {
    // Tracker formats (no sample rate)
    Mod,     // 0 - 4, 6 or 8 channel MOD
    Mod669,  // 1 - Renaissance 8 channel 669
    Stm,     // 2 - ScreamTracker 2
    S3m,     // 3 - ScreamTracker 3
    Mtm,     // 4 - MultiTracker
    Far,     // 5 - Farandole Composer
    Ult,     // 6 - UltraTracker
    Amf,     // 7 - Advanced Module Format
    Dmf,     // 8 - Delusion Digital Music Format
    Okt,     // 9 - Oktalyser
    Rol,     // 10 - AdLib ROL (FM)
    Cmf,     // 11 - Creative Music File (FM)
    Mid,     // 12 - MIDI
    Sadt,    // 13 - SAdT composer (FM)
    Voc,     // 14 - Creative Voice File
    Wav,     // 15 - Wave
    Smp8,    // 16 - Raw 8-bit mono sample
    Smp8s,   // 17 - Raw 8-bit stereo sample
    Smp16,   // 18 - Raw 16-bit mono sample
    Smp16s,  // 19 - Raw 16-bit stereo sample
    Patch8,  // 20 - 8-bit patch
    Patch16, // 21 - 16-bit patch
    Xm,      // 22 - FastTracker 2
    Hsc,     // 23 - HSC Tracker (FM)
    It,      // 24 - Impulse Tracker

    Unknown(u8),
}

impl AudioFormat {
    /// Parse from file type byte
    pub fn from_sauce(file_type: u8) -> Self {
        match file_type {
            0 => AudioFormat::Mod,
            1 => AudioFormat::Mod669,
            2 => AudioFormat::Stm,
            3 => AudioFormat::S3m,
            4 => AudioFormat::Mtm,
            5 => AudioFormat::Far,
            6 => AudioFormat::Ult,
            7 => AudioFormat::Amf,
            8 => AudioFormat::Dmf,
            9 => AudioFormat::Okt,
            10 => AudioFormat::Rol,
            11 => AudioFormat::Cmf,
            12 => AudioFormat::Mid,
            13 => AudioFormat::Sadt,
            14 => AudioFormat::Voc,
            15 => AudioFormat::Wav,
            16 => AudioFormat::Smp8,
            17 => AudioFormat::Smp8s,
            18 => AudioFormat::Smp16,
            19 => AudioFormat::Smp16s,
            20 => AudioFormat::Patch8,
            21 => AudioFormat::Patch16,
            22 => AudioFormat::Xm,
            23 => AudioFormat::Hsc,
            24 => AudioFormat::It,
            _ => AudioFormat::Unknown(file_type),
        }
    }

    /// Convert back to file type byte
    pub fn to_sauce(&self) -> u8 {
        match self {
            AudioFormat::Mod => 0,
            AudioFormat::Mod669 => 1,
            AudioFormat::Stm => 2,
            AudioFormat::S3m => 3,
            AudioFormat::Mtm => 4,
            AudioFormat::Far => 5,
            AudioFormat::Ult => 6,
            AudioFormat::Amf => 7,
            AudioFormat::Dmf => 8,
            AudioFormat::Okt => 9,
            AudioFormat::Rol => 10,
            AudioFormat::Cmf => 11,
            AudioFormat::Mid => 12,
            AudioFormat::Sadt => 13,
            AudioFormat::Voc => 14,
            AudioFormat::Wav => 15,
            AudioFormat::Smp8 => 16,
            AudioFormat::Smp8s => 17,
            AudioFormat::Smp16 => 18,
            AudioFormat::Smp16s => 19,
            AudioFormat::Patch8 => 20,
            AudioFormat::Patch16 => 21,
            AudioFormat::Xm => 22,
            AudioFormat::Hsc => 23,
            AudioFormat::It => 24,
            AudioFormat::Unknown(ft) => *ft,
        }
    }

    /// Check if this format stores sample rate in TInfo1
    pub fn has_sample_rate(&self) -> bool {
        matches!(
            self,
            AudioFormat::Smp8 | AudioFormat::Smp8s | AudioFormat::Smp16 | AudioFormat::Smp16s
        )
    }

    /// Check if this is a raw sample format
    pub fn is_raw_sample(&self) -> bool {
        matches!(
            self,
            AudioFormat::Smp8 | AudioFormat::Smp8s | AudioFormat::Smp16 | AudioFormat::Smp16s
        )
    }

    /// Check if this is a stereo format
    pub fn is_stereo(&self) -> bool {
        matches!(self, AudioFormat::Smp8s | AudioFormat::Smp16s)
    }

    /// Check if this is 16-bit audio
    pub fn is_16bit(&self) -> bool {
        matches!(
            self,
            AudioFormat::Smp16 | AudioFormat::Smp16s | AudioFormat::Patch16
        )
    }

    /// Check if this is an FM synthesis format
    pub fn is_fm_synthesis(&self) -> bool {
        matches!(
            self,
            AudioFormat::Rol | AudioFormat::Cmf | AudioFormat::Sadt | AudioFormat::Hsc
        )
    }

    /// Check if this is a tracker/module format
    pub fn is_tracker(&self) -> bool {
        matches!(
            self,
            AudioFormat::Mod
                | AudioFormat::Mod669
                | AudioFormat::Stm
                | AudioFormat::S3m
                | AudioFormat::Mtm
                | AudioFormat::Far
                | AudioFormat::Ult
                | AudioFormat::Amf
                | AudioFormat::Dmf
                | AudioFormat::Okt
                | AudioFormat::Xm
                | AudioFormat::It
        )
    }
}

#[derive(Debug, Clone)]
pub struct AudioCaps {
    pub format: AudioFormat,
    pub sample_rate: u16, // Only meaningful for raw sample formats
}

impl AudioCaps {
    pub(crate) fn from(header: &SauceHeader) -> crate::Result<Self> {
        if header.data_type != SauceDataType::Audio {
            return Err(crate::SauceError::UnsupportedDataType(header.data_type));
        }

        let format = AudioFormat::from_sauce(header.file_type);

        // Only raw sample formats use TInfo1 for sample rate
        let sample_rate = if format.has_sample_rate() {
            header.t_info1
        } else {
            0
        };

        Ok(AudioCaps {
            format,
            sample_rate,
        })
    }

    pub(crate) fn write_to_header(&self, header: &mut SauceHeader) -> crate::Result<()> {
        header.data_type = SauceDataType::Audio;
        header.file_type = self.format.to_sauce();

        // Set TInfo fields based on format
        if self.format.has_sample_rate() {
            // Raw sample formats store sample rate in TInfo1
            header.t_info1 = self.sample_rate;
            header.t_info2 = 0;
            header.t_info3 = 0;
            header.t_info4 = 0;
        } else {
            // All other audio formats have zeros for TInfo
            header.t_info1 = 0;
            header.t_info2 = 0;
            header.t_info3 = 0;
            header.t_info4 = 0;
        }

        // Audio formats don't use flags or TInfoS
        header.t_flags = 0;
        header.t_info_s.clear();

        Ok(())
    }
}
