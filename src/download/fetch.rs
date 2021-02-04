use crate::cli::CliOptions;

use super::types;
use anyhow::{bail, Result};
use std::{future::Future, path::Path, str::FromStr};
use types::Node;
// use chrono::prelude::Utc;
use reqwest::{self, Url};
// use serde::Serialize;
use tokio::{fs, io::AsyncWriteExt};

/// Several counter variables used to keep track of limits
#[derive(Debug, Clone)]
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

pub enum DownloadRecursiveStatus<'a> {
    Done,
    Do(Vec<(&'a Node, &'a CliOptions, &'a reqwest::Client)>),
}

pub async fn download_files_to_dir(
    destination: &Path,
    files: &Vec<&types::FileLinkMetaData>,
    client: &reqwest::Client,
    options: Option<&CliOptions>,
    mut counters: Option<&mut LimitCounts>,
    done_list: &mut Vec<String>,
) -> Result<()> {
    for file in files {
        let temp = Url::from_str(&file.url)?;
        let last_segment = get_last_segment(&temp);

        if done_list.contains(&file.url) {
            println!("(StateStore) Already have file {}", last_segment);
            return Ok(());
        }

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
            .write(true)
            .create(true)
            .open(&file_path)
            .await?;

        // Write the file to disk in chunks as they arrive from the network
        while let Some(chunk) = res.chunk().await? {
            file_handle.write_all(&chunk).await?;
        }

        // Append the file URL to the done_list
        done_list.push(file.url.clone());
    }

    Ok(())
}

pub async fn download_recursive<'a>(
    node: &'a Node,
    options: &'a CliOptions,
    client: &'a reqwest::Client,
    counters: &'a mut LimitCounts,
    done_list: &mut Vec<String>,
) -> Result<DownloadRecursiveStatus<'a>> {
    // ) -> Box<dyn Future<Output = ()>> {
    // Pin<Box<dyn Future<Output = Result<()>>>>
    // Pin<Box<dyn Future<Output = ()>>>

    if let Node::CrawledDir(meta, children) = node {
        // If no download is desired, skip the download
        if options.no_download {
            println!("Skipped download");
            return Ok(DownloadRecursiveStatus::Done);
        };

        // Increment the recursion depth
        counters.recursion_depth += 1;

        // The folder name from the server
        let server_path = meta.name.split('/').last().expect("Can't split");

        // Skip unwanted folders
        if let Some(regex) = &options.path_filter {
            if regex.is_match(&server_path) {
                println!("(Filter) Skip directory {}", server_path);
                return Ok(DownloadRecursiveStatus::Done);
            }
        }

        // Only download wanted folders
        if let Some(regex) = &options.path_matcher {
            if !regex.is_match(&server_path) {
                println!("(Matcher) Skip directory {}", server_path);
                return Ok(DownloadRecursiveStatus::Done);
            }
        }

        let mut url = options.url.clone();
        url.path_segments_mut()
            .unwrap()
            // TODO improve this somehow
            .pop_if_empty()
            .pop_if_empty()
            .pop_if_empty()
            .pop_if_empty()
            .pop_if_empty()
            .pop_if_empty()
            .pop_if_empty()
            .pop_if_empty();

        let last_segment = url.path_segments().unwrap().last().unwrap();

        // Create the directory (if it doesn't exist)
        let folder_path = Path::new(&options.destination)
            .join(last_segment)
            .join(server_path);

        println!("{}", folder_path.to_str().unwrap());

        fs::create_dir_all(&folder_path).await?;

        // Make a list of files
        let mut files = vec![];

        for node in children {
            if let Node::File(file) = node {
                files.push(file);
            }
        }

        // Download all the files (if they pass the filters)
        download_files_to_dir(
            &folder_path,
            &files,
            client,
            Some(options),
            Some(counters),
            done_list,
        )
        .await?;

        // A list of tuples containing arguments which which this function should be called again
        let mut to_do = vec![];

        // Iterate over the sub directories
        for directory in children {
            if let Node::CrawledDir(_, _) = directory {
                // Stop if the recursion limit is reached
                if let Some(rec_limit) = options.recursion_limit {
                    if counters.recursion_depth >= rec_limit {
                        println!("Reached recursion limit at {}", counters.recursion_depth);
                        return Ok(DownloadRecursiveStatus::Done);
                    }
                }

                if let Some(file_limit) = options.limit_count {
                    if counters.file_count >= file_limit {
                        println!("File limit reached at {} files", counters.file_count);
                        return Ok(DownloadRecursiveStatus::Done);
                    }
                }

                // TODO await the recursion (or maybe keep the return vector)
                // (*(download_recursive(directory, options, client, counters).await))?;

                // Start the next recursive iteration
                to_do.push((directory, options, client));
            } else if let Node::PendingDir(directory) = directory {
                let mut url = Url::from_str(&directory.url)?;
                url.path_segments_mut()
                    .unwrap()
                    // TODO improve this somehow
                    .pop_if_empty()
                    .pop_if_empty()
                    .pop_if_empty()
                    .pop_if_empty()
                    .pop_if_empty()
                    .pop_if_empty()
                    .pop_if_empty()
                    .pop_if_empty();

                let last_segment = url.path_segments().unwrap().last().unwrap();

                println!("(Skip) Directory not initialized: {}", last_segment);
            }
        }

        // Return the to_do list of tuples containing arguments with which this function should be called again
        if to_do.len() == 0 {
            Ok(DownloadRecursiveStatus::Done)
        } else {
            Ok(DownloadRecursiveStatus::Do(to_do))
        }
    } else {
        bail!("Cannot work with pending directory")
    }
}

/// Returns a reference to the last segment of a given URL as a &str
#[deprecated]
fn get_last_segment(url: &Url) -> &str {
    url.path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or_else(|| {
            // TODO this might need to be fixed
            // (url.clone().path_segments_mut().unwrap().pop_if_empty());
            // get_last_segment(url);
            return "unknown_segment";
        })

    // TODO Maybe provide a fallback
    // See https://rust-lang-nursery.github.io/rust-cookbook/web/clients/download.html
}
