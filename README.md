# LZMA Tarball Documentation

This library provides functionalities to compress directories and files into `.tar.xz` format and decompress `.tar.xz` files using configurable options.

## Examples

See the [examples](https://github.com/Drew-Chase/lzma_tarball/tree/master/examples) directory for more detailed examples.

## Creating an `LZMATarballWriter` Instance for Compression

You can create an `LZMATarballWriter` instance by specifying the input directory (or file) and the output file path.

```rust
use lzma_tarball::writer::LZMATarballWriter;

let input_path = "./";
let output_file = "../test/test.tar.xz";

let options = LZMATarballWriter::new(input_path, output_file).unwrap();
```

## Configuring Options for Compression

You can configure the compression level and buffer size using the provided methods.

```rust
let options = LZMATarballWriter::new(input_path, output_file)
    .unwrap()
    .with_compression_level(6)
    .with_buffer_size(64);
```

## Compression

To compress data, call the `compress` method and pass a callback function to report progress.

```rust
let result = options.compress(|progress| {
    let percentage = progress.percentage * 100f32;
    let processed = progress.bytes_processed;
    let bps = progress.bytes_per_second;
    let mbps = (bps as f32) / 1024f32 / 1024f32;

    print!("\x1b[1A"); // Move cursor up
    println!("Progress: {:.2}% - Processed: {}B - Speed: {:.2}Mb/s", percentage, processed, mbps);
}).unwrap();

assert!(result.is_ok());
```

## Full Example for Compression

This example demonstrates how to create an `LZMATarballWriter` instance, configure it, and use it to compress data while reporting progress.

```rust
use lzma_tarball::writer::LZMATarballWriter;

fn main() {
    let input_path = "./";
    let output_file = "../test/test.tar.xz";

    let result = LZMATarballWriter::new(input_path, output_file)
        .unwrap()
        .with_compression_level(6)
        .with_buffer_size(64)
        .compress(|progress| {
            let percentage = progress.percentage * 100f32;
            let processed = progress.bytes_processed;
            let bps = progress.bytes_per_second;
            let mbps = (bps as f32) / 1024f32 / 1024f32;

            print!("\x1b[1A"); // Move cursor up
            println!("Progress: {:.2}% - Processed: {}B - Speed: {:.2}Mb/s", percentage, processed, mbps);
        }).unwrap();

    let duration = result.elapsed_time;
    let size = result.size;
    let original_size = result.original_size;

    println!("Compression complete! Elapsed time: {:?}", duration);
    println!("Original size: {}B - Compressed size: {}B", original_size, size);
}
```

## Handling Errors during Compression

The `compress` method returns a `Result` which you can use to handle potential errors.

```rust
let result = options.compress(|progress| {
    let percentage = progress.percentage * 100f32;
    let processed = progress.bytes_processed;
    let bps = progress.bytes_per_second;
    let mbps = (bps as f32) / 1024f32 / 1024f32;

    print!("\x1b[1A"); // Move cursor up
    println!("Progress: {:.2}% - Processed: {}B - Speed: {:.2}Mb/s", percentage, processed, mbps);
});

match result {
    Ok(success) => println!("Compression successful: {:?}", success),
    Err(e) => eprintln!("Compression failed: {:?}", e),
}
```

## Creating an `LZMATarballReader` Instance for Decompression

You can create an `LZMATarballReader` instance by specifying the tarball file path.

```rust
use lzma_tarball::reader::LZMATarballReader;

let tar_file = "path/to/input.tar.xz";

let reader = LZMATarballReader::new(tar_file).unwrap();
```

## Configuring Options for Decompression

You can configure various options to control the decompression process.

```rust
let reader = LZMATarballReader::new(tar_file)
    .unwrap()
    .set_overwrite(true)
    .set_mask(0o755)
    .set_ignore_zeros(false)
    .set_preserve_mtime(true)
    .set_preserve_ownerships(true)
    .set_preserve_permissions(true);
```

## Decompression

To decompress data, use the `decompress` method and specify the output directory.

```rust
let output_dir = "./output_directory";
let result = reader.decompress(output_dir).unwrap();

println!("Decompression successful!");
println!("Elapsed time: {:?}", result.elapsed_time);
println!("Files decompressed: {:?}", result.files);
println!("Total size: {} bytes", result.total_size);
```

## Full Example for Decompression

This example demonstrates how to create an `LZMATarballReader` instance, configure it, and use it to decompress data.

```rust
use lzma_tarball::reader::LZMATarballReader;

fn main() {
    let tar_file = "path/to/input.tar.xz";
    let output_dir = "./output_directory";

    let reader = LZMATarballReader::new(tar_file)
        .unwrap()
        .set_overwrite(true)
        .set_mask(0o755)
        .set_ignore_zeros(false)
        .set_preserve_mtime(true)
        .set_preserve_ownerships(true)
        .set_preserve_permissions(true);

    let result = reader.decompress(output_dir).unwrap();

    println!("Decompression successful!");
    println!("Elapsed time: {:?}", result.elapsed_time);
    println!("Files decompressed: {:?}", result.files);
    println!("Total size: {} bytes", result.total_size);
}
```

## Handling Errors during Decompression

The `decompress` method returns a `Result` which you can use to handle potential errors.

```rust
let result = reader.decompress(output_dir);

match result {
    Ok(success) => {
        println!("Decompression successful!");
        println!("Elapsed time: {:?}", success.elapsed_time);
        println!("Files decompressed: {:?}", success.files);
        println!("Total size: {} bytes", success.total_size);
    },
    Err(e) => eprintln!("Decompression failed: {:?}", e),
}
```

## Struct Definitions

For reference, here are the definitions of the main structs used in this library.

### Writer

```rust
#[derive(Debug, Clone)]
pub struct LZMATarballWriter {
    pub compression_level: u8,
    pub buffer_size: u16,
    pub output_file: PathBuf,
    pub tar_file: PathBuf,
    pub input_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct LZMAResult {
    pub output_file: PathBuf,
    pub size: u64,
    pub original_size: u64,
    pub elapsed_time: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct LZMACallbackResult {
    pub bytes_processed: u64,
    pub bytes_per_second: u64,
    pub percentage: f32,
}
```

### Reader

```rust
#[derive(Debug, Clone)]
pub struct LZMATarballReader {
    pub tar_file: PathBuf,
    pub overwrite: bool,
    pub mask: u32,
    pub ignore_zeros: bool,
    pub preserve_mtime: bool,
    pub preserve_ownerships: bool,
    pub preserve_permissions: bool,
    pub unpack_xattrs: bool,
}

#[derive(Debug, Clone)]
pub struct DecompressionResult {
    pub elapsed_time: std::time::Duration,
    pub files: Vec<String>,
    pub total_size: u64,
}
```

## Methods

Here are some of the key methods available for compression and decompression:

### Writer

- `new`: Creates a new instance of `LZMATarballWriter`.
- `with_compression_level`: Sets the desired compression level (0-9).
- `with_buffer_size`: Specifies the buffer size to use (in KB).
- `compress`: Compresses the data and calls the provided callback function with progress updates.

### Reader

- `new`: Creates a new instance of `LZMATarballReader`.
- `set_overwrite`: Sets whether to overwrite existing files.
- `set_mask`: Sets the file permissions mask.
- `set_ignore_zeros`: Sets whether to ignore zero blocks.
- `set_preserve_mtime`: Sets whether to preserve modification times.
- `set_preserve_ownerships`: Sets whether to preserve file ownerships.
- `set_preserve_permissions`: Sets whether to preserve file permissions.
- `decompress`: Decompresses the data to the specified output directory.