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
//!    	 .with_compression_level(6)
//!    	 // Set the buffer size to 64KB - this is the default
//!    	 // The buffer size is used to read and write data
//!    	 // A larger buffer size speeds up compression but uses more memory
//!    	 // A smaller buffer size slows down compression but uses less memory
//!    	 .with_buffer_size(64)
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
use xz2::write::XzEncoder;

/// Options for LZMA compression
#[derive(Debug, Clone)]
pub struct LZMATarballWriter {
	pub compression_level: u8,
	pub buffer_size: u16,
	pub output_file: PathBuf,
	pub tar_file: PathBuf,
	pub input_path: PathBuf,
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

impl LZMATarballWriter {
	/// Creates new LZMAOptions with default settings
	/// - Default Compression level: 6
	/// - Default Buffer size: 64KB
	/// - Default Tar File: `%TEMP%/{filename|"archive"}-{timestamp}.tar`
	pub fn new(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
		let absolute_input = input.as_ref().canonicalize()?;
		if !absolute_input.exists() {
			#[cfg(feature = "log")]
			{
				error!("Input path does not exist: {:?}", absolute_input);
			}
			return Err("Input path does not exist".into());
		}
		let filename = match input.as_ref().file_name() {
			Some(name) => name.to_str(),
			None => Some("archive"),
		}
			.unwrap_or("archive");

		let tar_file_path = temp_dir().join(format!(
			"{}-{}.tar",
			filename,
			chrono::Utc::now().timestamp()
		));
		let output = output.as_ref();
		// create output directory if it doesn't exist
		if let Some(parent) = output.parent() {
			std::fs::create_dir_all(parent)?;
		}

		Ok(LZMATarballWriter {
			compression_level: 6,
			buffer_size: 64,
			output_file: output.to_path_buf(),
			tar_file: tar_file_path,
			input_path: absolute_input,
		})
	}

	/// Sets the compression level (clamps between 0 and 9)
	pub fn with_compression_level(&mut self, level: u8) -> &mut Self {
		self.compression_level = level.clamp(0, 9);
		self
	}

	/// Sets the buffer size in KB
	pub fn with_buffer_size(&mut self, size: u16) -> &mut Self {
		self.buffer_size = size;
		self
	}

	/// Sets the temporary tar file output path
	pub fn set_tar_file(&mut self, tar_file: PathBuf) -> &mut Self {
		self.tar_file = tar_file;
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
		let start = std::time::Instant::now();

		match create_tar(&self.input_path, &self.tar_file) {
			Ok(_) => (),
			Err(e) => return Err(format!("Failed to create tar file: {}", e).into()),
		};

		match compress_tar(
			&self.tar_file,
			self.output_file.to_str().unwrap(),
			self.compression_level,
			self.buffer_size,
			callback
		) {
			Ok(_) => (),
			Err(e) => return Err(format!("Failed to compress tar file: {}", e).into()),
		}

		let tarball_size = self.tar_file.metadata()?.len();
		#[cfg(feature = "log")]
		debug!("Removing tar file: {:?}", self.tar_file);

		std::fs::remove_file(&self.tar_file)
			.map_err(|e| format!("Failed to remove tar file: {}", e))?;

		let elapsed_time = start.elapsed();
		let size = self.output_file.metadata()?.len();

		Ok(LZMAResult {
			output_file: self.output_file.clone(),
			size,
			original_size: tarball_size,
			elapsed_time,
		})
	}
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
fn create_tar(filepath: &Path, tar_file_path: &Path) -> Result<(), Box<dyn Error>> {
	#[cfg(feature = "log")]
	debug!("Creating tar file: {:?}", tar_file_path);
	let tar_file = File::create(tar_file_path)?;
	let mut tar_builder = Builder::new(BufWriter::new(tar_file));

	let metadata = filepath.metadata()?;
	let is_directory = metadata.is_dir();

	if is_directory {
		compress_directory(filepath, filepath, &mut tar_builder)?;
	} else {
		let root = filepath.parent().unwrap();
		compress_file(filepath, root, &mut tar_builder)?;
	}

	tar_builder.into_inner()?;
	Ok(())
}

/// Compresses a directory recursively into a tarball
///
/// # Parameters
/// - `directory`: The directory to compress
/// - `root`: The root directory for relative paths
/// - `tar_builder`: The tar builder to use for compression
///
/// # Returns
/// - `Ok(())` on success
/// - `Box<dyn Error>` on failure
fn compress_directory(
	directory: impl AsRef<Path>,
	root: impl AsRef<Path>,
	tar_builder: &mut Builder<BufWriter<File>>,
) -> Result<(), Box<dyn Error>>
{
	#[cfg(feature = "log")]
	debug!("Compressing directory: {:?}", directory.as_ref());
	for entry in std::fs::read_dir(directory.as_ref())? {
		let entry = entry?;
		let path = entry.path();

		if entry.file_type()?.is_dir() {
			compress_directory(path, root.as_ref(), tar_builder)?;
		} else {
			compress_file(path, &root, tar_builder)?;
		}
	}
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
	file: impl AsRef<Path>,
	root: impl AsRef<Path>,
	tar_builder: &mut Builder<BufWriter<File>>,
) -> Result<(), Box<dyn Error>>
{
	let file = file.as_ref();
	let root = root.as_ref();

	let compressed_path = if file == root {
		file
	} else {
		file.strip_prefix(root)?
	};

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
	input_path: &Path,
	output_path: &str,
	level: u8,
	buffer_size: u16,
	callback: F
) -> Result<(), Box<dyn Error>>
	where
		F: Fn(LZMACallbackResult) + 'static + Send + Sync,
{
	let mut input_file = BufReader::new(File::open(input_path)?);
	let output_file = BufWriter::new(File::create(output_path)?);

	let mut compressor = XzEncoder::new(output_file, level as u32);
	let mut buffer = vec![0; 1024 * (buffer_size as usize)];

	let total_size = std::fs::metadata(input_path)?.len();
	#[cfg(feature = "log")]
	debug!("Balling up the tar with {}KB Buffer...", buffer_size);
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