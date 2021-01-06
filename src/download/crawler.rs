use super::types;
use anyhow::{bail, Result};
use std::path::Path;
// use chrono::prelude::Utc;
use reqwest::{self, Url};
use scraper::{Html, Selector};
// use serde::Serialize;
use tokio::{fs, io::AsyncWriteExt};

// Make-shift errors
const DIRECTORY_NOT_FOUND: &'static str = "Couldn't find the directory name";
const CANNOT_PARSE_DIRECTORY: &'static str = "Couldn't parse the directory name";
const EMPTY_RESPONSE: &'static str = "Got a empty response";

const EMPTY_SIZE_STRING: &'static str = "  - ";

/// Parses a given HTML-string and extracts the directory and file paths.
/// -  Not recursive
/// -  Does not make requests
pub fn extract_from_html(html_string: &str, base_url: &Url) -> Result<types::DirData> {
    // let temp_url = Url::parse("/").expect("Invalid base");
    // let base_url = base_url.unwrap_or(&temp_url);

    let dir_selector = &Selector::parse("body > h1").unwrap();
    let row_selector = &Selector::parse("body > table > tbody > tr:nth-child(3) ~ tr").unwrap();

    let document = Html::parse_document(html_string);

    let dir_name = match document.select(dir_selector).next() {
        Some(element) => match element.text().next() {
            Some(text) => {
                if text.starts_with("Index of ") {
                    text.split_at(9).1
                } else {
                    bail!(CANNOT_PARSE_DIRECTORY)
                }
            }
            None => bail!(DIRECTORY_NOT_FOUND),
        },
        None => bail!(DIRECTORY_NOT_FOUND),
    };

    let mut data = types::DirData::new(dir_name.to_owned());

    // Iterate over every row
    for row in document.select(row_selector) {
        // Get an iterator over the elements of the row
        let mut row = row.children().skip(1);

        let a = match row.next() {
            Some(element) => element.first_child().unwrap(),
            None => break,
        };

        let href = base_url
            .join(a.value().as_element().unwrap().attr("href").unwrap())
            .to_owned()?;

        let name = a
            .first_child()
            .unwrap()
            .value()
            .as_text()
            .unwrap()
            .to_string();

        let last_modified = row
            .next()
            .unwrap()
            .first_child()
            .unwrap()
            .value()
            .as_text()
            .unwrap()
            .to_string();

        let size = row
            .next()
            .unwrap()
            .first_child()
            .unwrap()
            .value()
            .as_text()
            .unwrap()
            .to_string();

        let description = row
            .next()
            .unwrap()
            .first_child()
            .unwrap()
            .value()
            .as_text()
            .unwrap()
            .to_string();

        // Check if the result is a directory
        if size == EMPTY_SIZE_STRING {
            data.sub_dirs.push((
                types::DirLinkMetaData {
                    url: href,
                    name,
                    last_modified,
                    description,
                },
                None,
            ))
        } else {
            data.files.push(types::FileLinkMetaData {
                url: href,
                name,
                last_modified,
                size,
                description,
            })
        }
    }

    Ok(data)
}

/// Takes a node (DirData) and crawls all its children (not recursive)
pub async fn expand_tree(dir_data: &mut types::DirData, client: &reqwest::Client) -> Result<()> {
    for sub_dir in &mut dir_data.sub_dirs {
        // Only crawl if needed
        if sub_dir.1.is_none() {
            let req = client.get(sub_dir.0.url.as_str()).send();

            // Get the HTML from the server
            let html = match req.await {
                Ok(res) => res.text().await.expect(EMPTY_RESPONSE),
                Err(err) => bail!(err),
            };

            // Perse the response
            match extract_from_html(&html, &sub_dir.0.url) {
                Ok(sub_dir_data) => sub_dir.1 = Some(sub_dir_data),
                Err(err) => bail!(err),
            };
        }
    }

    Ok(())
}

pub async fn download_files(
    destination: &Path,
    files: &Vec<types::FileLinkMetaData>,
    client: &reqwest::Client,
) -> Result<()> {
    for file in files {
        println!("Downloading file {}", file.name);

        // Request the file from the server
        let req = client.get(file.url.as_str()).send();

        let mut res = req.await?;

        let last_segment = res
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap();
        // TODO Maybe provide a fallback
        // See https://rust-lang-nursery.github.io/rust-cookbook/web/clients/download.html

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

pub async fn depth_first_crawl() {}

// pub async fn breadth_first_crawl(root_dir_data: &mut types::DirData, client: &reqwest::Client) {
//     expand_tree(root_dir_data, client);

// }

pub async fn get_root_dir(url: &Url, client: &reqwest::Client) -> Result<types::DirData> {
    let res = client
        .get(url.as_str())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let dir_data = extract_from_html(&res, url)?;

    Ok(dir_data)
}
