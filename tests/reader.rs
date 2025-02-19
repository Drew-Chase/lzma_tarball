// tests for the LZMATarballReader
#[cfg(test)]
mod tests {
    use chrono::Utc;
    use std::error::Error;
    use std::fs::{self, File};
    use std::io::Read;
    use std::path::PathBuf;
    use std::time::Duration;
    use tar::Builder;
    use xz2::write::XzEncoder;

    // Import the reader from your library. Adjust the path as needed.
    use lzma_tarball::reader::{DecompressionResult, LZMATarballReader};

    /// Create a temporary LZMA-compressed tar archive containing a single file "hello.txt"
    /// with the content "Hello, world!".
    fn create_test_tar_xz() -> Result<PathBuf, Box<dyn Error>> {
        // Create a unique temporary file path.
        let tmp_dir = std::env::temp_dir();
        let file_name = format!("lzma_tarball_test_{}.tar.xz", Utc::now().timestamp_millis());
        let archive_path = tmp_dir.join(file_name);

        // Open a file for writing.
        let file = File::create(&archive_path)?;

        // Wrap it in an XzEncoder with a compression level of 9.
        let encoder = XzEncoder::new(file, 9);

        // Create a tar archive using the encoder as the output.
        let mut tar_builder = Builder::new(encoder);

        // Create an in-memory file entry "hello.txt"
        let file_content = b"Hello, world!";
        {
            // Create a new tar header for the file entry.
            let mut header = tar::Header::new_gnu();
            header.set_size(file_content.len() as u64);
            header.set_cksum();
            tar_builder.append_data(&mut header, "hello.txt", file_content as &[u8])?;
        }

        // Finish the tar and then flush the encoder.
        tar_builder.finish()?;
        // The encoder must be finished to complete writing.
        // Get the encoder from the builder to call finish() on it:
        let encoder = tar_builder.into_inner()?;
        encoder.finish()?;

        Ok(archive_path)
    }

    #[test]
    fn test_setters_chainability() -> Result<(), Box<dyn Error>> {
        // Create a new instance and call several setters.
        let mut reader = LZMATarballReader::new();
        reader
            .set_archive("dummy_archive.tar.xz")? // using a dummy path for this test
            .set_output_directory("dummy_output")?
            .set_overwrite(true)
            .set_mask(0o644)
            .set_ignore_zeros(true)
            .set_preserve_mtime(true)
            .set_preserve_ownerships(true)
            .set_preserve_permissions(true);

        // If no panic occurs, we assume the chainability works.
        Ok(())
    }

    #[test]
    fn test_entries() -> Result<(), Box<dyn Error>> {
        // Create the test tar.xz archive.
        let archive_path = create_test_tar_xz()?;

        // Create a new reader and set the archive.
        let mut reader = LZMATarballReader::new();
        reader.set_archive(&archive_path)?;

        // Retrieve the list of entries
        let entries = reader.entries()?;

        // Check that the archive contains "hello.txt".
        assert!(
            entries.iter().any(|entry| entry.contains("hello.txt")),
            "The archive entries should contain 'hello.txt'"
        );

        // Clean up the temporary archive file.
        fs::remove_file(archive_path)?;
        Ok(())
    }

    #[test]
    fn test_decompress() -> Result<(), Box<dyn Error>> {
        // Create the test tar.xz archive.
        let archive_path = create_test_tar_xz()?;

        // Create a temporary directory for output.
        let tmp_dir = std::env::temp_dir();
        let output_dir = tmp_dir.join(format!(
            "lzma_tarball_output_{}",
            Utc::now().timestamp_millis()
        ));
        fs::create_dir_all(&output_dir)?;

        // Create a new reader, set the archive and output directory
        let mut reader = LZMATarballReader::new();
        reader
            .set_archive(&archive_path)?
            .set_output_directory(&output_dir)?;

        // Decompress the archive.
        let result: DecompressionResult = reader.decompress()?;

        // Check that some files were decompressed.
        assert!(
            !result.files.is_empty(),
            "There should be at least one decompressed file"
        );

        // Verify that the file "hello.txt" exists in the output directory.
        let output_file_path = output_dir.join("hello.txt");
        assert!(
            output_file_path.exists(),
            "The file 'hello.txt' should exist in the output directory"
        );

        // Read the file content and verify.
        let mut file = File::open(&output_file_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        assert_eq!(
            content, "Hello, world!",
            "The content of 'hello.txt' did not match expected"
        );

        // Optionally: Clean up. (In real tests, you might want to keep the file if debugging.)
        fs::remove_file(archive_path)?;
        fs::remove_file(output_file_path)?;
        fs::remove_dir(output_dir)?;

        // Check that some non-zero elapsed time was recorded.
        assert!(
            result.elapsed_time > Duration::new(0, 0),
            "Elapsed time should be greater than 0"
        );

        // The total size should equal the size of "Hello, world!"
        assert_eq!(
            result.total_size,
            "Hello, world!".len() as u64,
            "Total size should match file content length"
        );

        Ok(())
    }
    
    fn set_working_directory()->anyhow::Result<()> {
        std::env::set_current_dir("../dev-env").unwrap();
    }
}
