use std::str::FromStr;

use super::types::{DirLinkMetaData, FileLinkMetaData, Node};
use anyhow::{anyhow, bail, Result};
use html_escape::{decode_html_entities_to_string, decode_html_entities_to_vec};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use reqwest::{self, Url};
use scraper::{ElementRef, Html, Selector};
// use serde::Serialize;

// Make-shift errors
const DIRECTORY_NOT_FOUND_1: &'static str = "Couldn't find the directory name_1";
const DIRECTORY_NOT_FOUND_2: &'static str = "Couldn't find the directory name_2";
const CANNOT_PARSE_DIRECTORY: &'static str = "Couldn't parse the directory name";
const EMPTY_RESPONSE: &'static str = "Got a empty response";

const EMPTY_SIZE_STRING: &'static str = "  - ";

pub const POS_HREF: usize = 1;
pub const POS_NAME: usize = 2;
pub const POS_DATE: usize = 3;
pub const POS_SIZE: usize = 4;
pub const POS_DESC: usize = 5;

lazy_static! {
    /// This is an example for using doc comment attributes
    static ref RX_MAIN: Regex = Regex::new(
        "</td><td><a href=\"(.+?)\">(.+?)</a></td><td align=\"right\">(.+?)  </td><td align=\"right\">(.+?)</td><td>(.+?)</td></tr>"
    ).unwrap();

    /// This is an example for using doc comment attributes
    static ref RX_PARENT: Regex = Regex::new(
        "</td><td><a href=\"/(.+?)/\">Parent Directory</a>       </td><td> </td><td align=\"right\">  - </td><td> </td></tr>"
    ).unwrap();

    /// This is an example for using doc comment attributes
    static ref RX_TITLE: Regex = Regex::new("<h1>Index of (.+?)</h1>").unwrap();
}

/**
Returns the first match in a string with a given Regex pattern
*/
fn get_first<'a>(text: &'a str, regex: &Regex) -> Result<&'a str> {
    Ok(regex
        .captures(text)
        .ok_or(anyhow!(CANNOT_PARSE_DIRECTORY))?
        .get(1)
        .ok_or(anyhow!(CANNOT_PARSE_DIRECTORY))?
        .as_str())
}

/**
Parses a given HTML-string and extracts the directory and file paths.

-  Not recursive
-  Does not make requests

Returns a tuple containing the extracted name and the vector of extracted nodes.
*/
pub fn cheap_extract_from_html(html: &str, base_url: &Url) -> Result<(String, Vec<Node>)> {
    let dir_name = get_first(html, &RX_TITLE)?;

    // TODO maybe use the parent_href in the future
    // let parent_href = get_first(html, &RX_PARENT)?;

    // Split the string into lines
    let nodes = html
        .par_lines()
        .filter_map(cheap_process_row(base_url))
        .collect();

    Ok((dir_name.to_owned(), nodes))
}

/**
Turns an ElementRef (of a HTML table-row into a node (Either PendingDir or File)
*/
pub fn cheap_process_row<'a>(
    base_url: &'a Url,
) -> Box<dyn Fn(&str) -> Option<Node> + Send + Sync + 'a> {
    Box::new(move |line| {
        let captures = RX_MAIN.captures(line)?;

        // Calculate the absolute href using the base_url
        let mut href = base_url
            .join(captures.get(POS_HREF)?.as_str())
            .to_owned()
            .ok()?;

        // The other values get extracted using the regex
        let name = captures.get(POS_NAME)?.as_str().to_owned();
        let last_modified = captures.get(POS_DATE)?.as_str().to_owned();
        let size = captures.get(POS_SIZE)?.as_str().to_owned();
        let description = captures.get(POS_DESC)?.as_str().to_owned();

        // Check if the result is a directory (by examining its stated size)
        if captures.get(POS_SIZE)?.as_str() == EMPTY_SIZE_STRING {
            // TODO re-introduce count
            // println!("Got directory ({:4}): {}", nodes.len(), &name);
            println!("Got directory: {}", &name);

            Some(Node::PendingDir(DirLinkMetaData {
                url: href.to_string(),
                name,
                last_modified,
                description,
            }))
        } else {
            clean_url(&mut href);

            // TODO re-introduce count
            // println!("Got file ({:4}): {}", nodes.len(), &name);
            println!("Got file: {}", &name);
            println!("{}\n", &href);

            Some(Node::File(FileLinkMetaData {
                url: href.to_string(),
                name,
                last_modified,
                size,
                description,
            }))
        }
    })

    // unimplemented!()
}

