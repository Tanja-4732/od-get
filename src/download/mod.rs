use chrono::prelude::Utc;
use reqwest;
use scraper::{Html, Selector};
use serde::Serialize;
use tokio::{fs, io::AsyncWriteExt};

pub mod types;

/// Parses the HTML at a given URL and extracts the directory and file paths.
/// Not recursive
pub fn crawl_url(url: &str) -> Result<types::UrlData, anyhow::Error> {}
