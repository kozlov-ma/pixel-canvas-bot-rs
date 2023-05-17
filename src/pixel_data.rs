use image::imageops::grayscale;
use serde_json::json;
use strum_macros::EnumIter;
use serde::Serialize;
use image::Pixel;
use strum::IntoEnumIterator;

#[derive(Debug, Serialize)]
pub struct PixelData {
    x: i32,
    y: i32,
    color: PixelColor
}

impl PixelData {
    pub fn from_rgb8(x: i32, y: i32, pixel: &image::Rgb<u8>, grayscale: bool) -> Self {
        let channels = pixel.channels();
        let color = PixelData::pixel_color_from_channels(channels, grayscale);

        Self { x, y, color }
    }

    fn pixel_color_from_channels(channels: &[u8], grayscale: bool) -> PixelColor {
        if grayscale {
            [PixelColor::White, PixelColor::Black, PixelColor::LightGray, PixelColor::Gray].iter()
                .min_by_key(
                    |p| PixelData::get_color_diff(&p.get_channels(), &channels.to_owned()))
                .unwrap().to_owned()
        } else {
            PixelColor::iter()
                .min_by_key(
                    |p| PixelData::get_color_diff(&p.get_channels(), &channels.to_owned()))
                .unwrap()
        }

    }

    fn get_color_diff(c1: &[u8], c2: &[u8]) -> u64 {
        c1.iter()
            .zip(c2.iter())
            .map(|(a, b)| (a.to_owned() as i64 - b.to_owned() as i64).unsigned_abs())
            .sum()
    }

    pub fn get_json(self, fingerprint: &str) -> String {
        let a = self.x + self.y + 8;
        let color = self.color as i16;
        json!(
            {
                "x":self.x,
                "y":self.y,
                "a":a,
                "color":color,
                "fingerprint":fingerprint,
                "token":null
            })
            .to_string()
    }
}

#[derive(Debug, EnumIter, Serialize, Copy, Clone)]
#[repr(u8)]
enum PixelColor {
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
