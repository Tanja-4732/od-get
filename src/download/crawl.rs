use super::types::{DirLinkMetaData, FileLinkMetaData, Node};
use anyhow::{bail, Result};
use reqwest::{self, Url};
use scraper::{Html, Selector};
// use serde::Serialize;

// Make-shift errors
const DIRECTORY_NOT_FOUND: &'static str = "Couldn't find the directory name";
const CANNOT_PARSE_DIRECTORY: &'static str = "Couldn't parse the directory name";
const EMPTY_RESPONSE: &'static str = "Got a empty response";

const EMPTY_SIZE_STRING: &'static str = "  - ";

/**
Parses a given HTML-string and extracts the directory and file paths.

-  Not recursive
-  Does not make requests

Returns a tuple containing the extracted name and the vector of extracted nodes.
*/
pub fn extract_from_html(html_string: &str, base_url: &Url) -> Result<(String, Vec<Node>)> {
    // let temp_url = Url::parse("/").expect("Invalid base");
    // let base_url = base_url.unwrap_or(&temp_url);

    let dir_selector = &Selector::parse("body > h1").unwrap();
    let row_selector = &Selector::parse("body > table > tbody > tr:nth-child(3) ~ tr").unwrap();

    let document = Html::parse_document(html_string);

    let dir_name = match document.select(dir_selector).next() {
        Some(element) => match element.text().next() {
            Some(text) => {
                if text.starts_with("Index of ") {
                    text.split_at(9).1.to_owned()
                } else {
                    bail!(CANNOT_PARSE_DIRECTORY)
                }
            }
            None => bail!(DIRECTORY_NOT_FOUND),
        },
        None => bail!(DIRECTORY_NOT_FOUND),
    };

    let mut nodes = vec![];

    // Iterate over every row
    for row in document.select(row_selector) {
        // Get an iterator over the elements of the row
        let mut row = row.children().skip(1);

        let a = match row.next() {
            Some(element) => element.first_child().unwrap(),
            None => break,
        };

        let mut href = base_url
            .join(a.value().as_element().unwrap().attr("href").unwrap())
            .to_owned()?;

        clean_url(&mut href);

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
            nodes.push(Node::PendingDir(DirLinkMetaData {
                url: href,
                name,
                last_modified,
                description,
            }))
        } else {
            nodes.push(Node::File(FileLinkMetaData {
                url: href,
                name,
                last_modified,
                size,
                description,
            }))
        }
    }

    Ok((dir_name, nodes))
}

/**
Expand all PengingDir nodes
*/
pub async fn expand_node(nodes: &mut Vec<Node>, client: &reqwest::Client) -> Result<()> {
    for node in nodes {
        // Only crawl if needed
        if let Node::PendingDir(dir) = node {
            println!("Now crawling: {}", dir.name);
            let req = client.get(dir.url.clone()).send();

            // Get the HTML from the server
            let html = match req.await {
                Ok(res) => res.text().await.expect(EMPTY_RESPONSE),
                Err(err) => bail!(err),
            };

            // Perse the response
            match extract_from_html(&html, &dir.url) {
                Err(err) => bail!(err),
                Ok(dir_data) => {
                    // Replace the PendingDir node with a CrawledDir one
                    *node = Node::CrawledDir(
                        DirLinkMetaData {
                            url: dir.url.clone(), // TODO remove copy
                            name: dir_data.0,
                            description: dir.description.clone(), // TODO remove copy
                            last_modified: dir.last_modified.clone(), // TODO remove copy
                        },
                        dir_data.1,
                    )
                }
            };
        }
    }

    Ok(())
}

/**
Extracts the HTML from the root URL and returns a node
*/
pub async fn get_root_dir(url: &Url, client: &reqwest::Client) -> Result<Node> {
    println!("Crawling root URL");

    let res = client
        .get(url.as_str())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let root_data = extract_from_html(&res, url)?;

    Ok(Node::CrawledDir(
        DirLinkMetaData {
            url: url.clone(),
            name: root_data.0,
            description: String::new(),
            last_modified: String::new(),
        },
        root_data.1,
    ))
}

/// Clear a lot of trailing slashes
fn clean_url(url: &mut Url) -> () {
    // TODO Improve this
    url.path_segments_mut()
        .expect("Cannot use URL")
        .pop_if_empty()
        .pop_if_empty()
        .pop_if_empty()
        .pop_if_empty()
        .pop_if_empty()
        .pop_if_empty()
        .pop_if_empty()
        .pop_if_empty()
        .pop_if_empty()
        .pop_if_empty()
        .pop_if_empty()
        .pop_if_empty()
        .pop_if_empty()
        .pop_if_empty()
        .pop_if_empty()
        .pop_if_empty()
        .pop_if_empty();
}
