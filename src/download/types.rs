pub struct UrlData {
    pub path: String,
    pub files: Vec<FileData>,
    pub sub_dirs: Vec<DirData>,
}

pub struct FileData {
    pub url: String,
    pub name: String,
    pub last_modified: String,
    pub size: String,
    pub description: String,
}

pub struct DirData {
    pub url: String,
    pub name: String,
    pub last_modified: String,
    pub description: String,
}

impl UrlData {
    pub fn new(path: String) -> Self {
        UrlData {
            path,
            files: vec![],
            sub_dirs: vec![],
        }
    }
}
