//! # LZMA Tarball Writer
//! This documentation provides detailed instructions on how to use the LZMATarballWriter to compress files or directories into LZMA-compressed tarballs.
//!
//! ## Example
//! Below is a basic example demonstrating how to use the `LZMATarballWriter` to compress a directory or file.
//! ```rust
//! use lzma_tarball::writer::LZMATarballWriter;
//!
//! fn main() {
//!     // The input path can be any directory or file, specified as a relative or absolute path.
//!     let input_path = "./";
//!
//!     // Specify the output file path. This will create the parent directories if they don't exist.
//!     let output = "../test/test.tar.xz";
//!
//!     // Create a new LZMATarballWriter and configure it
//!     let result = LZMATarballWriter::new(input_path, output)
//!    	 .unwrap()
//!    	 // Set the compression level to 6 - this is the default
//!    	 // The range is 0-9, where 0 is no compression and 9 is the maximum compression
//!    	 .set_compression_level(6)
//!    	 // Set the buffer size to 64KB - this is the default
//!    	 // The buffer size is used to read and write data
//!    	 // A larger buffer size speeds up compression but uses more memory
//!    	 // A smaller buffer size slows down compression but uses less memory
//!    	 .set_buffer_size(64)
//!    	 // Compress the data and report progress
//!    	 .compress(|progress| {
//!    		 // The percentage of compression completed, ranging between 0.0 and 1.0
//!    		 // Multiply by 100 to get a percentage
//!    		 let percentage = progress.percentage * 100f32;
//!
//!    		 // The number of bytes processed
//!    		 let processed = progress.bytes_processed;
//!
//!    		 // The number of bytes processed per second
//!    		 let bps = progress.bytes_per_second;
//!
//!    		 // Convert bytes per second to megabytes per second
//!    		 let mbps = (bps as f32) / 1024f32 / 1024f32;
//!
//!    		 // Update progress on the same console line
//!    		 print!("\x1b[1A"); // Move cursor up
//!    		 println!("Progress: {:.2}% - Processed: {}B - Speed: {:.2}Mb/s", percentage, processed, mbps);
//!    	 }).unwrap();
//!
//!     // Retrieve and print compression results
//!     let duration = result.elapsed_time;
//!     let size = result.size;
//!     let original_size = result.original_size;
//!     println!("Compression complete! Elapsed time: {:?}", duration);
//!     println!("Original size: {}B - Compressed size: {}B", original_size, size);
//! }
//! ```
//!
//! ## Detailed Explanation
//!
//! ### LZMATarballWriter::new
//! - `new(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<Self, Box<dyn Error>>`
//! - Creates a new instance of the `LZMATarballWriter`.
//! - It takes the path of the input (file or directory) to be compressed and the path of the output file.
//! - The method ensures that the input path exists and resolves the output directory.
//!
//! ### LZMATarballWriter::with_compression_level
//! - `with_compression_level(&mut self, level: u8) -> &mut Self`
//! - Sets the compression level, clamping it between 0 (no compression) and 9 (maximum compression).
//! - The default compression level is 6.
//!
//! ### LZMATarballWriter::with_buffer_size
//! - `with_buffer_size(&mut self, size: u16) -> &mut Self`
//! - Sets the buffer size for reading and writing data during compression.
//! - The buffer size is in kilobytes (KB). The default is 64KB.
//!
//! ### LZMATarballWriter::compress
//! - `compress<F>(&self, callback: F) -> Result<LZMAResult, Box<dyn Error>> where F: Fn(LZMACallbackResult) + 'static + Send + Sync`
//! - Compresses the input path into an LZMA-compressed tarball.
//! - A callback function is provided to report progress, which includes the percentage completed, bytes processed, and the speed in bytes per second (converted to megabytes per second).
//! - Returns an `LZMAResult` on success, containing details about the compressed file size, original file size, and elapsed time of compression.

#[cfg(feature = "log")]
use log::{debug, error};
use std::env::temp_dir;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use tar::Builder;
use walkdir::DirEntry;
use xz2::write::XzEncoder;

/// Options for LZMA compression
#[derive(Debug, Clone)]
pub struct LZMATarballWriter {
	pub compression_level: u8,
	pub buffer_size: u16,
	pub output_file: Option<PathBuf>,
	pub tar_file: PathBuf,
	pub archive_paths: Vec<ArchiveEntry>,
}

/// Result of an LZMA compression operation
#[derive(Debug, Clone)]
pub struct LZMAResult {
	pub output_file: PathBuf,
	pub size: u64,
	pub original_size: u64,
	pub elapsed_time: std::time::Duration,
}

