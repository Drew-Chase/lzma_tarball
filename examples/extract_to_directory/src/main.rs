use lzma_tarball::reader::LZMATarballReader;

fn main() {
	// Initialize LZMATarballReader and configure it with the specified options
	let result = LZMATarballReader::new()
		// Set the archive file (tarball) path
		.set_archive("../test/test.tar.xz").unwrap()
		// Set the output directory for decompressed files
		.set_output_directory("../test/output").unwrap()
		// Enable overwriting of existing files
		.set_overwrite(true)
		// Set the file permission mask
		.set_mask(0o644)
		// Set to not ignore zero blocks in the archive
		.set_ignore_zeros(false)
		// Preserve the modification time of files
		.set_preserve_mtime(true)
		// Preserve file ownerships
		.set_preserve_ownerships(true)
		// Preserve file permissions
		.set_preserve_permissions(true)
		// Perform the decompression and get the result
		.decompress().unwrap();

	// Iterate over the list of extracted files and print their names
	for file in result.files {
		println!("Extracted: {}", file);
	}

	// Print the total size of decompressed files
	println!("Total size: {} bytes", result.total_size);

	// Print the elapsed time for the decompression process
	println!("Elapsed time: {:?}", result.elapsed_time);
}
