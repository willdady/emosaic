# emosaic

Mosaic generator written in [Rust!](https://www.rust-lang.org/)

![](example/warhol.png?raw=true)

## Building

To build [make sure you have rust installed](https://www.rust-lang.org/tools/install).

```
cargo build --release
```

Once compiled, the binary can be found at `target/release/emosaic` in the repository root.

## Usage

The command expects a path to a directory containing square 'tile' images and a source image.

```
emosaic /path/to/tile/images/ source.png
```

### Modes

The strategy used to generate the mosaic is controlled by the `-m, --mode` option.

#### 1to1 (Default)

For each pixel in the source image a tile with the nearest matching average color will be emitted.

Assuming a source image with dimensions 100x100 and default tile size of 16 the output image will be 1600x1600.

#### 4to1

For every 2x2 pixels one tile will be emitted. Tiles are divided into 2x2 segments and the average colour of each segment is stored. The tile with the nearest average color in _each_ segment to the target pixels will be chosen. This mode may provide smoother transitions between tile images and works best if you have a large tile set.

Assuming a source image with dimensions 100x100 and default tile size of 16 the output image will be 800x800.

#### random

The source image is not analysed and tiles are simply randomized in the output. This mode is best combined with the `-t, --tint-opacity` option to overlay the source image on top of the output. If your source image only contains a few colors (like a logo) this is the mode you want.

Assuming a source image with dimensions 100x100 and default tile size of 16 the output image will be 1600x1600.

### Output path

By default the resulting image will be output to the current directory as `output.png`. You can specify the output file with the `-o, --output` option e.g.

```
emosaic /path/to/tile/images/ source.png -o /foo/bar/myimage.png
```

Output format is _always_ PNG.

### Controlling tile size

Each 'tile' in the output image will be 16x16 by default. Provide a custom size with the `-s, --tile-size` option. Note the size of your source image and tile size dictate the final size of your output image. For example, if your source image is 100x200 and you specify a tile size of 32 with the default mode _1to1_ the output image will be 3200x6400! So be careful!

```
emosaic /path/to/tile/images/ source.png -s 32
```

### Tinting

Use the tinting option, `-t, --tint-opacity`, to control the transparency of the source image overlayed on top of the the output mosaic. This can be useful to push the overall color of each tile closer to the color(s) it was sampled from in the source image. Value must be between 0 and 1. Default is 0.

```
emosaic /path/to/tile/images/ source.png -t 0.5
```
