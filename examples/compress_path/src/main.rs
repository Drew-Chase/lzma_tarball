use lzma_tarball::lzma::LZMATarball;

fn main() {
	// this can be any directory or file, relative or absolute path
	let input_path = "./";

	// this is the output file.
	// this will create the parent directories if they don't exist.
	let output = "../test/test.tar.xz";

	let result = LZMATarball::new(input_path, output)
		.unwrap()
		// Set the compression level to 6 - this is the default
		// the range is 0-9, where 0 is no compression and 9 is maximum compression
		.with_compression_level(6)
		// Set the buffer size to 64 - this is the default
		// this is the size of the buffer used to read and write data
		// the larger, the buffer, the faster the compression, but the more memory it uses
		// the smaller, the buffer, the slower the compression, but the less memory it uses
		// the buffer size is in kilobytes
		.with_buffer_size(64)
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
