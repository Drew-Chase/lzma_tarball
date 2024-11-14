# LZMA Tarball Documentation

This library provides functionalities to compress directories and files into `.tar.xz` format and decompress `.tar.xz` files using configurable options.

## Examples

See the [examples](https://github.com/Drew-Chase/lzma_tarball/tree/master/examples) directory for more detailed examples.

## Creating an Archive

To create a `.tar.xz` archive, use the `LZMATarballWriter` struct.
first create a new instance of the `LZMATarballWriter` struct using the `new` method.

```rust
use lzma_tarball::writer::LZMATarballWriter;
// ...
let result = LZMATarballWriter::new();
```

By default the compression level is set to 6 and the buffer size is set to 64 kilobytes. These values can be changed using the `set_compression_level` and `set_buffer_size` methods.

```rust
// ...
.set_compression_level(6) // 0-9, where 0 is no compression and 9 is maximum compression
.set_buffer_size(64); // 64 kilobytes
```
### Adding Files and Directories
Next, add the files and directories to the archive using the `with_path` method. The first argument is the path to the file or directory to add to the archive, and the second argument is the path inside the archive. If the second argument is "/", the file or directory will be placed in the root of the archive.
This method will check if the provided path is a directory or file and call the appropriate method to add it to the archive.

```rust
// ...
.with_path("./", "/")
.unwrap(); // This throws an error if the path does not exist or it could not determine if it is a file or directory
```

Alternatively you can use the `with_file` and `with_directory` methods to add files and directories to the archive.

```rust
// ...
.with_file("./file.txt", "/file.txt")
.with_directory_contents("./directory", "/directory")
```

You can also add all files in a directory using a filter. The filter is a closure that takes a `&DirEntry` and returns a `bool`. If the closure returns `true`, the file will be added to the archive.

```rust
// ...
.with_filtered_directory_contents("./", "./rs", &|entry| { entry.path().extension().is_some_and(|ext| ext == "rs") })
```

Or you can add an array of file paths with the `with_files` method.

```rust
// ...
.with_files(
& mut vec![
	ArchiveEntry {
		filesystem_path: PathBuf::from("./test.txt"),
		archive_path: "/test.txt".to_string(),
	},
	ArchiveEntry {
		filesystem_path: PathBuf::from("./other.txt"),
		archive_path: "/other.txt".to_string(),
	}
]
)
```

All of these methods can be chained together to add multiple files and directories to the archive.

```rust
// ...
.with_path("./", "/")
.with_file("./file.txt", "/file.txt")
.with_directory_contents("./directory", "/directory")
.with_filtered_directory_contents("./", "./rs", &|entry| { entry.path().extension().is_some_and(|ext| ext == "rs") })
.with_files(
& mut vec![
	ArchiveEntry {
		filesystem_path: PathBuf::from("./test.txt"),
		archive_path: "/test.txt".to_string(),
	},
	ArchiveEntry {
		filesystem_path: PathBuf::from("./other.txt"),
		archive_path: "/other.txt".to_string(),
	}
]
)
```

Now set the output file using the `set_output` method. This will create the parent directories if they don't exist.

```rust
// ...
.set_output("../test/output.tar.xz")
```

Finally, call the `compress` method to compress the data. This method takes a closure that will be called with a `Progress` struct that contains information about the compression progress. The closure should return a `Result<(), Error>`.
The compress callback will be called with the progress of the compression. The progress struct contains the following fields:

- `percentage`: A float between 0.0 and 1.0 representing the percentage of the compression process that has been completed.
- `bytes_processed`: The number of bytes that have been processed so far.
- `bytes_per_second`: The number of bytes processed per second.   
  The callback is called everytime the buffer is filled and the data flushed to disk.
  So the larger the buffer size is, the less often the callback is called.

```rust
// ...
.compress( | progress| {
// Do something with the progress
//...
}).unwrap();
```

### Full Example

```rust
use lzma_tarball::writer::{ArchiveEntry, LZMATarballWriter};
use std::path::PathBuf;

fn main() {
	let result = LZMATarballWriter::new()
		// Set the compression level to 6 - this is the default
		// the range is 0-9, where 0 is no compression and 9 is maximum compression
		.set_compression_level(6)
		// Set the buffer size to 64 - this is the default
		// this is the size of the buffer used to read and write data
		// the larger, the buffer, the faster the compression, but the more memory it uses
		// the smaller, the buffer, the slower the compression, but the less memory it uses
		// the buffer size is in kilobytes
		.set_buffer_size(64)
		// The first argument is the path to the directory or file to compress
		// the second argument is the path inside the archive
		// if the second argument is "/", entry will be placed in the root of the archive.
		.with_path("./", "/")
		.unwrap()
		// Filter the contents of the directory.
		// Only files with a ".rs" extension will be included in the archive.
        .with_filtered_directory_contents("./", "./rs", &|entry| { entry.path().extension().is_some_and(|ext| ext == "rs") })
		// Add specific files to the archive.
		// The first file added is "test.txt", which will appear as "/test.txt" in the archive.
		// The second file added is "other.txt", which will appear as "/other.txt" in the archive.
		.with_files(
			&mut vec![
				ArchiveEntry {
					filesystem_path: PathBuf::from("./test.txt"),
					archive_path: "/test.txt".to_string(),
				},
				ArchiveEntry {
					filesystem_path: PathBuf::from("./other.txt"),
					archive_path: "/other.txt".to_string(),
				}
			]
		)
		// this is the output file.
		// this will create the parent directories if they don't exist.
		.set_output("../test/test.tar.xz")
		// Compress the data and report progress
		.compress(|progress| {
			// The percentage is between 0.0 and 1.0
			// Multiply by 100 to get a percentage
			let percentage = progress.percentage * 100f32;

			// The number of bytes processed
			let processed = progress.bytes_processed;

			// The number of bytes processed per second
			let bps = progress.bytes_per_second;

			// Convert bytes per second to megabytes per second
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

## Extracting an Archive
Extracting is pretty straight forward. First create a new instance of the `LZMATarballReader` struct using the `new` method.

```rust
use lzma_tarball::reader::LZMATarballReader;
// ...
let result = LZMATarballReader::new().unwrap();
```




### Full Example
```rust
use lzma_tarball::reader::LZMATarballReader;
fn main() {
	let archive = "../test/test.tar.xz";
	let result = LZMATarballReader::new(archive)
		.unwrap()
		.decompress("../test/output")
		.unwrap();

	let files = result.files;
	let duration = result.elapsed_time;
	let total_size = result.total_size;
	println!("Decompressed {} files in {:?} with a total size of {} bytes", files.len(), duration, total_size);
}
```

