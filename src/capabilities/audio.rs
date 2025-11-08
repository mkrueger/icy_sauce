//! Audio format capabilities as specified in the SAUCE v00 standard.
//!
//! This module provides types for describing audio files (tracker modules, MIDI, samples)
//! with SAUCE metadata. Audio formats can include sampling rate information for certain types.
//!
//! # SAUCE Field Mappings
//!
//! Audio formats use:
//! - DataType: Audio (4)
//! - FileType: Format variant (0-17)
//! - TInfo1: Sample rate in Hz (for some formats)
//! - TInfo2-4: All 0
//! - TFlags: 0
//! - TInfoS: Empty
//!
//! # Example
//!
//! ```
//! use icy_sauce::{AudioCapabilities, AudioFormat, SauceRecordBuilder, SauceDataType, Capabilities};
//!
//! // Build SAUCE metadata for a S3M module (tracker formats don't store sample rate)
//! let s3m_caps = AudioCapabilities { format: AudioFormat::S3m, sample_rate: 0 };
//! let s3m_sauce = SauceRecordBuilder::default()
//!     .data_type(SauceDataType::Audio)
//!     .capabilities(Capabilities::Audio(s3m_caps))
//!     .unwrap()
//!     .build();
//!
//! // Raw sample formats can include a sample rate (e.g. 44.1 kHz for 16-bit stereo)
//! let sample_caps = AudioCapabilities { format: AudioFormat::Smp16s, sample_rate: 44_100 };
//! let sample_sauce = SauceRecordBuilder::default()
//!     .data_type(SauceDataType::Audio)
//!     .capabilities(Capabilities::Audio(sample_caps))
//!     .unwrap()
//!     .build();
//! ```
use crate::{SauceDataType, header::SauceHeader};

/// Audio file format enumeration for SAUCE metadata.
///
/// Covers 25 distinct audio formats recognized by SAUCE, plus an unknown variant for
/// any future formats. Each variant has an associated SAUCE FileType code (0-24).
///
/// # Format Categories
///
/// ## Tracker/Module Formats (no sample rate metadata)
/// Formats for sequenced music with instrument data. Sample rate is fixed per format:
/// - MOD-family: 4/6/8-channel module format (stereo mixing configurable)
/// - S3M-family: ScreamTracker formats with variable sample rates
/// - XM/IT: FastTracker 2 and Impulse Tracker formats
/// - Other trackers: 669, STM, MTM, FAR, ULT, AMF, DMF, OKT
///
/// ## Raw Sample Formats (sample rate in TInfo1)
/// Uncompressed PCM audio samples with configurable sample rates:
/// - Smp8: 8-bit mono PCM
/// - Smp8s: 8-bit stereo PCM
/// - Smp16: 16-bit mono PCM
/// - Smp16s: 16-bit stereo PCM
///
/// ## FM Synthesis Formats
/// Music using frequency modulation synthesis (AdLib compatible):
/// - ROL: AdLib ROL (Roland formats)
/// - CMF: Creative Music File (AdLib format)
/// - HSC: HSC Tracker (FM composer)
/// - SAdT: SAdT Composer (FM synthesis)
///
/// ## Standard Audio Formats
/// - WAV: WAVE PCM audio container
/// - MIDI: Standard Musical Instrument Digital Interface
/// - VOC: Creative Voice File
/// - Patch8/Patch16: 8-bit and 16-bit patch formats
///
/// # Example
///
/// ```
/// use icy_sauce::AudioFormat;
///
/// // Parse from SAUCE file type code
/// let wav_format = AudioFormat::from_sauce(15);
/// assert_eq!(wav_format, AudioFormat::Wav);
///
/// // Convert back to file type code
/// assert_eq!(AudioFormat::Wav.to_sauce(), 15);
///
/// // Check format properties
/// assert!(!AudioFormat::Mod.has_sample_rate());   // Tracker format
/// assert!(AudioFormat::Smp16s.has_sample_rate()); // Raw sample format
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioFormat {
    /// MOD format: 4, 6, or 8 channel module (file type code 0)
    Mod,
    /// 669 format: Renaissance 8-channel module (file type code 1)
    Mod669,
    /// STM format: ScreamTracker 2 module (file type code 2)
    Stm,
    /// S3M format: ScreamTracker 3 module (file type code 3)
    S3m,
    /// MTM format: MultiTracker module (file type code 4)
    Mtm,
    /// FAR format: Farandole Composer module (file type code 5)
    Far,
    /// ULT format: UltraTracker module (file type code 6)
    Ult,
    /// AMF format: Advanced Module Format (file type code 7)
    Amf,
    /// DMF format: Delusion Digital Music Format (file type code 8)
    Dmf,
    /// OKT format: Oktalyser module (file type code 9)
    Okt,
    /// ROL format: AdLib ROL/Roland FM synthesis (file type code 10)
    Rol,
    /// CMF format: Creative Music File/AdLib FM synthesis (file type code 11)
    Cmf,
    /// MIDI format: Standard MIDI file (file type code 12)
    Mid,
    /// SAdT format: SAdT Composer FM synthesis (file type code 13)
    Sadt,
    /// VOC format: Creative Voice File (file type code 14)
    Voc,
    /// WAV format: WAVE PCM audio (file type code 15)
    Wav,
    /// Raw 8-bit mono PCM sample (file type code 16)
    Smp8,
    /// Raw 8-bit stereo PCM sample (file type code 17)
    Smp8s,
    /// Raw 16-bit mono PCM sample (file type code 18)
    Smp16,
    /// Raw 16-bit stereo PCM sample (file type code 19)
    Smp16s,
    /// 8-bit patch format (file type code 20)
    Patch8,
    /// 16-bit patch format (file type code 21)
    Patch16,
    /// XM format: FastTracker 2 module (file type code 22)
    Xm,
    /// HSC format: HSC Tracker FM synthesis (file type code 23)
    Hsc,
    /// IT format: Impulse Tracker module (file type code 24)
    It,

    /// Unknown audio format with arbitrary file type code
    Unknown(u8),
}

