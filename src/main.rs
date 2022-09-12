mod mosaic;

use std::fs;
use std::path::{Path, PathBuf};

use clap::{App, Arg};
use image::{imageops, DynamicImage, ImageFormat, RgbImage, Rgba, RgbaImage};

use mosaic::{
    image::{analyse_1to1, analyse_4to1, read_images_in_dir},
    render_1to1, render_4to1, render_random, Tile, TileSet,
};
use serde::Serialize;

fn generate_tile_set<T: Serialize>(
    tiles_path: &Path,
    cache_path: &Path,
    analyse: fn(Vec<(PathBuf, RgbImage)>) -> TileSet<T>,
) -> TileSet<T> {
    let images = read_images_in_dir(tiles_path);
    let tile_set = analyse(images);
    let encoded_tile_set = bincode::serialize(&tile_set).unwrap();
    fs::write(&cache_path, encoded_tile_set).unwrap();
    tile_set
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
            .default_value("1to1"))
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

    // Open the source image
    let img_path = Path::new(img);
    let img = match image::open(img_path) {
        Ok(img) => img.to_rgb(),
        Err(e) => {
            eprintln!("Failed to open source image: {}", e);
            std::process::exit(1);
        }
    };

    // Validate the source image dimensions when mode = 4to1
    if mode == "4to1" && (img.width() % 2 != 0 || img.height() % 2 != 0) {
        eprintln!("Invalid source dimensions ({}x{}): Dimensions must be divisible by 2 when mode is 4to1", img.width(), img.height());
        std::process::exit(1);
    }

    // Read all images in tiles directory
    let tiles_path = Path::new(tiles_dir_path);

    let output = match mode {
        "1to1" => {
            let analysis_cache_path = tiles_path.join("_emosaic_1to1");
            let tile_set = match fs::read(&analysis_cache_path) {
                Ok(bytes) => bincode::deserialize(&bytes).unwrap(),
                _ => generate_tile_set(tiles_path, &analysis_cache_path, analyse_1to1),
            };
            render_1to1(&img, &tile_set, tile_size)
        }
        "4to1" => {
            let analysis_cache_path = tiles_path.join("_emosaic_4to1");
            let tile_set = match fs::read(&analysis_cache_path) {
                Ok(bytes) => bincode::deserialize(&bytes).unwrap(),
                _ => generate_tile_set(tiles_path, &analysis_cache_path, analyse_4to1),
            };
            render_4to1(&img, &tile_set, tile_size)
        }
        "random" => {
            let images = read_images_in_dir(tiles_path);
            let mut tile_set = TileSet::<()>::new();
            for (path_buf, _) in images {
                let tile = Tile::<()>::new(path_buf, ());
                tile_set.push(tile);
            }
            render_random(&img, &tile_set, tile_size)
        }
        _ => {
            eprintln!("Invalid value for 'mode': Value must be 1to1, 4to1 or random");
            std::process::exit(1);
        }
    };

    if tint_opacity > 0.0 {
        let mut overlay = RgbaImage::new(img.width(), img.height());
        for x in 0..img.width() {
            for y in 0..img.height() {
                let p = img.get_pixel(x, y);
                let p2: Rgba<u8> = Rgba([p[0], p[1], p[2], (255_f64 * tint_opacity) as u8]);
                overlay.put_pixel(x, y, p2);
            }
        }
        // Scale up to match the output size
        let overlay = imageops::resize(
            &overlay,
            output.width(),
            output.height(),
            image::FilterType::Nearest,
        );
        // Apply overlay
        let mut output2 = DynamicImage::ImageRgb8(output.clone()).to_rgba();
        imageops::overlay(&mut output2, &overlay, 0, 0);
        output2
            .save_with_format(output_path, ImageFormat::PNG)
            .unwrap();
        return;
    }

    output
        .save_with_format(output_path, ImageFormat::PNG)
        .unwrap();
}
