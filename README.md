# LZMA Tarball Documentation

This library provides functionalities to compress directories and files into `.tar.xz` format using configurable options.

## Creating an `LZMATarball` Instance

You can create an `LZMATarball` instance by specifying the input directory (or file) and the output file path.

```rust
use lzma_tarball::LZMATarball;

let options = LZMATarball::new("./input_directory_or_file", "output.tar.xz");
```

## Configuring Options

You can configure the compression level and buffer size using the provided methods.

```rust
let options = LZMATarball::new("./input_directory_or_file", "output.tar.xz")
     .with_compression_level(6)
     .with_buffer_size(64);
```

## Compressing Data

To compress data, call the `compress` method and pass a callback function to report progress.

```rust
let result = options.compress(|progress| {
     println!("Progress: {:?}", progress);
});

assert!(result.is_ok());
```

## Full Example

This example demonstrates how to create an `LZMATarball` instance, configure it, and use it to compress data while reporting progress.

```rust
use lzma_tarball::LZMATarball;

let result = LZMATarball::new("./input_directory_or_file", "output.tar.xz")
     .with_compression_level(6)
     .with_buffer_size(64)
     .compress(|progress| {
    	 println!("Progress: {:?}", progress);
     });

assert!(result.is_ok());
```

## Setting a Custom TAR File Path

By default, the TAR file is created in the current directory. You can set a custom TAR file path using the `set_tar_file` method.

```rust
let options = LZMATarball::new("./input_directory_or_file", "output.tar.xz")
     .set_tar_file("./custom/path/custom.tar");

let result = options.compress(|progress| {
     println!("Progress: {:?}", progress);
});

assert!(result.is_ok());
```

## Handling Errors

The `compress` method returns a `Result` which you can use to handle potential errors.

```rust
let result = options.compress(|progress| {
     println!("Progress: {:?}", progress);
});

match result {
     Ok(success) => println!("Compression successful: {:?}", success),
     Err(e) => eprintln!("Compression failed: {:?}", e),
}
```

## Struct Definitions

For reference, here are the definitions of the main structs used in this library.

```rust
pub struct LZMATarball {
     pub compression_level: u8,
     pub buffer_size: u16,
     pub output_file: PathBuf,
     pub tar_file: PathBuf,
     pub input_path: PathBuf,
}

pub struct LZMAResult {
     pub output_file: PathBuf,
     pub size: u64,
     pub original_size: u64,
     pub elapsed_time: std::time::Duration,
}

pub struct LZMACallbackResult {
     pub bytes_processed: u64,
     pub bytes_per_second: u64,
}
```

## Methods

Here are some of the key methods available:

- `new`: Creates a new instance of `LZMATarball`.
- `with_compression_level`: Sets the desired compression level (0-9).
- `with_buffer_size`: Specifies the buffer size to use (in KB).
- `set_tar_file`: Sets a custom TAR file path (this is the temp file).
- `compress`: Compresses the data and calls the provided callback function with progress updates.