/// Callback result for reporting progress
#[derive(Debug, Clone)]
pub struct LZMACallbackResult {
	pub bytes_processed: u64,
	pub bytes_per_second: u64,
	pub percentage: f32,
}
#[derive(Debug, Clone)]
pub struct ArchiveEntry {
	pub filesystem_path: PathBuf,
	pub archive_path: String
}


impl Default for LZMATarballWriter {
	fn default() -> Self {
		Self::new()
	}
}

impl LZMATarballWriter {
	/// Creates new LZMAOptions with default settings
	/// - Default Compression level: 6
	/// - Default Buffer size: 64KB
	/// - Default Tar File: `%TEMP%/{filename|"archive"}-{timestamp}.tar`
	pub fn new() -> Self {
		let tar_file_path = temp_dir().join(format!(
			"archive-{}.tmp",
			chrono::Utc::now().timestamp()
		));

		LZMATarballWriter {
			compression_level: 6,
			buffer_size: 64,
			output_file: None,
			tar_file: tar_file_path,
			archive_paths: Vec::new(),
		}
	}

	/// Sets the compression level (clamps between 0 and 9)
	pub fn set_compression_level(&mut self, level: u8) -> &mut Self {
		self.compression_level = level.clamp(0, 9);
		self
	}

	/// Sets the buffer size in KB
	pub fn set_buffer_size(&mut self, size: u16) -> &mut Self {
		self.buffer_size = size;
		self
	}

	/// Sets the temporary tar file output path
	pub fn set_tar_file(&mut self, tar_file: impl AsRef<Path>) -> &mut Self {
		self.tar_file = tar_file.as_ref().to_path_buf();
		self
	}

	pub fn with_path(&mut self, input_path: impl AsRef<Path>, archive_path: impl AsRef<str>) -> Result<&mut Self, Box<dyn Error>> {
		let metadata = input_path.as_ref().metadata()?;
		if metadata.is_dir() {
			Ok(self.with_directory_contents(input_path, archive_path))
		} else {
			Ok(self.with_file(input_path, archive_path))
		}
	}

	pub fn with_file(&mut self, input_file: impl AsRef<Path>, archive_path: impl AsRef<str>) -> &mut Self {
		self.archive_paths.push(ArchiveEntry {
			filesystem_path: input_file.as_ref().to_path_buf(),
			archive_path: archive_path.as_ref().to_string()
		});
		self
	}

	pub fn with_files(&mut self, input_files: &mut Vec<ArchiveEntry>) -> &mut Self {
		self.archive_paths.append(input_files);
		self
	}

	pub fn with_directory_contents(&mut self, input_directory: impl AsRef<Path>, archive_path: impl AsRef<str>) -> &mut Self {
		self.with_filtered_directory_contents(input_directory, archive_path, &|_| true)
	}

	pub fn set_output(&mut self, output_file: impl AsRef<Path>) -> &mut Self {
		let output_file = output_file.as_ref().to_path_buf();
		std::fs::create_dir_all(output_file.parent().unwrap()).unwrap();
		self.output_file = Some(output_file);
		self
	}

	pub fn with_filtered_directory_contents(&mut self, input_directory: impl AsRef<Path>, archive_path: impl AsRef<str>, filter: &dyn Fn(&DirEntry) -> bool) -> &mut Self {
		walkdir::WalkDir::new(&input_directory)
			.into_iter()
			.filter_map(|e| e.ok())
			.filter(|e| e.file_type().is_file())
			.filter(filter)
			.for_each(|e| {
				self.archive_paths.push(ArchiveEntry {
					filesystem_path: e.path().to_path_buf(),
					archive_path: format!("{}/{}", archive_path.as_ref(), e.path().to_path_buf().strip_prefix(&input_directory).unwrap().to_str().unwrap())
				});
			});
		self
	}


	/// Compress the input path into an LZMA-compressed file
	///
	/// # Parameters
	/// - `callback`: A callback function to report progress
	///
	/// # Returns
	/// - `LZMAResult` on success
	/// - `Box<dyn Error>` on failure
	pub fn compress<F>(&self, callback: F) -> Result<LZMAResult, Box<dyn Error>>
	                   where
		                   F: Fn(LZMACallbackResult) + 'static + Send + Sync,
	{
		if self.archive_paths.is_empty() {
			#[cfg(feature = "log")]
			error!("No files or directories to compress");
			return Err("No files or directories to compress".into());
		}

		let output_file = match self.output_file {
			Some(ref file) => file,
			None => {
				#[cfg(feature = "log")]
				error!("Output file not set");
				return Err("Output file not set".into())
			}
		};

		let start = std::time::Instant::now();

		match self.create_tar() {
			Ok(_) => (),
			Err(e) => return Err(format!("Failed to create tar file: {}", e).into()),
		};

		match self.compress_tar(callback) {
			Ok(_) => (),
			Err(e) => return Err(format!("Failed to compress tar file: {}", e).into()),
		}

		let tarball_size = self.tar_file.metadata()?.len();
		#[cfg(feature = "log")]
		debug!("Removing tar file: {:?}", self.tar_file);

		std::fs::remove_file(&self.tar_file)
			.map_err(|e| format!("Failed to remove tar file: {}", e))?;

		let elapsed_time = start.elapsed();
		let size = output_file.metadata()?.len();

		Ok(LZMAResult {
			output_file: output_file.clone(),
			size,
			original_size: tarball_size,
			elapsed_time,
		})
	}

