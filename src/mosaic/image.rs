use std::fs::{self};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::thread;

use image::{DynamicImage, GenericImage, Pixel, RgbaImage, Rgba};

use super::color::{average_color, QuadRgba};
use crate::{Tile, TileSet};

pub fn fill_rect<T>(img: &mut T, color: &T::Pixel, rect: &(u32, u32, u32, u32))
where
    T: GenericImage,
{
    let (x, y, width, height) = *rect;
    for y2 in y..(y + height) {
        for x2 in x..(x + width) {
            let mut pixel = img.get_pixel(x2, y2);
            pixel.blend(color);
            img.put_pixel(x2, y2, pixel);
        }
    }
}

fn read_dir(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        paths.push(path);
    }
    Ok(paths)
}

pub fn read_images_in_dir(path: &Path) -> Vec<(PathBuf, RgbaImage)> {
    let mut images = vec![];
    for path_buf in read_dir(path).unwrap() {
        let path = path_buf.as_path();
        let img = match image::open(path) {
            Ok(im) => im,
            _ => continue,
        };
        let img = match img {
            DynamicImage::ImageRgba8(im) => im as RgbaImage,
            DynamicImage::ImageRgb8(_) => img.to_rgba(),
            _ => continue,
        };
        images.push((path_buf, img));
    }
    images
}

pub fn analyse(images: Vec<(PathBuf, RgbaImage)>) -> TileSet<Rgba<u8>> {
    let (tx, rx) = channel();
    let mut handles = vec![];
    for chunk in images.chunks(500) {
        let tx = tx.clone();
        let owned_chuck = chunk.to_owned();
        let handle = thread::spawn(move || {
            for (path_buf, img) in owned_chuck {
                let colors = average_color(&img, &(0, 0, img.width(), img.height()));
                tx.send((path_buf, colors)).unwrap();
            }
        });
        handles.push(handle);
    }
    let num_images = images.len();
    for handle in handles {
        handle.join().unwrap();
    }
    let mut tile_set = TileSet::new();
    for (count, (path_buf, colors)) in rx.iter().enumerate() {
        let tile = Tile::new(path_buf, colors);
        tile_set.push(tile);
        if count == num_images - 1 {
            break;
        }
    }
    tile_set
}

pub fn quad_analyse(images: Vec<(PathBuf, RgbaImage)>) -> TileSet<QuadRgba> {
    let (tx, rx) = channel();
    let mut handles = vec![];
    for chunk in images.chunks(500) {
        let tx = tx.clone();
        let owned_chuck = chunk.to_owned();
        let handle = thread::spawn(move || {
            for (path_buf, img) in owned_chuck {
                let half_width = (f64::from(img.width()) * 0.5).floor() as u32;
                let half_height = (f64::from(img.height()) * 0.5).floor() as u32;

                let rect_top_left = (0u32, 0u32, half_width, half_height);
                let rect_top_right = (half_width, 0u32, half_width, half_height);
                let rect_bottom_right = (half_width, half_height, half_width, half_height);
                let rect_bottom_left = (0u32, half_height, half_width, half_height);

                let top_left = average_color(&img, &rect_top_left);
                let top_right = average_color(&img, &rect_top_right);
                let bottom_right = average_color(&img, &rect_bottom_right);
                let bottom_left = average_color(&img, &rect_bottom_left);

                let colors: QuadRgba = [top_left, top_right, bottom_right, bottom_left];

                tx.send((path_buf, colors)).unwrap();
            }
        });
        handles.push(handle);
    }
    let num_images = images.len();
    for handle in handles {
        handle.join().unwrap();
    }
    let mut tile_set = TileSet::new();
    for (count, (path_buf, colors)) in rx.iter().enumerate() {
        let tile = Tile::new(path_buf, colors);
        tile_set.push(tile);
        if count == num_images - 1 {
            break;
        }
    }
    tile_set
}
