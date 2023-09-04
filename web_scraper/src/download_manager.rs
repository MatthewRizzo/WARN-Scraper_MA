//! File with structs representing ownership over downloading and managing the
//! existence of the files created

use std::{fs, fs::File, io, path::PathBuf};

use crate::error::{ScraperError, ScraperResult};
use reqwest;

/// Struct managing the downloading from a url, creation of the file, and
/// deletion of said file
pub(crate) struct Downloader {
    download_url: String,
    downloaded_file_path: PathBuf,
}

impl Drop for Downloader {
    /// Make sure to delete the downloaded file
    fn drop(&mut self) {
        self.delete_file();
    }
}

impl Downloader {
    pub(crate) fn new(download_url: String, downloaded_file_path: PathBuf) -> Downloader {
        Downloader {
            download_url,
            downloaded_file_path,
        }
    }
    pub(crate) fn download_file(&self) -> ScraperResult<()> {
        let mut download_response = reqwest::blocking::get(&self.download_url)?;
        let mut output_file = File::create(&self.downloaded_file_path)?;
        let _bytes_copied = io::copy(&mut download_response, &mut output_file)?;
        println!(
            "Downloaded yearly update file from {} to {}",
            self.download_url,
            self.downloaded_file_path.display()
        );
        Ok(())
    }

    pub fn get_path_to_file(&self) -> &PathBuf {
        &self.downloaded_file_path
    }

    fn delete_file(&mut self) {
        if self.downloaded_file_path.as_path().exists() {
            fs::remove_file(self.downloaded_file_path.clone()).unwrap();
        }
    }
}
