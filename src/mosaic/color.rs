use image::{Rgb, RgbImage};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct SerializableRgb(u8, u8, u8);

impl SerializableRgb {
    pub fn red(&self) -> u8 {
        self.0
    }

    pub fn green(&self) -> u8 {
        self.1
    }

    pub fn blue(&self) -> u8 {
        self.2
    }
}

pub trait IntoSerializableRgb {
    fn into_serializable_rgb(self) -> SerializableRgb;
}

impl IntoSerializableRgb for Rgb<u8> {
    fn into_serializable_rgb(self) -> SerializableRgb {
        SerializableRgb(self[0], self[1], self[2])
    }
}

impl From<SerializableRgb> for [u8; 3] {
    fn from(rgb: SerializableRgb) -> Self {
        [rgb.0, rgb.1, rgb.2]
    }
}

impl From<SerializableRgb> for [f64; 3] {
    fn from(rgb: SerializableRgb) -> Self {
        [f64::from(rgb.0), f64::from(rgb.1), f64::from(rgb.2)]
    }
}

pub fn average_color(img: &RgbImage, rect: &(u32, u32, u32, u32)) -> Rgb<u8> {
    let (left, top, width, height) = rect;
    let mut r = 0.0;
    let mut g = 0.0;
    let mut b = 0.0;
    let mut count = 0.0;
    for y in *top..*height {
        for x in *left..*width {
            let pixel = img.get_pixel(x, y);
            r += f64::from(pixel[0]);
            g += f64::from(pixel[1]);
            b += f64::from(pixel[2]);
            count += 1.0;
        }
    }
    let r = (r / count).round() as u8;
    let g = (g / count).round() as u8;
    let b = (b / count).round() as u8;
    Rgb([r, g, b])
}
