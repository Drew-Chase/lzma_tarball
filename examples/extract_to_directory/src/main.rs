use lzma_tarball::reader::LZMATarballReader;

fn main() {
	let archive = "../test/test.tar.xz";
	let result = LZMATarballReader::new(archive)
		.unwrap()
		.decompress("output")
		.unwrap();

	let files = result.files;
	let duration = result.elapsed_time;
	let total_size = result.total_size;
	println!("Decompressed {} files in {:?} with a total size of {} bytes", files.len(), duration, total_size);
}
