use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Archive;
use xz2::read::XzDecoder;

/// `LZMATarballReader` is used to read and decompress LZMA compressed tarball files.
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

/// `DecompressionResult` holds the result of a decompression operation.
#[derive(Debug, Clone)]
pub struct DecompressionResult {
	pub elapsed_time: std::time::Duration,
	pub files: Vec<String>,
	pub total_size: u64,
}

impl LZMATarballReader {
	/// Creates a new `LZMATarballReader`.
	///
	/// # Arguments
	///
	/// * `tar_file` - Path to the tarball file to be read.
	///
	/// # Errors
	///
	/// Returns an error if the file does not exist.
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

	/// Sets the overwrite flag.
	///
	/// # Arguments
	///
	/// * `overwrite` - A boolean flag to set whether to overwrite existing files.
	pub fn set_overwrite(&mut self, overwrite: bool) -> &mut Self {
		self.overwrite = overwrite;
		self
	}

	/// Sets the file permission mask.
	///
	/// # Arguments
	///
	/// * `mask` - A permission mask to apply.
	pub fn set_mask(&mut self, mask: u32) -> &mut Self {
		self.mask = mask;
		self
	}

	/// Sets the ignore_zeros flag.
	///
	/// # Arguments
	///
	/// * `ignore_zeros` - A boolean flag to set whether to ignore zero blocks in the archive.
	pub fn set_ignore_zeros(&mut self, ignore_zeros: bool) -> &mut Self {
		self.ignore_zeros = ignore_zeros;
		self
	}

	/// Sets the preserve modification time flag.
	///
	/// # Arguments
	///
	/// * `preserve_mtime` - A boolean flag to set whether to preserve modification times of files.
	pub fn set_preserve_mtime(&mut self, preserve_mtime: bool) -> &mut Self {
		self.preserve_mtime = preserve_mtime;
		self
	}

	/// Sets the preserve ownerships flag.
	///
	/// # Arguments
	///
	/// * `preserve_ownerships` - A boolean flag to set whether to preserve file ownerships.
	pub fn set_preserve_ownerships(&mut self, preserve_ownerships: bool) -> &mut Self {
		self.preserve_ownerships = preserve_ownerships;
		self
	}

	/// Sets the preserve permissions flag.
	///
	/// # Arguments
	///
	/// * `preserve_permissions` - A boolean flag to set whether to preserve file permissions.
	pub fn set_preserve_permissions(&mut self, preserve_permissions: bool) -> &mut Self {
		self.preserve_permissions = preserve_permissions;
		self
	}

	/// Lists entries in the tarball archive.
	///
	/// # Errors
	///
	/// Returns an error if the archive cannot be read.
	pub fn entries(&self) -> Result<Vec<String>, Box<dyn Error>> {
		let archive = &mut self.get_archive()?;
		let files = archive.entries()?;

		// Collect file paths as strings
		let files = files
			.filter_map(|file| {
				if let Ok(file) = file {
					Some(file.path().unwrap().to_str().unwrap().to_string())
				} else {
					None
				}
			})
			.collect();

		Ok(files)
	}

	/// Returns an `Archive` object for the tarball file.
	///
	/// # Errors
	///
	/// Returns an error if the file cannot be opened or if the archive cannot
	/// be created.
	///
	/// # Returns
	///
	/// A result containing the `Archive` object or an error.
	pub fn get_archive(&self) -> Result<Archive<XzDecoder<File>>, Box<dyn Error>> {
		let archive = &self.tar_file;
		let file = File::open(archive)?;
		let mut archive = Archive::new(XzDecoder::new(file));
		
		// Set archive options
		archive.set_overwrite(self.overwrite);
		archive.set_mask(self.mask);
		archive.set_ignore_zeros(self.ignore_zeros);
		archive.set_preserve_mtime(self.preserve_mtime);
		archive.set_preserve_ownerships(self.preserve_ownerships);
		archive.set_preserve_permissions(self.preserve_permissions);
		archive.set_unpack_xattrs(self.unpack_xattrs);
		
		Ok(archive)
	}

	/// Decompresses the tarball archive to the specified output directory.
	///
	/// # Arguments
	///
	/// * `output_dir` - The directory to decompress files into.
	///
	/// # Errors
	///
	/// Returns an error if the decompression fails.
	pub fn decompress(&self, output_dir: impl AsRef<Path>) -> Result<DecompressionResult, Box<dyn Error>> {
		let start = std::time::Instant::now();
		let output_dir = output_dir.as_ref();
		if !output_dir.exists() {
			fs::create_dir_all(output_dir)?;
		}

		// Get the list of files in the archive
		let files = self.entries()?;

		// Decompress the archive
		let mut archive = self.get_archive()?;
		archive.unpack(output_dir)?;

		// Calculate the total size of the decompressed files
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
