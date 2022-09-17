pub mod color;
pub mod image;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ::image::imageops;
use ::image::{FilterType, RgbImage};
use color::SerializableRgb;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use self::color::IntoSerializableRgb;

#[derive(Serialize, Deserialize, Clone)]
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

impl kd_tree::KdPoint for Tile<[SerializableRgb; 4]> {
    type Scalar = f64;
    type Dim = typenum::U12;
    fn at(&self, k: usize) -> f64 {
        if k < 3 {
            let color = <[u8; 3]>::from(self.colors[0]);
            return f64::from(color[k]);
        }
        if k < 6 {
            let color = <[u8; 3]>::from(self.colors[1]);
            return f64::from(color[k - 3]);
        }
        if k < 9 {
            let color = <[u8; 3]>::from(self.colors[2]);
            return f64::from(color[k - 6]);
        }
        let color = <[u8; 3]>::from(self.colors[3]);
        f64::from(color[k - 9])
    }
}

impl kd_tree::KdPoint for Tile<SerializableRgb> {
    type Scalar = f64;
    type Dim = typenum::U3;
    fn at(&self, k: usize) -> f64 {
        if k == 0 {
            return f64::from(self.colors.red());
        }
        if k == 1 {
            return f64::from(self.colors.green());
        }
        f64::from(self.colors.blue())
    }
}

#[derive(Serialize, Deserialize, Clone)]
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

pub fn render_1to1(
    source_img: &RgbImage,
    tile_set: &TileSet<SerializableRgb>,
    tile_size: u32,
) -> RgbImage {
    let mut output = RgbImage::new(
        source_img.width() * tile_size,
        source_img.height() * tile_size,
    );

    // Cache mapping file path to resized tile image
    let mut resize_cache = HashMap::new();

    // Create kd-tree
    let kdtree: kd_tree::KdTree<Tile<SerializableRgb>> =
        kd_tree::KdTree::build_by_ordered_float(tile_set.tiles.clone());

    for y in 0..source_img.height() {
        for x in 0..source_img.width() {
            let color = *source_img.get_pixel(x, y);

            let tile = kdtree
                .nearest(&<[f64; 3]>::from(color.into_serializable_rgb()))
                .unwrap()
                .item;

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
        }
    }
    output
}

pub fn render_4to1(
    source_img: &RgbImage,
    tile_set: &TileSet<[SerializableRgb; 4]>,
    tile_size: u32,
) -> RgbImage {
    let tile_size_halved = tile_size / 2;

    let mut output = RgbImage::new(
        source_img.width() * tile_size_halved,
        source_img.height() * tile_size_halved,
    );

    // Cache mapping file path to resized tile image
    let mut resize_cache = HashMap::new();

    // Create kd-tree
    let kdtree: kd_tree::KdTree<Tile<[SerializableRgb; 4]>> =
        kd_tree::KdTree::build_by_ordered_float(tile_set.tiles.clone());

    for y in (0..source_img.height()).step_by(2) {
        for x in (0..source_img.width()).step_by(2) {
            let colors: [SerializableRgb; 4] = [
                (*source_img.get_pixel(x, y)).into_serializable_rgb(),
                (*source_img.get_pixel(x + 1, y)).into_serializable_rgb(),
                (*source_img.get_pixel(x + 1, y + 1)).into_serializable_rgb(),
                (*source_img.get_pixel(x, y + 1)).into_serializable_rgb(),
            ];

            let tile = kdtree
                .nearest(&[
                    f64::from(colors[0].red()),
                    f64::from(colors[0].green()),
                    f64::from(colors[0].blue()),
                    f64::from(colors[1].red()),
                    f64::from(colors[1].green()),
                    f64::from(colors[1].blue()),
                    f64::from(colors[2].red()),
                    f64::from(colors[2].green()),
                    f64::from(colors[2].blue()),
                    f64::from(colors[3].red()),
                    f64::from(colors[3].green()),
                    f64::from(colors[3].blue()),
                ])
                .unwrap()
                .item;

            // Calculate tile coordinates in output image
            let tile_x = x / 2 * tile_size;
            let tile_y = y / 2 * tile_size;

            let path = tile.path().to_owned();
            match resize_cache.get(&path) {
                Some(tile_img) => {
                    imageops::overlay(&mut output, tile_img, tile_x, tile_y);
                }
                _ => {
                    let tile_img = ::image::open(&path).unwrap().to_rgb();
                    let tile_img =
                        imageops::resize(&tile_img, tile_size, tile_size, FilterType::Lanczos3);
                    imageops::overlay(&mut output, &tile_img, tile_x, tile_y);
                    resize_cache.insert(path, tile_img);
                }
            };
        }
    }
    output
}

pub fn render_random(source_img: &RgbImage, tile_set: &TileSet<()>, tile_size: u32) -> RgbImage {
    let mut output = RgbImage::new(
        source_img.width() * tile_size,
        source_img.height() * tile_size,
    );

    // Cache mapping file path to resized tile image
    let mut resize_cache = HashMap::new();

    for tile_y in 0..source_img.height() {
        for tile_x in 0..source_img.width() {
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
        }
    }
    output
}