	/// Creates a tarball from the specified filepath
	///
	/// # Parameters
	/// - `filepath`: The path to the file or directory to tar
	/// - `tar_file_path`: The path where the tar file will be created
	///
	/// # Returns
	/// - `Ok(())` on success
	/// - `Box<dyn Error>` on failure
	fn create_tar(&self) -> Result<(), Box<dyn Error>> {
		#[cfg(feature = "log")]
		debug!("Creating tar file: {:?}", &self.tar_file);
		let tar_file = File::create(&self.tar_file)?;
		let mut tar_builder = Builder::new(BufWriter::new(tar_file));

		for archive_path in self.archive_paths.iter() {
			match Self::compress_file(archive_path, &mut tar_builder){
				Ok(_) => (),
				Err(e) => return Err(format!("Failed to compress file: {}", e).into())
			}
		}

		tar_builder.into_inner()?;
		Ok(())
	}

	/// Compresses a single file into a tarball
	///
	/// # Parameters
	/// - `file`: The file to compress
	/// - `root`: The root directory for relative paths
	/// - `tar_builder`: The tar builder to use for compression
	///
	/// # Returns
	/// - `Ok(())` on success
	/// - `Box<dyn Error>` on failure
	fn compress_file(
		entry: &ArchiveEntry,
		tar_builder: &mut Builder<BufWriter<File>>,
	) -> Result<(), Box<dyn Error>>
	{
		let file = entry.filesystem_path.to_str().unwrap();
		let compressed_path = entry.archive_path.as_str();

		// trim leading slash
		let compressed_path = compressed_path.strip_prefix("/").unwrap_or(compressed_path);

		#[cfg(feature = "log")]
		debug!("Streamed file to tar: {:?}", compressed_path);
		let mut stream = File::open(file)?;
		tar_builder.append_file(compressed_path, &mut stream)?;

		Ok(())
	}

	/// Compresses a tar file into an LzMA-compressed file
	///
	/// # Parameters
	/// - `input_path`: The path to the input tar file
	/// - `output_path`: The path where the compressed file will be created
	/// - `level`: The compression level
	/// - `buffer_size`: The buffer size for compression
	///
	/// # Returns
	/// - `Ok(())` on success
	/// - `Box<dyn Error>` on failure
	fn compress_tar<F>(
		&self,
		callback: F
	) -> Result<(), Box<dyn Error>>
		where
			F: Fn(LZMACallbackResult) + 'static + Send + Sync,
	{
		let mut input_file = BufReader::new(File::open(&self.tar_file)?);

		let output_file = match &self.output_file {
			Some(file) => BufWriter::new(File::create(file)?),
			None => return Err("Output file not set".into())
		};

		let mut compressor = XzEncoder::new(output_file, self.compression_level as u32);
		let mut buffer = vec![0; 1024 * (self.buffer_size as usize)];

		let total_size = std::fs::metadata(&self.tar_file)?.len();
		#[cfg(feature = "log")]
		debug!("Balling up the tar with {}KB Buffer...", self.buffer_size);
		let mut bytes_processed = 0;
		let start = std::time::Instant::now();
		loop {
			let bytes_read = input_file.read(&mut buffer)?;
			if bytes_read == 0 {
				break; // End of file
			}
			bytes_processed += bytes_read as u64;
			let elapsed_seconds = start.elapsed().as_secs();
			if elapsed_seconds > 0 {
				let bytes_per_second = bytes_processed / elapsed_seconds;
				let percentage = bytes_processed as f32 / total_size as f32;
				callback(LZMACallbackResult {
					bytes_processed,
					bytes_per_second,
					percentage,
				});
			}
			compressor.write_all(&buffer[..bytes_read])?;
		}

		compressor.finish()?;
		#[cfg(feature = "log")]
		debug!("Compression complete!");
		Ok(())
	}
}
