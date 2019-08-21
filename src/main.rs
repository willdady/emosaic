mod mosaic;

use std::path::{Path, PathBuf};

use clap::{App, Arg};
use image::{ImageFormat, Rgba};
use rand::prelude::*;

use mosaic::{
    color::{compare_color, NilRgba, QuadRgba},
    image::{analyse_images, read_images_in_dir},
    render_4to1, render_random,
};

struct Tile<T> {
    path_buf: PathBuf,
    colors: T,
}

impl<T> Tile<T> {
    fn new(path_buf: PathBuf, colors: T) -> Tile<T> {
        Tile { path_buf, colors }
    }

    fn path(&self) -> &Path {
        self.path_buf.as_path()
    }
}

impl Tile<QuadRgba> {
    fn compare_top_left(&self, color: Rgba<u8>) -> f64 {
        compare_color(self.colors[0], color)
    }

    fn compare_top_right(&self, color: Rgba<u8>) -> f64 {
        compare_color(self.colors[1], color)
    }

    fn compare_bottom_right(&self, color: Rgba<u8>) -> f64 {
        compare_color(self.colors[2], color)
    }

    fn compare_bottom_left(&self, color: Rgba<u8>) -> f64 {
        compare_color(self.colors[3], color)
    }
}

pub struct TileSet<T> {
    tiles: Vec<Tile<T>>,
}

impl<T> TileSet<T> {
    fn new() -> TileSet<T> {
        TileSet::<T> { tiles: vec![] }
    }

    fn push(&mut self, tile: Tile<T>) {
        self.tiles.push(tile);
    }

    fn random_tile(&self) -> &Tile<T> {
        let mut rng = thread_rng();
        let i = rng.gen_range(0, self.tiles.len());
        &self.tiles[i]
    }
}

trait NearestTile<T> {
    fn nearest_tile(&self, colors: &T) -> &Tile<T>;
}

impl NearestTile<QuadRgba> for TileSet<QuadRgba> {
    fn nearest_tile(&self, colors: &QuadRgba) -> &Tile<QuadRgba> {
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

fn main() {
    let matches = App::new("emosaic")
        .version("0.2.0")
        .author("Will Dady <willdady@gmail.com>")
        .about("Mosaic generator")
        .arg(Arg::with_name("tile_size")
            .short("s")
            .long("tile-size")
            .value_name("UINT")
            .help("The size of each tile in the output image")
            .default_value("16"))
        .arg(Arg::with_name("tint_opacity")
            .short("t")
            .long("tint-opacity")
            .value_name("FLOAT")
            .help("Value between 0 and 1 indicating the opacity of the source image overlayed on the output image")
            .default_value("0"))
        .arg(Arg::with_name("output_path")
            .short("o")
            .long("output")
            .value_name("PATH")
            .help("Output image path")
            .default_value("./output.png"))
        .arg(Arg::with_name("mode")
            .short("m")
            .long("MODE")
            .value_name("STRING")
            .help("Mosaic mode to use")
            .default_value("4to1"))
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
    let output_path = matches.value_of("output_path").unwrap();
    let tiles_dir_path = matches.value_of("tiles_dir").unwrap();
    let mode = matches.value_of("mode").unwrap();

    let tile_size = match matches.value_of("tile_size").unwrap().parse::<u32>() {
        Ok(val) => val,
        _ => {
            eprintln!("Invalid value for 'tile-size': Value must be an unsigned integer");
            std::process::exit(1);
        }
    };

    let tint_opacity = match matches.value_of("tint_opacity").unwrap().parse::<f64>() {
        Ok(val) => {
            let val = val.abs();
            if val > 1.0 {
                eprintln!(
                    "Invalid value for 'tint-opacity': Value must be a float between 0 and 1"
                );
                std::process::exit(1);
            }
            val
        }
        _ => {
            eprintln!("Invalid value for 'tint-opacity': Value must be a float between 0 and 1");
            std::process::exit(1);
        }
    };

    // Open source image
    let img_path = Path::new(img);
    let img = match image::open(img_path) {
        Ok(img) => img.to_rgba(),
        Err(e) => {
            eprintln!("Failed to open source image: {}", e);
            std::process::exit(1);
        }
    };
    // Read all images in tiles directory
    let tiles_dir = Path::new(tiles_dir_path);
    let images = read_images_in_dir(tiles_dir);

    let output = match mode {
        "4to1" => {
            let tile_set = analyse_images(images);
            render_4to1(&img, &tile_set, tile_size, tint_opacity)
        }
        "random" => {
            let mut tile_set = TileSet::<NilRgba>::new();
            for (path_buf, _) in images {
                let tile = Tile::<NilRgba>::new(path_buf, []);
                tile_set.push(tile);
            }
            render_random(&img, &tile_set, tile_size, tint_opacity)
        }
        _ => {
            eprintln!("Invalid value for 'mode': Value must be 4to1 or random");
            std::process::exit(1);
        }
    };

    output
        .save_with_format(output_path, ImageFormat::PNG)
        .unwrap();
}