/**
Parses a given HTML-string and extracts the directory and file paths.

-  Not recursive
-  Does not make requests

Returns a tuple containing the extracted name and the vector of extracted nodes.
*/
#[deprecated]
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
            None => bail!(DIRECTORY_NOT_FOUND_1),
        },
        None => bail!(DIRECTORY_NOT_FOUND_2),
    };

    let mut nodes = vec![];

    // Match all rows
    let mut selected = document.select(row_selector);

    // Group the rows in 100-long batches
    let mut run = true;
    while run {
        let mut batch = vec![];

        while let Some(next) = selected.next() {
            if batch.len() == 100 {
                break;
            } else {
                batch.push(next);
            }
        }

        // Check, if the loop should run again
        run = batch.len() != 0;

        // TODO Make multi-threaded
        // tokio::spawn(async move { // Multi-threaded open
        let mut batch_out = vec![];
        for row in batch {
            print!("({:4}) ", nodes.len() + batch_out.len());
            batch_out.push(process_row(row, &base_url)?);
        }
        nodes.append(&mut batch_out);
        // }); // Multi-threaded close
    }

    Ok((dir_name, nodes))
}

/**
Turns an ElementRef (of a HTML table-row into a node (Either PendingDir or File)
*/
#[deprecated]
fn process_row(row: ElementRef, base_url: &Url) -> Result<Node> {
    // Get an iterator over the elements of the row
    let mut row = row.children().skip(1);

    let a = match row.next() {
        Some(element) => element.first_child().unwrap(),
        // None => break,
        None => bail!("No next element"),
    };

    let mut href = base_url
        .join(a.value().as_element().unwrap().attr("href").unwrap())
        .to_owned()?;

    // clean_url(&mut href);

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
        // TODO re-introduce count
        // println!("Got directory ({:4}): {}", nodes.len(), &name);
        println!("Got directory: {}", &name);

        Ok(Node::PendingDir(DirLinkMetaData {
            url: href.to_string(),
            name,
            last_modified,
            description,
        }))
    } else {
        clean_url(&mut href);

        // TODO re-introduce count
        // println!("Got file ({:4}): {}", nodes.len(), &name);
        println!("Got file: {}", &name);
        println!("{}\n", &href);

        Ok(Node::File(FileLinkMetaData {
            url: href.to_string(),
            name,
            last_modified,
            size,
            description,
        }))
    }
}

/**
Expand all PengingDir nodes
*/
pub async fn expand_node(nodes: &mut Vec<Node>, client: &reqwest::Client) -> Result<()> {
    for node in nodes {
        // Only crawl if needed
        if let Node::PendingDir(dir) = node {
            println!("Now crawling: {}", dir.name);
            let req = client.get(&dir.url).send();

            // Get the HTML from the server
            let html = match req.await {
                Ok(res) => sanitize_html(&res.text().await.expect(EMPTY_RESPONSE))?,
                Err(err) => bail!(err),
            };

            // Perse the response
            match cheap_extract_from_html(&html, &Url::from_str(&dir.url)?) {
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
    println!("Fetching root HTML");

    let res = client
        .get(url.as_str())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    // Sanitize the HTML
    let html = sanitize_html(&res)?;

    println!("Crawling root URL");

    let root_data = cheap_extract_from_html(&html, url)?;

    Ok(Node::CrawledDir(
        DirLinkMetaData {
            url: url.to_string(),
            name: root_data.0,
            description: String::new(),
            last_modified: String::new(),
        },
        root_data.1,
    ))
}

/**
Sanitize the HTML (use String::from_utf8 to check for valid UTF-8, the library is unsafe)
*/
pub fn sanitize_html(text: &str) -> Result<String> {
    let mut output = Vec::new();
    decode_html_entities_to_vec(text, &mut output);
    Ok(String::from_utf8(output)?)
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
