use gophermap::{GopherMenu, ItemType};
use std::{
    fs,
    io::prelude::*,
    io::{BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};
use threadpool::ThreadPool;

const MAX_WORKERS: usize = 10;
const MAX_PEEK_SIZE: usize = 1024;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub struct Request {
    stream: TcpStream,
    selector: String,
    root: String,
    host: String,
    port: String,
}

/// Starts a Gopher server at the specified root directory.
pub fn start(host: &str, port: &str, root_dir: &str) -> Result<()> {
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr)?;
    println!("-> Listening at {}", addr);
    let pool = ThreadPool::new(MAX_WORKERS);
    for stream in listener.incoming() {
        let stream = stream?;
        println!("-> Connection from: {}", stream.peer_addr()?);
        let mut req = Request::from(
            stream,
            root_dir.to_string(),
            host.to_string(),
            port.to_string(),
        );
        pool.execute(move || {
            if let Err(e) = req.serve() {
                eprintln!("-> {}", e);
            }
        });
    }
    Ok(())
}

impl Request {
    pub fn from(stream: TcpStream, root: String, host: String, port: String) -> Request {
        Request {
            stream,
            root,
            host,
            port,
            selector: String::new(),
        }
    }

    pub fn root_path_as_string(&self) -> Result<String> {
        Ok(fs::canonicalize(&self.root)?.to_string_lossy().to_string())
    }

    pub fn path(&self) -> Result<PathBuf> {
        let mut path = fs::canonicalize(&self.root)?;
        path.push(self.selector.replace("..", ".").trim_start_matches('/'));
        Ok(path)
    }

    pub fn path_as_string(&self) -> Result<String> {
        let mut path = self
            .path()?
            .to_string_lossy()
            .to_string()
            .replace(&self.root_path_as_string()?, "");
        if !path.ends_with('/') {
            path.push('/');
        }
        Ok(path)
    }

    /// Reads from the client and responds.
    fn serve(&mut self) -> Result<()> {
        let reader = BufReader::new(&self.stream);
        let mut lines = reader.lines();
        if let Some(Ok(line)) = lines.next() {
            println!("-> Received: {:?}", line);
            self.selector = line;
            self.respond()?;
        }
        Ok(())
    }

    /// Respond to a client's request.
    fn respond(&mut self) -> Result<()> {
        let md = fs::metadata(self.path()?)?;
        if md.is_file() {
            self.send_text()
        } else if md.is_dir() {
            self.send_dir()
        } else {
            Ok(())
        }
    }

    /// Send a directory listing (menu) to the client.
    fn send_dir(&mut self) -> Result<()> {
        let mut dir = fs::read_dir(self.path()?)?;
        let mut menu = GopherMenu::with_write(&self.stream);
        let path = self.path_as_string()?;
        while let Some(Ok(entry)) = dir.next() {
            let mut path = path.clone();
            let file_name = entry.file_name();
            path.push_str(&file_name.to_string_lossy());
            menu.write_entry(
                file_type(&entry),
                &file_name.to_string_lossy(),
                &path,
                &self.host,
                self.port.parse()?,
            )?;
        }
        menu.end()?;
        Ok(())
    }

    /// Send a text document to the client.
    fn send_text(&mut self) -> Result<()> {
        let path = self.path()?;
        let md = fs::metadata(&path)?;
        let mut f = fs::File::open(&path)?;
        let mut buf = [0; 1024];
        let mut bytes = md.len();
        while bytes > 0 {
            let n = f.read(&mut buf[..])?;
            bytes -= n as u64;
            self.stream.write_all(&buf[..n])?;
        }
        self.stream.write_all(b"\r\n.\r\n")?; // end gopher response
        Ok(())
    }
}

/// Determine the gopher type for a DirEntry on disk.
fn file_type(dir: &fs::DirEntry) -> ItemType {
    let metadata = dir.metadata();
    if metadata.is_err() {
        return ItemType::Error;
    }
    let metadata = metadata.unwrap();

    if metadata.is_file() {
        if let Ok(file) = fs::File::open(&dir.path()) {
            let mut buffer: Vec<u8> = vec![];
            let _ = file.take(MAX_PEEK_SIZE as u64).read_to_end(&mut buffer);
            if content_inspector::inspect(&buffer).is_binary() {
                ItemType::Binary
            } else {
                ItemType::File
            }
        } else {
            ItemType::Binary
        }
    } else if metadata.is_dir() {
        ItemType::Directory
    } else {
        ItemType::Error
    }
}
