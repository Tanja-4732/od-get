pub struct UrlData {
    path: String,
    files: Vec<FileData>,
    sub_dirs: Vec<DirData>,
}

pub struct FileData {
    url: String,
    name: String,
    last_modified: String,
    size: String,
    destination: Option<String>,
}

pub struct DirData {
    url: String,
    name: String,
    last_modified: String,
    destination: Option<String>,
}
