/**
This enum defines an entry in an open-directory listing.

- `Root` Represents the root URL of the open-directory listing
- `File` Represents a link to a file to be downloaded
- `PendingDir` Represents a link to a directory to be crawled
- `CrawledDir` Represents a link to a crawled directory
*/
#[derive(Debug)]
pub enum Node {
    /// The root URL of the open directory listing
    Root(String),

    /// This node is a file
    File(FileLinkMetaData),

    /// This node is an un-crawled directory
    PendingDir(DirLinkMetaData),

    /// This node is a crawled directory
    CrawledDir(Vec<Node>),
}

#[derive(Debug)]
#[deprecated]
pub struct DirData {
    pub path: String,
    pub files: Vec<FileLinkMetaData>,
    pub sub_dirs: Vec<(DirLinkMetaData, Option<DirData>)>,
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

impl DirData {
    pub fn new(path: String) -> Self {
        DirData {
            path,
            files: vec![],
            sub_dirs: vec![],
        }
    }
}
