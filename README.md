# emosaic

Mosaic generator written in Rust!

## Building

```
cargo build --release
```

Once compiled, binary can be found at `target/release/emosaic` in the repository root.

## Usage

```
emosaic /path/to/tile/images/ source.png
```

You may optionally specify the size of each tile in the output image, where each pixel of the source image will output a tile of the given size in the output image. For example, if your source image is 100x200 and you specify a tile size of 32 the output image will be 3200x6400! The default tile size is 16.

```
emosaic -s 32 /path/to/tile/images/ source.png
```
