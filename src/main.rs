extern crate image;
extern crate clap;

use std::io;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::thread;
use std::sync::mpsc::channel;
use std::collections::HashMap;

use clap::{Arg, App};
use image::{DynamicImage, Rgba, RgbaImage, FilterType};
use image::imageops;

struct Tile {
    path_buf: PathBuf,
    rgba: Rgba<u8>
}

impl Tile {
    fn new(path_buf: PathBuf, rgba: Rgba<u8>) -> Tile {
        Tile {
            path_buf,
            rgba
        }
    }

    fn dist(&self, rgba: &Rgba<u8>) -> f64 {
        let r1 = self.rgba[0] as f64;
        let g1 = self.rgba[1] as f64;
        let b1 = self.rgba[2] as f64;
        let r2 = rgba[0] as f64;
        let g2 = rgba[1] as f64;
        let b2 = rgba[2] as f64;
        ((r2 - r1) * 0.3).powi(2) + ((g2 - g1) * 0.59).powi(2) + ((b2 - b1) * 0.11).powi(2)
    }

    fn path(&self) -> &Path {
        self.path_buf.as_path()
    }
}

struct TileSet {
    tiles: Vec<Tile>
}

impl TileSet {
    fn new() -> TileSet {
        TileSet {
            tiles: vec!()
        }
    }

    fn push(&mut self, tile: Tile) {
        self.tiles.push(tile);
    }

    fn closest_tile(&self, rgba: &Rgba<u8>) -> &Tile {
        let mut d = std::f64::MAX;
        let mut t = &self.tiles[0];
        for tile in &self.tiles {
            let d2 = tile.dist(rgba);
            if d2 < d {
                d = d2;
                t = tile;
            }
        }
        t
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

fn get_average_color(img: RgbaImage) -> Option<Rgba<u8>> {
    let mut r = 0.0;
    let mut g = 0.0;
    let mut b = 0.0;
    let mut a = 0.0;
    let mut count = 0.0;
    let total_pixels = (img.width() * img.height()) as f64;
    let mut transparent_pixel_count = 0.0;
    for pixel in img.pixels() {
        // If more than 50% of pixels have a 0% alpha return None
        if pixel[3] == 0 {
            transparent_pixel_count += 1.0;
            if (transparent_pixel_count / total_pixels) > 0.5 {
                return None
            }
        }

        r += pixel[0] as f64;
        g += pixel[1] as f64;
        b += pixel[2] as f64;
        a += pixel[3] as f64;
        count += 1.0;
    }
    let r = (r / count) as u8;
    let g = (g / count) as u8;
    let b = (b / count) as u8;
    let a = (a / count) as u8;
    Some(Rgba([r, g, b, a]))
}

fn read_images_in_dir(path: &Path) -> Vec<(PathBuf, RgbaImage)> {
    let mut images = vec!();
    for path_buf in read_dir(path).unwrap() {
        let path = path_buf.as_path();
        let img = match image::open(path) {
            Ok(im) => im,
            _ => continue
        };
        let img = match img {
            DynamicImage::ImageRgba8(im) => im as RgbaImage,
            _ => continue
        };
        images.push((path_buf, img));
    };
    images
}

fn analyse_images(images: Vec<(PathBuf, RgbaImage)>) -> TileSet {
    let (tx, rx) = channel();
    let mut handles = vec!();
    for chunk in images.chunks(500) {
        let tx = tx.clone();
        let owned_chuck = chunk.to_owned();
        let handle = thread::spawn(move || {
            for (path_buf, img) in owned_chuck {
                let rgba = get_average_color(img);
                tx.send((path_buf, rgba)).unwrap();
            }
        });
        handles.push(handle);
    }
    let num_images = images.len();
    for handle in handles {
        handle.join().unwrap();
    }
    let mut tile_set = TileSet::new();
    let mut count: usize = 0;
    for (path_buf, rgba) in rx {
        match rgba {
            Some(rgba) => {
                let tile = Tile::new(path_buf, rgba);
                tile_set.push(tile);
            },
            _ => ()
        }
        count += 1;
        if count == num_images {
            break;
        }
    }
    tile_set
}

fn main() {
    let matches = App::new("emosaic")
        .version("0.1.0")
        .author("Will Dady <willdady@gmail.com>")
        .about("Mosaic generator")
        .arg(Arg::with_name("tile_size")
            .short("s")
            .long("tile-size")
            .value_name("UINT")
            .help("The size of each tile in the output image")
            .default_value("16"))
        .arg(Arg::with_name("tiles_dir")
            .value_name("DIR")
            .help("Directory containing tile images")
            .index(1)
            .required(true))
        .arg(Arg::with_name("IMG")
            .help("Input image")
            .index(2)
            .required(true))
        .get_matches();

    let img = matches.value_of("IMG").unwrap();
    let img_path = Path::new(img);

    let tiles_dir_path = matches.value_of("tiles_dir").unwrap();
    let tiles_dir = Path::new(tiles_dir_path);

    let images = read_images_in_dir(tiles_dir);
    let tile_set = analyse_images(images);

    // TODO: Show better error if parse fails
    let tile_size: u32 = matches.value_of("tile_size").unwrap().parse().unwrap();

    let img = image::open(img_path).unwrap();
    let img = img.to_rgba();
    let mut output = image::RgbaImage::new(&img.width() * tile_size, &img.height() * tile_size);

    let mut cache = HashMap::new();

    for (x, y, rgba) in img.enumerate_pixels() {
        let tile = tile_set.closest_tile(rgba);
        let path = tile.path();

        if cache.contains_key(path) {
            let tile_img = cache.get(path).unwrap();
            imageops::overlay(&mut output, tile_img, x * tile_size, y * tile_size);
            continue;
        }

        let tile_img = image::open(path).unwrap();
        let tile_img = imageops::resize(&tile_img, tile_size, tile_size, FilterType::Lanczos3);
        imageops::overlay(&mut output, &tile_img, x * tile_size, y * tile_size);
        cache.insert(path, tile_img);
    }

    output.save("./output.png").unwrap();
}
