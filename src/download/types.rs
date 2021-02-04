use anyhow::{bail, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/**
This enum defines an entry in an open-directory listing.

- `File` Represents a link to a file to be downloaded
- `PendingDir` Represents a link to a directory to be crawled
- `CrawledDir` Represents a link to a crawled directory
*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Node {
    /// This node is a file
    File(FileLinkMetaData),

    /// This node is an un-crawled directory
    PendingDir(DirLinkMetaData),

    /// This node is a crawled directory
    CrawledDir(DirLinkMetaData, Vec<Node>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileLinkMetaData {
    pub url: String,
    pub name: String,
    pub last_modified: String,
    pub size: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirLinkMetaData {
    pub url: String,
    pub name: String,
    pub last_modified: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CrawlingState {
    Complete(Node),
    Partial(Node),
    None,
}

/**

*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateStore {
    pub created_at: String,
    pub last_modified: String,
    pub crawling_state: CrawlingState,

    // TODO use borrowed string slices instead
    pub downloaded_urls: Vec<String>,
}

impl StateStore {
    pub const VERSION: u64 = 1;

    pub fn new() -> Self {
        let now = Utc::now().to_rfc3339();

        Self {
            created_at: now.clone(),
            last_modified: now,
            crawling_state: CrawlingState::None,
            downloaded_urls: vec![],
        }
    }

    pub fn update_modified_time(&mut self) {
        self.last_modified = Utc::now().to_rfc3339();
    }

    pub fn get_root_ref(&self) -> Result<&Node> {
        if let CrawlingState::Complete(root) = &self.crawling_state {
            Ok(root)
        } else {
            bail!("Crawl is not complete")
        }
    }

    pub fn get_root_ref_mut(&mut self) -> Result<&mut Node> {
        if let CrawlingState::Complete(root) = &mut self.crawling_state {
            Ok(root)
        } else {
            bail!("Crawl is not complete")
        }
    }
}
