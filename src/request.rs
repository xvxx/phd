use crate::Result;
use std::fs;

#[derive(Debug)]
pub struct Request {
    pub selector: String,
    pub root: String,
    pub host: String,
    pub port: u16,
}

impl Request {
    pub fn from(host: &str, port: u16, root: &str) -> Result<Request> {
        Ok(Request {
            host: host.into(),
            port: port,
            root: fs::canonicalize(root)?.to_string_lossy().into(),
            selector: String::new(),
        })
    }

    /// Path to the target file on disk requested by this request.
    pub fn file_path(&self) -> String {
        let mut path = self.root.to_string();
        if !path.ends_with('/') {
            path.push('/');
        }
        path.push_str(self.selector.replace("..", ".").trim_start_matches('/'));
        path
    }

    /// Path to the target file relative to the server root.
    pub fn relative_file_path(&self) -> String {
        self.file_path().replace(&self.root, "")
    }
}
