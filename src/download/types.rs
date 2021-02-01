/**
This enum defines an entry in an open-directory listing.

- `File` Represents a link to a file to be downloaded
- `PendingDir` Represents a link to a directory to be crawled
- `CrawledDir` Represents a link to a crawled directory
*/
#[derive(Debug)]
pub enum Node {
    /// This node is a file
    File(FileLinkMetaData),

    /// This node is an un-crawled directory
    PendingDir(DirLinkMetaData),

    /// This node is a crawled directory
    CrawledDir(DirLinkMetaData, Vec<Node>),
}

#[derive(Debug)]
pub struct FileLinkMetaData {
    pub url: reqwest::Url,
    pub name: String,
    pub last_modified: String,
    pub size: String,
    pub description: String,
}

#[derive(Debug)]
pub struct DirLinkMetaData {
    pub url: reqwest::Url,
    pub name: String,
    pub last_modified: String,
    pub description: String,
}