impl AudioFormat {
    /// Parse audio format from SAUCE file type code.
    ///
    /// Converts a SAUCE FileType byte (0-24) into the corresponding `AudioFormat` enum variant.
    /// Any file type code not in the range 0-24 is wrapped in `AudioFormat::Unknown`.
    ///
    /// # SAUCE Mapping
    ///
    /// File type codes for AudioData records:
    /// | Code | Format |
    /// |------|--------|
    /// | 0    | MOD    |
    /// | 1    | 669    |
    /// | 2    | STM    |
    /// | 3    | S3M    |
    /// | 4    | MTM    |
    /// | 5    | FAR    |
    /// | 6    | ULT    |
    /// | 7    | AMF    |
    /// | 8    | DMF    |
    /// | 9    | OKT    |
    /// | 10   | ROL    |
    /// | 11   | CMF    |
    /// | 12   | MIDI   |
    /// | 13   | SAdT   |
    /// | 14   | VOC    |
    /// | 15   | WAV    |
    /// | 16   | SMP8   |
    /// | 17   | SMP8S  |
    /// | 18   | SMP16  |
    /// | 19   | SMP16S |
    /// | 20   | PATCH8 |
    /// | 21   | PATCH16|
    /// | 22   | XM     |
    /// | 23   | HSC    |
    /// | 24   | IT     |
    ///
    /// # Arguments
    ///
    /// * `file_type` - SAUCE file type byte (typically from `SauceHeader::file_type`)
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::AudioFormat;
    ///
    /// assert_eq!(AudioFormat::from_sauce(15), AudioFormat::Wav);
    /// assert_eq!(AudioFormat::from_sauce(3), AudioFormat::S3m);
    /// assert_eq!(AudioFormat::from_sauce(99), AudioFormat::Unknown(99));
    /// ```
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

    /// Convert audio format back to SAUCE file type code.
    ///
    /// Inverse of [`Self::from_sauce`]. Converts an `AudioFormat` variant into its
    /// corresponding SAUCE FileType byte (0-24) for serialization into headers.
    ///
    /// # Returns
    ///
    /// A SAUCE file type code (0-24) or the wrapped value for `Unknown` variants.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::AudioFormat;
    ///
    /// assert_eq!(AudioFormat::Wav.to_sauce(), 15);
    /// assert_eq!(AudioFormat::Xm.to_sauce(), 22);
    /// assert_eq!(AudioFormat::Unknown(99).to_sauce(), 99);
    /// ```
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

