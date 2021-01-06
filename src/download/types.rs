pub struct DirData {
    pub path: String,
    pub files: Vec<FileLinkMetaData>,
    pub sub_dirs: Vec<(DirLinkMetaData, Option<DirData>)>,
}

pub struct FileLinkMetaData {
    pub url: String,
    pub name: String,
    pub last_modified: String,
    pub size: String,
    pub description: String,
}

pub struct DirLinkMetaData {
    pub url: String,
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
