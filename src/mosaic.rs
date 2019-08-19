use std::collections::HashMap;

use image::{RgbaImage, FilterType};
use image::imageops;

use crate::imageutils::{fill_rect};
use crate::{QuadRgba, TileSet};


pub fn render(source_img: &RgbaImage, tile_set: &TileSet, tile_size: u32, tint_opacity: f64) -> RgbaImage {
    let tile_size_halved = tile_size / 2;

    let mut output = RgbaImage::new(source_img.width() * tile_size / 2, source_img.height() * tile_size / 2);

    // Cache mapping file path to resized tile image
    let mut resize_cache = HashMap::new();

    for y in (0..source_img.height()).step_by(2) {
        for x in (0..source_img.width()).step_by(2) {
            let mut colors: QuadRgba = [
                *source_img.get_pixel(x, y),
                *source_img.get_pixel(x + 1, y),
                *source_img.get_pixel(x + 1, y + 1),
                *source_img.get_pixel(x, y + 1)
            ];

            let tile = tile_set.closest_tile(&colors);

            // Calculate tile coordinates in output image
            let tile_x = x / 2 * tile_size;
            let tile_y = y / 2 * tile_size;

            let path = tile.path();
            match resize_cache.get(path) {
                Some(tile_img) => {
                    imageops::overlay(&mut output, tile_img, tile_x, tile_y);
                },
                _ => {
                    let tile_img = image::open(path).unwrap();
                    let tile_img = imageops::resize(&tile_img, tile_size, tile_size, FilterType::Lanczos3);
                    imageops::overlay(&mut output, &tile_img, tile_x, tile_y);
                    resize_cache.insert(path, tile_img);
                }
            };
            // Apply tint to each quadrant of the output tile
            if tint_opacity <= 0.0 {
                continue;
            }
            colors[0][3] = (255_f64 * tint_opacity).round() as u8;
            colors[1][3] = (255_f64 * tint_opacity).round() as u8;
            colors[2][3] = (255_f64 * tint_opacity).round() as u8;
            colors[3][3] = (255_f64 * tint_opacity).round() as u8;
            fill_rect(&mut output, &colors[0], &(tile_x, tile_y, tile_size_halved, tile_size_halved));
            fill_rect(&mut output, &colors[1], &(tile_x + tile_size_halved, tile_y, tile_size_halved, tile_size_halved));
            fill_rect(&mut output, &colors[2], &(tile_x + tile_size_halved, tile_y + tile_size_halved, tile_size_halved, tile_size_halved));
            fill_rect(&mut output, &colors[3], &(tile_x, tile_y + tile_size_halved, tile_size_halved, tile_size_halved));
        }
    }
    output
}
