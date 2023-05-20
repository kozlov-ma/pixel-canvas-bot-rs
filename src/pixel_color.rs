use strum_macros::EnumIter;
use serde::Serialize;

#[derive(Debug, EnumIter, Serialize, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum PixelColor {
    White,
    LightGray,
    Gray,
    Black,
    Pink,
    Red,
    Orange,
    Brown,
    Yellow,
    LightGreen,
    Green,
    Sky,
    Teal,
    Blue,
    Magenta,
    Purple
}

impl PixelColor {
    pub fn get_channels(self) -> Vec<u8> {
        match self {
            PixelColor::White => vec![255, 255, 255],
            PixelColor::LightGray => vec![228, 228, 228],
            PixelColor::Gray => vec![136, 136, 136],
            PixelColor::Black => vec![34, 34, 34],
            PixelColor::Pink => vec![255, 167, 209],
            PixelColor::Red => vec![229, 0, 0],
            PixelColor::Orange => vec![229, 149, 0],
            PixelColor::Brown => vec![160, 106, 66],
            PixelColor::Yellow => vec![229, 217, 0],
            PixelColor::LightGreen => vec![148, 224, 68],
            PixelColor::Green => vec![2, 190, 1],
            PixelColor::Sky => vec![0, 211, 211],
            PixelColor::Teal => vec![0, 131, 199],
            PixelColor::Blue => vec![0, 0, 234],
            PixelColor::Magenta => vec![207, 110, 228],
            PixelColor::Purple => vec![130, 0, 128],
        }
    }
}
