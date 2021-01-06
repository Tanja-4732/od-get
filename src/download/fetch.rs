use crate::cli::CliOptions;

use super::types;
use anyhow::{bail, Result};
use std::path::Path;
// use chrono::prelude::Utc;
use reqwest::{self, Url};
// use serde::Serialize;
use tokio::{fs, io::AsyncWriteExt};

pub struct LimitCounts {
    recursion_depth: u64,
    file_count: u64,
    skipped_files: u64,
}

impl LimitCounts {
    pub fn new() -> Self {
        Self {
            recursion_depth: 0,
            file_count: 0,
            skipped_files: 0,
        }
    }
}

pub async fn download_files_to_dir(
    destination: &Path,
    files: &Vec<types::FileLinkMetaData>,
    client: &reqwest::Client,
    options: Option<&CliOptions>,
    mut counters: Option<&mut LimitCounts>,
) -> Result<()> {
    for file in files {
        let last_segment = get_last_segment(&file.url);

        // Follow options (if specified)
        if let Some(options) = options {
            if let Some(counters) = &mut counters {
                // Check for the download limit
                if let Some(file_limit) = options.limit_count {
                    if counters.file_count >= file_limit {
                        return Ok(());
                    }
                }

                // Skip unwanted files
                if let Some(regex) = &options.file_filter {
                    if regex.is_match(last_segment) {
                        // println!("(Filter) Skip file {} ({})", last_segment, file.name);
                        println!("(Filter) Skip file {}", last_segment);
                        continue;
                    }
                }

                // Only download wanted files
                if let Some(regex) = &options.file_matcher {
                    if !regex.is_match(last_segment) {
                        // println!("(Matcher) Skip file {} ({})", last_segment, file.name);
                        println!("(Matcher) Skip file {}", last_segment);
                        continue;
                    }
                }

                // Skip files if desired
                if let Some(skip) = options.skip_count {
                    if counters.skipped_files < skip {
                        counters.skipped_files += 1;
                        continue;
                    }
                }

                // Increment download counter
                counters.file_count += 1;
            } else {
                panic!("Cannot specify only one of `client` or `options` (need both or niether)");
            }
        } else {
            if counters.is_some() {
                panic!("Cannot specify only one of `client` or `options` (need both or niether)");
            }
        }

        // println!("Downloading file {} ({})", last_segment, file.name);
        println!("Downloading file {}", last_segment);

        // Request the file from the server
        let req = client.get(file.url.as_str()).send();

        let mut res = req.await?;

        // Obtain the last segment from the server to follow redirects
        let last_segment = get_last_segment(res.url());

        let file_path = destination.join(last_segment);

        // Use Tokio to open the target file
        let mut file_handle = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .await?;

        // Write the file to disk in chunks as they arrive from the network
        while let Some(chunk) = res.chunk().await? {
            file_handle.write_all(&chunk).await?;
        }
    }

    Ok(())
}

pub async fn download_recursive(
    root_dir: &types::DirData,
    options: &CliOptions,
    client: &reqwest::Client,
    counters: &mut LimitCounts,
) -> Result<()> {
    // Pin<Box<dyn Future<Output = Result<()>>>>
    // Pin<Box<dyn Future<Output = ()>>>

    // If no download is desired, skip the download
    if options.no_download {
        println!("Skipped download");
        return Ok(());
    };

    // Increment the recursion depth
    counters.recursion_depth += 1;

    // The folder name from the server
    let server_path = root_dir.path.split('/').last().expect("Can't split");

    // Skip unwanted folders
    if let Some(regex) = &options.path_filter {
        if regex.is_match(&server_path) {
            println!("(Filter) Skip directory {}", server_path);
            return Ok(());
        }
    }

    // Only download wanted folders
    if let Some(regex) = &options.path_matcher {
        if !regex.is_match(&server_path) {
            println!("(Matcher) Skip directory {}", server_path);
            return Ok(());
        }
    }

    // Create the directory (if it doesn't exist)
    let folder_path = Path::new(&options.destination).join(server_path);
    fs::create_dir_all(&folder_path).await?;

    // Download all the files (if they pass the filters)
    download_files_to_dir(
        &folder_path,
        &root_dir.files,
        client,
        Some(options),
        Some(counters),
    )
    .await?;

    // Iterate over the sub directories
    for directory in &root_dir.sub_dirs {
        let dir = &directory.1;
        if dir.is_some() {
            // Stop if the recursion limit is reached
            if let Some(rec_limit) = options.recursion_limit {
                if counters.recursion_depth >= rec_limit {
                    println!("Reached recursion limit at {}", counters.recursion_depth);
                    return Ok(());
                }
            }

            if let Some(file_limit) = options.limit_count {
                if counters.file_count >= file_limit {
                    println!("File limit reached at {} files", counters.file_count);
                    return Ok(());
                }
            }

            // TODO await the recursion
            // download_recursive(root_dir, options, client, counters).await?;

            // Start the next recursive iteration
            download_recursive(root_dir, options, client, counters);
        } else {
            bail!(
                "Directory not initialized: {}",
                get_last_segment(&directory.0.url)
            );
        }
    }

    return Ok(());
}

/// Returns a reference to the last segment of a given URL as a &str
fn get_last_segment(url: &Url) -> &str {
    url.path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or_else(|| panic!("Cannot parse last segment of url: {}", url))

    // TODO Maybe provide a fallback
    // See https://rust-lang-nursery.github.io/rust-cookbook/web/clients/download.html
}
