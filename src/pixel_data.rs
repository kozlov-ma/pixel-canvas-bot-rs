use std::hash::{Hash, Hasher};
use serde_json::json;
use serde::Serialize;
use image::Pixel;
use strum::IntoEnumIterator;
use crate::pixel_color::PixelColor;

#[derive(Debug, Serialize, Hash, PartialEq, Eq)]
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
