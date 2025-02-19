// tests for the LZMATarballReader
#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::env::current_dir;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    // Import the reader from your library. Adjust the path as needed.
    use lzma_tarball::reader::LZMATarballReader;

    #[test]
    fn test_extract_to_directory() {
        let archive_file = setup_testing_environment().unwrap();
        let mut reader = LZMATarballReader::new();
        reader.set_output_directory("output").unwrap();
        reader.set_overwrite(true);
        reader.set_archive(archive_file.clone()).unwrap();
        reader.decompress().unwrap();

        let extracted_file = Path::new("output/hello.txt");
        assert!(extracted_file.exists());
        let extracted_file_contents = fs::read_to_string(extracted_file.to_str().unwrap()).unwrap();
        assert_eq!(extracted_file_contents, "Hello, world!");
    }

    #[test]
    fn test_read_entries(){
        let archive_file = setup_testing_environment().unwrap();
        let mut reader = LZMATarballReader::new();
        reader.set_archive(archive_file.clone()).unwrap();
        let entries = reader.entries().unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], "hello.txt");
    }
    
    

    fn create_test_tar_xz() -> Result<PathBuf> {
        let mut archive_path = current_dir()?;
        archive_path.push("test.tar.xz");
        let mut writer = lzma_tarball::writer::LZMATarballWriter::new();
        writer.set_compression_level(1);
        writer.set_output(archive_path.clone());
        writer.with_file("./hello.txt", "/hello.txt");
        writer.compress(|_| {})?;

        Ok(archive_path)
    }

    fn setup_testing_environment() -> Result<PathBuf> {
        let dir = "./dev-env";
        fs::create_dir_all(dir)?;
        std::env::set_current_dir(dir)?;

        let mut test_file = File::create("./hello.txt")?;
        test_file.write_all(b"Hello, world!")?;
        test_file.sync_all()?;

        create_test_tar_xz()
    }
}
