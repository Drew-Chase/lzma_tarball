use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct LZMATarballReader {
	tar_file: PathBuf,
	overwrite: bool,
	mask: u32,
	ignore_zeros: bool,
	preserve_mtime: bool,
	preserve_ownerships: bool,
	preserve_permissions: bool,
	unpack_xattrs: bool,

}

#[derive(Debug, Clone)]
pub struct DecompressionResult {
	pub elapsed_time: std::time::Duration,
	pub files: Vec<String>,
	pub total_size: u64,
}

impl LZMATarballReader {
	pub fn new(tar_file: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
		let tar_file = tar_file.as_ref();
		if !tar_file.exists() {
			return Err(format!("File not found: {:?}", tar_file).into());
		}
		Ok(Self {
			tar_file: tar_file.to_path_buf(),
			overwrite: false,
			mask: 0,
			ignore_zeros: false,
			preserve_mtime: true,
			preserve_ownerships: true,
			preserve_permissions: true,
			unpack_xattrs: false,
		})
	}

	pub fn set_overwrite(&mut self, overwrite: bool) -> &mut Self {
		self.overwrite = overwrite;
		self
	}

	pub fn set_mask(&mut self, mask: u32) -> &mut Self {
		self.mask = mask;
		self
	}

	pub fn set_ignore_zeros(&mut self, ignore_zeros: bool) -> &mut Self {
		self.ignore_zeros = ignore_zeros;
		self
	}
	pub fn set_preserve_mtime(&mut self, preserve_mtime: bool) -> &mut Self {
		self.preserve_mtime = preserve_mtime;
		self
	}

	pub fn set_preserve_ownerships(&mut self, preserve_ownerships: bool) -> &mut Self {
		self.preserve_ownerships = preserve_ownerships;
		self
	}

	pub fn set_preserve_permissions(&mut self, preserve_permissions: bool) -> &mut Self {
		self.preserve_permissions = preserve_permissions;
		self
	}

	pub fn decompress(&self, output_dir: impl AsRef<Path>) -> Result<DecompressionResult, Box<dyn Error>> {
		let start = std::time::Instant::now();
		let output_dir = output_dir.as_ref();
		if !output_dir.exists() {
			fs::create_dir_all(output_dir)?;
		}

		let archive = &self.tar_file;
		let file = fs::File::open(archive)?;
		let mut archive = tar::Archive::new(xz2::read::XzDecoder::new(file));
		archive.set_overwrite(self.overwrite);
		archive.set_mask(self.mask);
		archive.set_ignore_zeros(self.ignore_zeros);
		archive.set_preserve_mtime(self.preserve_mtime);
		archive.set_preserve_ownerships(self.preserve_ownerships);
		archive.set_preserve_permissions(self.preserve_permissions);
		archive.set_unpack_xattrs(self.unpack_xattrs);

		let files = archive.entries()?.map(|entry| entry.unwrap().path().unwrap().to_path_buf().to_str().unwrap().to_string()).collect::<Vec<String>>();
		archive.unpack(output_dir)?;

		// calculate the total size of the decompressed files
		let mut size = 0;
		for file in &files {
			let file = output_dir.join(file);
			let metadata = fs::metadata(file)?;
			size += metadata.len();
		}

		Ok(DecompressionResult {
			elapsed_time: start.elapsed(),
			files,
			total_size: size,
		})
	}
}
