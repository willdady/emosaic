use image::{Rgba, RgbaImage};

pub type QuadRgba = [Rgba<u8>; 4];
pub type NilRgba = [Rgba<u8>; 0];

pub fn average_color(img: &RgbaImage, rect: &(u32, u32, u32, u32)) -> Rgba<u8> {
    let (left, top, width, height) = rect;
    let mut r = 0.0;
    let mut g = 0.0;
    let mut b = 0.0;
    let mut a = 0.0;
    let mut count = 0.0;
    for y in *top..*height {
        for x in *left..*width {
            let pixel = img.get_pixel(x, y);
            r += f64::from(pixel[0]);
            g += f64::from(pixel[1]);
            b += f64::from(pixel[2]);
            a += f64::from(pixel[3]);
            count += 1.0;
        }
    }
    let r = (r / count).round() as u8;
    let g = (g / count).round() as u8;
    let b = (b / count).round() as u8;
    let a = (a / count).round() as u8;
    Rgba([r, g, b, a])
}

pub fn compare_color(a: Rgba<u8>, b: Rgba<u8>) -> f64 {
    let r1 = f64::from(a[0]);
    let g1 = f64::from(a[1]);
    let b1 = f64::from(a[2]);
    let r2 = f64::from(b[0]);
    let g2 = f64::from(b[1]);
    let b2 = f64::from(b[2]);
    ((r2 - r1) * 0.3).powi(2) + ((g2 - g1) * 0.59).powi(2) + ((b2 - b1) * 0.11).powi(2)
}
