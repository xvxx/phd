//! A Request represents a Gopher request made by a client. phd can
//! serve directory listings as Gopher Menus, plain text files as
//! Text, binary files as downloads, Gophermap files as menus, or
//! executable files as dynamic content.

use crate::Result;
use std::fs;

/// This struct represents a single gopher request.
#[derive(Debug, Clone)]
pub struct Request {
    /// Gopher selector requested
    pub selector: String,
    /// Search query string, if any.
    pub query: String,
    /// Root directory of the server. Can't serve outside of this.
    pub root: String,
    /// Host of the currently running server.
    pub host: String,
    /// Port of the currently running server.
    pub port: u16,
}

impl Request {
    /// Try to create a new request state object.
    pub fn from(host: &str, port: u16, root: &str) -> Result<Request> {
        Ok(Request {
            host: host.into(),
            port,
            root: fs::canonicalize(root)?.to_string_lossy().into(),
            selector: String::new(),
            query: String::new(),
        })
    }

    /// Path to the target file on disk requested by this request.
    pub fn file_path(&self) -> String {
        format!(
            "{}/{}",
            self.root.to_string().trim_end_matches('/'),
            self.selector.replace("..", ".").trim_start_matches('/')
        )
    }

    /// Path to the target file relative to the server root.
    pub fn relative_file_path(&self) -> String {
        self.file_path().replace(&self.root, "")
    }

    /// Set selector + query based on what the client sent.
    pub fn parse_request(&mut self, line: &str) {
        self.query.clear();
        self.selector.clear();
        if let Some((i, _)) = line
            .chars()
            .enumerate()
            .find(|&(_, c)| c == '\t' || c == '?')
        {
            if line.len() > i {
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
