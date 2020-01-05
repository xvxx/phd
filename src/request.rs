use crate::Result;
use std::fs;

/// This struct represents a single gopher request.
#[derive(Debug, Clone)]
pub struct Request {
    pub selector: String,
    pub query: String,
    pub root: String,
    pub host: String,
    pub port: u16,
}

impl Request {
    /// Try to create a new request state object.
    pub fn from(host: &str, port: u16, root: &str) -> Result<Request> {
        Ok(Request {
            host: host.into(),
            port: port,
            root: fs::canonicalize(root)?.to_string_lossy().into(),
            selector: String::new(),
            query: String::new(),
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

    /// Set selector + query based on what the client sent.
    pub fn parse_request(&mut self, line: &str) {
        self.query.clear();
        self.selector.clear();
        if let Some(i) = line.find('\t') {
            if line.len() >= i + 1 {
                self.query.push_str(&line[i + 1..]);
                self.selector.push_str(&line[..i]);
                return;
            }
        }
        self.selector.push_str(line);

        // strip trailing /
        if let Some(last) = self.selector.chars().last() {
            if last == '/' {
                self.selector.pop();
            }
        }
    }
}
