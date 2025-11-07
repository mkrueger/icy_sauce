use crate::{SauceDataType, header::SauceHeader};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraphicsFormat {
    // Bitmap formats (DataType::Bitmap)
    Gif,    // 0
    Pcx,    // 1
    LbmIff, // 2
    Tga,    // 3
    Fli,    // 4
    Flc,    // 5
    Bmp,    // 6
    Gl,     // 7
    Dl,     // 8
    Wpg,    // 9
    Png,    // 10
    Jpg,    // 11
    Mpg,    // 12
    Avi,    // 13

    // Vector formats (DataType::Vector)
    Dxf,       // 0
    Dwg,       // 1
    WpgVector, // 2
    ThreeDs,   // 3

    // Special case: RipScript is Character type 3
    RipScript,

    Unknown(SauceDataType, u8),
}

impl GraphicsFormat {
    /// Parse from data type and file type bytes
    pub fn from_sauce(data_type: SauceDataType, file_type: u8) -> Self {
        match data_type {
            SauceDataType::Character if file_type == 3 => GraphicsFormat::RipScript,
            SauceDataType::Bitmap => match file_type {
                0 => GraphicsFormat::Gif,
                1 => GraphicsFormat::Pcx,
                2 => GraphicsFormat::LbmIff,
                3 => GraphicsFormat::Tga,
                4 => GraphicsFormat::Fli,
                5 => GraphicsFormat::Flc,
                6 => GraphicsFormat::Bmp,
                7 => GraphicsFormat::Gl,
                8 => GraphicsFormat::Dl,
                9 => GraphicsFormat::Wpg,
                10 => GraphicsFormat::Png,
                11 => GraphicsFormat::Jpg,
                12 => GraphicsFormat::Mpg,
                13 => GraphicsFormat::Avi,
                _ => GraphicsFormat::Unknown(data_type, file_type),
            },
            SauceDataType::Vector => match file_type {
                0 => GraphicsFormat::Dxf,
                1 => GraphicsFormat::Dwg,
                2 => GraphicsFormat::WpgVector,
                3 => GraphicsFormat::ThreeDs,
                _ => GraphicsFormat::Unknown(data_type, file_type),
            },
            _ => GraphicsFormat::Unknown(data_type, file_type),
        }
    }

