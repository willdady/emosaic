mod mosaic;

use std::path::Path;

use clap::{App, Arg};
use image::ImageFormat;

use mosaic::{
    image::{analyse, quad_analyse, read_images_in_dir},
    render_1to1, render_4to1, render_random, Tile, TileSet,
};

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
    let tiles_dir = Path::new(tiles_dir_path);
    let images = read_images_in_dir(tiles_dir);

    let output = match mode {
        "1to1" => {
            let tile_set = analyse(images);
            render_1to1(&img, &tile_set, tile_size)
        }
        "4to1" => {
            let tile_set = quad_analyse(images);
            render_4to1(&img, &tile_set, tile_size)
        }
        "random" => {
            let mut tile_set = TileSet::<()>::new();
            for (path_buf, _) in images {
                let tile = Tile::<()>::new(path_buf, ());
                tile_set.push(tile);
            }
            render_random(&img, &tile_set, tile_size, tint_opacity)
        }
        _ => {
            eprintln!("Invalid value for 'mode': Value must be 1to1, 4to1 or random");
            std::process::exit(1);
        }
    };

    output
        .save_with_format(output_path, ImageFormat::PNG)
        .unwrap();
}