    /// Check if this format stores sample rate in SAUCE TInfo1 field.
    ///
    /// Only raw PCM sample formats (8-bit and 16-bit, mono and stereo) store the sample
    /// rate in the SAUCE header. All other formats (tracker, FM synthesis, etc.) have
    /// fixed or format-dependent sample rates and use TInfo1 = 0.
    ///
    /// # Returns
    ///
    /// `true` if sample rate is stored in SAUCE TInfo1, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::AudioFormat;
    ///
    /// // Raw sample formats store sample rate
    /// assert!(AudioFormat::Smp8.has_sample_rate());
    /// assert!(AudioFormat::Smp16s.has_sample_rate());
    ///
    /// // Tracker formats have fixed/format-dependent rates
    /// assert!(!AudioFormat::Mod.has_sample_rate());
    /// assert!(!AudioFormat::S3m.has_sample_rate());
    ///
    /// // WAV is a container and can vary, so TInfo1 is not used
    /// assert!(!AudioFormat::Wav.has_sample_rate());
    /// ```
    pub fn has_sample_rate(&self) -> bool {
        matches!(
            self,
            AudioFormat::Smp8 | AudioFormat::Smp8s | AudioFormat::Smp16 | AudioFormat::Smp16s
        )
    }

    /// Check if this is a raw PCM sample format.
    ///
    /// Raw sample formats represent uncompressed PCM audio data (8-bit or 16-bit, mono or stereo).
    /// These formats may store sample rate information in the SAUCE TInfo1 field.
    ///
    /// # Returns
    ///
    /// `true` if this is a raw sample format (Smp8, Smp8s, Smp16, or Smp16s), `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::AudioFormat;
    ///
    /// assert!(AudioFormat::Smp8.is_raw_sample());
    /// assert!(AudioFormat::Smp16s.is_raw_sample());
    /// assert!(!AudioFormat::Wav.is_raw_sample());
    /// assert!(!AudioFormat::Mod.is_raw_sample());
    /// ```
    pub fn is_raw_sample(&self) -> bool {
        matches!(
            self,
            AudioFormat::Smp8 | AudioFormat::Smp8s | AudioFormat::Smp16 | AudioFormat::Smp16s
        )
    }

    /// Check if this is a stereo audio format.
    ///
    /// Stereo formats represent two-channel audio data. Only applies to raw sample formats;
    /// tracker and FM synthesis formats may have configurable mono/stereo mixing but
    /// are not classified as "stereo" by this format.
    ///
    /// # Returns
    ///
    /// `true` if this is a stereo raw sample format (Smp8s or Smp16s), `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::AudioFormat;
    ///
    /// assert!(AudioFormat::Smp8s.is_stereo());
    /// assert!(AudioFormat::Smp16s.is_stereo());
    /// assert!(!AudioFormat::Smp8.is_stereo());
    /// assert!(!AudioFormat::Smp16.is_stereo());
    /// ```
    pub fn is_stereo(&self) -> bool {
        matches!(self, AudioFormat::Smp8s | AudioFormat::Smp16s)
    }

    /// Check if this is a 16-bit audio format.
    ///
    /// 16-bit formats provide higher audio quality (65536 amplitude levels) compared to
    /// 8-bit formats (256 amplitude levels). Includes 16-bit raw samples and patch formats.
    ///
    /// # Returns
    ///
    /// `true` if this is a 16-bit format (Smp16, Smp16s, or Patch16), `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::AudioFormat;
    ///
    /// assert!(AudioFormat::Smp16.is_16bit());
    /// assert!(AudioFormat::Smp16s.is_16bit());
    /// assert!(AudioFormat::Patch16.is_16bit());
    /// assert!(!AudioFormat::Smp8.is_16bit());
    /// assert!(!AudioFormat::Patch8.is_16bit());
    /// ```
    pub fn is_16bit(&self) -> bool {
        matches!(
            self,
            AudioFormat::Smp16 | AudioFormat::Smp16s | AudioFormat::Patch16
        )
    }

    /// Check if this is an FM synthesis format.
    ///
    /// FM synthesis formats use frequency modulation to generate sound, typically on
    /// AdLib-compatible FM synthesizer hardware. These formats cannot store sample rate
    /// metadata in SAUCE (TInfo1 is always 0).
    ///
    /// # Returns
    ///
    /// `true` if this is an FM synthesis format (ROL, CMF, SAdT, or HSC), `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::AudioFormat;
    ///
    /// assert!(AudioFormat::Rol.is_fm_synthesis());
    /// assert!(AudioFormat::Cmf.is_fm_synthesis());
    /// assert!(AudioFormat::Hsc.is_fm_synthesis());
    /// assert!(!AudioFormat::Wav.is_fm_synthesis());
    /// assert!(!AudioFormat::Smp16.is_fm_synthesis());
    /// ```
    pub fn is_fm_synthesis(&self) -> bool {
        matches!(
            self,
            AudioFormat::Rol | AudioFormat::Cmf | AudioFormat::Sadt | AudioFormat::Hsc
        )
    }

