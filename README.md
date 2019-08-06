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

The command expects a path to a directory containing square 'tile' images and a source image. For each pixel in the source image a tile with the closest average color will be output.

```
emosaic /path/to/tile/images/ source.png
```

### Output path

By default the resulting image will be output to the current directory as `output.png`. You can specify the output file with the `-o` option e.g.

```
emosaic -o /foo/bar/myimage.png /path/to/tile/images/ source.png
```

Output format is _always_ PNG.

### Controlling tile size

Each 'tile' in the output image will be 16x16 by default. Provide a custom size with the `-s` option. Note the size of your source image and tile size dictate the final size of your output image. For example, if your source image is 100x200 and you specify a tile size of 32 the output image will be 3200x6400! So be careful!

```
emosaic -s 32 /path/to/tile/images/ source.png
```
