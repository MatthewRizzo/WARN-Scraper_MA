//! File with structs representing ownership over downloading and managing the
//! existence of the files created
use std::fs::DirEntry;
use std::process::Command;
use std::{fs, path::PathBuf, str::FromStr};

use crate::error::{ScraperError, ScraperResult};

/// Struct managing the downloading from a url, creation of the file, and
/// deletion of said file
pub(crate) struct DownloaderWrapper<'a> {
    download_url: String,
    downloaded_file_path: &'a PathBuf,
    download_directory: &'a PathBuf,
    is_verbose: bool,
}

impl<'a> Drop for DownloaderWrapper<'a> {
    /// Make sure to delete the downloaded file
    /// leave no trace of the download
    fn drop(&mut self) {
        self.delete_file();
        self.delete_download_dir();
    }
}

impl<'a> DownloaderWrapper<'a> {
    pub(crate) fn new(
        download_url: String,
        downloaded_file_path: &'a PathBuf,
        download_directory: &'a PathBuf,
        is_verbose: bool,
    ) -> DownloaderWrapper<'a> {
        DownloaderWrapper {
            download_url,
            downloaded_file_path,
            download_directory,
            is_verbose,
        }
    }
    pub(crate) fn download_file(&self) -> ScraperResult<()> {
        if self.download_directory.exists() {
            fs::remove_dir_all(self.download_directory)?;
        }
        fs::create_dir_all(self.download_directory)?;

        let curl_location = Self::get_path_to_curl_exe()?;

        // curling a .xslx file downgrades it to a zip file unless the -J flag
        // is used in tandem with -O to force the usage of the page's headers.
        // Side affects include the file having a garbage name.
        let output = Command::new(curl_location)
            .current_dir(self.download_directory)
            .arg("-O")
            .arg("-J")
            .arg("https://www.mass.gov/doc/warn-report-for-the-week-ending-august-25-2023/download")
            .output()
            .map_err(|err| {
                ScraperError::Downloading(format!(
                    "Error running curl command: {}",
                    err.to_string()
                ))
            })?;

        if self.is_verbose {
            println!("Curl output:");
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("{}", String::from_utf8_lossy(&output.stderr));
        }

        let tmp_download_location = self.get_path_to_file_downloaded();
        if self.is_verbose {
            println!(
                "Downloaded yearly update file from {} to {}",
                self.download_url,
                &tmp_download_location.display()
            );
        }

        let move_str: String = format!(
            "Moving {} to {}",
            &tmp_download_location.display(),
            self.downloaded_file_path.display()
        );

        if self.is_verbose {
            println!("{}", move_str);
        }

        fs::rename(&tmp_download_location, self.downloaded_file_path).map_err(|err| {
            ScraperError::Downloading(format!(
                "Error renaming file from {} to {}: {}",
                tmp_download_location.display(),
                self.downloaded_file_path.display(),
                err.to_string()
            ))
        })?;

        Ok(())
    }

    fn delete_file(&mut self) {
        if self.downloaded_file_path.as_path().exists() {
            fs::remove_file(self.downloaded_file_path.clone()).unwrap();
        }
    }

    fn delete_download_dir(&mut self) {
        if self.download_directory.as_path().exists() {
            fs::remove_dir(self.download_directory.clone()).unwrap();
        }
    }

    fn get_path_to_curl_exe() -> ScraperResult<PathBuf> {
        if cfg!(windows) {
            Self::generic_get_path_to_exe("where")
        } else if cfg!(linux) {
            Self::generic_get_path_to_exe("which")
        } else {
            PathBuf::from_str("/usr/bin/curl").map_err(|err| {
                ScraperError::Downloading(format!(
                    "Error converting string to pathbuf for curl command: {}",
                    err.to_string()
                ))
            })
        }
    }

    /// Generic function to run a command and find the path to a given exe
    /// # Param
    /// * cmd_name - the OS specifc program to run that gets paths to exe's.
    ///     i.e. which for linux and where for windows
    fn generic_get_path_to_exe(cmd_name: &str) -> ScraperResult<PathBuf> {
        let cmd_handler = Command::new(cmd_name).arg("curl").spawn().map_err(|err| {
            ScraperError::Downloading(format!("Error running curl: {}", err.to_string()))
        })?;

        let output = cmd_handler.wait_with_output().map_err(|err| {
            ScraperError::Downloading(format!(
                "Error waiting for command {}to get path to curl command to finish: {}",
                cmd_name,
                err.to_string()
            ))
        })?;

        let path_str = String::from_utf8(output.stdout).expect("Error converting output to string");
        PathBuf::from_str(path_str.as_str()).map_err(|err| {
            ScraperError::Downloading(format!(
                "Error getting path to curl command exe: {}",
                err.to_string()
            ))
        })
    }

    /// The file downloaded tends to have a random name with escaped characters
    /// in html format (i.e. %20). This is an unavoidable side affect of
    /// curling a .xsxl file.
    /// Retrieve the path to the file (including its name) so we can rename it
    fn get_path_to_file_downloaded(&self) -> PathBuf {
        let valid_xlsx_files = fs::read_dir(&self.download_directory)
            .expect(
                format!(
                    "Couldn't read the directory {}",
                    &self.download_directory.display()
                )
                .as_str(),
            )
            .filter_map(|file| match file {
                Err(_) => None,
                Ok(entry) => Self::is_xlsx(entry).unwrap(),
            })
            .max_by_key(|entry| entry.metadata().unwrap().modified().unwrap())
            .expect("Last modified file not found");

        valid_xlsx_files.path()
    }

    /// returns the entry if it is an xlsx, None otherwise
    fn is_xlsx(entry: DirEntry) -> ScraperResult<Option<DirEntry>> {
        let file_pathbuf =
            PathBuf::from_str(entry.file_name().to_str().unwrap()).map_err(|err| {
                ScraperError::Downloading(format!(
                    "Error converting file OsStr to a path buffer: {}",
                    err.to_string()
                ))
            })?;
        let extension = file_pathbuf.as_path().extension();

        match extension {
            None => Ok(None),
            Some(ext) => {
                if ext.to_str() == Some("xlsx") {
                    Ok(Some(entry))
                } else {
                    Ok(None)
                }
            }
        }
    }
}
