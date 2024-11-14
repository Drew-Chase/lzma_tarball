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
		.with_path("./", "/all")
		.unwrap() // This will throw an error if the path does not exist, or it could not be determined if it is a file or directory
		// Add the contents of the directory to the archive.
		// The first argument is the path to the directory to compress
		// the second argument is the path inside the archive
		.with_directory_contents("./", "/contents")
		// Filter the contents of the directory.
		// Only files with a ".rs" extension will be included in the archive.
		.with_filtered_directory_contents("./", "./rs", &|entry| { entry.path().extension().is_some_and(|ext| ext == "rs") })
		// Add specific files to the archive.
		// The first file added is "test.txt", which will appear as "/test.txt" in the archive.
		// The second file added is "other.txt", which will appear as "/other.txt" in the archive.
		.with_files(
			&mut vec![
				ArchiveEntry {
					filesystem_path: PathBuf::from("./Cargo.toml"),
					archive_path: "/individual/Cargo.toml".to_string(),
				},
				ArchiveEntry {
					filesystem_path: PathBuf::from("./Cargo.lock"),
					archive_path: "/individual/Cargo.lock".to_string(),
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
