pub mod color;
pub mod image;

use std::collections::HashMap;

use ::image::imageops;
use ::image::{FilterType, RgbaImage};

use crate::{
    mosaic::color::{NilRgba, QuadRgba},
    mosaic::image::fill_rect,
    NearestTile, TileSet,
};

pub fn render_4to1(
    source_img: &RgbaImage,
    tile_set: &TileSet<QuadRgba>,
    tile_size: u32,
    tint_opacity: f64,
) -> RgbaImage {
    let tile_size_halved = tile_size / 2;

    let mut output = RgbaImage::new(
        source_img.width() * tile_size / 2,
        source_img.height() * tile_size / 2,
    );

    // Cache mapping file path to resized tile image
    let mut resize_cache = HashMap::new();

    for y in (0..source_img.height()).step_by(2) {
        for x in (0..source_img.width()).step_by(2) {
            let mut colors: QuadRgba = [
                *source_img.get_pixel(x, y),
                *source_img.get_pixel(x + 1, y),
                *source_img.get_pixel(x + 1, y + 1),
                *source_img.get_pixel(x, y + 1),
            ];

            let tile = tile_set.nearest_tile(&colors);

            // Calculate tile coordinates in output image
            let tile_x = x / 2 * tile_size;
            let tile_y = y / 2 * tile_size;

            let path = tile.path();
            match resize_cache.get(path) {
                Some(tile_img) => {
                    imageops::overlay(&mut output, tile_img, tile_x, tile_y);
                }
                _ => {
                    let tile_img = ::image::open(path).unwrap();
                    let tile_img =
                        imageops::resize(&tile_img, tile_size, tile_size, FilterType::Lanczos3);
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
            fill_rect(
                &mut output,
                &colors[0],
                &(tile_x, tile_y, tile_size_halved, tile_size_halved),
            );
            fill_rect(
                &mut output,
                &colors[1],
                &(
                    tile_x + tile_size_halved,
                    tile_y,
                    tile_size_halved,
                    tile_size_halved,
                ),
            );
            fill_rect(
                &mut output,
                &colors[2],
                &(
                    tile_x + tile_size_halved,
                    tile_y + tile_size_halved,
                    tile_size_halved,
                    tile_size_halved,
                ),
            );
            fill_rect(
                &mut output,
                &colors[3],
                &(
                    tile_x,
                    tile_y + tile_size_halved,
                    tile_size_halved,
                    tile_size_halved,
                ),
            );
        }
    }
    output
}

pub fn render_random(
    source_img: &RgbaImage,
    tile_set: &TileSet<NilRgba>,
    tile_size: u32,
    tint_opacity: f64,
) -> RgbaImage {
    let mut output = RgbaImage::new(
        source_img.width() * tile_size,
        source_img.height() * tile_size,
    );

    // Cache mapping file path to resized tile image
    let mut resize_cache = HashMap::new();

    for tile_y in 0..source_img.height() {
        for tile_x in 0..source_img.width() {
            let mut pixel = *source_img.get_pixel(tile_x, tile_y);

            let tile = tile_set.random_tile();
            let path = tile.path();
            match resize_cache.get(path) {
                Some(tile_img) => {
                    imageops::overlay(
                        &mut output,
                        tile_img,
                        tile_x * tile_size,
                        tile_y * tile_size,
                    );
                }
                _ => {
                    let tile_img = ::image::open(path).unwrap();
                    let tile_img =
                        imageops::resize(&tile_img, tile_size, tile_size, FilterType::Lanczos3);
                    imageops::overlay(
                        &mut output,
                        &tile_img,
                        tile_x * tile_size,
                        tile_y * tile_size,
                    );
                    resize_cache.insert(path, tile_img);
                }
            };
            // Apply tint to each tile in the output tile
            if tint_opacity <= 0.0 {
                continue;
            }
            pixel[3] = (255_f64 * tint_opacity).round() as u8;
            fill_rect(
                &mut output,
                &pixel,
                &(tile_x * tile_size, tile_y * tile_size, tile_size, tile_size),
            );
        }
    }
    output
}
