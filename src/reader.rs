use anyhow::{Result, Context};
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Archive;
use xz2::read::XzDecoder;

#[cfg(feature = "log")]
use log::*;
#[cfg(not(feature = "log"))]
use crate::*;

/// `LZMATarballReader` is used to read and decompress LZMA compressed tarball files.
#[derive(Debug, Clone)]
pub struct LZMATarballReader {
	archive_file: Option<PathBuf>,
	output: Option<PathBuf>,
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

impl Default for LZMATarballReader {
	fn default() -> Self {
		debug!("Creating default LZMATarballReader instance.");
		Self::new()
	}
}

impl LZMATarballReader {
	/// Creates a new `LZMATarballReader`.
	pub fn new() -> Self {
		debug!("Initializing a new LZMATarballReader with default settings.");
		Self {
			archive_file: None,
			output: None,
			overwrite: false,
			mask: 0,
			ignore_zeros: false,
			preserve_mtime: true,
			preserve_ownerships: true,
			preserve_permissions: true,
			unpack_xattrs: false,
		}
	}

	/// Sets the archive file path.
	pub fn set_archive(&mut self, archive: impl AsRef<Path>) -> Result<&mut Self> {
		debug!("Attempting to set archive file: {:?}", archive.as_ref());
		if !archive.as_ref().exists() {
			error!("Archive file not found: {:?}", archive.as_ref());
			anyhow::bail!("File not found: {:?}", archive.as_ref());
		}
		self.archive_file = Some(archive.as_ref().to_path_buf());
		info!("Archive file set to: {:?}", archive.as_ref());
		Ok(self)
	}

	/// Sets the output directory for decompressed files.
	pub fn set_output_directory(&mut self, output_dir: impl AsRef<Path>) -> Result<&mut Self> {
		let output_dir = output_dir.as_ref().to_path_buf();
		info!("Setting output directory: {:?}", &output_dir);
		debug!("Attempting to create output directory if it doesn't exist.");
		fs::create_dir_all(&output_dir).context("Failed to create output directory")?;
		self.output = Some(output_dir);
		Ok(self)
	}

	/// Sets the overwrite flag.
	pub fn set_overwrite(&mut self, overwrite: bool) -> &mut Self {
		debug!("Setting overwrite flag to: {}", overwrite);
		self.overwrite = overwrite;
		self
	}

	/// Sets the file permission mask.
	pub fn set_mask(&mut self, mask: u32) -> &mut Self {
		debug!("Setting file permission mask to: {}.", mask);
		self.mask = mask;
		self
	}

	/// Sets the ignore_zeros flag.
	pub fn set_ignore_zeros(&mut self, ignore_zeros: bool) -> &mut Self {
		debug!("Setting ignore_zeros flag to: {}.", ignore_zeros);
		self.ignore_zeros = ignore_zeros;
		self
	}

	/// Sets the preserve modification time flag.
	pub fn set_preserve_mtime(&mut self, preserve_mtime: bool) -> &mut Self {
		debug!("Setting preserve_mtime flag to: {}.", preserve_mtime);
		self.preserve_mtime = preserve_mtime;
		self
	}

	/// Sets the preserve ownerships flag.
	pub fn set_preserve_ownerships(&mut self, preserve_ownerships: bool) -> &mut Self {
		debug!("Setting preserve_ownerships flag to: {}.", preserve_ownerships);
		self.preserve_ownerships = preserve_ownerships;
		self
	}

	/// Sets the preserve permissions flag.
	pub fn set_preserve_permissions(&mut self, preserve_permissions: bool) -> &mut Self {
		debug!("Setting preserve_permissions flag to: {}.", preserve_permissions);
		self.preserve_permissions = preserve_permissions;
		self
	}

	/// Lists entries in the tarball archive.
	pub fn entries(&self) -> Result<Vec<String>> {
		debug!("Fetching entries from archive.");
		let archive = &mut self.get_archive()?;
		let files = archive.entries().context("Failed to get entries from archive")?;
		let files: Vec<String> = files
			.filter_map(|file| {
				file.ok().and_then(|f| {
					f.path().ok().and_then(|p| {
						let path_str = p.to_str().map(|s| s.to_string());
						if let Some(ref s) = path_str {
							debug!("Found file: {}", s);
						}
						path_str
					})
				})
			})
			.collect();
		info!("Total entries fetched: {}", files.len());
		Ok(files)
	}

	/// Returns an `Archive` object for the tarball file.
	pub fn get_archive(&self) -> Result<Archive<XzDecoder<File>>> {
		debug!("Retrieving archive from LZMATarballReader.");
		if let Some(archive) = &self.archive_file {
			debug!("Opening archive file: {:?}", archive);
			let file = File::open(archive).context("Failed to open archive file")?;
			let mut archive = Archive::new(XzDecoder::new(file));
			archive.set_overwrite(self.overwrite);
			archive.set_mask(self.mask);
			archive.set_ignore_zeros(self.ignore_zeros);
			archive.set_preserve_mtime(self.preserve_mtime);
			archive.set_preserve_ownerships(self.preserve_ownerships);
			archive.set_preserve_permissions(self.preserve_permissions);
			archive.set_unpack_xattrs(self.unpack_xattrs);
			info!("Archive successfully initialized with provided configurations.");
			Ok(archive)
		} else {
			error!("No archive file specified in LZMATarballReader.");
			anyhow::bail!("No archive file specified");
		}
	}

	/// Decompresses the tarball archive to the specified output directory.
	pub fn decompress(&self) -> Result<DecompressionResult> {
		debug!("Starting decompression process.");
		if let Some(output_dir) = &self.output {
			info!("Using output directory: {:?}", output_dir);
			let start = std::time::Instant::now();
			if !output_dir.exists() {
				debug!("Output directory does not exist; attempting to create: {:?}", output_dir);
				fs::create_dir_all(output_dir).context("Failed to create output directory")?;
			}
			let files = self.entries()?;
			debug!("Unpacking archive into output directory.");
			let mut archive = self.get_archive()?;
			archive.unpack(output_dir).context("Failed to unpack archive")?;
			let mut size = 0;
			for file in &files {
				let file_path = output_dir.join(file);
				debug!("Processing file: {:?}", file_path);
				let metadata = fs::metadata(&file_path).context("Failed to get metadata for file")?;
				size += metadata.len();
			}
			let elapsed = start.elapsed();
			info!("Decompression completed in {:?}", elapsed);
			Ok(DecompressionResult {
				elapsed_time: elapsed,
				files,
				total_size: size,
			})
		} else {
			error!("Output directory not specified when decompress() was called.");
			anyhow::bail!("No output directory specified");
		}
	}
}