    /// Convert back to data type and file type
    pub fn to_sauce(&self) -> (SauceDataType, u8) {
        match self {
            // RipScript special case
            GraphicsFormat::RipScript => (SauceDataType::Character, 3),

            // Bitmap formats
            GraphicsFormat::Gif => (SauceDataType::Bitmap, 0),
            GraphicsFormat::Pcx => (SauceDataType::Bitmap, 1),
            GraphicsFormat::LbmIff => (SauceDataType::Bitmap, 2),
            GraphicsFormat::Tga => (SauceDataType::Bitmap, 3),
            GraphicsFormat::Fli => (SauceDataType::Bitmap, 4),
            GraphicsFormat::Flc => (SauceDataType::Bitmap, 5),
            GraphicsFormat::Bmp => (SauceDataType::Bitmap, 6),
            GraphicsFormat::Gl => (SauceDataType::Bitmap, 7),
            GraphicsFormat::Dl => (SauceDataType::Bitmap, 8),
            GraphicsFormat::Wpg => (SauceDataType::Bitmap, 9),
            GraphicsFormat::Png => (SauceDataType::Bitmap, 10),
            GraphicsFormat::Jpg => (SauceDataType::Bitmap, 11),
            GraphicsFormat::Mpg => (SauceDataType::Bitmap, 12),
            GraphicsFormat::Avi => (SauceDataType::Bitmap, 13),

            // Vector formats
            GraphicsFormat::Dxf => (SauceDataType::Vector, 0),
            GraphicsFormat::Dwg => (SauceDataType::Vector, 1),
            GraphicsFormat::WpgVector => (SauceDataType::Vector, 2),
            GraphicsFormat::ThreeDs => (SauceDataType::Vector, 3),

            GraphicsFormat::Unknown(dt, ft) => (*dt, *ft),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PixelCaps {
    pub format: GraphicsFormat,
    pub width: u16,
    pub height: u16,
    pub pixel_depth: u16,
}

impl PixelCaps {
    pub fn new(graphics_format: GraphicsFormat) -> Self {
        PixelCaps {
            format: graphics_format,
            width: 0,
            height: 0,
            pixel_depth: 0,
        }
    }

    pub(crate) fn from(header: &SauceHeader) -> crate::Result<Self> {
        let graphics_format = GraphicsFormat::from_sauce(header.data_type, header.file_type);

        let (width, height, pixel_depth) = match header.data_type {
            SauceDataType::Character if header.file_type == 3 => {
                // RipScript has fixed dimensions per spec
                (640, 350, 16) // 640x350 pixels, 16 colors
            }
            SauceDataType::Bitmap => {
                // All bitmap formats use TInfo1-3 for dimensions
                (header.t_info1, header.t_info2, header.t_info3)
            }
            SauceDataType::Vector => {
                // Vector formats don't have pixel dimensions
                (0, 0, 0)
            }
            _ => {
                // Shouldn't happen but handle gracefully
                (0, 0, 0)
            }
        };

        Ok(PixelCaps {
            format: graphics_format,
            width,
            height,
            pixel_depth,
        })
    }

    pub(crate) fn write_to_header(&self, header: &mut SauceHeader) -> crate::Result<()> {
        let (data_type, file_type) = self.format.to_sauce();
        header.data_type = data_type;
        header.file_type = file_type;

        match self.format {
            GraphicsFormat::RipScript => {
                // RipScript always has fixed values
                header.t_info1 = 640;
                header.t_info2 = 350;
                header.t_info3 = 16;
                header.t_info4 = 0;
                header.t_flags = 0;
                header.t_info_s.clear();
            }
            GraphicsFormat::Gif
            | GraphicsFormat::Pcx
            | GraphicsFormat::LbmIff
            | GraphicsFormat::Tga
            | GraphicsFormat::Fli
            | GraphicsFormat::Flc
            | GraphicsFormat::Bmp
            | GraphicsFormat::Gl
            | GraphicsFormat::Dl
            | GraphicsFormat::Wpg
            | GraphicsFormat::Png
            | GraphicsFormat::Jpg
            | GraphicsFormat::Mpg
            | GraphicsFormat::Avi => {
                // Bitmap formats store dimensions
                header.t_info1 = self.width;
                header.t_info2 = self.height;
                header.t_info3 = self.pixel_depth;
                header.t_info4 = 0;
                header.t_flags = 0;
                header.t_info_s.clear();
            }
            GraphicsFormat::Dxf
            | GraphicsFormat::Dwg
            | GraphicsFormat::WpgVector
            | GraphicsFormat::ThreeDs => {
                // Vector formats have no pixel info
                header.t_info1 = 0;
                header.t_info2 = 0;
                header.t_info3 = 0;
                header.t_info4 = 0;
                header.t_flags = 0;
                header.t_info_s.clear();
            }
            GraphicsFormat::Unknown(_, _) => {
                // Preserve whatever values are there
                header.t_info1 = self.width;
                header.t_info2 = self.height;
                header.t_info3 = self.pixel_depth;
            }
        }

        Ok(())
    }

    /// Check if this format has pixel dimensions
    pub fn has_dimensions(&self) -> bool {
        !matches!(
            self.format,
            GraphicsFormat::Dxf
                | GraphicsFormat::Dwg
                | GraphicsFormat::WpgVector
                | GraphicsFormat::ThreeDs
        )
    }

    /// Check if this is an animated format
    pub fn is_animated(&self) -> bool {
        matches!(
            self.format,
            GraphicsFormat::Fli
                | GraphicsFormat::Flc
                | GraphicsFormat::Gl
                | GraphicsFormat::Dl
                | GraphicsFormat::Mpg
                | GraphicsFormat::Avi
        )
    }

    /// Check if this is a vector format
    pub fn is_vector(&self) -> bool {
        matches!(
            self.format,
            GraphicsFormat::Dxf
                | GraphicsFormat::Dwg
                | GraphicsFormat::WpgVector
                | GraphicsFormat::ThreeDs
        )
    }
}
