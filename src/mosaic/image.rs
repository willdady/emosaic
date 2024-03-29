use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::thread;

use image::{DynamicImage, RgbImage};

use super::color::{average_color, IntoSerializableRgb, SerializableRgb};
use crate::{Tile, TileSet};

fn read_dir(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        paths.push(path);
    }
    Ok(paths)
}

pub fn read_images_in_dir(path: &Path) -> Vec<(PathBuf, RgbImage)> {
    let mut images = vec![];
    for path_buf in read_dir(path).unwrap() {
        let path = path_buf.as_path();
        let img = match image::open(path) {
            Ok(im) => im,
            _ => continue,
        };
        let img: RgbImage = match img {
            DynamicImage::ImageRgba8(_) => match img.as_rgb8() {
                Some(x) => x.to_owned(),
                _ => continue,
            },
            DynamicImage::ImageRgb8(im) => im,
            _ => continue,
        };
        images.push((path_buf, img));
    }
    images
}

pub fn analyse_1to1(images: Vec<(PathBuf, RgbImage)>) -> TileSet<SerializableRgb> {
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
        let tile = Tile::new(path_buf, colors.into_serializable_rgb());
        tile_set.push(tile);
        if count == num_images - 1 {
            break;
        }
    }
    tile_set
}

pub fn analyse_4to1(images: Vec<(PathBuf, RgbImage)>) -> TileSet<[SerializableRgb; 4]> {
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

                let colors: [SerializableRgb; 4] = [
                    top_left.into_serializable_rgb(),
                    top_right.into_serializable_rgb(),
                    bottom_right.into_serializable_rgb(),
                    bottom_left.into_serializable_rgb(),
                ];

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
