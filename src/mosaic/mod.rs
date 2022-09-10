pub mod color;
pub mod image;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ::image::imageops;
use ::image::{FilterType, Rgb, RgbImage};
use rand::prelude::*;

use crate::{mosaic::color::compare_color, mosaic::image::fill_rect};

pub struct Tile<T> {
    path_buf: PathBuf,
    colors: T,
}

impl<T> Tile<T> {
    pub fn new(path_buf: PathBuf, colors: T) -> Tile<T> {
        Tile { path_buf, colors }
    }

    pub fn path(&self) -> &Path {
        self.path_buf.as_path()
    }
}

impl Tile<[Rgb<u8>; 4]> {
    fn compare_top_left(&self, color: Rgb<u8>) -> f64 {
        compare_color(self.colors[0].0, color.0)
    }

    fn compare_top_right(&self, color: Rgb<u8>) -> f64 {
        compare_color(self.colors[1].0, color.0)
    }

    fn compare_bottom_right(&self, color: Rgb<u8>) -> f64 {
        compare_color(self.colors[2].0, color.0)
    }

    fn compare_bottom_left(&self, color: Rgb<u8>) -> f64 {
        compare_color(self.colors[3].0, color.0)
    }
}

impl Tile<Rgb<u8>> {
    fn compare(&self, color: Rgb<u8>) -> f64 {
        compare_color(self.colors.0, color.0)
    }
}

pub struct TileSet<T> {
    tiles: Vec<Tile<T>>,
}

impl<T> TileSet<T> {
    pub fn new() -> TileSet<T> {
        TileSet::<T> { tiles: vec![] }
    }

    pub fn push(&mut self, tile: Tile<T>) {
        self.tiles.push(tile);
    }

    pub fn random_tile(&self) -> &Tile<T> {
        let mut rng = thread_rng();
        let i = rng.gen_range(0, self.tiles.len());
        &self.tiles[i]
    }
}

trait NearestTile<T> {
    fn nearest_tile(&self, colors: &T) -> &Tile<T>;
}

impl NearestTile<[Rgb<u8>; 4]> for TileSet<[Rgb<u8>; 4]> {
    fn nearest_tile(&self, colors: &[Rgb<u8>; 4]) -> &Tile<[Rgb<u8>; 4]> {
        let mut d = std::f64::MAX;
        let mut t = &self.tiles[0];

        for tile in &self.tiles {
            let [top_left, top_right, bottom_right, bottom_left] = colors;

            let tl2 = tile.compare_top_left(*top_left);
            let tr2 = tile.compare_top_right(*top_right);
            let br2 = tile.compare_bottom_right(*bottom_right);
            let bl2 = tile.compare_bottom_left(*bottom_left);

            let d2 = tl2 + tr2 + br2 + bl2;

            if d2 < d {
                d = d2;
                t = tile;
            }
        }
        t
    }
}

impl NearestTile<Rgb<u8>> for TileSet<Rgb<u8>> {
    fn nearest_tile(&self, colors: &Rgb<u8>) -> &Tile<Rgb<u8>> {
        let mut d = std::f64::MAX;
        let mut t = &self.tiles[0];
        for tile in &self.tiles {
            let d2 = tile.compare(*colors);
            if d2 < d {
                d = d2;
                t = tile;
            }
        }
        t
    }
}

pub fn render_1to1(
    source_img: &RgbImage,
    tile_set: &TileSet<Rgb<u8>>,
    tile_size: u32,
    tint_opacity: f64,
) -> RgbImage {
    let mut output = RgbImage::new(
        source_img.width() * tile_size,
        source_img.height() * tile_size,
    );

    // Cache mapping file path to resized tile image
    let mut resize_cache = HashMap::new();

    for y in 0..source_img.height() {
        for x in 0..source_img.width() {
            let mut colors = *source_img.get_pixel(x, y);

            let tile = tile_set.nearest_tile(&colors);

            // Calculate tile coordinates in output image
            let tile_x = x * tile_size;
            let tile_y = y * tile_size;

            let path = tile.path();
            match resize_cache.get(path) {
                Some(tile_img) => {
                    imageops::overlay(&mut output, tile_img, tile_x, tile_y);
                }
                _ => {
                    let tile_img = ::image::open(path).unwrap().to_rgb();
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
            colors[3] = (255_f64 * tint_opacity).round() as u8;
            fill_rect(
                &mut output,
                &colors,
                &(tile_x, tile_y, tile_size, tile_size),
            );
        }
    }
    output
}

pub fn render_4to1(
    source_img: &RgbImage,
    tile_set: &TileSet<[Rgb<u8>; 4]>,
    tile_size: u32,
    tint_opacity: f64,
) -> RgbImage {
    let tile_size_halved = tile_size / 2;

    let mut output = RgbImage::new(
        source_img.width() * tile_size / 2,
        source_img.height() * tile_size / 2,
    );

    // Cache mapping file path to resized tile image
    let mut resize_cache = HashMap::new();

    for y in (0..source_img.height()).step_by(2) {
        for x in (0..source_img.width()).step_by(2) {
            let mut colors: [Rgb<u8>; 4] = [
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
                    let tile_img = ::image::open(path).unwrap().to_rgb();
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
    source_img: &RgbImage,
    tile_set: &TileSet<()>,
    tile_size: u32,
    tint_opacity: f64,
) -> RgbImage {
    let mut output = RgbImage::new(
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
                    let tile_img = ::image::open(path).unwrap().to_rgb();
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