    /// Check if this is a tracker/module format.
    ///
    /// Tracker formats represent sequenced music with instrument data, patterns, and effects.
    /// These formats typically have fixed or format-dependent sample rates and do not store
    /// sample rate metadata in SAUCE (TInfo1 is always 0).
    ///
    /// # Returns
    ///
    /// `true` if this is a tracker/module format, `false` otherwise.
    ///
    /// # Covered Formats
    ///
    /// * MOD, 669, STM, S3M, MTM, FAR, ULT, AMF, DMF, OKT, XM, IT, MIDI
    ///
    /// # Example
    ///
    /// ```
    /// use icy_sauce::AudioFormat;
    ///
    /// assert!(AudioFormat::Mod.is_tracker());
    /// assert!(AudioFormat::S3m.is_tracker());
    /// assert!(AudioFormat::It.is_tracker());
    /// assert!(!AudioFormat::Wav.is_tracker());
    /// assert!(!AudioFormat::Smp16.is_tracker());
    /// ```
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

/// Audio file capabilities for SAUCE records.
///
/// Represents audio-specific metadata parsed from or to be written to a SAUCE header.
/// This includes the audio format and optional sample rate for raw sample formats.
///
/// # SAUCE Integration
///
/// Audio records require DataType = Audio (10). Specific format capabilities are encoded
/// in the FileType byte (0-24), with optional sample rate in TInfo1 for raw samples.
///
/// # Example
///
/// ```
/// use icy_sauce::{AudioCapabilities, AudioFormat};
///
/// // Create raw 16-bit stereo at 44.1kHz
/// let audio = AudioCapabilities {
///     format: AudioFormat::Smp16s,
///     sample_rate: 44100,
/// };
///
/// assert!(audio.format.is_16bit());
/// assert!(audio.format.is_stereo());
/// ```
#[derive(Debug, Clone)]
pub struct AudioCapabilities {
    /// Audio format (MOD, WAV, raw samples, etc.)
    pub format: AudioFormat,
    /// Sample rate in Hz (only used for raw sample formats; 0 for others)
    pub sample_rate: u16,
}

impl AudioCapabilities {
    /// Parse audio capabilities from a SAUCE header.
    ///
    /// # Arguments
    ///
    /// * `header` - The SAUCE header to parse
    ///
    /// # Returns
    ///
    /// Audio capabilities extracted from header fields.
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::UnsupportedDataType`] if DataType is not Audio.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Internal parsing example (ignored because `AudioCapabilities::from` is not public)
    /// use icy_sauce::{header::SauceHeader, SauceDataType, AudioFormat};
    /// use icy_sauce::AudioCapabilities;
    ///
    /// let mut header = SauceHeader::default();
    /// header.data_type = SauceDataType::Audio;
    /// header.file_type = 3; // S3M
    /// header.t_info1 = 44100;
    /// let caps = AudioCapabilities::from(&header).unwrap();
    /// assert_eq!(caps.format, AudioFormat::S3m);
    /// assert_eq!(caps.sample_rate, 44100);
    /// ```
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

        Ok(AudioCapabilities {
            format,
            sample_rate,
        })
    }

    /// Serialize audio capabilities into a SAUCE header.
    ///
    /// # Arguments
    ///
    /// * `header` - Mutable reference to the SAUCE header to populate
    ///
    /// # Errors
    ///
    /// Returns [`SauceError::UnsupportedDataType`] if DataType cannot be set to Audio.
    ///
    /// # Behavior
    ///
    /// Sets the following header fields:
    /// - DataType = Audio
    /// - FileType = Format variant (0-17)
    /// - TInfo1 = Sample rate (or 0 if not applicable)
    /// - TInfo2-4 = 0
    /// - TFlags = 0
    /// - TInfoS = Empty
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Internal serialization example (ignored because `encode_into_header` is not public)
    /// use icy_sauce::{AudioCapabilities, AudioFormat};
    /// use icy_sauce::header::SauceHeader;
    ///
    /// let caps = AudioCapabilities { format: AudioFormat::Mod, sample_rate: 0 };
    /// let mut header = SauceHeader::default();
    /// caps.encode_into_header(&mut header).unwrap();
    /// assert_eq!(header.file_type, 0);
    /// ```
    pub(crate) fn encode_into_header(&self, header: &mut SauceHeader) -> crate::Result<()> {
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
