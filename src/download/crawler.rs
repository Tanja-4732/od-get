use super::types;
use anyhow::{bail, Result};
// use chrono::prelude::Utc;
// use reqwest;
use scraper::{Html, Selector};
// use serde::Serialize;
// use tokio::{fs, io::AsyncWriteExt};

// Make-shift errors
const DIRECTORY_NOT_FOUND: &'static str = "Couldn't find the directory name";
const CANNOT_PARSE_DIRECTORY: &'static str = "Couldn't parse the directory name";

const EMPTY_SIZE_STRING: &'static str = "  - ";

/// Parses a given HTML-string and extracts the directory and file paths.
/// -  Not recursive
/// -  Does not make requests
pub fn extract_from_html(html_string: &str) -> Result<types::UrlData> {
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

    let mut data = types::UrlData::new(dir_name.to_owned());

    // Iterate over every row
    for row in document.select(row_selector) {
        // Get an iterator over the elements of the row
        let mut row = row.children().skip(1);

        let a = row.next().unwrap().first_child().unwrap();

        let href = a
            .value()
            .as_element()
            .unwrap()
            .attr("href")
            .unwrap()
            .to_owned();

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
            data.sub_dirs.push(types::DirData {
                url: href,
                name,
                last_modified,
                description,
            })
        } else {
            data.files.push(types::FileData {
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

// let res = client.get(url).send().await;
