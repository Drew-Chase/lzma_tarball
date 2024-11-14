use lzma_tarball::reader::LZMATarballReader;
fn main() {
	let entries: Vec<String> = LZMATarballReader::new()
		// Set the archive file (tarball) path
		.set_archive("../test/test.tar.xz").unwrap()
		// Read the entries in the archive
		.entries().unwrap();

	for entry in entries {
		println!("Entry: {}", entry);
	}
}
